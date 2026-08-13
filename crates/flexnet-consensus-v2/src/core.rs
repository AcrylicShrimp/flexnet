mod effect;
mod input;
mod lock;
mod proposal;
mod proposal_hint;
mod stage;
mod state;
mod timeout_stage;
mod transition;
mod valid_proposal;
mod vote_target;

use crate::core::{
    effect::CoreEffect, input::CoreInput, stage::CoreStage, state::CoreState,
    transition::transition,
};

/// Deterministic safety state machine for one consensus height.
///
/// `Core` is fixed to a single height. Height changes are handled by dropping
/// this instance and creating a new one after the chain or sync layer has
/// verified the previous height.
#[derive(Debug, Clone, Hash)]
pub struct Core {
    state: CoreState,
}

impl Core {
    pub fn new(height: u128) -> (Self, Vec<CoreEffect>) {
        (
            Self {
                state: CoreState {
                    height,
                    round: 0,
                    stage: CoreStage::Propose,
                    lock: None,
                    valid_proposal: None,
                },
            },
            vec![CoreEffect::EnteredPropose {
                round: 0,
                proposal_hint: None,
            }],
        )
    }

    pub fn position(&self) -> (u128, u64) {
        (self.state.height, self.state.round)
    }

    pub fn stage(&self) -> CoreStage {
        self.state.stage
    }

    pub fn apply(
        &mut self,
        input: CoreInput,
    ) -> Result<Vec<CoreEffect>, transition::TransitionError> {
        let (state, effects) = transition(self.state.clone(), input)?;
        self.state = state;
        Ok(effects)
    }
}
