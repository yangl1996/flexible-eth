use serde::{Deserialize, Serialize};

pub type Root = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentifiedData<T> {
    pub root: Root,
    pub data: T,
}

pub const HEADER_GENESIS_ROOT: &str =
    "0x4d611d5b93fdab69013a7f0a2f961caca0c853f87cfe9595fe50038163079360";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    pub slot: usize,
    pub proposer_index: usize,
    pub parent_root: Root,
    pub state_root: Root,
    pub body_root: Root,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub slot: usize,
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
    pub slot: usize,
    pub index: usize,
    pub beacon_block_root: Root,
    pub source: Checkpoint,
    pub target: Checkpoint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub epoch: usize,
    pub root: Root,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorAssignment {
    pub index: usize,
    pub balance: usize,
    pub status: String,
    pub validator: Validator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    pub pubkey: String,
    pub effective_balance: usize,
    pub slashed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitteeAssignment {
    pub index: usize,
    pub slot: usize,
    pub validators: Vec<usize>,
}
