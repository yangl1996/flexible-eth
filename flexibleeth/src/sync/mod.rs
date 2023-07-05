use bincode;
use ratelimit::Ratelimiter;
use reqwest;
use rocksdb::DB;

mod api;
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

        ratelimiter_wait(&mut ratelimiter);
        match api::get_block(&mut rpc, &rpc_url, &slot).await? {
            Some(blk) => {
                log::debug!("Block: {:?}", blk);
                db.put(format!("block_{}", &slot), bincode::serialize(&blk)?)?;
            }
            None => {}
        }

        if is_epoch_boundary_slot(slot) {
            ratelimiter_wait(&mut ratelimiter);
            let (cp_previous_justified, cp_current_justified, cp_finalized) =
                api::get_state_finality_checkpoints(&mut rpc, &rpc_url, &slot).await?;
            db.put(
                format!("state_{}_finality_checkpoints", slot),
                bincode::serialize(&(cp_previous_justified, cp_current_justified, cp_finalized))?,
            )?;

            // ratelimiter_wait(&mut ratelimiter);
            // let vals = api::get_state_validators(&mut rpc, &rpc_url, &slot).await?;
            // db.put(
            //     format!("state_{}_validators", slot),
            //     bincode::serialize(&vals)?,
            // )?;

            // ratelimiter_wait(&mut ratelimiter);
            // let comms = api::get_state_committees(&mut rpc, &rpc_url, &slot).await?;
            // db.put(
            //     format!("state_{}_committees", slot),
            //     bincode::serialize(&comms)?,
            // )?;
        }

        db.put("sync_progress", bincode::serialize(&slot)?)?;
    }

    Ok(())
}
