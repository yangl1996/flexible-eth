use crate::data::{self};
use serde::{Deserialize, Deserializer, Serialize};
use serde_aux::prelude::*;
use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::str::FromStr;

// ERROR HANDLING

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseError {
    pub code: usize,
    pub message: String,
}

impl fmt::Display for ResponseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl Error for ResponseError {}

// API TYPES

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiHeader {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub slot: usize,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub proposer_index: usize,
    pub parent_root: data::Root,
    pub state_root: data::Root,
    pub body_root: data::Root,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiBlock {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub slot: usize,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub proposer_index: usize,
    pub parent_root: data::Root,
    pub state_root: data::Root,
    pub body: ApiBlockBody,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiBlockBody {
    pub attestations: Vec<ApiAttestation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiAttestation {
    pub aggregation_bits: String,
    pub data: ApiAttestationData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiAttestationData {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub slot: usize,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub index: usize,
    pub beacon_block_root: data::Root,
    pub source: ApiCheckpoint,
    pub target: ApiCheckpoint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiCheckpoint {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub epoch: usize,
    pub root: data::Root,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiValidatorAssignment {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub index: usize,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub balance: usize,
    pub status: String,
    pub validator: ApiValidator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiValidator {
    pub pubkey: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub effective_balance: usize,
    pub slashed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiCommitteeAssignment {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub index: usize,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub slot: usize,
    #[serde(deserialize_with = "deserialize_vec_number_from_vec_string")]
    pub validators: Vec<usize>,
}

pub fn deserialize_vec_number_from_vec_string<'de, T, D>(
    deserializer: D,
) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr + serde::Deserialize<'de>,
    <T as FromStr>::Err: Display,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum VecStringOrInt<T> {
        VecString(Vec<String>),
        VecNumber(Vec<T>),
    }

    match VecStringOrInt::<T>::deserialize(deserializer)? {
        VecStringOrInt::VecString(s) => {
            let items: Result<Vec<_>, _> = s.iter().map(|e| e.parse::<T>()).collect();
            items.map_err(serde::de::Error::custom)
        }
        VecStringOrInt::VecNumber(i) => Ok(i),
    }
}

// CONVERSION API TYPES TO DATABASE TYPES

impl From<ApiHeader> for data::Header {
    fn from(api_header: ApiHeader) -> Self {
        data::Header {
            slot: api_header.slot,
            proposer_index: api_header.proposer_index,
            parent_root: api_header.parent_root,
            state_root: api_header.state_root,
            body_root: api_header.body_root,
        }
    }
}

impl From<ApiBlock> for data::Block {
    fn from(api_block: ApiBlock) -> Self {
        data::Block {
            slot: api_block.slot,
            proposer_index: api_block.proposer_index,
            parent_root: api_block.parent_root,
            state_root: api_block.state_root,
            body: api_block.body.into(),
        }
    }
}

impl From<ApiBlockBody> for data::BlockBody {
    fn from(api_block_body: ApiBlockBody) -> Self {
        data::BlockBody {
            attestations: api_block_body
                .attestations
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

impl From<ApiAttestation> for data::Attestation {
    fn from(api_attestation: ApiAttestation) -> Self {
        data::Attestation {
            aggregation_bits: api_attestation.aggregation_bits,
            data: api_attestation.data.into(),
        }
    }
}

impl From<ApiAttestationData> for data::AttestationData {
    fn from(api_attestation_data: ApiAttestationData) -> Self {
        data::AttestationData {
            slot: api_attestation_data.slot,
            index: api_attestation_data.index,
            beacon_block_root: api_attestation_data.beacon_block_root,
            source: api_attestation_data.source.into(),
            target: api_attestation_data.target.into(),
        }
    }
}

impl From<ApiCheckpoint> for data::Checkpoint {
    fn from(api_checkpoint: ApiCheckpoint) -> Self {
        data::Checkpoint {
            epoch: api_checkpoint.epoch,
            root: api_checkpoint.root,
        }
    }
}

impl From<ApiValidatorAssignment> for data::ValidatorAssignment {
    fn from(api_validator_assignment: ApiValidatorAssignment) -> Self {
        data::ValidatorAssignment {
            index: api_validator_assignment.index,
            balance: api_validator_assignment.balance,
            status: api_validator_assignment.status,
            validator: api_validator_assignment.validator.into(),
        }
    }
}

impl From<ApiValidator> for data::Validator {
    fn from(api_validator: ApiValidator) -> Self {
        data::Validator {
            pubkey: api_validator.pubkey,
            effective_balance: api_validator.effective_balance,
            slashed: api_validator.slashed,
        }
    }
}

impl From<ApiCommitteeAssignment> for data::CommitteeAssignment {
    fn from(api_committee_assignment: ApiCommitteeAssignment) -> Self {
        data::CommitteeAssignment {
            index: api_committee_assignment.index,
            slot: api_committee_assignment.slot,
            validators: api_committee_assignment.validators,
        }
    }
}

// HEADERS

#[allow(dead_code)]
pub async fn get_headers_by_slot(
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
        message: ApiHeader,
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
                    data: hdr.header.message.into(),
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

#[allow(dead_code)]
pub async fn get_blockroot_by_slot(
    client: &mut reqwest::Client,
    rpc_url: &str,
    slot: &usize,
) -> Result<Option<data::Root>, Box<dyn std::error::Error>> {
    #[derive(Debug, Clone, Deserialize)]
    struct GetBlockRootResponse {
        data: GetBlockRootResponseData,
    }

    #[derive(Debug, Clone, Deserialize)]
    struct GetBlockRootResponseData {
        root: data::Root,
    }

    let json_string = client
        .get(format!("{}/eth/v1/beacon/blocks/{}/root", rpc_url, slot))
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .await?
        .text()
        .await?;

    match serde_json::from_str::<GetBlockRootResponse>(&json_string) {
        Ok(resp) => Ok(Some(resp.data.root)),
        Err(_) => {
            let _err = serde_json::from_str::<ResponseError>(&json_string)?;
            Ok(None)
        }
    }
}

pub async fn get_block_by_blockroot(
    client: &mut reqwest::Client,
    rpc_url: &str,
    root: &data::Root,
) -> Result<Option<data::Block>, Box<dyn std::error::Error>> {
    #[derive(Debug, Clone, Deserialize)]
    struct GetBlockResponse {
        data: GetBlockResponseData,
    }

    #[derive(Debug, Clone, Deserialize)]
    struct GetBlockResponseData {
        message: ApiBlock,
    }

    let json_string = client
        .get(format!("{}/eth/v2/beacon/blocks/{}", rpc_url, root))
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .await?
        .text()
        .await?;

    match serde_json::from_str::<GetBlockResponse>(&json_string) {
        Ok(resp) => Ok(Some(resp.data.message.into())),
        Err(_) => {
            let _err = serde_json::from_str::<ResponseError>(&json_string)?;
            Ok(None)
        }
    }
}

// STATE

pub async fn get_stateroot_by_slot(
    client: &mut reqwest::Client,
    rpc_url: &str,
    slot: &usize,
) -> Result<data::Root, Box<dyn std::error::Error>> {
    #[derive(Debug, Clone, Deserialize)]
    struct GetStateRootResponse {
        data: GetStateRootResponseData,
    }

    #[derive(Debug, Clone, Deserialize)]
    struct GetStateRootResponseData {
        root: data::Root,
    }

    let json_string = client
        .get(format!("{}/eth/v1/beacon/states/{}/root", rpc_url, slot))
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .await?
        .text()
        .await?;

    match serde_json::from_str::<GetStateRootResponse>(&json_string) {
        Ok(resp) => Ok(resp.data.root),
        Err(_) => {
            let err = serde_json::from_str::<ResponseError>(&json_string)?;
            Err(Box::new(err))
        }
    }
}

#[allow(dead_code)]
pub async fn get_state_finality_checkpoints_by_slot(
    client: &mut reqwest::Client,
    rpc_url: &str,
    slot: &usize,
) -> Result<(data::Checkpoint, data::Checkpoint, data::Checkpoint), Box<dyn std::error::Error>> {
    #[derive(Debug, Clone, Deserialize)]
    struct GetStateFinalityCheckpointsResponse {
        data: GetStateFinalityCheckpointsResponseData,
    }

    #[derive(Debug, Clone, Deserialize)]
    struct GetStateFinalityCheckpointsResponseData {
        previous_justified: ApiCheckpoint,
        current_justified: ApiCheckpoint,
        finalized: ApiCheckpoint,
    }

    let json_string = client
        .get(format!(
            "{}/eth/v1/beacon/states/{}/finality_checkpoints",
            rpc_url, slot
        ))
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .await?
        .text()
        .await?;

    match serde_json::from_str::<GetStateFinalityCheckpointsResponse>(&json_string) {
        Ok(resp) => Ok((
            resp.data.previous_justified.into(),
            resp.data.current_justified.into(),
            resp.data.finalized.into(),
        )),
        Err(_) => {
            let err = serde_json::from_str::<ResponseError>(&json_string)?;
            Err(Box::new(err))
        }
    }
}

pub async fn get_state_committees_by_slot(
    client: &mut reqwest::Client,
    rpc_url: &str,
    slot: &usize,
) -> Result<Vec<data::CommitteeAssignment>, Box<dyn std::error::Error>> {
    #[derive(Debug, Clone, Deserialize)]
    struct GetStateCommitteesResponse {
        data: Vec<ApiCommitteeAssignment>,
    }

    let json_string = client
        .get(format!(
            "{}/eth/v1/beacon/states/{}/committees",
            rpc_url, slot
        ))
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .await?
        .text()
        .await?;

    match serde_json::from_str::<GetStateCommitteesResponse>(&json_string) {
        Ok(resp) => Ok(resp.data.into_iter().map(Into::into).collect()),
        Err(_) => {
            let err = serde_json::from_str::<ResponseError>(&json_string)?;
            Err(Box::new(err))
        }
    }
}
