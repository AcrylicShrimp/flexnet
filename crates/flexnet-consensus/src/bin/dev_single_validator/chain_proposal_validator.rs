use flexnet_chain::{
    chain::Chain, chain_config::ChainConfig, rules::rule_block::execute_block, state::WritableState,
};
use flexnet_consensus::{
    consensus_config::ConsensusConfig, consensus_driver::proposal_block::ProposalBlock,
    proposal_validator::ProposalValidator,
};
use parking_lot::Mutex;
use std::sync::Arc;

pub struct ChainProposalValidator<S>
where
    S: 'static + WritableState + Send + Sync,
{
    chain: Arc<Mutex<Chain<S>>>,
}

impl<S> ChainProposalValidator<S>
where
    S: 'static + WritableState + Send + Sync,
{
    pub fn new(chain: Arc<Mutex<Chain<S>>>) -> Self {
        Self { chain }
    }
}

impl<S> ProposalValidator<ProposalBlock> for ChainProposalValidator<S>
where
    S: 'static + WritableState + Send + Sync,
{
    fn validate(
        &self,
        height: u128,
        _round: u32,
        proposal: &ProposalBlock,
        chain_config: &ChainConfig,
        _consensus_config: &ConsensusConfig,
    ) -> bool {
        let chain = self.chain.lock();
        let expected_height = match chain.next_block_height() {
            Some(height) => height,
            None => {
                return false;
            }
        };

        if expected_height != height {
            return false;
        }

        execute_block(proposal.as_block(), chain_config, chain.state()).is_ok()
    }
}
