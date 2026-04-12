#![allow(dead_code)]

use std::collections::BTreeMap;

use ed25519_dalek::{Signer, SigningKey};
use flexnet_core::{
    Account, Address, Block, Hash, State, Transfer,
    codec::encode_transfer_signing_payload,
    constants::{CURRENT_CHAIN_ID, CURRENT_CHAIN_VERSION},
    hash_state,
};

pub fn signing_key(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

pub fn address_for(signing_key: &SigningKey) -> Address {
    Address::new(signing_key.verifying_key().to_bytes())
}

pub fn state_with_accounts(accounts: &[(Address, Account)]) -> State {
    let accounts = BTreeMap::from_iter(accounts.iter().copied());
    State::new(accounts)
}

pub fn signed_transfer(
    signing_key: &SigningKey,
    to: Address,
    amount: u128,
    nonce: u128,
) -> Transfer {
    let from = address_for(signing_key);
    let unsigned = Transfer::unsigned(
        CURRENT_CHAIN_ID,
        CURRENT_CHAIN_VERSION,
        from,
        to,
        amount,
        nonce,
    );
    let signature = signing_key.sign(&encode_transfer_signing_payload(&unsigned));

    unsigned.with_signature(signature.to_bytes())
}

pub fn block_with_transfers(
    previous_state: &State,
    previous_block_hash: Hash,
    block_height: u128,
    transfers: Vec<Transfer>,
) -> (Block, State) {
    let mut next_state = previous_state.clone();

    for transfer in &transfers {
        let delta = transfer
            .apply(&next_state)
            .expect("block helper expects valid transfers");
        next_state.apply_delta(delta);
    }

    let block = Block::new(
        CURRENT_CHAIN_ID,
        CURRENT_CHAIN_VERSION,
        block_height,
        previous_block_hash,
        hash_state(&next_state),
        transfers,
    );

    (block, next_state)
}
