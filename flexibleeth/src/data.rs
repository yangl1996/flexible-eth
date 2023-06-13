use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentifiedData<T> {
    pub root: String,
    pub data: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    pub slot: usize,
    pub proposer_index: usize,
    pub parent_root: String,
    pub state_root: String,
    pub body_root: String,
}
