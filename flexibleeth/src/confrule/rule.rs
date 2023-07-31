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

#[derive(Debug, Clone)]
pub struct ConfirmationRuleState {
    quorum: f64,
    tip_blkroot: data::Root,
    tip_slot: usize,
}

impl ConfirmationRuleState {
    pub fn new(quorum: f64, tip_blkroot: data::Root, tip_slot: usize) -> Self {
        Self {
            quorum,
            tip_blkroot,
            tip_slot,
        }
    }

    pub fn count_votes_for_confirmation(&mut self, slot_em1: usize, slot_e: usize, blkroot_em1: &data::Root, committees: &[data::CommitteeAssignment], blkroots: &[data::Root], blks: &[data::Block]) -> bool {
        assert!(blkroots.len() == blks.len());
        
        let mut accounting_committees = HashSet::new();
        let mut accounting_validators = HashSet::new();
        let mut validators_n: usize = 0;
        for committee in committees {
            assert!(committee.slot >= utils::most_recent_epoch_boundary_slot_for_slot(slot_em1));
            assert!(committee.slot < slot_e);

            let is_new = accounting_committees.insert((committee.slot, committee.index));
            assert!(is_new);

            for validator in &committee.validators {
                let is_new = accounting_validators.insert(validator);
                assert!(is_new);
                validators_n += 1;
            }
        }

        let validators_q = (validators_n as f64 * self.quorum).ceil() as usize;
        log::info!("{:?}: Validator n={} q={}", self, validators_n, validators_q);
        let mut validators_votes: usize = 0;

        let mut votes_counted_aggregators = HashMap::new();
        for blk_chain in blks.iter() {
            log::debug!("{:?}: Block: {:?}", self, blk_chain);

            for attestation in &blk_chain.body.attestations {
                if attestation.data.slot < slot_em1 {
                    // skip attestations from before the epoch in question
                    continue;
                }
                if attestation.data.slot >= slot_e {
                    // skip attestations from after the epoch in question
                    continue;
                }
                if attestation.data.target.root != *blkroot_em1 {
                    // skip attestations that are not for the target in question
                    continue;
                }

                assert!(accounting_committees.contains(&(attestation.data.slot, attestation.data.index)));
                if !votes_counted_aggregators.contains_key(&(attestation.data.slot, attestation.data.index)) {
                    votes_counted_aggregators.insert((attestation.data.slot, attestation.data.index), utils::AggregationBits::new_from_0xhex_str_zeroed(&attestation.aggregation_bits));
                }
                let votes_counted_aggregator = votes_counted_aggregators.get_mut(&(attestation.data.slot, attestation.data.index)).unwrap();

                let new_aggregate_aggregator = utils::AggregationBits::new_from_0xhex_str(&attestation.aggregation_bits);
                let new_votes = votes_counted_aggregator.incorporate_delta(&new_aggregate_aggregator);
                validators_votes += new_votes.count();
            }
        }

        log::info!("{:?}: votes={}", self, validators_votes);

        if validators_votes >= validators_q {
            log::info!(
                "{:?}: Quorum OK! v/n={}/{}>={}=q @ {}",
                self,
                validators_votes,
                validators_n,
                self.quorum,
                blkroot_em1,
            );

            true
        } else {
            log::info!(
                "{:?}: Quorum FAIL! v/n={}/{}<{}=q @ {}",
                self,
                validators_votes,
                validators_n,
                self.quorum,
                blkroot_em1,
            );

            false
        }
    }

    pub fn get_tip_blkroot(&self) -> data::Root {
        self.tip_blkroot.clone()
    }

    #[allow(dead_code)]
    pub fn get_tip_slot(&self) -> usize {
        self.tip_slot
    }

    pub fn update_tip(&mut self, tip_blkroot: data::Root, tip_slot: usize) {
        self.tip_blkroot = tip_blkroot;
        self.tip_slot = tip_slot;
    }
}
