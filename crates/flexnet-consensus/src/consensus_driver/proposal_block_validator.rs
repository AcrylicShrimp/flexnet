use crate::{
    consensus_config::ConsensusConfig, consensus_driver::proposal_block::ProposalBlock,
    proposal_validator::ProposalValidator,
};
use flexnet_chain::{chain_config::ChainConfig, rules::rule_block::verify_block_stateless};

pub struct ProposalBlockValidator;

impl ProposalValidator<ProposalBlock> for ProposalBlockValidator {
    fn validate(
        &self,
        height: u128,
        _round: u32,
        proposal: &ProposalBlock,
        chain_config: &ChainConfig,
        _consensus_config: &ConsensusConfig,
    ) -> bool {
        let block = proposal.as_block();

        if block.block_height != height {
            return false;
        }

        verify_block_stateless(proposal.as_block(), chain_config).is_ok()

        // TODO: verify the block against the current chain state
    }
}
