use crate::data::{self, GetHeadersResponse, IdentifiedData, ResponseError};
use ratelimit::Ratelimiter;
use reqwest;
use rocksdb::{Options, DB};
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

    println!("{}", json_string);
    match serde_json::from_str::<GetHeadersResponse>(&json_string) {
        Ok(resp) => {
            let mut headers = Vec::new();
            for hdr in resp.data {
                println!("{:?}", hdr.header.message);
                //headers.push(IdentifiedData {
                //    root: hdr["root"].as_str().unwrap().to_string(),
                //     data: data::Header {
                //         slot: hdr["header"].as_object().unwrap()["message"]
                //             .as_object()
                //             .unwrap()["slot"]
                //             .as_u64()
                //             .unwrap() as usize,
                //         proposer_index: hdr["header"].as_object().unwrap()["message"]
                //             .as_object()
                //             .unwrap()["proposer_index"]
                //             .as_u64()
                //             .unwrap() as usize,
                //         parent_root: hdr["header"].as_object().unwrap()["message"]
                //             .as_object()
                //             .unwrap()["parent_root"]
                //             .as_str()
                //             .unwrap()
                //             .to_string(),
                //         state_root: hdr["header"].as_object().unwrap()["message"]
                //             .as_object()
                //             .unwrap()["state_root"]
                //             .as_str()
                //             .unwrap()
                //             .to_string(),
                //         body_root: hdr["header"].as_object().unwrap()["message"]
                //             .as_object()
                //             .unwrap()["body_root"]
                //             .as_str()
                //             .unwrap()
                //             .to_string(),
                //     },
                // });
            }
            Ok(headers)
        }
        Err(_) => {
            // It could either be that the slot is empty, or we encountered a server error.
            // Figure it out by parsing the error message.
            let err = serde_json::from_str::<ResponseError>(&json_string)?;
            match err.code {
                404 => Ok(vec![]),
                _ => Err(Box::new(err)),
            }
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

    // slot number should be stored as u256 but u64 is fine
    let begin_slot = match db.get("sync_progress")? {
        Some(serialized) => usize::from_le_bytes(serialized.try_into().unwrap()) + 1,
        None => 0,
    };

    for slot in begin_slot..max_slot {
        ratelimiter_wait(&mut ratelimiter);
        let headers = get_headers(&mut rpc, &rpc_url, slot).await?;
        println!("Headers: {:?}", headers);
        // let body = reqwest::get(format!("{}/eth/v1/beacon/headers", https://www.rust-lang.org")
        //     .await?
        //     .text()
        //     .await?;
        db.put("sync_progress", slot.to_le_bytes())?;
    }

    Ok(())
}
