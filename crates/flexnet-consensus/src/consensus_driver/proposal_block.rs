use crate::proposal::Proposal;
use flexnet_chain::{
    block::Block,
    hash::{Hash, compute_block_hash},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProposalBlock(Block);

impl ProposalBlock {
    pub fn new(block: Block) -> Self {
        Self(block)
    }

    pub fn as_block(&self) -> &Block {
        &self.0
    }

    pub fn into_block(self) -> Block {
        self.0
    }
}

impl Proposal for ProposalBlock {
    fn hash(&self) -> Hash {
        compute_block_hash(self.as_block())
    }
}
