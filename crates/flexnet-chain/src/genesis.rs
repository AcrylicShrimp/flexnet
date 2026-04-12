use crate::{
    block::Block,
    chain_config::ChainConfig,
    hash::{Hash, compute_state_hash},
    state::StateView,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Genesis<S>
where
    S: StateView,
{
    pub config: ChainConfig,
    pub initial_state: S,
}

impl<S> Genesis<S>
where
    S: StateView,
{
    pub fn new(config: ChainConfig, initial_state: S) -> Self {
        Self {
            config,
            initial_state,
        }
    }

    pub fn state_hash(&self) -> Hash {
        compute_state_hash(&self.initial_state)
    }

    pub fn block(&self) -> Block {
        Block::new(
            self.config.chain_id,
            self.config.chain_version,
            0,
            Hash::ZERO,
            self.state_hash(),
            Vec::new(),
        )
    }

    pub fn into_genesis_block(self) -> (ChainConfig, S, Block) {
        let block = self.block();
        (self.config, self.initial_state, block)
    }
}

#[cfg(test)]
mod tests {
    use super::Genesis;
    use crate::{
        account::Account,
        address::Address,
        chain_config::ChainConfig,
        chain_id::ChainId,
        chain_version::ChainVersion,
        hash::{Hash, compute_state_hash},
        state::State,
    };
    use std::collections::BTreeMap;

    #[test]
    fn genesis_builds_empty_genesis_block_with_state_hash() {
        let state = State::new(BTreeMap::from([(
            Address::new([1; 32]),
            Account::new(10, 0),
        )]));
        let genesis = Genesis::new(
            ChainConfig {
                chain_id: ChainId::new(1),
                chain_version: ChainVersion::new(1),
                max_transactions_per_block: 16,
            },
            state.clone(),
        );
        let block = genesis.block();

        assert_eq!(block.block_height, 0);
        assert_eq!(block.previous_block_hash, Hash::ZERO);
        assert!(block.transactions.is_empty());
        assert_eq!(block.state_hash, compute_state_hash(&state));
    }
}
