use crate::{
    justification::{Evidence, Justification},
    lock::Lock,
    polka::Polka,
    proposal::Proposal,
    proposal_validator::ProposalValidator,
    state::State,
    state_machine::StateMachine,
    state_output::StateOutput,
};
use flexnet_chain::{address::Address, crypto::Signature, hash::Hash};

pub(crate) struct PrevoteQuorumEntry<P> {
    proposal: Option<P>,
    prevote: Option<Hash>,
    quorum_hash: Option<Hash>,
    evidences: Vec<Evidence>,
}

impl<P, V> StateMachine<P, V>
where
    P: Proposal,
    V: ProposalValidator<P>,
{
    pub fn on_prevote_received(
        &mut self,
        height: u128,
        round: u32,
        address: Address,
        proposal_hash: Option<Hash>,
        signature: Signature,
    ) -> Vec<StateOutput<P>> {
        if self.is_older(height, round) {
            // prevote is behind the current round; ignore input
            return vec![];
        }

        if self.is_newer(height, round) {
            // TODO: support out-of-order inputs later
            return vec![];
        }

        self.collect_prevote(address, proposal_hash, signature);

        let entry = match self.any_prevote_quorum_satisfied() {
            Some(entry) => entry,
            None => {
                // already precommitted or no quorum yet; ignore input
                return vec![];
            }
        };

        match (entry.proposal, entry.prevote, entry.quorum_hash) {
            (Some(proposal), Some(prevote), Some(quorum_hash)) if prevote == quorum_hash => {
                // a quorum is satisfied and the same as the prevote

                // good polka candidate
                self.accept_polka_candidate(Polka {
                    proposal: proposal.clone(),
                    proposal_hash: quorum_hash,
                    justification: Justification {
                        height: self.height,
                        round: self.round,
                        evidences: entry.evidences,
                    },
                });

                // good lock candidate
                self.accept_lock_candidate(Lock {
                    proposal_hash: quorum_hash,
                    round: self.round,
                });

                // precommit for the quorum hash
                self.precommit(Some(quorum_hash))
            }
            (Some(proposal), _, Some(quorum_hash)) if proposal.hash() == quorum_hash => {
                // a quorum is satisfied but not the same as the prevote

                // good polka candidate
                self.accept_polka_candidate(Polka {
                    proposal: proposal.clone(),
                    proposal_hash: quorum_hash,
                    justification: Justification {
                        height: self.height,
                        round: self.round,
                        evidences: entry.evidences,
                    },
                });

                // but still precommit for nil
                self.precommit(None)
            }
            _ => {
                // conflict; precommit for nil
                self.precommit(None)
            }
        }
    }

    /// Collects a prevote.
    pub(crate) fn collect_prevote(
        &mut self,
        address: Address,
        hash: Option<Hash>,
        signature: Signature,
    ) {
        let prevote_set = match &mut self.state {
            State::Propose { prevote_set, .. } => prevote_set,
            State::Prevote { prevote_set, .. } => prevote_set,
            State::Precommit { prevote_set, .. } => prevote_set,
            _ => {
                return;
            }
        };

        prevote_set.add_vote(address, hash, signature);
    }

    /// Checks if a quorum of prevotes is satisfied for the current round.
    pub(crate) fn any_prevote_quorum_satisfied(&self) -> Option<PrevoteQuorumEntry<P>> {
        match &self.state {
            State::Prevote {
                proposal,
                prevote,
                prevote_set,
                ..
            } => prevote_set
                .any_quorum_satisfied(self.consensus_config.quorum)
                .map(|(quorum_hash, evidences)| PrevoteQuorumEntry {
                    proposal: proposal.as_ref().cloned(),
                    prevote: prevote.as_ref().cloned(),
                    quorum_hash: quorum_hash.as_ref().cloned(),
                    evidences,
                }),
            _ => None,
        }
    }

    /// Precommits for a proposal or nil.
    pub(crate) fn precommit(&mut self, precommit: Option<Hash>) -> Vec<StateOutput<P>> {
        self.transite_to_precommit(precommit)
    }

    /// Transites to the precommit state.
    pub(crate) fn transite_to_precommit(&mut self, precommit: Option<Hash>) -> Vec<StateOutput<P>> {
        let (proposal, prevote) = match &mut self.state {
            State::Prevote {
                proposal, prevote, ..
            } => (proposal.take(), prevote.take()),
            State::Precommit {
                proposal, prevote, ..
            } => (proposal.take(), prevote.take()),
            _ => (None, None),
        };

        let (prevote_set, precommit_set) = match &mut self.state {
            State::Propose {
                prevote_set,
                precommit_set,
            } => (std::mem::take(prevote_set), std::mem::take(precommit_set)),
            State::Prevote {
                prevote_set,
                precommit_set,
                ..
            } => (std::mem::take(prevote_set), std::mem::take(precommit_set)),
            State::Precommit {
                prevote_set,
                precommit_set,
                ..
            } => (std::mem::take(prevote_set), std::mem::take(precommit_set)),
            State::Commit => (Default::default(), Default::default()),
            State::Failure => (Default::default(), Default::default()),
        };

        self.state = State::Precommit {
            proposal,
            prevote,
            prevote_set,
            precommit,
            precommit_set,
        };

        vec![StateOutput::Precommit {
            height: self.height,
            round: self.round,
            proposal_hash: precommit,
        }]
    }
}
