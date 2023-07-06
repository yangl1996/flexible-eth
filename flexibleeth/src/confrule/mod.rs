use bincode;
use rocksdb::DB;
use std::collections::{HashSet, HashMap};

mod rule;
use crate::data;
use crate::utils;

use self::rule::ConfirmationRuleState;

pub async fn main(
    db_path: String,
    quorum: f64,
    max_slot: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let db = DB::open_default(db_path)?;

    // ensure confirmation is up to a reasonable target
    let mut max_slot = max_slot;
    let now_unixtime = utils::get_unixtime();
    let now_slot = utils::unixtime_to_slot(now_unixtime);
    if max_slot > now_slot - utils::GAP_LATEST_SLOT_NOW_SLOT_CANONICAL_CHAIN_STABILITY {
        let new_max_slot = now_slot - utils::GAP_LATEST_SLOT_NOW_SLOT_CANONICAL_CHAIN_STABILITY;
        log::warn!(
            "Maximum slot {} is too recent, using {} instead to avoid undetected reorgs of the canonical chain",
            max_slot,
            new_max_slot
        );
        max_slot = new_max_slot;
    }
    if max_slot != utils::most_recent_epoch_boundary_slot_for_slot(max_slot) {
        let new_max_slot = utils::most_recent_epoch_boundary_slot_for_slot(max_slot);
        log::warn!(
            "Maximum slot {} is not an epoch boundary, using {} instead",
            max_slot,
            new_max_slot
        );
        max_slot = new_max_slot;
    }

    // ensure necessary data has been sync'ed
    let last_synced_slot = match db.get("sync_progress")? {
        Some(serialized) => bincode::deserialize::<usize>(&serialized)?,
        None => 0,
    };
    if last_synced_slot < max_slot {
        log::error!(
            "Sync is not up to slot {}, only up to slot {}",
            max_slot,
            last_synced_slot
        );
        return Err("Sync is not complete".into());
    }

    // run confirmation rule
    let mut conf_rule_state = ConfirmationRuleState::new(quorum, data::HEADER_GENESIS_ROOT.to_string());
    let mut current_tip = data::HEADER_GENESIS_ROOT.to_string();
    println!(
        "CONFIRMATION({}/{}={}>={}): t={} tip_root={} tip_slot={}",
        0,
        0,
        1.0,
        1.0,
        0,
        current_tip,
        0
    );
    for epoch in 1..(utils::slot_to_epoch(max_slot) + 1) {
        log::info!("Running confirmation rules for epoch {}", epoch);

        let slot_e = utils::epoch_to_slot(epoch);
        let slot_em1 = utils::epoch_to_slot(epoch - 1);

        let blkroot_e = match &db.get(&format!("block_{}", slot_e))? {
            Some(serialized_blkroot) => bincode::deserialize::<data::Root>(serialized_blkroot)?,
            None => {
                log::warn!("Block at slot {} not found", slot_e);
                continue;
            }
        };
        let blkroot_em1 = match &db.get(&format!("block_{}", slot_em1))? {
            Some(serialized_blkroot) => bincode::deserialize::<data::Root>(serialized_blkroot)?,
            None => {
                log::warn!("Block at slot {} not found", slot_em1);
                continue;
            }
        };

        log::info!("Block-roots: e-1: {} / e: {}", blkroot_em1, blkroot_e);

        // let blk_e = bincode::deserialize::<data::Block>(
        //     &db.get(&format!("block_{}", blkroot_e))?
        //         .expect("Block for blkroot_e not found"),
        // )?;
        let blk_em1 = bincode::deserialize::<data::Block>(
            &db.get(&format!("block_{}", blkroot_em1))?
                .expect("Block for blkroot_em1 not found"),
        )?;

        let chain_e = bincode::deserialize::<Vec<data::Root>>(
            &db.get(&format!("chain_{}", blkroot_e))?
                .expect("Chain of block-roots for blkroot_e not found"),
        )?;
        let chain_em1 = bincode::deserialize::<Vec<data::Root>>(
            &db.get(&format!("chain_{}", blkroot_em1))?
                .expect("Chain of block-roots for blkroot_em1 not found"),
        )?;
        assert!(utils::is_prefix_of(&chain_em1, &chain_e));

        let committees = bincode::deserialize::<Vec<data::CommitteeAssignment>>(
            &db.get(&format!("state_{}_committees", blk_em1.state_root))?
                .expect("Committees not found"),
        )?;
        // let blkroots = chain_e[chain_em1.len() - 1..].to_vec();
        // let blocks = blkroots.iter().map(|blkroot_chain| {
        //     let blk_chain = bincode::deserialize::<data::Block>(
        //         &db.get(&format!("block_{}", blkroot_chain))?
        //             .expect("Block not found"),
        //     )?;
        //     log::debug!("Block: {:?}", blk_chain);
        //     Ok(blk_chain)
        // }).collect::<Vec<_>>()?;

        let mut accounting_committees = HashSet::new();
        let mut accounting_validators = HashSet::new();
        let mut validators_n: usize = 0;
        for committee in committees {
            assert!(committee.slot >= slot_em1);
            assert!(committee.slot < slot_e);

            let is_new = accounting_committees.insert((committee.slot, committee.index));
            assert!(is_new);

            for validator in committee.validators {
                let is_new = accounting_validators.insert(validator);
                assert!(is_new);
                validators_n += 1;
            }
        }

        log::info!("Validator n: {}", validators_n);
        let validators_q = (validators_n as f64 * quorum).ceil() as usize;
        log::info!("Validator q: {}", validators_q);
        let mut validators_votes: usize = 0;

        let mut votes_counted_aggregators = HashMap::new();
        for blkroot_chain in chain_e[chain_em1.len() - 1..].iter() {
            log::debug!("Counting votes in block-root {}", blkroot_chain);

            let blk_chain = bincode::deserialize::<data::Block>(
                &db.get(&format!("block_{}", blkroot_chain))?
                    .expect("Block not found"),
            )?;
            log::debug!("Block: {:?}", blk_chain);

            for attestation in blk_chain.body.attestations {
                if attestation.data.slot < slot_em1 {
                    // skip attestations from before the epoch in question
                    continue;
                }
                if attestation.data.slot >= slot_e {
                    // skip attestations from after the epoch in question
                    continue;
                }
                if attestation.data.target.root != blkroot_em1 {
                    // skip attestations that are not for the target in question
                    continue;
                }

                assert!(accounting_committees.contains(&(attestation.data.slot, attestation.data.index)));
                if !votes_counted_aggregators.contains_key(&(attestation.data.slot, attestation.data.index)) {
                    votes_counted_aggregators.insert((attestation.data.slot, attestation.data.index), utils::AggregationBits::new_from_0xhex_str_zeroed(&attestation.aggregation_bits));
                }
                let votes_counted_aggregator = votes_counted_aggregators.get_mut(&(attestation.data.slot, attestation.data.index)).unwrap();

                let new_aggregate_aggregator = utils::AggregationBits::new_from_0xhex_str(&attestation.aggregation_bits);
                let new_votes = votes_counted_aggregator.incorporate_delta(&new_aggregate_aggregator);
                validators_votes += new_votes.count();
            }
        }

        log::info!("Validator votes: {}", validators_votes);

        if validators_votes >= validators_q {
            log::info!(
                "Quorum {}/{} >= {} have acknowledged {} ...",
                validators_votes,
                validators_n,
                quorum,
                blkroot_em1
            );

            let (_cp_previous_justified, _cp_current_justified, cp_finalized) =
                bincode::deserialize::<(data::Checkpoint, data::Checkpoint, data::Checkpoint)>(
                    &db.get(&format!(
                        "state_{}_finality_checkpoints",
                        blk_em1.state_root
                    ))?
                    .expect("Finality checkpoints not found"),
                )?;
            let mut new_tip = cp_finalized.root;

            log::info!(
                "... which commits them to considering finalized: {}",
                new_tip
            );
            if new_tip == "0x0000000000000000000000000000000000000000000000000000000000000000" {
                new_tip = data::HEADER_GENESIS_ROOT.to_string();
            }

            let chain_current_tip = bincode::deserialize::<Vec<data::Root>>(
                &db.get(&format!("chain_{}", current_tip))?
                    .expect("Chain of block-roots for current_tip not found"),
            )?;
            let chain_new_tip = bincode::deserialize::<Vec<data::Root>>(
                &db.get(&format!("chain_{}", new_tip))?
                    .expect("Chain of block-roots for new_tip not found"),
            )?;
            assert!(utils::is_prefix_of(&chain_current_tip, &chain_new_tip));

            current_tip = new_tip;

            let blk_tip = bincode::deserialize::<data::Block>(
                &db.get(&format!("block_{}", current_tip))?
                    .expect("Block for current_tip not found"),
            )?;

            println!(
                "CONFIRMATION({}/{}={}>={}): t={} tip_root={} tip_slot={}",
                validators_votes,
                validators_n,
                validators_votes as f64 / validators_n as f64,
                quorum,
                slot_e,
                current_tip,
                blk_tip.slot
            );
        }
    }

    Ok(())
}
