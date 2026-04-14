use crate::{
    proposal::Proposal, proposal_validator::ProposalValidator, state::State,
    state_machine::StateMachine, state_output::StateOutput, vote_set::VoteSet,
};
use flexnet_chain::address::Address;

impl<P, V> StateMachine<P, V>
where
    P: Proposal,
    V: ProposalValidator<P>,
{
    pub fn on_start_round(&mut self, height: u128, round: u32) -> Vec<StateOutput<P>> {
        if self.is_older(height, round) || self.is_same(height, round) {
            // the round is behind or the same; ignore input
            return vec![];
        }

        let mut outputs = self.transition_to_propose(height, round);

        if self.is_proposer() {
            // this node is proposer
            outputs.push(StateOutput::Propose {
                height: self.height,
                round: self.round,
                polka: self.polka.clone(),
            })
        }

        outputs
    }

    pub(crate) fn compute_proposer(&self) -> Address {
        let validators_len = self.config.validators.len() as u128;
        let proposer_index = (((self.height % validators_len)
            + (self.round as u128 % validators_len))
            % validators_len) as usize;
        self.config.validators[proposer_index]
    }

    pub(crate) fn is_proposer(&self) -> bool {
        self.compute_proposer() == self.config.address
    }

    /// Transitions to the propose state.
    pub(crate) fn transition_to_propose(
        &mut self,
        height: u128,
        round: u32,
    ) -> Vec<StateOutput<P>> {
        self.height = height;
        self.round = round;
        self.state = State::Propose {
            prevote_set: VoteSet::new(),
            precommit_set: VoteSet::new(),
        };

        vec![StateOutput::StartTimeout {
            height,
            round,
            timeout_ms: self.config.round_timeout_ms,
        }]
    }
}
