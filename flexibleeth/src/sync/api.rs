// use ruint::{uint, Uint};
use crate::data;
use serde::{Deserialize, Serialize};
use serde_aux::prelude::*;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseError {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub code: u64,
    pub message: String,
}

impl fmt::Display for ResponseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl Error for ResponseError {}

pub async fn get_headers(
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
                headers.push(data::IdentifiedData {
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

#[derive(Debug, Clone, Deserialize)]
pub struct GetHeadersResponse {
    pub data: Vec<GetHeadersResponseData>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetHeadersResponseData {
    pub root: data::Root,
    pub header: GetHeadersResponseDataHeader,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetHeadersResponseDataHeader {
    pub message: data::Header,
}

pub async fn get_block(
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
        Ok(resp) => Ok(data::IdentifiedData {
            root: root,
            data: resp.data.message,
        }),
        Err(_) => {
            let err = serde_json::from_str::<ResponseError>(&json_string)?;
            Err(Box::new(err))
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetBlockResponse {
    pub data: GetBlockResponseData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetBlockResponseData {
    pub message: data::Block,
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct StateFinalityCheckpoints {
//     pub previous_justified: Checkpoint,
//     pub current_justified: Checkpoint,
//     pub finalized: Checkpoint,
// }

// #[derive(Debug, Clone, Deserialize)]
// pub struct GetStateFinalityCheckpointsResponse {
//     pub data: StateFinalityCheckpoints,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct StateValidators {
//     pub
// }

// #[derive(Debug, Clone, Deserialize)]
// pub struct GetStateValidatorsResponse {
//     pub data: Vec<GetStateValidatorsResponseData>,
// }

// {"data":[{"index":"0","balance":"32044256882","status":"active_ongoing","validator":{"pubkey":"0x933ad9491b62059dd065b560d256d8957a8c402cc6e8d8ee7290ae11e8f7329267a8811c397529dac52ae1342ba58c95","withdrawal_credentials":"0x0100000000000000000000000d369bb49efa5100fd3b86a9f828c55da04d2d50","effective_balance":"32000000000","slashed":false,"activation_eligibility_epoch":"0","activation_epoch":"0","exit_epoch":"18446744073709551615","withdrawable_epoch":"18446744073709551615"}},{"index":"1","balance":"32009779801","status":"active_ongoing","validator":{"pubkey":"0xa1d1ad0714035353258038e964ae9675dc0252ee22cea896825c01458e1807bfad2f9969338798548d9858a571f7425c","withdrawal_credentials":"0x01000000000000000000000015f4b914a0ccd14333d850ff311d6dafbfbaa32b","effective_balance":"32000000000","slashed":false,"activation_eligibility_epoch":"0","activation_epoch":"0","exit_epoch":"18446744073709551615","withdrawable_epoch":"18446744073709551615"}},{"index":"2","balance":"32009712065","status":"active_ongoing","validator":{"pubkey":"0xb2ff4716ed345b05dd1dfc6a5a9fa70856d8c75dcc9e881dd2f766d5f891326f0d10e96f3a444ce6c912b69c22c6754d","withdrawal_credent"

// #[derive(Debug, Clone, Deserialize)]
// pub struct GetStateFinalityCheckpointsResponse {
//     pub data: GetStateFinalityCheckpointsResponseData,
// }

// #[derive(Debug, Clone, Deserialize)]
// pub struct GetStateFinalityCheckpointsResponseData {
//     pub previous_justified: Checkpoint,
//     pub current_justified: Checkpoint,
//     pub finalized: Checkpoint,
// }
