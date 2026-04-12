use crate::{
    account::Account, address::Address, block::Block, state::StateView, transaction::Transaction,
};
use sha2::{Digest, Sha256};
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hash([u8; 32]);

impl Hash {
    pub const ZERO: Self = Self([0; 32]);

    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn compute(data: &[u8]) -> Self {
        Self::new(Sha256::digest(data).into())
    }
}

impl Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{}", hex::encode(self.0))
    }
}

pub fn encode_transactions_hash_preimage(transactions: &[Transaction]) -> Vec<u8> {
    let mut preimage_size = 2;

    for transaction in transactions {
        preimage_size += transaction.encoded_len();
    }

    let mut preimage = Vec::with_capacity(preimage_size);
    preimage.extend_from_slice(&(transactions.len() as u16).to_le_bytes());

    for transaction in transactions {
        transaction.encode_canonical(&mut preimage);
    }

    preimage
}

pub fn compute_transactions_hash(transactions: &[Transaction]) -> Hash {
    Hash::compute(&encode_transactions_hash_preimage(transactions))
}

pub fn encode_state_hash_preimage(rolling: &Hash, address: &Address, account: &Account) -> Vec<u8> {
    let preimage_size = 32 + 32 + 16 + 16;

    let mut preimage = Vec::with_capacity(preimage_size);
    preimage.extend_from_slice(rolling.as_bytes());
    preimage.extend_from_slice(address.as_bytes());
    preimage.extend_from_slice(&account.balance.to_le_bytes());
    preimage.extend_from_slice(&account.nonce.to_le_bytes());
    preimage
}

pub fn compute_state_hash<S>(state: &S) -> Hash
where
    S: StateView,
{
    let mut rolling = Hash::ZERO;

    for (address, account) in state
        .all_accounts_in_order()
        .filter(|(_, account)| !account.is_empty())
    {
        rolling = Hash::compute(&encode_state_hash_preimage(&rolling, &address, &account));
    }

    rolling
}

pub fn encode_block_hash_preimage(block: &Block) -> Vec<u8> {
    let preimage_size = 2 + 2 + 16 + 32 + 32 + 32;

    let transactions_hash = compute_transactions_hash(&block.transactions);

    let mut preimage = Vec::with_capacity(preimage_size);
    preimage.extend_from_slice(&block.chain_id.into_u16().to_le_bytes());
    preimage.extend_from_slice(&block.chain_version.into_u16().to_le_bytes());
    preimage.extend_from_slice(&block.block_height.to_le_bytes());
    preimage.extend_from_slice(block.previous_block_hash.as_bytes());
    preimage.extend_from_slice(block.state_hash.as_bytes());
    preimage.extend_from_slice(transactions_hash.as_bytes());
    preimage
}

pub fn compute_block_hash(block: &Block) -> Hash {
    Hash::compute(&encode_block_hash_preimage(block))
}
