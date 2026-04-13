use crate::{consensus_config::ConsensusConfig, proposal::Proposal};

pub trait ProposalValidator<P>
where
    P: Proposal,
{
    fn validate(&self, height: u128, round: u32, proposal: &P, config: &ConsensusConfig) -> bool;
}
