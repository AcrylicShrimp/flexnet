use crate::core::{lock::CoreLock, stage::CoreStage, valid_proposal::ValidProposal};

#[derive(Debug, Clone, Hash)]
pub struct CoreState {
    pub height: u128,
    pub round: u64,
    pub stage: CoreStage,
    pub lock: Option<CoreLock>,
    pub valid_proposal: Option<ValidProposal>,
}
