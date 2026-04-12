use crate::{
    block::Block,
    chain_config::ChainConfig,
    chain_id::ChainId,
    chain_version::ChainVersion,
    hash::{Hash, compute_state_hash},
    state::{StateTransition, StateView},
    transaction::{TransactionExecutionError, TransactionVerificationError},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BlockVerifyError {
    #[error("invalid chain id: expected {expected}, got {actual}")]
    InvalidChainId { expected: ChainId, actual: ChainId },
    #[error("invalid chain version: expected {expected}, got {actual}")]
    InvalidChainVersion {
        expected: ChainVersion,
        actual: ChainVersion,
    },
    #[error("genesis block must use the zero previous block hash")]
    InvalidGenesisPreviousBlockHash,
    #[error("genesis block must not contain any transactions")]
    NonEmptyTransactionsInGenesisBlock,
    #[error("too many transactions: received {count}, maximum allowed is {max}")]
    TooManyTransactions { count: usize, max: usize },
    #[error("transaction at index {index} verification failed: {error}")]
    TxVerifyError {
        index: usize,
        error: TransactionVerificationError,
    },
}

pub fn verify_block_stateless(block: &Block, config: &ChainConfig) -> Result<(), BlockVerifyError> {
    if block.chain_id != config.chain_id {
        return Err(BlockVerifyError::InvalidChainId {
            expected: config.chain_id,
            actual: block.chain_id,
        });
    }

    if block.chain_version != config.chain_version {
        return Err(BlockVerifyError::InvalidChainVersion {
            expected: config.chain_version,
            actual: block.chain_version,
        });
    }

    if block.is_genesis() {
        if block.previous_block_hash != Hash::ZERO {
            return Err(BlockVerifyError::InvalidGenesisPreviousBlockHash);
        }

        if !block.transactions.is_empty() {
            return Err(BlockVerifyError::NonEmptyTransactionsInGenesisBlock);
        }
    }

    if block.transactions.len() > config.max_transactions_per_block {
        return Err(BlockVerifyError::TooManyTransactions {
            count: block.transactions.len(),
            max: config.max_transactions_per_block,
        });
    }

    for (index, transaction) in block.transactions.iter().enumerate() {
        transaction
            .verify_stateless(config)
            .map_err(|error| BlockVerifyError::TxVerifyError { index, error })?;
    }

    Ok(())
}

#[derive(Error, Debug)]
pub enum BlockExecuteError {
    #[error("block verification failed: {0}")]
    VerifyError(#[from] BlockVerifyError),
    #[error("transaction at index {index} execution failed: {error}")]
    TxExecuteError {
        index: usize,
        error: TransactionExecutionError,
    },
    #[error("invalid state hash: expected {expected}, got {actual}")]
    InvalidStateHash { expected: Hash, actual: Hash },
}

pub fn execute_block<S, T>(
    block: &Block,
    config: &ChainConfig,
    mut state: S,
    state_transition: T,
) -> Result<S, BlockExecuteError>
where
    S: StateView,
    T: StateTransition,
{
    verify_block_stateless(block, config)?;

    for (index, transaction) in block.transactions.iter().enumerate() {
        let delta = transaction
            .execute(config, &state)
            .map_err(|error| BlockExecuteError::TxExecuteError { index, error })?;
        state = state_transition.apply(state, delta);
    }

    let state_hash = compute_state_hash(&state);

    if block.state_hash != state_hash {
        return Err(BlockExecuteError::InvalidStateHash {
            expected: block.state_hash,
            actual: state_hash,
        });
    }

    Ok(state)
}
