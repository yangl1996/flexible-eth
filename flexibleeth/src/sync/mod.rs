use crate::data::{self, GetBlockResponse, GetHeadersResponse, IdentifiedData, ResponseError};
use bincode;
use ratelimit::Ratelimiter;
use reqwest;
use rocksdb::DB;
use serde_json;

fn ratelimiter_wait(ratelimiter: &mut Ratelimiter) {
    while let Err(sleep) = ratelimiter.try_wait() {
        std::thread::sleep(sleep);
    }
}

async fn get_headers(
    client: &mut reqwest::Client,
    rpc_url: &str,
    slot: usize,
) -> Result<Vec<data::IdentifiedData<data::Header>>, Box<dyn std::error::Error>> {
    let json_string = client
        .get(format!("{}/eth/v1/beacon/headers", rpc_url))
        .query(&[("slot", slot)])
        .send()
        .await?
        .text()
        .await?;

    match serde_json::from_str::<GetHeadersResponse>(&json_string) {
        Ok(resp) => {
            let mut headers = Vec::new();
            for hdr in resp.data {
                headers.push(IdentifiedData {
                    root: hdr.root,
                    data: hdr.header.message,
                });
            }
            assert!(
                slot != 0 || (headers.len() == 1 && headers[0].root == data::HEADER_GENESIS_ROOT)
            );
            Ok(headers)
        }
        Err(_) => {
            let err = serde_json::from_str::<ResponseError>(&json_string)?;
            Err(Box::new(err))
        }
    }
}

async fn get_block(
    client: &mut reqwest::Client,
    rpc_url: &str,
    root: data::Root,
) -> Result<data::IdentifiedData<data::Block>, Box<dyn std::error::Error>> {
    let json_string = client
        .get(format!("{}/eth/v2/beacon/blocks/{}", rpc_url, root))
        .send()
        .await?
        .text()
        .await?;

    match serde_json::from_str::<GetBlockResponse>(&json_string) {
        Ok(resp) => Ok(IdentifiedData {
            root: root,
            data: resp.data.message,
        }),
        Err(_) => {
            let err = serde_json::from_str::<ResponseError>(&json_string)?;
            Err(Box::new(err))
        }
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
        let headers = get_headers(&mut rpc, &rpc_url, slot).await?;

        let mut headers_roots = Vec::new();
        for hdr in headers {
            log::debug!("Header: {:?}", hdr);
            db.put(format!("header_{}", hdr.root), bincode::serialize(&hdr)?)?;
            headers_roots.push(hdr.root.clone());

            ratelimiter_wait(&mut ratelimiter);
            let blk = get_block(&mut rpc, &rpc_url, hdr.root).await?;
            log::debug!("Block: {:?}", blk);
            db.put(format!("block_{}", blk.root), bincode::serialize(&blk)?)?;
        }
        db.put(
            format!("headers_for_slot_{}", slot),
            bincode::serialize(&headers_roots)?,
        )?;

        db.put("sync_progress", bincode::serialize(&slot)?)?;
    }

    Ok(())
}
