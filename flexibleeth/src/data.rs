// use ruint::{uint, Uint};
use serde::{Deserialize, Serialize};
use serde_aux::prelude::*;

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

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct StateValidators {
//     pub
// }

// #[derive(Debug, Clone, Deserialize)]
// pub struct GetStateValidatorsResponse {
//     pub data: Vec<GetStateValidatorsResponseData>,
// }

// {"data":[{"index":"0","balance":"32044256882","status":"active_ongoing","validator":{"pubkey":"0x933ad9491b62059dd065b560d256d8957a8c402cc6e8d8ee7290ae11e8f7329267a8811c397529dac52ae1342ba58c95","withdrawal_credentials":"0x0100000000000000000000000d369bb49efa5100fd3b86a9f828c55da04d2d50","effective_balance":"32000000000","slashed":false,"activation_eligibility_epoch":"0","activation_epoch":"0","exit_epoch":"18446744073709551615","withdrawable_epoch":"18446744073709551615"}},{"index":"1","balance":"32009779801","status":"active_ongoing","validator":{"pubkey":"0xa1d1ad0714035353258038e964ae9675dc0252ee22cea896825c01458e1807bfad2f9969338798548d9858a571f7425c","withdrawal_credentials":"0x01000000000000000000000015f4b914a0ccd14333d850ff311d6dafbfbaa32b","effective_balance":"32000000000","slashed":false,"activation_eligibility_epoch":"0","activation_epoch":"0","exit_epoch":"18446744073709551615","withdrawable_epoch":"18446744073709551615"}},{"index":"2","balance":"32009712065","status":"active_ongoing","validator":{"pubkey":"0xb2ff4716ed345b05dd1dfc6a5a9fa70856d8c75dcc9e881dd2f766d5f891326f0d10e96f3a444ce6c912b69c22c6754d","withdrawal_credent
