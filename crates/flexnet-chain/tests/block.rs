mod shared;

use flexnet_chain::{
    account::Account,
    block::Block,
    hash::{Hash, compute_block_hash, compute_state_hash, compute_transactions_hash},
    rules::{
        rule_block::{BlockExecutionError, execute_block},
        rule_transfer::TransferExecutionError,
    },
    state::{StateView, WritableState},
    transaction::TransactionExecutionError,
};
use shared::common::{address_for, config, secret_key, signed_transfer, state_with_accounts};

#[test]
fn execute_block_matches_expected_hashes_and_state() {
    let alice_key = secret_key(1);
    let bob_key = secret_key(2);
    let carol_key = secret_key(3);
    let alice = address_for(&alice_key);
    let bob = address_for(&bob_key);
    let carol = address_for(&carol_key);
    let previous_state =
        state_with_accounts(&[(alice, Account::new(1_000, 0)), (bob, Account::new(100, 0))]);
    let previous_block_hash = Hash::new([7; 32]);
    let tx1 = signed_transfer(&alice_key, bob, 150, 0);
    let tx2 = signed_transfer(&bob_key, carol, 75, 0);
    let expected_state = {
        let mut next = previous_state.clone();
        let delta1 = tx1.execute(&config(), &next).unwrap();
        WritableState::apply_delta(&mut next, delta1);
        let delta2 = tx2.execute(&config(), &next).unwrap();
        WritableState::apply_delta(&mut next, delta2);
        next
    };
    let block = Block::new(
        config().chain_id,
        config().chain_version,
        1,
        previous_block_hash,
        compute_state_hash(&expected_state),
        vec![tx1.clone(), tx2.clone()],
    );

    let outcome = execute_block(&block, &config(), &previous_state).unwrap();
    let mut next_state = previous_state.clone();
    WritableState::apply_delta(&mut next_state, outcome.state_delta.clone());

    assert_eq!(next_state, expected_state);
    assert_eq!(outcome.state_hash, compute_state_hash(&expected_state));
    assert_eq!(
        compute_transactions_hash(&block.transactions),
        compute_transactions_hash(&[tx1, tx2])
    );
    assert_eq!(compute_block_hash(&block), compute_block_hash(&block));
    assert_eq!(next_state.get_account(&alice), Account::new(850, 1));
    assert_eq!(next_state.get_account(&bob), Account::new(175, 1));
    assert_eq!(next_state.get_account(&carol), Account::new(75, 0));
}

#[test]
fn reject_block_when_any_transaction_is_invalid() {
    let alice_key = secret_key(1);
    let bob_key = secret_key(2);
    let alice = address_for(&alice_key);
    let bob = address_for(&bob_key);
    let previous_state = state_with_accounts(&[(alice, Account::new(1_000, 0))]);
    let tx1 = signed_transfer(&alice_key, bob, 100, 0);
    let tx2 = signed_transfer(&alice_key, bob, 50, 0);
    let block = Block::new(
        config().chain_id,
        config().chain_version,
        1,
        Hash::ZERO,
        Hash::ZERO,
        vec![tx1, tx2],
    );

    assert_eq!(
        execute_block(&block, &config(), &previous_state),
        Err(BlockExecutionError::TxExecuteError {
            index: 1,
            error: TransactionExecutionError::Transfer(TransferExecutionError::InvalidNonce {
                expected: 1,
                actual: 0,
            },),
        })
    );
}
