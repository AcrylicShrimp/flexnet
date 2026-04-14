use crate::{
    proposal::Proposal,
    proposal_validator::ProposalValidator,
    state::State,
    state_machine::StateMachine,
    state_output::{RoundFailureReason, StateOutput},
};
use flexnet_chain::{address::Address, crypto::Signature, hash::Hash};

pub(crate) struct PrecommitQuorumEntry<P> {
    proposal: Option<P>,
    precommit: Option<Hash>,
    quorum_hash: Option<Hash>,
}

impl<P, V> StateMachine<P, V>
where
    P: Proposal,
    V: ProposalValidator<P>,
{
    pub fn on_precommit_received(
        &mut self,
        height: u128,
        round: u32,
        address: Address,
        proposal_hash: Option<Hash>,
        signature: Signature,
    ) -> Vec<StateOutput<P>> {
        if self.is_older(height, round) {
            // precommit is behind the current round; ignore input
            return vec![];
        }

        if self.is_newer(height, round) {
            // TODO: support out-of-order inputs later
            return vec![];
        }

        self.collect_precommit(address, proposal_hash, signature);

        let entry = match self.any_precommit_quorum_satisfied() {
            Some(entry) => entry,
            None => {
                // already precommitted or no quorum yet; ignore input
                return vec![];
            }
        };

        match (entry.proposal, entry.precommit, entry.quorum_hash) {
            (Some(proposal), Some(precommit), Some(quorum_hash)) if precommit == quorum_hash => {
                // a quorum is satisfied and the same as the precommit
                // commit the proposal
                self.commit(proposal.clone())
            }
            _ => {
                // conflict; fail the round
                self.fail_round(RoundFailureReason::NoDecision)
            }
        }
    }

    pub(crate) fn collect_precommit(
        &mut self,
        address: Address,
        proposal_hash: Option<Hash>,
        signature: Signature,
    ) {
        let precommit_set = match &mut self.state {
            State::Propose { precommit_set, .. } => precommit_set,
            State::Prevote { precommit_set, .. } => precommit_set,
            State::Precommit { precommit_set, .. } => precommit_set,
            _ => {
                return;
            }
        };

        precommit_set.add_vote(address, proposal_hash, signature);
    }

    /// Checks if a quorum of precommits is satisfied for the current round.
    pub(crate) fn any_precommit_quorum_satisfied(&self) -> Option<PrecommitQuorumEntry<P>> {
        match &self.state {
            State::Precommit {
                proposal,
                precommit,
                precommit_set,
                ..
            } => precommit_set
                .any_quorum_satisfied(self.consensus_config.quorum)
                .map(|(quorum_hash, _)| PrecommitQuorumEntry {
                    proposal: proposal.as_ref().cloned(),
                    precommit: precommit.as_ref().cloned(),
                    quorum_hash: quorum_hash.as_ref().cloned(),
                }),
            _ => None,
        }
    }

    /// Commits a proposal.
    pub(crate) fn commit(&mut self, proposal: P) -> Vec<StateOutput<P>> {
        self.transition_to_commit(proposal)
    }

    /// Transitions to the commit state.
    pub(crate) fn transition_to_commit(&mut self, proposal: P) -> Vec<StateOutput<P>> {
        self.state = State::Commit;
        vec![StateOutput::Commit {
            height: self.height,
            round: self.round,
            proposal,
        }]
    }

    /// Fails the round.
    pub(crate) fn fail_round(&mut self, reason: RoundFailureReason) -> Vec<StateOutput<P>> {
        self.transition_to_failure(reason)
    }

    /// Transitions to the failure state.
    pub(crate) fn transition_to_failure(
        &mut self,
        reason: RoundFailureReason,
    ) -> Vec<StateOutput<P>> {
        self.state = State::Failure;
        vec![StateOutput::RoundFailure {
            height: self.height,
            round: self.round,
            reason,
        }]
    }
}
