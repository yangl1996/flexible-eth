use bincode;
use rocksdb::{DB, Options};

mod rule;
use crate::data;
use crate::utils;

pub  fn get_first_block(
    db: &DB,
    epoch: usize,
) -> Result<Option<(data::Root, usize)>, Box<dyn std::error::Error>> {
    for s in utils::epoch_to_slot(epoch)..utils::epoch_to_slot(epoch+1) {
        match &db.get(&format!("block_{}", s))? {
            Some(serialized_blkroot) => {
                return Ok(Some((bincode::deserialize::<data::Root>(serialized_blkroot)?, s)));
            }
            None => {
                continue;
            }
        };
    }
    return Ok(None);
}

pub async fn main(
    db_path: String,
    quorum: Vec<f64>,
    min_slot: usize,
    max_slot: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut db_opts = Options::default();
    // db_opts.create_if_missing(true);
    db_opts.increase_parallelism(utils::get_available_cpucores() as i32);
    db_opts.optimize_level_style_compaction(utils::get_available_ram() / 4);
    db_opts.optimize_for_point_lookup(utils::get_available_ram() as u64 / 4);
    let db = DB::open_for_read_only(&db_opts, db_path, true)?;

    // ensure confirmation is up to a reasonable target
    if max_slot < min_slot {
        log::error!(
            "Maximum slot cannot be smaller than the minimum slot"
            );
    }
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
    let mut min_slot = min_slot;
    if min_slot != utils::most_recent_epoch_boundary_slot_for_slot(min_slot) {
        let new_min_slot = utils::most_recent_epoch_boundary_slot_for_slot(min_slot);
        log::warn!(
            "Minimum slot {} is not an epoch boundary, using {} instead",
            min_slot,
            new_min_slot
        );
        min_slot = new_min_slot;
    }

    // ensure necessary data has been sync'ed
    for slot in min_slot..max_slot {
        if db.get(format!("slot_{}_synched", slot))?.is_none() {
            log::error!(
                "Slot {} not synched",
                slot
                );
            return Err("Sync is not complete".into());
        }
    }

    // setup confirmation rules
    let mut conf_rule_states = Vec::new();
    for (idx, q) in quorum.iter().enumerate() {
        // FIXME: we are initing the confirmation rule with genesis root, which is not correct. Fortunately the confirmed tip (which this root is affecting) is not used in the conf rule for now (all relevant checks are disabled).
        conf_rule_states.push(rule::ConfirmationRuleState::new(*q, data::HEADER_GENESIS_ROOT.to_string(), 0));
        println!("LEDGER t={} {:?}", 0, conf_rule_states[idx]);
    }

    // run confirmation rules
    for epoch in (utils::slot_to_epoch(min_slot) + 1)..(utils::slot_to_epoch(max_slot) + 1) {
        log::info!("Running confirmation rules for epoch {}", epoch);

        // epoch boundary block-roots and slot numbers (skip epoch if the entire epoch is empty)
        let (blkroot_e, slot_e) = match get_first_block(&db, epoch)? {
            Some(v) => v,
            None => {
                log::warn!("Epoch {} is empty", epoch);
                continue;
            },
        };
        let (blkroot_em1, slot_em1) = match get_first_block(&db, epoch-1)? {
            Some(v) => v,
            None => {
                log::warn!("Epoch {} is empty", epoch-1);
                continue;
            },
        };

        log::info!("First-block roots and slots: e-1: {} ({}) / e: {} ({})", blkroot_em1, slot_em1, blkroot_e, blkroot_e);

        // epoch boundary blocks
        // let blk_e = bincode::deserialize::<data::Block>(
        //     &db.get(&format!("block_{}", blkroot_e))?
        //         .expect("Block for blkroot_e not found"),
        // )?;
        let blk_em1 = bincode::deserialize::<data::Block>(
            &db.get(&format!("block_{}", blkroot_em1))?
                .expect("Block for blkroot_em1 not found"),
        )?;

        // chains of epoch boundary blocks (to ensure consistency of the blocks)
        // FIXME: disabling the check since we assume there hasn't been a 51% attack
        /*
        let chain_e = bincode::deserialize::<Vec<data::Root>>(
            &db.get(&format!("chain_{}", blkroot_e))?
                .expect("Chain of block-roots for blkroot_e not found"),
        )?;
        let chain_em1 = bincode::deserialize::<Vec<data::Root>>(
            &db.get(&format!("chain_{}", blkroot_em1))?
                .expect("Chain of block-roots for blkroot_em1 not found"),
        )?;
        assert!(utils::is_prefix_of(&chain_em1, &chain_e));
        */

        // load committee information necessary for confirmation rule to count votes
        let committees = bincode::deserialize::<Vec<data::CommitteeAssignment>>(
            &db.get(&format!("state_{}_committees", blk_em1.state_root))?
                .expect("Committees not found"),
        )?;

        // load blocks that contain the votes in question
        let mut blkroots = vec![];
        for s in slot_em1..=slot_e {
            match &db.get(&format!("block_{}", s))? {
                Some(serialized_blkroot) => {
                    blkroots.push(bincode::deserialize::<data::Root>(serialized_blkroot)?);
                }
                None => {
                    continue;
                }
            };
        };
        //let blkroots = chain_e[chain_em1.len() - 1..].to_vec();
        let blks = blkroots.iter().map(|blkroot_chain| {
            let blk_chain = bincode::deserialize::<data::Block>(
                &db.get(&format!("block_{}", blkroot_chain))?
                    .expect("Block not found"),
            )?;
            log::debug!("Block: {:?}", blk_chain);
            Ok(blk_chain)
        }).collect::<Result<Vec<data::Block>, Box<dyn std::error::Error>>>()?.to_vec();

        // load checkpoint information of what is the confirmation target in question
        // TODO: check if the following assumption is correct---the checkpoint loaded here is the same as the one from the EBB for epoch e-1
        let (_cp_previous_justified, _cp_current_justified, cp_finalized) =
        bincode::deserialize::<(data::Checkpoint, data::Checkpoint, data::Checkpoint)>(
            &db.get(&format!(
                "state_{}_finality_checkpoints",
                blk_em1.state_root
            ))?
            .expect("Finality checkpoints not found"),
        )?;
        let mut cp_finalized_blkroot = cp_finalized.root;
        if cp_finalized_blkroot == "0x0000000000000000000000000000000000000000000000000000000000000000" {
            cp_finalized_blkroot = data::HEADER_GENESIS_ROOT.to_string();
        }

        // load block information of the confirmation target in question
        let cp_finalized_blk = bincode::deserialize::<data::Block>(
            &db.get(&format!("block_{}", cp_finalized_blkroot))?
                .expect("Block for cp_finalized_blk not found"),
        )?;
        log::info!(
            "Block to confirm: blkroot={}, slot={}",
            cp_finalized_blkroot, cp_finalized_blk.slot,
        );
        // FIXME: disabling the check
        /*
        let chain_tip_new = bincode::deserialize::<Vec<data::Root>>(
            &db.get(&format!("chain_{}", cp_finalized_blkroot))?
                .expect("Chain of block-roots for cp_finalized_blkroot not found"),
        )?;
        */

        // invoke confirmation rules
        for rule in conf_rule_states.iter_mut() {
            if rule.count_votes_for_confirmation(slot_em1, slot_e, &blkroot_em1, &committees, &blkroots, &blks) {
                // confirmation takes place according to the rule
                // FIXME: disabling the check
                /*
                let tip_old = rule.get_tip_blkroot();
                let chain_tip_old = bincode::deserialize::<Vec<data::Root>>(
                    &db.get(&format!("chain_{}", tip_old))?
                        .expect("Chain of block-roots for tip_old not found"),
                )?;
                assert!(utils::is_prefix_of(&chain_tip_old, &chain_tip_new));
                */

                rule.update_tip(cp_finalized_blkroot.clone(), cp_finalized_blk.slot);
                println!("LEDGER t={} {:?}", slot_e, rule);
            }
        }
    }

    Ok(())
}
