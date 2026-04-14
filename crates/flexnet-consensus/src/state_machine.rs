mod on_precommit_received;
mod on_prevote_received;
mod on_proposal_received;
mod on_start_round;

use crate::{
    consensus_config::ConsensusConfig,
    lock::Lock,
    polka::Polka,
    proposal::Proposal,
    proposal_validator::ProposalValidator,
    state::State,
    state_input::StateInput,
    state_output::{RoundFailureReason, StateOutput},
    vote_set::VoteSet,
};
use std::marker::PhantomData;
use thiserror::Error;

pub struct StateMachine<P, V>
where
    P: Proposal,
    V: ProposalValidator<P>,
{
    config: ConsensusConfig,
    height: u128,
    round: u32,
    state: State<P>,
    lock: Option<Lock>,      // lock persists across rounds within the same height
    polka: Option<Polka<P>>, // next candidate proposal with enough justification
    proposal_validator: V,
    _phantom: PhantomData<P>,
}

#[derive(Error, Debug)]
pub enum StateMachineInitError {
    #[error("validator set is empty")]
    ValidatorSetEmpty,
    #[error("quorum is zero")]
    QuorumZero,
}

impl<P, V> StateMachine<P, V>
where
    P: Proposal,
    V: ProposalValidator<P>,
{
    pub fn new(
        height: u128,
        config: ConsensusConfig,
        proposal_validator: V,
    ) -> Result<Self, StateMachineInitError> {
        if config.validators.is_empty() {
            return Err(StateMachineInitError::ValidatorSetEmpty);
        }

        if config.quorum == 0 {
            return Err(StateMachineInitError::QuorumZero);
        }

        Ok(Self {
            config,
            height,
            round: 0,
            state: State::Propose {
                prevote_set: VoteSet::new(),
                precommit_set: VoteSet::new(),
            },
            lock: None,
            polka: None,
            proposal_validator,
            _phantom: PhantomData,
        })
    }

    pub fn is_older(&self, height: u128, round: u32) -> bool {
        (height, round) < (self.height, self.round)
    }

    pub fn is_same(&self, height: u128, round: u32) -> bool {
        (height, round) == (self.height, self.round)
    }

    pub fn is_newer(&self, height: u128, round: u32) -> bool {
        (height, round) > (self.height, self.round)
    }

    /// Accepts a new lock candidate if it is better than the current lock.
    pub(crate) fn accept_lock_candidate(&mut self, lock: Lock) {
        match &self.lock {
            Some(prior_lock) if lock.round > prior_lock.round => {
                self.lock = Some(lock);
            }
            None => {
                self.lock = Some(lock);
            }
            _ => {}
        }
    }

    /// Accepts a new polka candidate if it is better than the current polka.
    pub(crate) fn accept_polka_candidate(&mut self, polka: Polka<P>) {
        if polka.justification.height != self.height {
            return;
        }

        match &self.polka {
            Some(prior_polka) if polka.justification.round > prior_polka.justification.round => {
                self.polka = Some(polka);
            }
            None => {
                self.polka = Some(polka);
            }
            _ => {}
        }
    }

    pub fn step(&mut self, input: StateInput<P>) -> Vec<StateOutput<P>> {
        match input {
            StateInput::StartHeight { height } => {
                if self.is_older(height, 0) {
                    // height is behind; ignore input
                    return vec![];
                }

                // a new height starts; clear the lock and polka
                self.lock = None;
                self.polka = None;

                vec![StateOutput::StartRound { height, round: 0 }]
            }
            StateInput::StartRound { height, round } => self.on_start_round(height, round),
            StateInput::ProposalReceived {
                height,
                round,
                proposal,
                justification,
            } => self.on_proposal_received(height, round, proposal, justification),
            StateInput::PrevoteReceived {
                height,
                round,
                address,
                proposal_hash,
                signature,
            } => self.on_prevote_received(height, round, address, proposal_hash, signature),
            StateInput::PrecommitReceived {
                height,
                round,
                address,
                proposal_hash,
                signature,
            } => self.on_precommit_received(height, round, address, proposal_hash, signature),
            StateInput::RoundTimeout { height, round } => {
                if !self.is_same(height, round) {
                    // timeout is not for the current round; ignore input
                    return vec![];
                }

                vec![StateOutput::RoundFailure {
                    height,
                    round,
                    reason: RoundFailureReason::Timeout,
                }]
            }
        }
    }
}
