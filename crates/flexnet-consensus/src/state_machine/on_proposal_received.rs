use crate::{
    justification::Justification, polka::Polka, proposal::Proposal,
    proposal_validator::ProposalValidator, state::State, state_machine::StateMachine,
    state_output::StateOutput,
};
use flexnet_chain::hash::Hash;

impl<P, V> StateMachine<P, V>
where
    P: Proposal,
    V: ProposalValidator<P>,
{
    pub fn on_proposal_received(
        &mut self,
        height: u128,
        round: u32,
        proposal: P,
        justification: Option<Justification>,
    ) -> Vec<StateOutput<P>> {
        if self.is_older(height, round) {
            // proposal is behind the current round; ignore input
            return vec![];
        }

        if self.is_newer(height, round) {
            // TODO: support out-of-order inputs later
            return vec![];
        }

        if !self.is_first_proposal_for_round() {
            // not the first proposal for the round; ignore input
            return vec![];
        }

        if !self
            .proposal_validator
            .validate(self.height, self.round, &proposal, &self.config)
        {
            // bad proposal; prevote nil
            return self.prevote(None);
        }

        let proposal_hash = proposal.hash();

        // try to accept a new polka candidate if a justification is provided.
        if let Some(justification) = &justification {
            self.accept_polka_candidate(Polka {
                proposal: proposal.clone(),
                proposal_hash,
                justification: justification.clone(),
            });
        }

        // try to unlock the lock if a justification is provided.
        self.try_unlock(justification.as_ref());

        if self.can_prevote_for_proposal(&proposal_hash) {
            // can prevote for the proposal; prevote for it
            return self.prevote(Some((proposal, proposal_hash)));
        }

        // cannot prevote for the proposal; prevote nil
        self.prevote(None)
    }

    /// Checks if the current state is the first proposal for the round.
    pub(crate) fn is_first_proposal_for_round(&self) -> bool {
        matches!(&self.state, State::Propose { .. })
    }

    /// Tries to unlock the lock if a justification is provided.
    pub(crate) fn try_unlock(&mut self, justification: Option<&Justification>) {
        let unlocked = match (&self.lock, justification) {
            (Some(lock), Some(justification)) => {
                justification.height == self.height
                    && justification.round < self.round
                    && justification.evidences.len() >= self.config.quorum
                    && lock.round < justification.round
            }
            _ => false,
        };

        if !unlocked {
            return;
        }

        self.lock = None;
    }

    /// Checks if the proposal hash is the same as the lock's proposal hash.
    pub(crate) fn can_prevote_for_proposal(&self, proposal_hash: &Hash) -> bool {
        match &self.lock {
            Some(lock) => &lock.proposal_hash == proposal_hash,
            None => true,
        }
    }

    /// Prevotes for a proposal or nil.
    pub(crate) fn prevote(&mut self, proposal_and_hash: Option<(P, Hash)>) -> Vec<StateOutput<P>> {
        match proposal_and_hash {
            Some((proposal, proposal_hash)) => {
                self.transite_to_prevote(Some(proposal), Some(proposal_hash))
            }
            None => self.transite_to_prevote(None, None),
        }
    }

    /// Transites to the prevote state.
    pub(crate) fn transite_to_prevote(
        &mut self,
        proposal: Option<P>,
        prevote: Option<Hash>,
    ) -> Vec<StateOutput<P>> {
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

        self.state = State::Prevote {
            proposal,
            prevote,
            prevote_set,
            precommit_set,
        };

        vec![StateOutput::Prevote {
            height: self.height,
            round: self.round,
            proposal_hash: prevote,
        }]
    }
}
