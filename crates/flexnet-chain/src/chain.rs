use crate::{
    block::Block,
    chain_config::ChainConfig,
    genesis::Genesis,
    hash::{Hash, compute_block_hash},
    rules::rule_block::{BlockExecuteError, execute_block},
    state::{StateTransition, StateView},
};
use thiserror::Error;

pub struct Chain<S>
where
    S: StateView,
{
    config: ChainConfig,
    state: S,
    tip_block: Block,
    tip_block_hash: Hash,
}

#[derive(Error, Debug)]
pub enum ChainAppendError {
    #[error("block height overflow")]
    BlockHeightOverflow,
    #[error("unexpected block height: expected {expected}, got {actual}")]
    UnexpectedBlockHeight { expected: u128, actual: u128 },
    #[error("previous block hash mismatch: expected {expected}, got {actual}")]
    PreviousBlockHashMismatch { expected: Hash, actual: Hash },
    #[error("block execution failed: {0}")]
    BlockExecutionError(#[from] BlockExecuteError),
}

impl<S> Chain<S>
where
    S: StateView,
{
    pub fn new(genesis: Genesis<S>) -> Self {
        let (config, state, tip_block) = genesis.into_genesis_block();
        let tip_block_hash = compute_block_hash(&tip_block);

        Self {
            config,
            state,
            tip_block,
            tip_block_hash,
        }
    }

    pub fn append_block<T>(
        self,
        block: Block,
        state_transition: T,
    ) -> Result<Self, ChainAppendError>
    where
        T: StateTransition,
    {
        let expected_height = match self.tip_block.block_height.checked_add(1) {
            Some(height) => height,
            None => {
                return Err(ChainAppendError::BlockHeightOverflow);
            }
        };

        if block.block_height != expected_height {
            return Err(ChainAppendError::UnexpectedBlockHeight {
                expected: expected_height,
                actual: block.block_height,
            });
        }

        if block.previous_block_hash != self.tip_block_hash {
            return Err(ChainAppendError::PreviousBlockHashMismatch {
                expected: self.tip_block_hash,
                actual: block.previous_block_hash,
            });
        }

        let state = execute_block(&block, &self.config, self.state, state_transition)?;
        let block_hash = compute_block_hash(&block);

        Ok(Self {
            config: self.config,
            state,
            tip_block: block,
            tip_block_hash: block_hash,
        })
    }
}
