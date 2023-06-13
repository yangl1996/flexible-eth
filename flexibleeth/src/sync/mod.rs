use crate::data::{self, IdentifiedData};
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

    let parsed_json: serde_json::Value = serde_json::from_str(&json_string)?;

    let mut headers = Vec::new();
    for hdr in parsed_json["data"].as_array().unwrap() {
        println!("{:?}", hdr.get("root"));
        // headers.push(IdentifiedData {
        //     root: hdr["root"].as_str().unwrap().to_string(),
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

pub async fn main(
    db_path: String,
    rpc_url: String,
    max_slot: usize,
    mut ratelimiter: Ratelimiter,
) -> Result<(), Box<dyn std::error::Error>> {
    let db = DB::open_default(db_path)?;
    let mut rpc = reqwest::Client::new();

    for slot in 0..max_slot {
        if db.get(format!("slot-{}-synced", slot))?.is_some() {
            log::info!("Slot {} already synchronized", slot);
            continue;
        }

        ratelimiter_wait(&mut ratelimiter);
        let headers = get_headers(&mut rpc, &rpc_url, slot).await?;
        println!("Headers: {:?}", headers);
        // let body = reqwest::get(format!("{}/eth/v1/beacon/headers", https://www.rust-lang.org")
        //     .await?
        //     .text()
        //     .await?;
    }

    Ok(())
}
