use crate::{
    block::Block,
    error::BlockError,
    hash::{Hash, hash_block_with_transactions_hash, hash_state, hash_transactions},
    state::State,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionOutcome {
    pub state: State,
    pub state_hash: Hash,
    pub transactions_hash: Hash,
    pub block_hash: Hash,
}

pub fn execute_block(
    previous_state: &State,
    previous_block_hash: Hash,
    block: &Block,
) -> Result<ExecutionOutcome, BlockError> {
    block.validate_structure()?;

    if block.block_height == 0 {
        return Err(BlockError::UnexpectedGenesisBlock);
    }
    if block.previous_block_hash != previous_block_hash {
        return Err(BlockError::PreviousBlockHashMismatch {
            expected: previous_block_hash,
            actual: block.previous_block_hash,
        });
    }

    let mut next_state = previous_state.clone();

    for (index, transfer) in block.transactions.iter().enumerate() {
        let delta = transfer
            .apply(&next_state)
            .map_err(|error| BlockError::Transaction { index, error })?;
        next_state.apply_delta(delta);
    }

    let state_hash = hash_state(&next_state);
    if block.state_hash != state_hash {
        return Err(BlockError::InvalidStateHash {
            expected: state_hash,
            actual: block.state_hash,
        });
    }

    let transactions_hash = hash_transactions(&block.transactions);
    let block_hash = hash_block_with_transactions_hash(block, transactions_hash);

    Ok(ExecutionOutcome {
        state: next_state,
        state_hash,
        transactions_hash,
        block_hash,
    })
}
