use bincode;
use ratelimit::Ratelimiter;
use reqwest;
use rocksdb::DB;

mod api;
use crate::data;
use crate::utils::{self, is_epoch_boundary_slot};

fn ratelimiter_wait(ratelimiter: &mut Ratelimiter) {
    while let Err(sleep) = ratelimiter.try_wait() {
        std::thread::sleep(sleep);
    }
}

pub async fn main(
    db_path: String,
    rpc_url: String,
    max_slot: usize,
    mut ratelimiter: Ratelimiter,
) -> Result<(), Box<dyn std::error::Error>> {
    let db = DB::open_default(db_path)?;
    let mut rpc = reqwest::Client::new();

    // ensure sync is up to a reasonable target
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

    // track sync progress
    let begin_slot = match db.get("sync_progress")? {
        Some(serialized) => bincode::deserialize::<usize>(&serialized)? + 1,
        None => 0,
    };
    max_slot += 1; // include last epoch boundary block in sync
    log::info!("Syncing slots {}..{}", begin_slot, max_slot);

    // sync
    for slot in begin_slot..max_slot {
        log::info!("Syncing slot {}", slot);

        // sync canonical chain blocks
        ratelimiter_wait(&mut ratelimiter);
        let blk_root = match api::get_blockroot_by_slot(&mut rpc, &rpc_url, &slot).await? {
            Some(root) => {
                log::debug!("Canonical block root: {:?}", &root);
                db.put(format!("block_{}", &slot), bincode::serialize(&root)?)?;
                root
            }
            None => continue, // skip empty slots
        };

        // sync block
        ratelimiter_wait(&mut ratelimiter);
        let blk = api::get_block_by_blockroot(&mut rpc, &rpc_url, &blk_root)
            .await?
            .expect("Block not found");
        log::debug!("Canonical block: {:?}", &blk);
        db.put(format!("block_{}", &blk_root), bincode::serialize(&blk)?)?;

        // sync block chain structure
        if blk.parent_root == "0x0000000000000000000000000000000000000000000000000000000000000000" {
            assert!(blk.slot == 0);
            assert!(blk.proposer_index == 0);
            assert!(blk_root == data::HEADER_GENESIS_ROOT);
            log::debug!("Genesis block, trivial block chain!");
            db.put(
                format!("chain_{}", &blk_root),
                bincode::serialize(&vec![blk_root])?,
            )?;
        } else {
            let mut chain = bincode::deserialize::<Vec<data::Root>>(
                &db.get(&format!("chain_{}", blk.parent_root))?
                    .expect("Parent chain not found"),
            )?;
            chain.push(blk_root.clone());
            log::debug!("Block chain: {:?}", &chain);
            db.put(format!("chain_{}", &blk_root), bincode::serialize(&chain)?)?;
        }

        // sync state at epoch boundaries
        if is_epoch_boundary_slot(slot) {
            // TODO: while Prysm retains old states, it seems they can only be accessed by-slot, not by-state-root.
            // so we make sure that retrieving by slot gives us data that belongs to the right state-root (that of the block)
            ratelimiter_wait(&mut ratelimiter);
            let tmp_state_root = api::get_stateroot_by_slot(&mut rpc, &rpc_url, &slot).await?;
            log::debug!(
                "State-root by block: {:?} / state-root by slot: {:?}",
                &blk.state_root,
                &tmp_state_root
            );
            assert!(tmp_state_root == blk.state_root);

            ratelimiter_wait(&mut ratelimiter);
            let (cp_previous_justified, cp_current_justified, cp_finalized) =
                api::get_state_finality_checkpoints_by_slot(&mut rpc, &rpc_url, &slot).await?;
            log::debug!(
                "Finality checkpoints: {:?}, {:?}, {:?}",
                &cp_previous_justified,
                &cp_current_justified,
                &cp_finalized
            );
            db.put(
                format!("state_{}_finality_checkpoints", blk.state_root),
                bincode::serialize(&(cp_previous_justified, cp_current_justified, cp_finalized))?,
            )?;

            // ratelimiter_wait(&mut ratelimiter);
            // let vals = api::get_state_validators(&mut rpc, &rpc_url, &slot).await?;
            // db.put(
            //     format!("state_{}_validators", slot),
            //     bincode::serialize(&vals)?,
            // )?;

            ratelimiter_wait(&mut ratelimiter);
            let committees = api::get_state_committees_by_slot(&mut rpc, &rpc_url, &slot).await?;
            log::debug!("Committees: {:?}", &committees);
            db.put(
                format!("state_{}_committees", blk.state_root),
                bincode::serialize(&committees)?,
            )?;

            // TODO: while Prysm retains old states, it seems they can only be accessed by-slot, not by-state-root.
            // so we make sure that retrieving by slot gives us data that belongs to the right state-root (that of the block)
            ratelimiter_wait(&mut ratelimiter);
            let tmp_state_root = api::get_stateroot_by_slot(&mut rpc, &rpc_url, &slot).await?;
            log::debug!(
                "State-root by block: {:?} / state-root by slot: {:?}",
                &blk.state_root,
                &tmp_state_root
            );
            assert!(tmp_state_root == blk.state_root);
        }

        db.put("sync_progress", bincode::serialize(&slot)?)?;
    }

    Ok(())
}
