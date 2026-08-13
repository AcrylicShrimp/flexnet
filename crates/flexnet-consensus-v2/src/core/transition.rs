use crate::core::{
    effect::CoreEffect, input::CoreInput, lock::CoreLock, proposal::Proposal,
    proposal_hint::ProposalHint, stage::CoreStage, state::CoreState,
    timeout_stage::CoreTimeoutStage, valid_proposal::ValidProposal, vote_target::VoteTarget,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TransitionError {
    #[error("round overflow")]
    RoundOverflow,
}

pub fn transition(
    state: CoreState,
    input: CoreInput,
) -> Result<(CoreState, Vec<CoreEffect>), TransitionError> {
    if matches!(&state.stage, CoreStage::Commit { .. }) {
        // already committed, so we ignore any further inputs for this height
        return Ok((state, vec![]));
    }

    if let CoreInput::PrecommitQuorumReady { round, target } = &input
        && let VoteTarget::Block(block_hash) = target
    {
        let round = *round;
        let block_hash = *block_hash;

        // we observed a precommit quorum for a block, so we can commit it
        return Ok((
            CoreState {
                round,
                stage: CoreStage::Commit { block_hash },
                ..state
            },
            vec![CoreEffect::Commit { round, block_hash }],
        ));
    }

    match state.stage {
        CoreStage::Propose => transition_propose(state, input),
        CoreStage::Prevote => transition_prevote(state, input),
        CoreStage::Precommit => transition_precommit(state, input),
        CoreStage::Commit { .. } => Ok((state, vec![])),
    }
}

fn transition_propose(
    mut state: CoreState,
    input: CoreInput,
) -> Result<(CoreState, Vec<CoreEffect>), TransitionError> {
    fn vote_for(state: CoreState, target: VoteTarget) -> (CoreState, Vec<CoreEffect>) {
        let round = state.round;

        (
            CoreState {
                stage: CoreStage::Prevote,
                ..state
            },
            vec![
                CoreEffect::EnteredPrevote { round },
                CoreEffect::VotePrevote { round, target },
            ],
        )
    }

    match input {
        CoreInput::ProposalReady { round, proposal } => {
            if round != state.round {
                return Ok((state, vec![]));
            }

            match (proposal, state.lock) {
                (Proposal::Fresh { block_hash }, None) => {
                    // can vote this block because we have no lock
                    Ok(vote_for(state, VoteTarget::Block(block_hash)))
                }
                (
                    Proposal::Fresh { block_hash },
                    Some(CoreLock {
                        block_hash: lock_block_hash,
                        ..
                    }),
                ) if block_hash == lock_block_hash => {
                    // can vote this block because it matches our lock
                    Ok(vote_for(state, VoteTarget::Block(block_hash)))
                }
                (Proposal::Justified { block_hash, .. }, None) => {
                    // can vote this block because it is justified and we have no lock
                    Ok(vote_for(state, VoteTarget::Block(block_hash)))
                }
                (
                    Proposal::Justified { block_hash, .. },
                    Some(CoreLock {
                        block_hash: lock_block_hash,
                        ..
                    }),
                ) if block_hash == lock_block_hash => {
                    // can vote this block because it matches our lock
                    Ok(vote_for(state, VoteTarget::Block(block_hash)))
                }
                (
                    Proposal::Justified {
                        valid_round,
                        block_hash,
                    },
                    Some(CoreLock { locked_round, .. }),
                ) if valid_round > locked_round => {
                    // can vote this block because it is justified and supersedes our lock
                    Ok(vote_for(state, VoteTarget::Block(block_hash)))
                }
                _ => {
                    // cannot vote for this block, so we vote nil
                    Ok(vote_for(state, VoteTarget::Nil))
                }
            }
        }
        CoreInput::PrevoteQuorumReady { round, target } => {
            if round != state.round {
                return Ok((state, vec![]));
            }

            if let VoteTarget::Block(block_hash) = target {
                update_valid_proposal(&mut state, ValidProposal { round, block_hash });
            }

            // block and prevote quorum are ready before a proposal input has driven
            // this round forward; record the valid value and wait for the proposal
            Ok((state, vec![]))
        }
        CoreInput::PrecommitQuorumReady { .. } => {
            // precommit quorum for nil observed;
            // just ignore it and wait for timeout or any further inputs to drive the core forward
            Ok((state, vec![]))
        }
        CoreInput::TimedOut { round, stage } => {
            if round != state.round || !is_timeout_stage(&state.stage, stage) {
                return Ok((state, vec![]));
            }

            // timeout in propose stage, so we vote nil
            Ok(vote_for(state, VoteTarget::Nil))
        }
    }
}

fn transition_prevote(
    mut state: CoreState,
    input: CoreInput,
) -> Result<(CoreState, Vec<CoreEffect>), TransitionError> {
    fn vote_for(state: CoreState, target: VoteTarget) -> (CoreState, Vec<CoreEffect>) {
        let round = state.round;

        (
            CoreState {
                stage: CoreStage::Precommit,
                ..state
            },
            vec![
                CoreEffect::EnteredPrecommit { round },
                CoreEffect::VotePrecommit { round, target },
            ],
        )
    }

    match input {
        CoreInput::ProposalReady { .. } => {
            // proposal arrived after we already prevoted,
            // so we ignore it and wait for further inputs
            Ok((state, vec![]))
        }
        CoreInput::PrevoteQuorumReady { round, target } => {
            if round != state.round {
                return Ok((state, vec![]));
            }

            if let VoteTarget::Block(block_hash) = target {
                state.lock = Some(CoreLock {
                    locked_round: round,
                    block_hash,
                });
                update_valid_proposal(&mut state, ValidProposal { round, block_hash });
            }

            Ok(vote_for(state, target))
        }
        CoreInput::PrecommitQuorumReady { .. } => {
            // precommit quorum for nil observed;
            // just ignore it and wait for timeout or any further inputs to drive the core forward
            Ok((state, vec![]))
        }
        CoreInput::TimedOut { round, stage } => {
            if round != state.round || !is_timeout_stage(&state.stage, stage) {
                return Ok((state, vec![]));
            }

            // timeout in prevote stage, so we vote nil
            Ok(vote_for(state, VoteTarget::Nil))
        }
    }
}

fn transition_precommit(
    mut state: CoreState,
    input: CoreInput,
) -> Result<(CoreState, Vec<CoreEffect>), TransitionError> {
    match input {
        CoreInput::ProposalReady { .. } => {
            // proposal arrived after we already precommit,
            // so we ignore it and wait for further inputs
            Ok((state, vec![]))
        }
        CoreInput::PrevoteQuorumReady { round, target } => {
            if round != state.round {
                return Ok((state, vec![]));
            }

            if let VoteTarget::Block(block_hash) = target {
                update_valid_proposal(&mut state, ValidProposal { round, block_hash });
            }

            // prevote quorum observed in the precommit state; ignore it
            Ok((state, vec![]))
        }
        CoreInput::PrecommitQuorumReady { .. } => {
            // precommit quorum for nil observed;
            // just ignore it and wait for timeout or any further inputs to drive the core forward
            Ok((state, vec![]))
        }
        CoreInput::TimedOut { round, stage } => {
            if round != state.round || !is_timeout_stage(&state.stage, stage) {
                return Ok((state, vec![]));
            }

            let next_round = next_round(round)?;
            let proposal_hint = state
                .valid_proposal
                .as_ref()
                .map(valid_proposal_to_proposal_hint);

            Ok((
                CoreState {
                    stage: CoreStage::Propose,
                    round: next_round,
                    ..state
                },
                vec![CoreEffect::EnteredPropose {
                    round: next_round,
                    proposal_hint,
                }],
            ))
        }
    }
}

fn update_valid_proposal(state: &mut CoreState, new: ValidProposal) {
    match &mut state.valid_proposal {
        Some(valid_proposal) if valid_proposal.round < new.round => {
            *valid_proposal = new;
        }
        None => {
            state.valid_proposal = Some(new);
        }
        _ => {
            // do not update valid proposal if the new one is not higher than the current one
        }
    }
}

fn valid_proposal_to_proposal_hint(valid_proposal: &ValidProposal) -> ProposalHint {
    ProposalHint {
        valid_round: valid_proposal.round,
        block_hash: valid_proposal.block_hash,
    }
}

fn is_timeout_stage(stage: &CoreStage, timeout_stage: CoreTimeoutStage) -> bool {
    matches!(
        (stage, timeout_stage),
        (CoreStage::Propose, CoreTimeoutStage::Propose)
            | (CoreStage::Prevote, CoreTimeoutStage::Prevote)
            | (CoreStage::Precommit, CoreTimeoutStage::Precommit)
    )
}

fn next_round(round: u64) -> Result<u64, TransitionError> {
    round.checked_add(1).ok_or(TransitionError::RoundOverflow)
}

#[cfg(test)]
mod tests {
    use super::*;
    use flexnet_chain::hash::Hash;

    fn hash(byte: u8) -> Hash {
        Hash::new([byte; 32])
    }

    fn propose_state(round: u64) -> CoreState {
        CoreState {
            height: 10,
            round,
            stage: CoreStage::Propose,
            lock: None,
            valid_proposal: None,
        }
    }

    fn locked_propose_state(round: u64, locked_round: u64, block_hash: Hash) -> CoreState {
        CoreState {
            lock: Some(CoreLock {
                locked_round,
                block_hash,
            }),
            ..propose_state(round)
        }
    }

    fn prevote_state(round: u64) -> CoreState {
        CoreState {
            stage: CoreStage::Prevote,
            ..propose_state(round)
        }
    }

    fn precommit_state(round: u64) -> CoreState {
        CoreState {
            stage: CoreStage::Precommit,
            ..propose_state(round)
        }
    }

    #[test]
    fn propose_timeout_prevotes_nil_without_advancing_round() {
        let state = propose_state(3);

        let (next_state, effects) = transition(
            state,
            CoreInput::TimedOut {
                round: 3,
                stage: CoreTimeoutStage::Propose,
            },
        )
        .expect("transition should not fail");

        assert_eq!(next_state.round, 3);
        assert_eq!(next_state.stage, CoreStage::Prevote);
        assert_eq!(
            effects,
            vec![
                CoreEffect::EnteredPrevote { round: 3 },
                CoreEffect::VotePrevote {
                    round: 3,
                    target: VoteTarget::Nil
                }
            ]
        );
    }

    #[test]
    fn fresh_proposal_without_lock_prevotes_the_block() {
        let block_hash = hash(1);
        let state = propose_state(0);

        let (next_state, effects) = transition(
            state,
            CoreInput::ProposalReady {
                round: 0,
                proposal: Proposal::Fresh { block_hash },
            },
        )
        .expect("transition should not fail");

        assert_eq!(next_state.stage, CoreStage::Prevote);
        assert_eq!(
            effects,
            vec![
                CoreEffect::EnteredPrevote { round: 0 },
                CoreEffect::VotePrevote {
                    round: 0,
                    target: VoteTarget::Block(block_hash)
                }
            ]
        );
    }

    #[test]
    fn proposal_matching_existing_lock_prevotes_the_locked_block() {
        let block_hash = hash(1);
        let state = locked_propose_state(5, 3, block_hash);

        let (next_state, effects) = transition(
            state,
            CoreInput::ProposalReady {
                round: 5,
                proposal: Proposal::Fresh { block_hash },
            },
        )
        .expect("transition should not fail");

        assert_eq!(next_state.stage, CoreStage::Prevote);
        assert_eq!(
            next_state.lock,
            Some(CoreLock {
                locked_round: 3,
                block_hash,
            })
        );
        assert_eq!(
            effects,
            vec![
                CoreEffect::EnteredPrevote { round: 5 },
                CoreEffect::VotePrevote {
                    round: 5,
                    target: VoteTarget::Block(block_hash),
                },
            ]
        );
    }

    #[test]
    fn conflicting_fresh_proposal_does_not_supersede_lock() {
        let locked_block_hash = hash(1);
        let proposed_block_hash = hash(2);
        let state = locked_propose_state(5, 3, locked_block_hash);

        let (next_state, effects) = transition(
            state,
            CoreInput::ProposalReady {
                round: 5,
                proposal: Proposal::Fresh {
                    block_hash: proposed_block_hash,
                },
            },
        )
        .expect("transition should not fail");

        assert_eq!(
            next_state
                .lock
                .expect("lock should be preserved")
                .block_hash,
            locked_block_hash
        );
        assert_eq!(next_state.stage, CoreStage::Prevote);
        assert_eq!(
            effects,
            vec![
                CoreEffect::EnteredPrevote { round: 5 },
                CoreEffect::VotePrevote {
                    round: 5,
                    target: VoteTarget::Nil
                }
            ]
        );
    }

    #[test]
    fn conflicting_justified_proposal_does_not_supersede_lock_without_newer_valid_round() {
        let locked_block_hash = hash(1);
        let proposed_block_hash = hash(2);

        for valid_round in [2, 3] {
            let state = locked_propose_state(5, 3, locked_block_hash);

            let (next_state, effects) = transition(
                state,
                CoreInput::ProposalReady {
                    round: 5,
                    proposal: Proposal::Justified {
                        valid_round,
                        block_hash: proposed_block_hash,
                    },
                },
            )
            .expect("transition should not fail");

            assert_eq!(next_state.stage, CoreStage::Prevote);
            assert_eq!(
                next_state.lock,
                Some(CoreLock {
                    locked_round: 3,
                    block_hash: locked_block_hash,
                })
            );
            assert_eq!(
                effects,
                vec![
                    CoreEffect::EnteredPrevote { round: 5 },
                    CoreEffect::VotePrevote {
                        round: 5,
                        target: VoteTarget::Nil,
                    },
                ]
            );
        }
    }

    #[test]
    fn justified_proposal_with_newer_valid_round_can_supersede_lock_for_prevote() {
        let locked_block_hash = hash(1);
        let proposed_block_hash = hash(2);
        let state = locked_propose_state(5, 3, locked_block_hash);

        let (next_state, effects) = transition(
            state,
            CoreInput::ProposalReady {
                round: 5,
                proposal: Proposal::Justified {
                    valid_round: 4,
                    block_hash: proposed_block_hash,
                },
            },
        )
        .expect("transition should not fail");

        assert_eq!(
            next_state
                .lock
                .expect("prevoting must not update the lock")
                .block_hash,
            locked_block_hash
        );
        assert_eq!(next_state.stage, CoreStage::Prevote);
        assert_eq!(
            effects,
            vec![
                CoreEffect::EnteredPrevote { round: 5 },
                CoreEffect::VotePrevote {
                    round: 5,
                    target: VoteTarget::Block(proposed_block_hash)
                }
            ]
        );
    }

    #[test]
    fn justified_proposal_replaces_lock_only_when_its_precommit_is_emitted() {
        let locked_block_hash = hash(1);
        let proposed_block_hash = hash(2);
        let state = locked_propose_state(3, 1, locked_block_hash);

        let (state, effects) = transition(
            state,
            CoreInput::ProposalReady {
                round: 3,
                proposal: Proposal::Justified {
                    valid_round: 2,
                    block_hash: proposed_block_hash,
                },
            },
        )
        .expect("transition should not fail");

        assert_eq!(
            state.lock,
            Some(CoreLock {
                locked_round: 1,
                block_hash: locked_block_hash,
            })
        );
        assert_eq!(
            effects,
            vec![
                CoreEffect::EnteredPrevote { round: 3 },
                CoreEffect::VotePrevote {
                    round: 3,
                    target: VoteTarget::Block(proposed_block_hash),
                },
            ]
        );

        let (state, effects) = transition(
            state,
            CoreInput::PrevoteQuorumReady {
                round: 3,
                target: VoteTarget::Block(proposed_block_hash),
            },
        )
        .expect("transition should not fail");

        assert_eq!(
            state.lock,
            Some(CoreLock {
                locked_round: 3,
                block_hash: proposed_block_hash,
            })
        );
        assert_eq!(
            effects,
            vec![
                CoreEffect::EnteredPrecommit { round: 3 },
                CoreEffect::VotePrecommit {
                    round: 3,
                    target: VoteTarget::Block(proposed_block_hash),
                },
            ]
        );
    }

    #[test]
    fn prevote_quorum_for_block_in_propose_updates_valid_without_voting_or_locking() {
        let block_hash = hash(1);
        let state = propose_state(2);

        let (next_state, effects) = transition(
            state,
            CoreInput::PrevoteQuorumReady {
                round: 2,
                target: VoteTarget::Block(block_hash),
            },
        )
        .expect("transition should not fail");

        assert!(matches!(next_state.stage, CoreStage::Propose));
        assert!(next_state.lock.is_none());
        assert_eq!(
            next_state.valid_proposal,
            Some(ValidProposal {
                round: 2,
                block_hash
            })
        );
        assert!(effects.is_empty());
    }

    #[test]
    fn prevote_quorum_in_propose_updates_valid_without_changing_existing_lock() {
        let locked_block_hash = hash(1);
        let quorum_block_hash = hash(2);
        let state = locked_propose_state(5, 3, locked_block_hash);

        let (next_state, effects) = transition(
            state,
            CoreInput::PrevoteQuorumReady {
                round: 5,
                target: VoteTarget::Block(quorum_block_hash),
            },
        )
        .expect("transition should not fail");

        assert_eq!(next_state.stage, CoreStage::Propose);
        assert_eq!(
            next_state.lock,
            Some(CoreLock {
                locked_round: 3,
                block_hash: locked_block_hash,
            })
        );
        assert_eq!(
            next_state.valid_proposal,
            Some(ValidProposal {
                round: 5,
                block_hash: quorum_block_hash,
            })
        );
        assert!(effects.is_empty());
    }

    #[test]
    fn proposal_after_early_prevote_quorum_can_complete_precommit_when_evidence_is_replayed() {
        let block_hash = hash(1);
        let state = propose_state(2);

        let (state, effects) = transition(
            state,
            CoreInput::PrevoteQuorumReady {
                round: 2,
                target: VoteTarget::Block(block_hash),
            },
        )
        .expect("transition should not fail");
        assert!(effects.is_empty());

        let (state, effects) = transition(
            state,
            CoreInput::ProposalReady {
                round: 2,
                proposal: Proposal::Fresh { block_hash },
            },
        )
        .expect("transition should not fail");
        assert_eq!(
            effects,
            vec![
                CoreEffect::EnteredPrevote { round: 2 },
                CoreEffect::VotePrevote {
                    round: 2,
                    target: VoteTarget::Block(block_hash),
                },
            ]
        );

        let (state, effects) = transition(
            state,
            CoreInput::PrevoteQuorumReady {
                round: 2,
                target: VoteTarget::Block(block_hash),
            },
        )
        .expect("transition should not fail");

        assert_eq!(state.stage, CoreStage::Precommit);
        assert_eq!(
            state.lock,
            Some(CoreLock {
                locked_round: 2,
                block_hash,
            })
        );
        assert_eq!(
            effects,
            vec![
                CoreEffect::EnteredPrecommit { round: 2 },
                CoreEffect::VotePrecommit {
                    round: 2,
                    target: VoteTarget::Block(block_hash),
                },
            ]
        );
    }

    #[test]
    fn proposal_for_another_round_cannot_change_state_or_emit_a_vote() {
        let locked_block_hash = hash(1);
        let state = locked_propose_state(5, 3, locked_block_hash);

        for proposal_round in [4, 6] {
            let (next_state, effects) = transition(
                state.clone(),
                CoreInput::ProposalReady {
                    round: proposal_round,
                    proposal: Proposal::Fresh {
                        block_hash: hash(2),
                    },
                },
            )
            .expect("transition should not fail");

            assert_eq!(next_state.round, 5);
            assert_eq!(next_state.stage, CoreStage::Propose);
            assert_eq!(
                next_state.lock,
                Some(CoreLock {
                    locked_round: 3,
                    block_hash: locked_block_hash,
                })
            );
            assert!(effects.is_empty());
        }
    }

    #[test]
    fn nil_precommit_quorum_in_propose_does_not_commit_or_advance_round() {
        let state = propose_state(7);

        let (next_state, effects) = transition(
            state,
            CoreInput::PrecommitQuorumReady {
                round: 7,
                target: VoteTarget::Nil,
            },
        )
        .expect("transition should not fail");

        assert_eq!(next_state.round, 7);
        assert!(matches!(next_state.stage, CoreStage::Propose));
        assert!(effects.is_empty());
    }

    #[test]
    fn block_precommit_quorum_commits_even_when_core_is_still_proposing() {
        let block_hash = hash(1);
        let state = propose_state(7);

        let (next_state, effects) = transition(
            state,
            CoreInput::PrecommitQuorumReady {
                round: 5,
                target: VoteTarget::Block(block_hash),
            },
        )
        .expect("transition should not fail");

        assert!(matches!(
            next_state.stage,
            CoreStage::Commit {
                block_hash: committed_block_hash
            } if committed_block_hash == block_hash
        ));
        assert_eq!(next_state.round, 5);
        assert_eq!(
            effects,
            vec![CoreEffect::Commit {
                round: 5,
                block_hash
            }]
        );
    }

    #[test]
    fn prevote_quorum_for_block_in_prevote_updates_valid_locks_and_precommits_block() {
        let block_hash = hash(1);
        let state = prevote_state(3);

        let (next_state, effects) = transition(
            state,
            CoreInput::PrevoteQuorumReady {
                round: 3,
                target: VoteTarget::Block(block_hash),
            },
        )
        .expect("transition should not fail");

        assert_eq!(next_state.stage, CoreStage::Precommit);
        assert_eq!(
            next_state.lock,
            Some(CoreLock {
                locked_round: 3,
                block_hash
            })
        );
        assert_eq!(
            next_state.valid_proposal,
            Some(ValidProposal {
                round: 3,
                block_hash
            })
        );
        assert_eq!(
            effects,
            vec![
                CoreEffect::EnteredPrecommit { round: 3 },
                CoreEffect::VotePrecommit {
                    round: 3,
                    target: VoteTarget::Block(block_hash)
                }
            ]
        );
    }

    #[test]
    fn prevote_quorum_for_different_block_still_locks_and_precommits_quorum_block() {
        let prevoted_block_hash = hash(1);
        let quorum_block_hash = hash(2);
        let state = propose_state(3);

        let (state, prevote_effects) = transition(
            state,
            CoreInput::ProposalReady {
                round: 3,
                proposal: Proposal::Fresh {
                    block_hash: prevoted_block_hash,
                },
            },
        )
        .expect("transition should not fail");

        assert_eq!(
            prevote_effects,
            vec![
                CoreEffect::EnteredPrevote { round: 3 },
                CoreEffect::VotePrevote {
                    round: 3,
                    target: VoteTarget::Block(prevoted_block_hash),
                },
            ]
        );

        let (next_state, effects) = transition(
            state,
            CoreInput::PrevoteQuorumReady {
                round: 3,
                target: VoteTarget::Block(quorum_block_hash),
            },
        )
        .expect("transition should not fail");

        assert_eq!(next_state.stage, CoreStage::Precommit);
        assert_eq!(
            next_state.lock,
            Some(CoreLock {
                locked_round: 3,
                block_hash: quorum_block_hash
            })
        );
        assert_eq!(
            next_state.valid_proposal,
            Some(ValidProposal {
                round: 3,
                block_hash: quorum_block_hash
            })
        );
        assert_eq!(
            effects,
            vec![
                CoreEffect::EnteredPrecommit { round: 3 },
                CoreEffect::VotePrecommit {
                    round: 3,
                    target: VoteTarget::Block(quorum_block_hash)
                }
            ]
        );
    }

    #[test]
    fn prevote_quorum_for_nil_in_prevote_precommits_nil_without_lock_or_valid_update() {
        let state = prevote_state(3);

        let (next_state, effects) = transition(
            state,
            CoreInput::PrevoteQuorumReady {
                round: 3,
                target: VoteTarget::Nil,
            },
        )
        .expect("transition should not fail");

        assert_eq!(next_state.stage, CoreStage::Precommit);
        assert!(next_state.lock.is_none());
        assert!(next_state.valid_proposal.is_none());
        assert_eq!(
            effects,
            vec![
                CoreEffect::EnteredPrecommit { round: 3 },
                CoreEffect::VotePrecommit {
                    round: 3,
                    target: VoteTarget::Nil
                }
            ]
        );
    }

    #[test]
    fn nil_prevote_quorum_preserves_existing_lock_and_valid_proposal() {
        let locked_block_hash = hash(1);
        let valid_block_hash = hash(2);
        let state = CoreState {
            stage: CoreStage::Prevote,
            lock: Some(CoreLock {
                locked_round: 1,
                block_hash: locked_block_hash,
            }),
            valid_proposal: Some(ValidProposal {
                round: 2,
                block_hash: valid_block_hash,
            }),
            ..propose_state(3)
        };

        let (next_state, effects) = transition(
            state,
            CoreInput::PrevoteQuorumReady {
                round: 3,
                target: VoteTarget::Nil,
            },
        )
        .expect("transition should not fail");

        assert_eq!(next_state.stage, CoreStage::Precommit);
        assert_eq!(
            next_state.lock,
            Some(CoreLock {
                locked_round: 1,
                block_hash: locked_block_hash,
            })
        );
        assert_eq!(
            next_state.valid_proposal,
            Some(ValidProposal {
                round: 2,
                block_hash: valid_block_hash,
            })
        );
        assert_eq!(
            effects,
            vec![
                CoreEffect::EnteredPrecommit { round: 3 },
                CoreEffect::VotePrecommit {
                    round: 3,
                    target: VoteTarget::Nil,
                },
            ]
        );
    }

    #[test]
    fn prevote_timeout_precommits_nil_without_lock_or_valid_update() {
        let state = prevote_state(3);

        let (next_state, effects) = transition(
            state,
            CoreInput::TimedOut {
                round: 3,
                stage: CoreTimeoutStage::Prevote,
            },
        )
        .expect("transition should not fail");

        assert_eq!(next_state.stage, CoreStage::Precommit);
        assert!(next_state.lock.is_none());
        assert!(next_state.valid_proposal.is_none());
        assert_eq!(
            effects,
            vec![
                CoreEffect::EnteredPrecommit { round: 3 },
                CoreEffect::VotePrecommit {
                    round: 3,
                    target: VoteTarget::Nil
                }
            ]
        );
    }

    #[test]
    fn prevote_timeout_preserves_existing_lock_and_valid_proposal() {
        let locked_block_hash = hash(1);
        let valid_block_hash = hash(2);
        let state = CoreState {
            stage: CoreStage::Prevote,
            lock: Some(CoreLock {
                locked_round: 1,
                block_hash: locked_block_hash,
            }),
            valid_proposal: Some(ValidProposal {
                round: 2,
                block_hash: valid_block_hash,
            }),
            ..propose_state(3)
        };

        let (next_state, effects) = transition(
            state,
            CoreInput::TimedOut {
                round: 3,
                stage: CoreTimeoutStage::Prevote,
            },
        )
        .expect("transition should not fail");

        assert_eq!(next_state.stage, CoreStage::Precommit);
        assert_eq!(
            next_state.lock,
            Some(CoreLock {
                locked_round: 1,
                block_hash: locked_block_hash,
            })
        );
        assert_eq!(
            next_state.valid_proposal,
            Some(ValidProposal {
                round: 2,
                block_hash: valid_block_hash,
            })
        );
        assert_eq!(
            effects,
            vec![
                CoreEffect::EnteredPrecommit { round: 3 },
                CoreEffect::VotePrecommit {
                    round: 3,
                    target: VoteTarget::Nil,
                },
            ]
        );
    }

    #[test]
    fn block_precommit_quorum_commits_even_when_core_is_in_prevote() {
        let block_hash = hash(1);
        let state = prevote_state(7);

        let (next_state, effects) = transition(
            state,
            CoreInput::PrecommitQuorumReady {
                round: 5,
                target: VoteTarget::Block(block_hash),
            },
        )
        .expect("transition should not fail");

        assert!(matches!(
            next_state.stage,
            CoreStage::Commit {
                block_hash: committed_block_hash
            } if committed_block_hash == block_hash
        ));
        assert_eq!(next_state.round, 5);
        assert_eq!(
            effects,
            vec![CoreEffect::Commit {
                round: 5,
                block_hash
            }]
        );
    }

    #[test]
    fn prevote_quorum_in_precommit_only_advances_valid_proposal() {
        let locked_block_hash = hash(1);
        let old_valid_block_hash = hash(2);
        let new_valid_block_hash = hash(3);
        let state = CoreState {
            stage: CoreStage::Precommit,
            lock: Some(CoreLock {
                locked_round: 1,
                block_hash: locked_block_hash,
            }),
            valid_proposal: Some(ValidProposal {
                round: 2,
                block_hash: old_valid_block_hash,
            }),
            ..propose_state(3)
        };

        let (next_state, effects) = transition(
            state,
            CoreInput::PrevoteQuorumReady {
                round: 3,
                target: VoteTarget::Block(new_valid_block_hash),
            },
        )
        .expect("transition should not fail");

        assert_eq!(next_state.stage, CoreStage::Precommit);
        assert_eq!(
            next_state.lock,
            Some(CoreLock {
                locked_round: 1,
                block_hash: locked_block_hash,
            })
        );
        assert_eq!(
            next_state.valid_proposal,
            Some(ValidProposal {
                round: 3,
                block_hash: new_valid_block_hash,
            })
        );
        assert!(effects.is_empty());
    }

    #[test]
    fn prevote_quorum_cannot_replace_valid_proposal_at_the_same_round() {
        let valid_block_hash = hash(1);
        let conflicting_block_hash = hash(2);
        let state = CoreState {
            stage: CoreStage::Precommit,
            valid_proposal: Some(ValidProposal {
                round: 3,
                block_hash: valid_block_hash,
            }),
            ..propose_state(3)
        };

        let (next_state, effects) = transition(
            state,
            CoreInput::PrevoteQuorumReady {
                round: 3,
                target: VoteTarget::Block(conflicting_block_hash),
            },
        )
        .expect("transition should not fail");

        assert_eq!(
            next_state.valid_proposal,
            Some(ValidProposal {
                round: 3,
                block_hash: valid_block_hash,
            })
        );
        assert!(effects.is_empty());
    }

    #[test]
    fn precommit_timeout_enters_the_next_round_without_a_proposal_hint() {
        let state = precommit_state(3);

        let (next_state, effects) = transition(
            state,
            CoreInput::TimedOut {
                round: 3,
                stage: CoreTimeoutStage::Precommit,
            },
        )
        .expect("transition should not fail");

        assert_eq!(next_state.round, 4);
        assert_eq!(next_state.stage, CoreStage::Propose);
        assert!(next_state.lock.is_none());
        assert!(next_state.valid_proposal.is_none());
        assert_eq!(
            effects,
            vec![CoreEffect::EnteredPropose {
                round: 4,
                proposal_hint: None,
            }]
        );
    }

    #[test]
    fn precommit_timeout_preserves_lock_and_uses_valid_proposal_as_the_next_round_hint() {
        let locked_block_hash = hash(1);
        let valid_block_hash = hash(2);
        let state = CoreState {
            stage: CoreStage::Precommit,
            lock: Some(CoreLock {
                locked_round: 1,
                block_hash: locked_block_hash,
            }),
            valid_proposal: Some(ValidProposal {
                round: 2,
                block_hash: valid_block_hash,
            }),
            ..propose_state(3)
        };

        let (next_state, effects) = transition(
            state,
            CoreInput::TimedOut {
                round: 3,
                stage: CoreTimeoutStage::Precommit,
            },
        )
        .expect("transition should not fail");

        assert_eq!(next_state.round, 4);
        assert_eq!(next_state.stage, CoreStage::Propose);
        assert_eq!(
            next_state.lock,
            Some(CoreLock {
                locked_round: 1,
                block_hash: locked_block_hash,
            })
        );
        assert_eq!(
            next_state.valid_proposal,
            Some(ValidProposal {
                round: 2,
                block_hash: valid_block_hash,
            })
        );
        assert_eq!(
            effects,
            vec![CoreEffect::EnteredPropose {
                round: 4,
                proposal_hint: Some(ProposalHint {
                    valid_round: 2,
                    block_hash: valid_block_hash,
                }),
            }]
        );
    }

    #[test]
    fn replayed_inputs_cannot_emit_a_second_vote_in_the_same_round() {
        let block_hash = hash(1);
        let conflicting_block_hash = hash(2);
        let mut state = propose_state(0);
        let mut all_effects = Vec::new();

        let inputs = [
            CoreInput::ProposalReady {
                round: 0,
                proposal: Proposal::Fresh { block_hash },
            },
            CoreInput::ProposalReady {
                round: 0,
                proposal: Proposal::Fresh { block_hash },
            },
            CoreInput::TimedOut {
                round: 0,
                stage: CoreTimeoutStage::Propose,
            },
            CoreInput::PrevoteQuorumReady {
                round: 0,
                target: VoteTarget::Block(block_hash),
            },
            CoreInput::PrevoteQuorumReady {
                round: 0,
                target: VoteTarget::Block(block_hash),
            },
            CoreInput::TimedOut {
                round: 0,
                stage: CoreTimeoutStage::Prevote,
            },
            CoreInput::TimedOut {
                round: 0,
                stage: CoreTimeoutStage::Precommit,
            },
            CoreInput::TimedOut {
                round: 0,
                stage: CoreTimeoutStage::Precommit,
            },
            CoreInput::ProposalReady {
                round: 1,
                proposal: Proposal::Fresh {
                    block_hash: conflicting_block_hash,
                },
            },
            CoreInput::ProposalReady {
                round: 1,
                proposal: Proposal::Fresh {
                    block_hash: conflicting_block_hash,
                },
            },
            CoreInput::PrevoteQuorumReady {
                round: 1,
                target: VoteTarget::Nil,
            },
            CoreInput::PrevoteQuorumReady {
                round: 1,
                target: VoteTarget::Nil,
            },
        ];

        for input in inputs {
            let (next_state, effects) =
                transition(state, input).expect("transition should not fail");
            state = next_state;
            all_effects.extend(effects);
        }

        let votes = all_effects
            .into_iter()
            .filter(|effect| {
                matches!(
                    effect,
                    CoreEffect::VotePrevote { .. } | CoreEffect::VotePrecommit { .. }
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(
            votes,
            vec![
                CoreEffect::VotePrevote {
                    round: 0,
                    target: VoteTarget::Block(block_hash),
                },
                CoreEffect::VotePrecommit {
                    round: 0,
                    target: VoteTarget::Block(block_hash),
                },
                CoreEffect::VotePrevote {
                    round: 1,
                    target: VoteTarget::Nil,
                },
                CoreEffect::VotePrecommit {
                    round: 1,
                    target: VoteTarget::Nil,
                },
            ]
        );
        assert_eq!(state.stage, CoreStage::Precommit);
        assert_eq!(
            state.lock,
            Some(CoreLock {
                locked_round: 0,
                block_hash,
            })
        );
    }

    #[test]
    fn stale_precommit_timeout_is_ignored() {
        let state = precommit_state(3);

        let (next_state, effects) = transition(
            state,
            CoreInput::TimedOut {
                round: 2,
                stage: CoreTimeoutStage::Precommit,
            },
        )
        .expect("transition should not fail");

        assert_eq!(next_state.round, 3);
        assert_eq!(next_state.stage, CoreStage::Precommit);
        assert!(effects.is_empty());
    }

    #[test]
    fn precommit_timeout_at_the_maximum_round_reports_overflow() {
        let state = precommit_state(u64::MAX);

        let result = transition(
            state,
            CoreInput::TimedOut {
                round: u64::MAX,
                stage: CoreTimeoutStage::Precommit,
            },
        );

        assert!(matches!(result, Err(TransitionError::RoundOverflow)));
    }

    #[test]
    fn block_precommit_quorum_commits_even_when_core_is_in_precommit() {
        let block_hash = hash(1);
        let state = precommit_state(7);

        let (next_state, effects) = transition(
            state,
            CoreInput::PrecommitQuorumReady {
                round: 5,
                target: VoteTarget::Block(block_hash),
            },
        )
        .expect("transition should not fail");

        assert_eq!(next_state.round, 5);
        assert_eq!(next_state.stage, CoreStage::Commit { block_hash });
        assert_eq!(
            effects,
            vec![CoreEffect::Commit {
                round: 5,
                block_hash,
            }]
        );
    }

    #[test]
    fn committed_core_ignores_every_further_input() {
        let committed_block_hash = hash(1);
        let state = CoreState {
            stage: CoreStage::Commit {
                block_hash: committed_block_hash,
            },
            ..propose_state(5)
        };

        let inputs = [
            CoreInput::ProposalReady {
                round: 5,
                proposal: Proposal::Fresh {
                    block_hash: hash(2),
                },
            },
            CoreInput::PrevoteQuorumReady {
                round: 5,
                target: VoteTarget::Block(hash(2)),
            },
            CoreInput::PrecommitQuorumReady {
                round: 6,
                target: VoteTarget::Block(hash(2)),
            },
            CoreInput::PrecommitQuorumReady {
                round: 5,
                target: VoteTarget::Nil,
            },
            CoreInput::TimedOut {
                round: 5,
                stage: CoreTimeoutStage::Precommit,
            },
        ];

        for input in inputs {
            let (next_state, effects) =
                transition(state.clone(), input).expect("transition should not fail");

            assert_eq!(next_state.height, state.height);
            assert_eq!(next_state.round, 5);
            assert_eq!(
                next_state.stage,
                CoreStage::Commit {
                    block_hash: committed_block_hash,
                }
            );
            assert_eq!(next_state.lock, state.lock);
            assert_eq!(next_state.valid_proposal, state.valid_proposal);
            assert!(effects.is_empty());
        }
    }

    #[test]
    fn bounded_input_sequences_preserve_local_safety_invariants() {
        #[derive(Clone, Default)]
        struct History {
            prevote_rounds: Vec<u64>,
            precommit_rounds: Vec<u64>,
            commit: Option<(u64, Hash)>,
        }

        fn candidate_inputs(state: &CoreState) -> Vec<CoreInput> {
            let block_a = hash(1);
            let block_b = hash(2);
            let mut rounds = vec![state.round];

            if let Some(stale_round) = state.round.checked_sub(1) {
                rounds.push(stale_round);
            }
            if let Some(future_round) = state.round.checked_add(1) {
                rounds.push(future_round);
            }

            let mut inputs = Vec::new();
            for round in rounds {
                inputs.push(CoreInput::ProposalReady {
                    round,
                    proposal: Proposal::Fresh {
                        block_hash: block_a,
                    },
                });
                inputs.push(CoreInput::ProposalReady {
                    round,
                    proposal: Proposal::Fresh {
                        block_hash: block_b,
                    },
                });
                if let Some(valid_round) = round.checked_sub(1) {
                    inputs.push(CoreInput::ProposalReady {
                        round,
                        proposal: Proposal::Justified {
                            valid_round,
                            block_hash: block_b,
                        },
                    });
                }
                for target in [
                    VoteTarget::Nil,
                    VoteTarget::Block(block_a),
                    VoteTarget::Block(block_b),
                ] {
                    inputs.push(CoreInput::PrevoteQuorumReady { round, target });
                    inputs.push(CoreInput::PrecommitQuorumReady { round, target });
                }
                for stage in [
                    CoreTimeoutStage::Propose,
                    CoreTimeoutStage::Prevote,
                    CoreTimeoutStage::Precommit,
                ] {
                    inputs.push(CoreInput::TimedOut { round, stage });
                }
            }

            inputs
        }

        fn assert_invariants(
            before: &CoreState,
            after: &CoreState,
            effects: &[CoreEffect],
            history: &mut History,
        ) {
            assert_eq!(after.height, before.height);

            if let Some(previous_lock) = before.lock {
                assert!(
                    after.lock.is_some(),
                    "a transition cleared an existing lock"
                );
                if let Some(next_lock) = after.lock
                    && next_lock != previous_lock
                {
                    assert!(next_lock.locked_round >= previous_lock.locked_round);
                }
            }

            if let Some(previous_valid) = before.valid_proposal {
                let next_valid = after
                    .valid_proposal
                    .expect("a transition cleared the valid proposal");
                assert!(next_valid.round >= previous_valid.round);
                if next_valid.round == previous_valid.round {
                    assert_eq!(next_valid.block_hash, previous_valid.block_hash);
                }
            }

            for effect in effects {
                match *effect {
                    CoreEffect::VotePrevote { round, .. } => {
                        assert_eq!(before.stage, CoreStage::Propose);
                        assert_eq!(round, before.round);
                        assert!(!history.prevote_rounds.contains(&round));
                        history.prevote_rounds.push(round);
                    }
                    CoreEffect::VotePrecommit { round, target } => {
                        assert_eq!(before.stage, CoreStage::Prevote);
                        assert_eq!(round, before.round);
                        assert!(!history.precommit_rounds.contains(&round));
                        history.precommit_rounds.push(round);

                        if let VoteTarget::Block(block_hash) = target {
                            assert_eq!(
                                after.lock,
                                Some(CoreLock {
                                    locked_round: round,
                                    block_hash,
                                })
                            );
                        } else {
                            assert_eq!(after.lock, before.lock);
                        }
                    }
                    CoreEffect::Commit { round, block_hash } => {
                        assert!(history.commit.is_none());
                        assert_eq!(after.round, round);
                        assert_eq!(after.stage, CoreStage::Commit { block_hash });
                        history.commit = Some((round, block_hash));
                    }
                    CoreEffect::EnteredPropose { .. }
                    | CoreEffect::EnteredPrevote { .. }
                    | CoreEffect::EnteredPrecommit { .. } => {}
                }
            }

            if before.lock != after.lock {
                let new_lock = after.lock.expect("a changed lock must remain non-nil");
                assert!(effects.iter().any(|effect| {
                    matches!(
                        effect,
                        CoreEffect::VotePrecommit {
                            round,
                            target: VoteTarget::Block(block_hash),
                        } if *round == new_lock.locked_round && *block_hash == new_lock.block_hash
                    )
                }));
            }

            if matches!(before.stage, CoreStage::Commit { .. }) {
                assert!(effects.is_empty());
                assert_eq!(after.round, before.round);
                assert_eq!(after.stage, before.stage);
                assert_eq!(after.lock, before.lock);
                assert_eq!(after.valid_proposal, before.valid_proposal);
            }
        }

        fn explore(state: CoreState, history: History, remaining_depth: usize) {
            if remaining_depth == 0 || matches!(state.stage, CoreStage::Commit { .. }) {
                return;
            }

            for input in candidate_inputs(&state) {
                let before = state.clone();
                let (after, effects) =
                    transition(state.clone(), input).expect("transition should not fail");
                let mut next_history = history.clone();

                assert_invariants(&before, &after, &effects, &mut next_history);
                explore(after, next_history, remaining_depth - 1);
            }
        }

        explore(propose_state(0), History::default(), 5);
    }
}
