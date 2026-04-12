use crate::{
    block::Block,
    chain_config::ChainConfig,
    hash::{Hash, compute_state_hash},
    state::StateView,
};

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

    pub fn into_genesis_block(self) -> (ChainConfig, S, Block) {
        let state_hash = compute_state_hash(&self.initial_state);
        let genesis_block = Block::new(
            self.config.chain_id,
            self.config.chain_version,
            0,
            Hash::ZERO,
            state_hash,
            Vec::new(),
        );

        (self.config, self.initial_state, genesis_block)
    }
}
