// use ruint::{uint, Uint};
use serde::{Deserialize, Deserializer, Serialize};
use serde_aux::prelude::*;
use std::fmt::Display;
use std::str::FromStr;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorAssignment {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub index: usize,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub balance: usize,
    pub status: String,
    pub validator: Validator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    pub pubkey: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub effective_balance: usize,
    pub slashed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitteeAssignment {
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
