use helper_functions;
use types::{ beacon_state::*, config::{ Config, MainnetConfig }};
// use types::consts::*;
// use types::types::*;
use core::consts::ExpConst;
use helper_functions::math::*;
use types::primitives::*;
use helper_functions::beacon_state_accessors::*;
use helper_functions::beacon_state_mutators::*;
use helper_functions::predicates::*;
use crate::attestations::attestations::AttestableBlock;

pub trait StakeholderBlock<T>
where
    T: Config + ExpConst,
{
    fn get_base_reward(&self, index: ValidatorIndex) -> Gwei;
    fn get_attestation_deltas(&self) -> (Vec<Gwei>, Vec<Gwei>);
    fn process_rewards_and_penalties(&mut self);
}

impl<T> StakeholderBlock<T> for BeaconState<T>
where
    T: Config + ExpConst,
{
    fn get_base_reward(
        &self,
        index: ValidatorIndex
    ) -> Gwei {
        let total_balance = get_total_active_balance(&self).unwrap();
        let effective_balance = self.validators[index as usize].effective_balance;
        return (effective_balance * T::base_reward_factor() / integer_squareroot(total_balance) / T::base_rewards_per_epoch()) as Gwei;
    }

    fn get_attestation_deltas(
        &self
    ) -> (Vec<Gwei>, Vec<Gwei>) {
        let previous_epoch = self.get_previous_epoch();
        let total_balance = self.get_total_active_balance();
        let mut rewards = Vec::new();
        let mut penalties = Vec::new();
        for i in 0..(self.validators.len()) {
            rewards.push(0 as Gwei);
            penalties.push(0 as Gwei);
        }
        let mut eligible_validator_indices: Vec<ValidatorIndex> = Vec::new();

        for (index, v) in self.validators.iter().enumerate() {
            if is_active_validator(v, previous_epoch) || (v.slashed && previous_epoch + 1 < v.withdrawable_epoch) {
                eligible_validator_indices.push(index as ValidatorIndex);
            }
        }
        
        //# Micro-incentives for matching FFG source, FFG target, and head
        let matching_source_attestations = self.get_matching_source_attestations(previous_epoch);
        let matching_target_attestations = self.get_matching_target_attestations(previous_epoch);
        let matching_head_attestations = self.get_matching_head_attestations(previous_epoch);
        let vec = vec![matching_source_attestations, matching_target_attestations, matching_head_attestations];

        for attestations in vec.iter() {
            let unslashed_attesting_indices = self.get_unslashed_attesting_indices(*attestations);
            let attesting_balance = get_total_balance(self, &unslashed_attesting_indices).unwrap();

            for index in eligible_validator_indices.iter() {
                if unslashed_attesting_indices.contains(&index) {
                    rewards[*index as usize] += self.get_base_reward(*index) * attesting_balance; // total_balance
                }
                else {
                    penalties[*index as usize] += self.get_base_reward(*index);
                }
            }
        }

        //# Proposer and inclusion delay micro-rewards
        for index in self.get_unslashed_attesting_indices(matching_source_attestations).iter() {
            let attestation = matching_source_attestations.iter().fold(None, |min, x| match min {
                None => Some(x),
                Some(y) => Some(
                    if get_attesting_indices(self, &x.data, &x.aggregation_bits).unwrap().any(|&x| x == *index)
                    && x.inclusion_delay < y.inclusion_delay { x } else { y }),
            }).unwrap();

            let proposer_reward = (self.get_base_reward(*index) / T::proposer_reward_quotient()) as Gwei;
            rewards[attestation.proposer_index as usize] += proposer_reward;
            let max_attester_reward = self.get_base_reward(*index) - proposer_reward;
            rewards[*index as usize] += (max_attester_reward / attestation.inclusion_delay) as Gwei;
        }
        //# Inactivity penalty
        let finality_delay = previous_epoch - self.finalized_checkpoint.epoch;
        if finality_delay > T::min_epochs_to_inactivity_penalty() {
            let matching_target_attesting_indices = self.get_unslashed_attesting_indices(matching_target_attestations);
            for index in eligible_validator_indices {
                penalties[index as usize] += (T::base_rewards_per_epoch() * self.get_base_reward(index)) as Gwei;
                if !(matching_target_attesting_indices.contains(&index)) {
                    penalties[index as usize] += ((self.validators[index as usize].effective_balance * finality_delay) / T::inactivity_penalty_quotient()) as Gwei;
                }
            }
        }
        return (rewards, penalties);
    }

    fn process_rewards_and_penalties(
        &mut self
    ) {
        if get_current_epoch(&self) == T::genesis_epoch() {
            return;
        }
        let (rewards, penalties) = self.get_attestation_deltas();
        for index in 0..self.validators.len() {
            increase_balance(&mut self, index as u64, rewards[index]);
            decrease_balance(&mut self, index as u64, penalties[index]);
        }
    }
}

#[test]
fn test_base_reward() {
    assert_eq!(1,1);
    let mut bs: BeaconState<MainnetConfig> = BeaconState {
        ..BeaconState::default()
    };
    let mut index = 0;
    
}