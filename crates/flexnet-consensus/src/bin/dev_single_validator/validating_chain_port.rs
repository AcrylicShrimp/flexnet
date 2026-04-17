use flexnet_chain::{block::Block, chain::Chain, state::WritableState};
use flexnet_consensus::ports::chain_port::ChainPort;
use parking_lot::Mutex;
use std::sync::Arc;

pub struct ValidatingChainPort<S>
where
    S: WritableState + Send + Sync,
{
    chain: Arc<Mutex<Chain<S>>>,
}

impl<S> ValidatingChainPort<S>
where
    S: WritableState + Send + Sync,
{
    pub fn new(chain: Arc<Mutex<Chain<S>>>) -> Self {
        Self { chain }
    }
}

impl<S> ChainPort for ValidatingChainPort<S>
where
    S: 'static + WritableState + Send + Sync,
{
    fn commit(&self, height: u128, block: Block) {
        if height != block.block_height {
            return;
        }

        let mut chain = self.chain.lock();

        match chain.append_block(block) {
            Ok(_) => {}
            Err(err) => {
                println!(
                    "ValidatingChainPort::commit: appending block failed: {:?}",
                    err
                );
            }
        }
    }
}
