// use ruint::{uint, Uint};
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

// pub type Root = Uint<256, 4>;
pub type Root = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentifiedData<T> {
    pub root: Root,
    pub data: T,
}

// pub const HEADER_GENESIS_ROOT: Root =
// uint!(0x4d611d5b93fdab69013a7f0a2f961caca0c853f87cfe9595fe50038163079360_U256);
pub const HEADER_GENESIS_ROOT: &str =
    "0x4d611d5b93fdab69013a7f0a2f961caca0c853f87cfe9595fe50038163079360";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub slot: usize,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub proposer_index: usize,
    pub parent_root: Root,
    pub state_root: Root,
    pub body_root: Root,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetHeadersResponse {
    pub data: Vec<GetHeadersResponseData>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetHeadersResponseData {
    pub root: Root,
    pub header: GetHeadersResponseDataHeader,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetHeadersResponseDataHeader {
    pub message: Header,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub slot: usize,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub proposer_index: usize,
    pub parent_root: Root,
    pub state_root: Root,
    pub body: BlockBody,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockBody {
    pub attestations: Vec<Attestation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attestation {
    pub aggregation_bits: String,
    pub data: AttestationData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationData {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub slot: usize,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub index: usize,
    pub beacon_block_root: Root,
    pub source: Checkpoint,
    pub target: Checkpoint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub epoch: usize,
    pub root: Root,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetBlockResponse {
    pub data: GetBlockResponseData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetBlockResponseData {
    pub message: Block,
}
