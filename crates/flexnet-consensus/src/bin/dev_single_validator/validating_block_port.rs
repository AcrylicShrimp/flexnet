use flexnet_chain::{
    block::Block, chain::Chain, hash::compute_state_hash_from_delta,
    rules::rule_block::compute_state_delta_from_transactions, state::WritableState,
};
use flexnet_consensus::ports::block_port::BlockPort;
use parking_lot::Mutex;
use std::sync::Arc;

pub struct ValidatingBlockPort<S>
where
    S: WritableState + Send + Sync,
{
    chain: Arc<Mutex<Chain<S>>>,
}

impl<S> ValidatingBlockPort<S>
where
    S: WritableState + Send + Sync,
{
    pub fn new(chain: Arc<Mutex<Chain<S>>>) -> Self {
        Self { chain }
    }
}

impl<S> BlockPort for ValidatingBlockPort<S>
where
    S: 'static + WritableState + Send + Sync,
{
    async fn next_candidate(&self, height: u128) -> Option<Block> {
        let chain = self.chain.lock();
        let next_height = chain.next_block_height()?;

        if next_height != height {
            return None;
        }

        let tx = vec![];
        let state = chain.state();
        let config = chain.config();
        let state_delta = compute_state_delta_from_transactions(state, &tx, config).ok()?;
        let state_hash = compute_state_hash_from_delta(state, &state_delta);

        Some(Block::new(
            config.chain_id,
            config.chain_version,
            next_height,
            chain.tip_block_hash(),
            state_hash,
            tx,
        ))
    }
}
