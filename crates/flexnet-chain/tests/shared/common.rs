use crate::shared::memory_state::MemoryState;
use flexnet_chain::{
    account::Account,
    address::Address,
    block::Block,
    chain_config::ChainConfig,
    chain_id::ChainId,
    chain_version::ChainVersion,
    crypto::address_from_secret_key,
    crypto::{SecretKey, sign},
    hash::{Hash, compute_state_hash},
    state::WritableState,
    transaction::Transaction,
    transactions::tx_transfer::{TransferPayload, TxTransfer},
};
use std::collections::BTreeMap;

pub fn config() -> ChainConfig {
    ChainConfig {
        chain_id: ChainId::new(1),
        chain_version: ChainVersion::new(1),
        max_transactions_per_block: 128,
    }
}

pub fn secret_key(seed: u8) -> SecretKey {
    SecretKey::new([seed; 32])
}

pub fn address_for(secret_key: &SecretKey) -> Address {
    address_from_secret_key(secret_key)
}

pub fn state_with_accounts(accounts: &[(Address, Account)]) -> MemoryState {
    MemoryState::new(BTreeMap::from_iter(accounts.iter().copied()))
}

pub fn signed_transfer(
    secret_key: &SecretKey,
    to: Address,
    amount: u128,
    nonce: u128,
) -> Transaction {
    let payload = TransferPayload::new(
        ChainId::new(1),
        ChainVersion::new(1),
        address_for(secret_key),
        to,
        amount,
        nonce,
    );
    let mut signing_payload = Vec::with_capacity(payload.signing_payload_len());
    payload.encode_signing_payload(&mut signing_payload);
    Transaction::Transfer(TxTransfer::new(payload, sign(secret_key, &signing_payload)))
}

pub fn block_with_transactions(
    previous_state: &MemoryState,
    previous_block_hash: Hash,
    block_height: u128,
    transactions: Vec<Transaction>,
) -> (Block, MemoryState) {
    let mut next_state = previous_state.clone();

    for transaction in &transactions {
        let delta = transaction
            .execute(&config(), &next_state)
            .expect("block helper expects valid transactions");
        next_state.apply_delta(delta);
    }

    let block = Block::new(
        ChainId::new(1),
        ChainVersion::new(1),
        block_height,
        previous_block_hash,
        compute_state_hash(&next_state),
        transactions,
    );

    (block, next_state)
}
