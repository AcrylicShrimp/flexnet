use flexnet_chain::{block::Block, chain::Chain, state::WritableState};
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

        let config = chain.config();
        let tip_block = chain.tip_block();

        Some(Block::new(
            config.chain_id,
            config.chain_version,
            next_height,
            tip_block.previous_block_hash,
            tip_block.state_hash,
            Vec::new(),
        ))
    }
}
