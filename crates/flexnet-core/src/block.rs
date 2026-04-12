use crate::{
    codec::{EncodeCanonical, append_fixed, append_u16_le, append_u128_le},
    constants::{CURRENT_CHAIN_ID, CURRENT_CHAIN_VERSION, MAX_TRANSACTIONS_PER_BLOCK},
    error::BlockError,
    hash::Hash,
    transfer::Transfer,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub chain_id: u16,
    pub chain_version: u16,
    pub block_height: u128,
    pub previous_block_hash: Hash,
    pub state_hash: Hash,
    pub transactions: Vec<Transfer>,
}

impl Block {
    pub fn new(
        chain_id: u16,
        chain_version: u16,
        block_height: u128,
        previous_block_hash: Hash,
        state_hash: Hash,
        transactions: Vec<Transfer>,
    ) -> Self {
        Self {
            chain_id,
            chain_version,
            block_height,
            previous_block_hash,
            state_hash,
            transactions,
        }
    }

    pub fn validate_structure(&self) -> Result<(), BlockError> {
        if self.chain_id != CURRENT_CHAIN_ID {
            return Err(BlockError::InvalidChainId);
        }
        if self.chain_version != CURRENT_CHAIN_VERSION {
            return Err(BlockError::InvalidChainVersion);
        }
        if self.transactions.len() > MAX_TRANSACTIONS_PER_BLOCK {
            return Err(BlockError::TooManyTransactions {
                count: self.transactions.len(),
                max: MAX_TRANSACTIONS_PER_BLOCK,
            });
        }
        if self.block_height == 0 && self.previous_block_hash != Hash::ZERO {
            return Err(BlockError::InvalidGenesisPreviousBlockHash);
        }

        Ok(())
    }

    pub fn hash_view<'a>(&'a self, transactions_hash: &'a Hash) -> BlockHashView<'a> {
        BlockHashView {
            block: self,
            transactions_hash,
        }
    }
}

pub struct BlockHashView<'a> {
    block: &'a Block,
    transactions_hash: &'a Hash,
}

impl EncodeCanonical for BlockHashView<'_> {
    fn encode_into(&self, out: &mut Vec<u8>) {
        append_u16_le(out, self.block.chain_id);
        append_u16_le(out, self.block.chain_version);
        append_u128_le(out, self.block.block_height);
        append_fixed(out, self.block.previous_block_hash.as_bytes());
        append_fixed(out, self.block.state_hash.as_bytes());
        append_fixed(out, self.transactions_hash.as_bytes());
    }
}
