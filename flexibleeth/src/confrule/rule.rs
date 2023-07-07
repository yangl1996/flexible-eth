use std::collections::{HashMap, HashSet};

use crate::data;
use crate::utils;

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
            assert!(committee.slot >= slot_em1);
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
