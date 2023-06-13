use crate::data::{Block, Header, IdentifiedData};
use bincode;
use ratelimit::Ratelimiter;
use reqwest;
use rocksdb::DB;
use serde_json;

mod api;

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

    let begin_slot = match db.get("sync_progress")? {
        Some(serialized) => bincode::deserialize::<usize>(&serialized)? + 1,
        None => 0,
    };
    log::info!("Syncing slots {}..{}", begin_slot, max_slot);

    for slot in begin_slot..max_slot {
        log::debug!("Syncing slot {}", slot);

        ratelimiter_wait(&mut ratelimiter);
        let headers = api::get_headers(&mut rpc, &rpc_url, slot).await?;

        let mut headers_roots = Vec::new();
        for hdr in headers {
            log::debug!("Header: {:?}", hdr);
            db.put(format!("header_{}", hdr.root), bincode::serialize(&hdr)?)?;
            headers_roots.push(hdr.root.clone());

            ratelimiter_wait(&mut ratelimiter);
            let blk = api::get_block(&mut rpc, &rpc_url, hdr.root).await?;
            log::debug!("Block: {:?}", blk);
            db.put(format!("block_{}", blk.root), bincode::serialize(&blk)?)?;

            ratelimiter_wait(&mut ratelimiter);
        }
        db.put(
            format!("headers_for_slot_{}", slot),
            bincode::serialize(&headers_roots)?,
        )?;

        db.put("sync_progress", bincode::serialize(&slot)?)?;
    }

    Ok(())
}
