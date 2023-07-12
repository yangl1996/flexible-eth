use std::time::{SystemTime, UNIX_EPOCH};
use libc;
use num_cpus;

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

pub fn is_prefix_of<T: PartialEq>(prefix: &[T], superfix: &[T]) -> bool {
    if prefix.len() > superfix.len() {
        return false;
    }
    for i in 0..prefix.len() {
        if prefix[i] != superfix[i] {
            return false;
        }
    }
    true
}

#[allow(dead_code)]
pub fn is_consistent_with<T: PartialEq>(vec1: &[T], vec2: &[T]) -> bool {
    is_prefix_of(vec1, vec2) || is_prefix_of(vec2, vec1)
}

#[derive(Debug)]
pub struct AggregationBits {
    bits: Vec<u8>,
}

impl AggregationBits {
    pub fn new_from_0xhex_str(bits: &str) -> Self {
        assert!(bits.starts_with("0x"));
        let mut bits = bits[2..].to_string();
        if bits.len() % 2 != 0 {
            bits = format!("0{}", bits);
        }
        let bits = (0..bits.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&bits[i..i + 2], 16).unwrap())
            .collect();
        Self { bits }
    }

    pub fn new_from_0xhex_str_zeroed(bits: &str) -> Self {
        let mut ret = Self::new_from_0xhex_str(bits);
        for i in 0..ret.bits.len() {
            ret.bits[i] = 0;
        }
        ret
    }

    pub fn incorporate_delta(&mut self, additional: &Self) -> Self {
        assert!(self.bits.len() == additional.bits.len());
        let mut delta = Self { bits: vec![0; self.bits.len()] };
        for i in 0..self.bits.len() {
            delta.bits[i] = additional.bits[i] & !(self.bits[i]);
            self.bits[i] = self.bits[i] | additional.bits[i];
        }
        delta
    }

    pub fn count(&self) -> usize {
        let mut cnt = 0;
        for val in &self.bits {
            cnt += val.count_ones() as usize;
        }
        cnt
    }
}

pub fn get_available_ram() -> usize {
    let pages = unsafe { libc::sysconf(libc::_SC_PAGESIZE) } as libc::c_ulong;
    let num_pages = unsafe { libc::sysconf(libc::_SC_PHYS_PAGES) } as libc::c_ulong;
    let available_memory = pages * num_pages;
    available_memory as usize
}

pub fn get_available_cpucores() -> usize {
    num_cpus::get()
}
