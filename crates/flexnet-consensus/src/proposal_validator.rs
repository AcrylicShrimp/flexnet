use crate::{consensus_config::ConsensusConfig, proposal::Proposal};
use flexnet_chain::chain_config::ChainConfig;

pub trait ProposalValidator<P>
where
    P: Proposal,
{
    fn validate(
        &self,
        height: u128,
        round: u32,
        proposal: &P,
        chain_config: &ChainConfig,
        consensus_config: &ConsensusConfig,
    ) -> bool;
}
