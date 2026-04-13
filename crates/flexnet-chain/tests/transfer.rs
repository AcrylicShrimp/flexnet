mod shared;

use flexnet_chain::{
    account::Account,
    rules::rule_transfer::{
        TransferExecutionError, TransferVerificationError, execute_transfer,
        verify_transfer_stateless,
    },
    state::{StateView, WritableState},
    transaction::Transaction,
    transactions::tx_transfer::{TransferPayload, TxTransfer},
};
use shared::common::{address_for, config, secret_key, signed_transfer, state_with_accounts};

#[test]
fn apply_valid_transfer_updates_balances_and_nonce() {
    let alice_key = secret_key(1);
    let bob_key = secret_key(2);
    let alice = address_for(&alice_key);
    let bob = address_for(&bob_key);
    let state = state_with_accounts(&[(alice, Account::new(1_000, 0)), (bob, Account::new(5, 0))]);
    let tx = signed_transfer(&alice_key, bob, 250, 0);

    let delta = match &tx {
        Transaction::Transfer(tx) => execute_transfer(tx, &config(), &state).unwrap(),
    };
    let mut next_state = state.clone();
    WritableState::apply_delta(&mut next_state, delta);

    assert_eq!(next_state.get_account(&alice), Account::new(750, 1));
    assert_eq!(next_state.get_account(&bob), Account::new(255, 0));
}

#[test]
fn reject_transfer_with_invalid_signature() {
    let alice_key = secret_key(1);
    let bob_key = secret_key(2);
    let alice = address_for(&alice_key);
    let bob = address_for(&bob_key);
    let state = state_with_accounts(&[(alice, Account::new(1_000, 0))]);
    let tx = signed_transfer(&alice_key, bob, 250, 0);
    let tampered = match tx {
        Transaction::Transfer(tx) => Transaction::Transfer(TxTransfer {
            payload: TransferPayload {
                amount: 251,
                ..tx.payload
            },
            signature: tx.signature,
        }),
    };

    assert!(matches!(
        match &tampered {
            Transaction::Transfer(tx) => verify_transfer_stateless(tx, &config()),
        },
        Err(TransferVerificationError::InvalidSignature(_))
    ));
    assert_eq!(state.get_account(&alice), Account::new(1_000, 0));
}

#[test]
fn reject_transfer_with_wrong_nonce() {
    let alice_key = secret_key(1);
    let bob_key = secret_key(2);
    let alice = address_for(&alice_key);
    let bob = address_for(&bob_key);
    let state = state_with_accounts(&[(alice, Account::new(1_000, 4)), (bob, Account::new(0, 0))]);
    let tx = signed_transfer(&alice_key, bob, 250, 3);

    assert_eq!(
        match &tx {
            Transaction::Transfer(tx) => execute_transfer(tx, &config(), &state),
        },
        Err(TransferExecutionError::InvalidNonce {
            expected: 4,
            actual: 3,
        })
    );
}
