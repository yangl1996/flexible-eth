// use ruint::{uint, Uint};
use crate::data;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

// ERROR HANDLING

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseError {
    #[serde(rename = "statusCode")]
    pub code: usize,
    pub message: String,
}

impl fmt::Display for ResponseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl Error for ResponseError {}

// HEADERS

pub async fn get_headers(
    client: &mut reqwest::Client,
    rpc_url: &str,
    slot: &usize,
) -> Result<Vec<data::IdentifiedData<data::Header>>, Box<dyn std::error::Error>> {
    #[derive(Debug, Clone, Deserialize)]
    struct GetHeadersResponse {
        data: Vec<GetHeadersResponseData>,
    }

    #[derive(Debug, Clone, Deserialize)]
    struct GetHeadersResponseData {
        root: data::Root,
        header: GetHeadersResponseDataHeader,
    }

    #[derive(Debug, Clone, Deserialize)]
    struct GetHeadersResponseDataHeader {
        message: data::Header,
    }

    let json_string = client
        .get(format!("{}/eth/v1/beacon/headers", rpc_url))
        .query(&[("slot", slot)])
        .header(reqwest::header::ACCEPT, "application/json")
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
                *slot != 0 || (headers.len() == 1 && headers[0].root == data::HEADER_GENESIS_ROOT)
            );
            Ok(headers)
        }
        Err(_) => {
            let err = serde_json::from_str::<ResponseError>(&json_string)?;
            Err(Box::new(err))
        }
    }
}

// BLOCKS

pub async fn get_block(
    client: &mut reqwest::Client,
    rpc_url: &str,
    root: &data::Root,
) -> Result<data::IdentifiedData<data::Block>, Box<dyn std::error::Error>> {
    #[derive(Debug, Clone, Deserialize)]
    struct GetBlockResponse {
        data: GetBlockResponseData,
    }

    #[derive(Debug, Clone, Deserialize)]
    struct GetBlockResponseData {
        message: data::Block,
    }

    let json_string = client
        .get(format!("{}/eth/v2/beacon/blocks/{}", rpc_url, root))
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .await?
        .text()
        .await?;

    match serde_json::from_str::<GetBlockResponse>(&json_string) {
        Ok(resp) => Ok(data::IdentifiedData {
            root: root.clone(),
            data: resp.data.message,
        }),
        Err(_) => {
            let err = serde_json::from_str::<ResponseError>(&json_string)?;
            Err(Box::new(err))
        }
    }
}

// STATE

pub async fn get_state_finality_checkpoints(
    client: &mut reqwest::Client,
    rpc_url: &str,
    root: &data::Root,
) -> Result<(data::Checkpoint, data::Checkpoint, data::Checkpoint), Box<dyn std::error::Error>> {
    #[derive(Debug, Clone, Deserialize)]
    struct GetStateFinalityCheckpointsResponse {
        data: GetStateFinalityCheckpointsResponseData,
    }

    #[derive(Debug, Clone, Deserialize)]
    struct GetStateFinalityCheckpointsResponseData {
        previous_justified: data::Checkpoint,
        current_justified: data::Checkpoint,
        finalized: data::Checkpoint,
    }

    let json_string = client
        .get(format!(
            "{}/eth/v1/beacon/states/{}/finality_checkpoints",
            rpc_url, root
        ))
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .await?
        .text()
        .await?;

    match serde_json::from_str::<GetStateFinalityCheckpointsResponse>(&json_string) {
        Ok(resp) => Ok((
            resp.data.previous_justified,
            resp.data.current_justified,
            resp.data.finalized,
        )),
        Err(_) => {
            let err = serde_json::from_str::<ResponseError>(&json_string)?;
            Err(Box::new(err))
        }
    }
}

pub async fn get_state_validators(
    client: &mut reqwest::Client,
    rpc_url: &str,
    root: &data::Root,
) -> Result<Vec<data::ValidatorAssignment>, Box<dyn std::error::Error>> {
    #[derive(Debug, Clone, Deserialize)]
    struct GetStateValidatorsResponse {
        data: Vec<data::ValidatorAssignment>,
    }

    let json_string = client
        .get(format!(
            "{}/eth/v1/beacon/states/{}/validators",
            rpc_url, root
        ))
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .await?
        .text()
        .await?;

    match serde_json::from_str::<GetStateValidatorsResponse>(&json_string) {
        Ok(resp) => Ok(resp.data),
        Err(_) => {
            let err = serde_json::from_str::<ResponseError>(&json_string)?;
            Err(Box::new(err))
        }
    }
}

pub async fn get_state_committees(
    client: &mut reqwest::Client,
    rpc_url: &str,
    root: &data::Root,
) -> Result<Vec<data::CommitteeAssignment>, Box<dyn std::error::Error>> {
    #[derive(Debug, Clone, Deserialize)]
    struct GetStateCommitteesResponse {
        data: Vec<data::CommitteeAssignment>,
    }

    let json_string = client
        .get(format!(
            "{}/eth/v1/beacon/states/{}/committees",
            rpc_url, root
        ))
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .await?
        .text()
        .await?;

    log::info!("Committee: {:?}", json_string);

    match serde_json::from_str::<GetStateCommitteesResponse>(&json_string) {
        Ok(resp) => Ok(resp.data),
        Err(_) => {
            let err = serde_json::from_str::<ResponseError>(&json_string)?;
            Err(Box::new(err))
        }
    }
}
