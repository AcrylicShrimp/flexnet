use crate::{
    consensus_config::ConsensusConfig,
    lock::Lock,
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
    lock: Option<Lock>, // lock persists across rounds within the same height
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
            proposal_validator,
            _phantom: PhantomData,
        })
    }

    pub fn step(&mut self, input: StateInput<P>) -> Vec<StateOutput<P>> {
        match input {
            StateInput::StartHeight { height } => {
                if height <= self.height {
                    // height is behind or the same; ignore input
                    return vec![];
                }

                // a new height starts; clear the lock
                self.lock = None;

                vec![StateOutput::StartRound { height, round: 0 }]
            }
            StateInput::StartRound { height, round } => {
                if (height, round) <= (self.height, self.round) {
                    // the round is behind or the same; ignore input
                    return vec![];
                }

                self.height = height;
                self.round = round;
                self.state = State::Propose {
                    prevote_set: VoteSet::new(),
                    precommit_set: VoteSet::new(),
                };

                let validators_len = self.config.validators.len() as u128;
                let proposer_index = (((self.height % validators_len)
                    + (self.round as u128 % validators_len))
                    % validators_len) as usize;
                let proposer = self.config.validators[proposer_index];

                if proposer != self.config.address {
                    return vec![StateOutput::StartTimeout {
                        height: self.height,
                        round: self.round,
                        timeout_ms: self.config.round_timeout_ms,
                    }];
                }

                // this node is proposer
                vec![
                    StateOutput::StartTimeout {
                        height: self.height,
                        round: self.round,
                        timeout_ms: self.config.round_timeout_ms,
                    },
                    StateOutput::Propose {
                        height: self.height,
                        round: self.round,
                        address: self.config.address,
                    },
                ]
            }
            StateInput::ProposalReceived {
                height,
                round,
                proposal,
            } => {
                if (height, round) < (self.height, self.round) {
                    // proposal is behind the current round; ignore input
                    return vec![];
                }

                if (height, round) != (self.height, self.round) {
                    // TODO: support out-of-order inputs later
                    return vec![];
                }

                match &mut self.state {
                    State::Propose {
                        prevote_set,
                        precommit_set,
                    } => {
                        if !self
                            .proposal_validator
                            .validate(height, round, &proposal, &self.config)
                        {
                            // bad proposal; prevote nil
                            self.state = State::Prevote {
                                proposal: None,
                                prevote: None,
                                prevote_set: std::mem::take(prevote_set),
                                precommit_set: std::mem::take(precommit_set),
                            };
                            return vec![StateOutput::Prevote {
                                height,
                                round,
                                proposal_hash: None,
                            }];
                        }

                        let proposal_hash = proposal.hash();

                        match &self.lock {
                            Some(lock) if proposal_hash == lock.proposal_hash => {
                                // good proposal with the same lock; prevote for it
                                self.state = State::Prevote {
                                    proposal: Some(proposal),
                                    prevote: Some(proposal_hash),
                                    prevote_set: std::mem::take(prevote_set),
                                    precommit_set: std::mem::take(precommit_set),
                                };
                                vec![StateOutput::Prevote {
                                    height,
                                    round,
                                    proposal_hash: Some(proposal_hash),
                                }]
                            }
                            Some(_) => {
                                // good proposal with a different lock; prevote nil
                                self.state = State::Prevote {
                                    proposal: None, // invalidate proposal
                                    prevote: None,
                                    prevote_set: std::mem::take(prevote_set),
                                    precommit_set: std::mem::take(precommit_set),
                                };
                                vec![StateOutput::Prevote {
                                    height,
                                    round,
                                    proposal_hash: None,
                                }]
                            }
                            None => {
                                // good proposal without previous lock; prevote for it
                                self.state = State::Prevote {
                                    proposal: Some(proposal),
                                    prevote: Some(proposal_hash),
                                    prevote_set: std::mem::take(prevote_set),
                                    precommit_set: std::mem::take(precommit_set),
                                };
                                vec![StateOutput::Prevote {
                                    height,
                                    round,
                                    proposal_hash: Some(proposal_hash),
                                }]
                            }
                        }
                    }
                    _ => {
                        // already provoted; do not prevote again
                        vec![]
                    }
                }
            }
            StateInput::PrevoteReceived {
                height,
                round,
                address,
                proposal_hash,
            } => {
                if (height, round) < (self.height, self.round) {
                    // prevote is behind the current round; ignore input
                    return vec![];
                }

                if (height, round) != (self.height, self.round) {
                    // TODO: support out-of-order inputs later
                    return vec![];
                }

                match &mut self.state {
                    State::Propose { prevote_set, .. } => {
                        // proposal not yet received; just collect prevotes
                        prevote_set.add_vote(address, proposal_hash);
                        vec![]
                    }
                    State::Prevote {
                        proposal,
                        prevote,
                        prevote_set,
                        precommit_set,
                    } => {
                        // first collect prevotes
                        prevote_set.add_vote(address, proposal_hash);

                        let quorum_hash = match prevote_set.any_quorum_satisfied(self.config.quorum)
                        {
                            Some(hash) => hash,
                            None => {
                                // no quorum yet; wait for more prevotes
                                return vec![];
                            }
                        };

                        match (prevote, &quorum_hash) {
                            (prevote, quorum_hash) if prevote != quorum_hash => {
                                // conflict: prevote and quorum hash are different
                                // give up on this round and precommit for nil
                                self.state = State::Precommit {
                                    proposal: None, // invalidate proposal
                                    prevote: std::mem::take(prevote),
                                    precommit: None,
                                    precommit_set: std::mem::take(precommit_set),
                                };
                                vec![StateOutput::Precommit {
                                    height,
                                    round,
                                    proposal_hash: None,
                                }]
                            }
                            (Some(prevote), Some(quorum_hash)) => {
                                // happy path: prevote and quorum hash are the same
                                // lock for the proposal (if no prior lock exists) and precommit for the quorum hash
                                if self.lock.is_none() {
                                    self.lock = Some(Lock {
                                        round: self.round,
                                        proposal_hash: *quorum_hash,
                                    });
                                }

                                self.state = State::Precommit {
                                    proposal: std::mem::take(proposal),
                                    prevote: Some(*prevote),
                                    precommit: Some(*quorum_hash),
                                    precommit_set: std::mem::take(precommit_set),
                                };
                                vec![StateOutput::Precommit {
                                    height,
                                    round,
                                    proposal_hash: Some(*quorum_hash),
                                }]
                            }
                            (None, None) => {
                                // happy path for nil: prevote and quorum hash are both nil
                                // give up on this round and precommit for nil
                                self.state = State::Precommit {
                                    proposal: None, // invalidate proposal
                                    prevote: None,
                                    precommit: None,
                                    precommit_set: std::mem::take(precommit_set),
                                };
                                vec![StateOutput::Precommit {
                                    height,
                                    round,
                                    proposal_hash: None,
                                }]
                            }
                            (_, _) => {
                                unreachable!();
                            }
                        }
                    }
                    _ => {
                        // already prevoted; do not prevote again
                        vec![]
                    }
                }
            }
            StateInput::PrecommitReceived {
                height,
                round,
                address,
                proposal_hash,
            } => {
                if (height, round) < (self.height, self.round) {
                    // precommit is behind the current round; ignore input
                    return vec![];
                }

                if (height, round) != (self.height, self.round) {
                    // TODO: support out-of-order inputs later
                    return vec![];
                }

                match &mut self.state {
                    State::Propose {
                        prevote_set,
                        precommit_set,
                    } => {
                        // proposal not yet received; just collect precommits
                        precommit_set.add_vote(address, proposal_hash);
                        self.state = State::Propose {
                            prevote_set: std::mem::take(prevote_set),
                            precommit_set: std::mem::take(precommit_set),
                        };
                        vec![]
                    }
                    State::Prevote { precommit_set, .. } => {
                        // prevote quorum not yet satisfied; just collect precommits
                        precommit_set.add_vote(address, proposal_hash);
                        vec![]
                    }
                    State::Precommit {
                        proposal,
                        precommit,
                        precommit_set,
                        ..
                    } => {
                        // first collect precommits
                        precommit_set.add_vote(address, proposal_hash);

                        match precommit_set.any_quorum_satisfied(self.config.quorum) {
                            Some(quorum_hash) if &quorum_hash == precommit => {
                                match proposal.take() {
                                    Some(proposal) => {
                                        // happy path: proposal is valid and precommit is the same as the quorum hash
                                        self.state = State::Commit;
                                        vec![StateOutput::Commit {
                                            height: self.height,
                                            round: self.round,
                                            proposal,
                                        }]
                                    }
                                    None => {
                                        // invalid proposal: give up on this round
                                        self.state = State::Failure;
                                        vec![StateOutput::RoundFailure {
                                            height: self.height,
                                            round: self.round,
                                            reason: RoundFailureReason::Conflict,
                                        }]
                                    }
                                }
                            }
                            Some(_) => {
                                // conflict: precommit is not the same as the quorum hash
                                // give up on this round
                                self.state = State::Failure;
                                vec![StateOutput::RoundFailure {
                                    height: self.height,
                                    round: self.round,
                                    reason: RoundFailureReason::Conflict,
                                }]
                            }
                            None => {
                                // no quorum yet; wait for more precommits
                                vec![]
                            }
                        }
                    }
                    _ => {
                        // already committed or failed; do not precommit again
                        vec![]
                    }
                }
            }
            StateInput::RoundTimeout { height, round } => {
                if (height, round) != (self.height, self.round) {
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
