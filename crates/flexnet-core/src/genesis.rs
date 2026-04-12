use std::collections::BTreeMap;

use serde::Deserialize;

use crate::{
    account::Account,
    address::Address,
    block::Block,
    constants::{CURRENT_CHAIN_ID, CURRENT_CHAIN_VERSION},
    error::GenesisError,
    hash::{Hash, hash_block, hash_state},
    state::State,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Genesis {
    pub chain_id: u16,
    pub chain_version: u16,
    pub block_height: u128,
    pub previous_block_hash: Hash,
    pub state: State,
}

impl Genesis {
    pub fn from_json_str(input: &str) -> Result<Self, GenesisError> {
        let raw: GenesisJson = serde_json::from_str(input)?;
        let genesis = Self {
            chain_id: raw.chain_id,
            chain_version: raw.chain_version,
            block_height: raw.block_height,
            previous_block_hash: raw.previous_block_hash,
            state: State::new(raw.state.accounts),
        };

        genesis.validate()?;
        Ok(genesis)
    }

    pub fn validate(&self) -> Result<(), GenesisError> {
        if self.chain_id != CURRENT_CHAIN_ID {
            return Err(GenesisError::InvalidChainId);
        }
        if self.chain_version != CURRENT_CHAIN_VERSION {
            return Err(GenesisError::InvalidChainVersion);
        }
        if self.block_height != 0 {
            return Err(GenesisError::InvalidBlockHeight);
        }
        if self.previous_block_hash != Hash::ZERO {
            return Err(GenesisError::InvalidPreviousBlockHash);
        }

        Ok(())
    }

    pub fn state_hash(&self) -> Hash {
        hash_state(&self.state)
    }

    pub fn block(&self) -> Block {
        Block::new(
            self.chain_id,
            self.chain_version,
            self.block_height,
            self.previous_block_hash,
            self.state_hash(),
            Vec::new(),
        )
    }

    pub fn block_hash(&self) -> Hash {
        hash_block(&self.block())
    }
}

#[derive(Deserialize)]
struct GenesisJson {
    chain_id: u16,
    chain_version: u16,
    block_height: u128,
    previous_block_hash: Hash,
    state: GenesisStateJson,
}

#[derive(Deserialize)]
struct GenesisStateJson {
    accounts: BTreeMap<Address, Account>,
}
