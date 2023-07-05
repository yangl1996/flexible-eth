use std::time::{SystemTime, UNIX_EPOCH};

pub const SLOTS_PER_EPOCH: usize = 32;
pub const SECONDS_PER_SLOT: usize = 12;

pub const GAP_LATEST_SLOT_NOW_SLOT_CANONICAL_CHAIN_STABILITY: usize = 5 * SLOTS_PER_EPOCH;

pub fn slot_to_epoch(slot: usize) -> usize {
    slot / SLOTS_PER_EPOCH
}

pub fn epoch_to_slot(epoch: usize) -> usize {
    epoch * SLOTS_PER_EPOCH
}

pub fn most_recent_epoch_boundary_slot_for_slot(slot: usize) -> usize {
    slot - (slot % SLOTS_PER_EPOCH)
}

pub fn is_epoch_boundary_slot(slot: usize) -> bool {
    slot % SLOTS_PER_EPOCH == 0
}

pub fn unixtime_to_slot(unixtime: u64) -> usize {
    unixtime as usize / SECONDS_PER_SLOT
}

pub fn get_unixtime() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
