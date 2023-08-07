use std::collections::{HashMap, HashSet};

use crate::data;
use crate::utils;

#[derive(Debug)]
pub struct ConfirmationState {
    quorum: f64,
    last_processed_slot: usize,
    confirmed_tip_slot: usize,
    confirmation_targets: Vec<TargetConfirmationState>,
}

impl ConfirmationState {
    pub fn new(quorum: f64) -> Self {
        Self {
            quorum,
            last_processed_slot: 0,
            confirmed_tip_slot: 0,
            confirmation_targets: vec![],
        }
    }

    pub fn get_quorum(&self) -> f64 {
        return self.quorum;
    }

    pub fn process_block(&mut self, blk: &data::Block) -> Option<usize> {
        assert!(blk.slot > self.last_processed_slot);

        let mut acted = false;

        for t in self.confirmation_targets.iter_mut() {
            let r = t.process_block(blk);
            if r {
                if self.confirmed_tip_slot < t.finalized_slot {
                    self.confirmed_tip_slot = t.finalized_slot;
                    acted = true;
                }
            }
        }

        self.last_processed_slot = blk.slot;
        if acted {
            return Some(self.confirmed_tip_slot);
        } else {
            return None;
        }
    }

    pub fn register_first_block_of_epoch(&mut self, epoch: usize, ebb_root: data::Root, finalized_slot: usize, committees: &[data::CommitteeAssignment]) {
        // remove confirmation targets that are 2 or more epoches old, since they will not
        // receive any more votes
        self.confirmation_targets.retain(|s| s.epoch > epoch-2);
        let nc = TargetConfirmationState::new(epoch, ebb_root, finalized_slot, committees, self.quorum);
        self.confirmation_targets.push(nc);
    }
}

#[derive(Debug)]
pub struct TargetConfirmationState {
    epoch: usize,
    vote_target: data::Root,
    finalized_slot: usize,
    quorum: usize,
    committees: HashSet<(usize, usize)>,    // slot, index
    vote_aggregators: HashMap<(usize, usize), utils::AggregationBits>,  // slot, index to aggregation bits
    num_votes: usize,
    confirmed: bool,
}

impl TargetConfirmationState {
    pub fn new(epoch: usize, vote_target: data::Root, finalized_slot: usize, committees: &[data::CommitteeAssignment], quorum: f64) -> Self {
        // parse committee info
        let mut accounting_committees = HashSet::new();
        let mut accounting_validators = HashSet::new();
        let mut validators_n: usize = 0;
        for committee in committees {
            assert!(committee.slot >= utils::epoch_to_slot(epoch));
            assert!(committee.slot < utils::epoch_to_slot(epoch+1));
            let is_new = accounting_committees.insert((committee.slot, committee.index));
            assert!(is_new);

            for validator in &committee.validators {
                let is_new = accounting_validators.insert(validator);
                assert!(is_new);
                validators_n += 1;
            }
        }

        let validators_q = (validators_n as f64 * quorum).ceil() as usize;
        log::info!("EBB {}: Validator n={} q={}", vote_target, validators_n, validators_q);

        Self {
            epoch,
            vote_target,
            finalized_slot,
            quorum: validators_q,
            committees: accounting_committees,
            vote_aggregators: HashMap::new(),
            num_votes: 0,
            confirmed: false,
        }
    }

    pub fn process_block(&mut self, blk: &data::Block) -> bool {
        if self.confirmed {
            return false
        }
        for attestation in &blk.body.attestations {
            if attestation.data.slot < utils::epoch_to_slot(self.epoch) {
                // skip attestations from before the epoch in question
                continue;
            }
            if attestation.data.slot >= utils::epoch_to_slot(self.epoch+1) {
                // skip attestations from after the epoch in question
                continue;
            }
            if attestation.data.target.root != *self.vote_target {
                // skip attestations that are not for the target in question
                continue;
            }

            assert!(self.committees.contains(&(attestation.data.slot, attestation.data.index)));
            if !self.vote_aggregators.contains_key(&(attestation.data.slot, attestation.data.index)) {
                self.vote_aggregators.insert((attestation.data.slot, attestation.data.index), utils::AggregationBits::new_from_0xhex_str_zeroed(&attestation.aggregation_bits));
            }
            let votes_counted_aggregator = self.vote_aggregators.get_mut(&(attestation.data.slot, attestation.data.index)).unwrap();

            let new_aggregate_aggregator = utils::AggregationBits::new_from_0xhex_str(&attestation.aggregation_bits);
            let new_votes = votes_counted_aggregator.incorporate_delta(&new_aggregate_aggregator);
            self.num_votes += new_votes.count();
        }

        if self.num_votes >= self.quorum {
            if !self.confirmed {
                log::info!(
                    "{}: Quorum OK! votes={}, quorum={}",
                    self.vote_target,
                    self.num_votes,
                    self.quorum,
                );
            }
            self.confirmed = true;
            true
        } else {
            false
        }
    }
}

