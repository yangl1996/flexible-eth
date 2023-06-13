use ruint::Uint;
use serde::{Deserialize, Serialize};
use serde_aux::prelude::*;
use std::error::Error;
use std::fmt;
use std::str::FromStr;

type U256 = Uint<256, 4>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentifiedData<T> {
    pub root: U256,
    pub data: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub slot: usize,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub proposer_index: usize,
    pub parent_root: U256,
    pub state_root: U256,
    pub body_root: U256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetHeadersResponse {
    pub data: Vec<GetHeadersResponseData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetHeadersResponseData {
    pub root: U256,
    pub header: GetHeadersResponseDataHeader,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetHeadersResponseDataHeader {
    pub message: Header,
}

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
