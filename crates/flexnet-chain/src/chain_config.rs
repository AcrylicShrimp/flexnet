use crate::{chain_id::ChainId, chain_version::ChainVersion};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChainConfig {
    pub chain_id: ChainId,
    pub chain_version: ChainVersion,
    pub max_transactions_per_block: usize,
}

impl ChainConfig {
    pub fn new(
        chain_id: ChainId,
        chain_version: ChainVersion,
        max_transactions_per_block: usize,
    ) -> Self {
        Self {
            chain_id,
            chain_version,
            max_transactions_per_block,
        }
    }
}
