use bincode;
use rocksdb::{DB, Options};

mod rule;
use crate::data;
use crate::utils;

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
    for (_, q) in quorum.iter().enumerate() {
        conf_rule_states.push(rule::ConfirmationState::new(*q));
    }

    let mut last_registered_epoch = utils::slot_to_epoch(min_slot);
    for slot in min_slot..=max_slot {
        let blkroot = match &db.get(&format!("block_{}", slot))? {
            Some(serialized_blkroot) => {
                bincode::deserialize::<data::Root>(serialized_blkroot)?
            }
            None => {
                continue;
            }
        };
        let blk = bincode::deserialize::<data::Block>(
            &db.get(&format!("block_{}", blkroot))?
                .expect("Block not found when block root is present in db"),
        )?;

        // if this is the first block of an epoch, register it as a confirmation target
        let epoch = utils::slot_to_epoch(slot);
        if epoch > last_registered_epoch {
            // load committees
            let committees = bincode::deserialize::<Vec<data::CommitteeAssignment>>(
                &db.get(&format!("state_{}_committees", blk.state_root))?
                .expect("Committees not found"),
                )?;

            // load checkpoint information of what is the confirmation target in question
            let (_cp_previous_justified, _cp_current_justified, cp_finalized) =
                bincode::deserialize::<(data::Checkpoint, data::Checkpoint, data::Checkpoint)>(
                    &db.get(&format!(
                            "state_{}_finality_checkpoints",
                            blk.state_root
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
            log::info!("Registering blkroot {} slot {} as confirmation target for epoch {}", cp_finalized_blkroot, cp_finalized_blk.slot, epoch);

            let ebb_root = bincode::deserialize::<data::Root>(
                &db.get(&format!("ebb_{}_root", epoch))?
                .expect("EBB root for current epoch not found"),
                )?;

            for rule in conf_rule_states.iter_mut() {
                rule.register_first_block_of_epoch(epoch, ebb_root.clone(), cp_finalized_blk.slot, &committees);
            }

            last_registered_epoch = epoch;
        }
        for rule in conf_rule_states.iter_mut() {
            match rule.process_block(&blk) {
                Some(s) => {
                    println!("LEDGER t={} tip={}, quorum={}", slot, s, rule.get_quorum());
                }
                None => {},
            };
        }
    }

    Ok(())
}
