#![allow(dead_code)]

use std::collections::BTreeMap;

use ed25519_dalek::{Signer, SigningKey};
use flexnet_core::{
    Account, Address, State, Transfer,
    codec::encode_transfer_signing_payload,
    constants::{CURRENT_CHAIN_ID, CURRENT_CHAIN_VERSION},
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
    let signature =
        signing_key.sign(&encode_transfer_signing_payload(&unsigned));

    unsigned.with_signature(signature.to_bytes())
}
