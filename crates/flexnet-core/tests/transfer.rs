#[path = "shared/common.rs"]
mod common;

use flexnet_core::{Account, TransferError};

use self::common::{address_for, signed_transfer, signing_key, state_with_accounts};

#[test]
fn apply_valid_transfer_updates_balances_and_nonce() {
    let alice_key = signing_key(1);
    let bob_key = signing_key(2);
    let alice = address_for(&alice_key);
    let bob = address_for(&bob_key);
    let state = state_with_accounts(&[(alice, Account::new(1_000, 0)), (bob, Account::new(5, 0))]);
    let transfer = signed_transfer(&alice_key, bob, 250, 0);

    let next_state = state.applying(transfer.apply(&state).unwrap());

    assert_eq!(next_state.get_account(alice), Account::new(750, 1));
    assert_eq!(next_state.get_account(bob), Account::new(255, 0));
}

#[test]
fn reject_transfer_with_invalid_signature() {
    let alice_key = signing_key(1);
    let bob_key = signing_key(2);
    let alice = address_for(&alice_key);
    let bob = address_for(&bob_key);
    let state = state_with_accounts(&[(alice, Account::new(1_000, 0))]);
    let transfer = signed_transfer(&alice_key, bob, 250, 0);
    let tampered = flexnet_core::Transfer {
        amount: 251,
        ..transfer
    };

    assert_eq!(
        tampered.verify(&state),
        Err(TransferError::InvalidSignature)
    );
}

#[test]
fn reject_transfer_with_wrong_nonce() {
    let alice_key = signing_key(1);
    let bob_key = signing_key(2);
    let alice = address_for(&alice_key);
    let bob = address_for(&bob_key);
    let state = state_with_accounts(&[(alice, Account::new(1_000, 4)), (bob, Account::new(0, 0))]);
    let transfer = signed_transfer(&alice_key, bob, 250, 3);

    assert_eq!(transfer.verify(&state), Err(TransferError::InvalidNonce));
}
