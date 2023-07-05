use bincode;
use rocksdb::DB;

use crate::data;
use crate::utils;

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
    let mut current_tip = data::HEADER_GENESIS_ROOT.to_string();
    for epoch in 1..(utils::slot_to_epoch(max_slot) + 1) {
        log::info!("Running confirmation rule for epoch {}", epoch);

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

        let blk_e = bincode::deserialize::<data::Block>(
            &db.get(&format!("block_{}", blkroot_e))?
                .expect("Block not found"),
        )?;
        let blk_em1 = bincode::deserialize::<data::Block>(
            &db.get(&format!("block_{}", blkroot_em1))?
                .expect("Block not found"),
        )?;

        let chain_e = bincode::deserialize::<Vec<data::Root>>(
            &db.get(&format!("chain_{}", blkroot_e))?
                .expect("Chain of block-roots not found"),
        )?;
        let chain_em1 = bincode::deserialize::<Vec<data::Root>>(
            &db.get(&format!("chain_{}", blkroot_em1))?
                .expect("Chain of block-roots not found"),
        )?;
        assert!(utils::is_prefix_of(&chain_em1, &chain_e));

        // let (cp_previous_justified, cp_current_justified, cp_finalized) =
        //     bincode::deserialize::<(data::Checkpoint, data::Checkpoint, data::Checkpoint)>(
        //         &db.get(&format!("state_{}_finality_checkpoints", slot_e_minus_1))?
        //             .expect("Finality checkpoints not found"),
        //     )?;

        // log::info!("Finalized checkpoint: {:?}", cp_finalized);
        // let mut finalized_root = cp_finalized.root;
        // // if finalized_root == "0x0000000000000000000000000000000000000000000000000000000000000000" {
        // //     finalized_root = data::HEADER_GENESIS_ROOT.to_string();
        // // }
        // if finalized_root != "0x0000000000000000000000000000000000000000000000000000000000000000" {
        //     log::info!(
        //         "Finalized block: {:?}",
        //         bincode::deserialize::<data::Block>(
        //             &db.get(&format!("block_{}", finalized_root))?
        //                 .expect("Block not found")
        //         )
        //     );
        // }
        // // log::info!("Current justified checkpoint: {:?}", cp_current_justified);
        // // log::info!("Previous justified checkpoint: {:?}", cp_previous_justified);
    }

    // println!("Confirmation rule called!");
    // println!("DB path: {:?}", db_path);
    // println!("Quorum: {:?}", quorum);
    // println!("Max slot: {:?}", max_slot);

    Ok(())
}
