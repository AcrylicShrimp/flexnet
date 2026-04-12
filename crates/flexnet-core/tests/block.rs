#[path = "shared/common.rs"]
mod common;

use flexnet_core::{
    Account, Block, BlockError, Hash,
    constants::{CURRENT_CHAIN_ID, CURRENT_CHAIN_VERSION},
    execute_block, hash_block, hash_state, hash_transactions,
};

use self::common::{address_for, signed_transfer, signing_key, state_with_accounts};

#[test]
fn execute_block_matches_expected_hashes_and_state() {
    let alice_key = signing_key(1);
    let bob_key = signing_key(2);
    let carol_key = signing_key(3);
    let alice = address_for(&alice_key);
    let bob = address_for(&bob_key);
    let carol = address_for(&carol_key);
    let previous_state =
        state_with_accounts(&[(alice, Account::new(1_000, 0)), (bob, Account::new(100, 0))]);
    let previous_block_hash = Hash::new([7; 32]);
    let tx1 = signed_transfer(&alice_key, bob, 150, 0);
    let tx2 = signed_transfer(&bob_key, carol, 75, 0);
    let expected_state = {
        let after_tx1 = previous_state.applying(tx1.apply(&previous_state).unwrap());
        after_tx1.applying(tx2.apply(&after_tx1).unwrap())
    };
    let block = Block::new(
        CURRENT_CHAIN_ID,
        CURRENT_CHAIN_VERSION,
        1,
        previous_block_hash,
        hash_state(&expected_state),
        vec![tx1, tx2],
    );

    let outcome = execute_block(&previous_state, previous_block_hash, &block).unwrap();

    assert_eq!(outcome.state, expected_state);
    assert_eq!(outcome.state_hash, hash_state(&expected_state));
    assert_eq!(
        outcome.transactions_hash,
        hash_transactions(&block.transactions)
    );
    assert_eq!(outcome.block_hash, hash_block(&block));
    assert_eq!(outcome.state.get_account(alice), Account::new(850, 1));
    assert_eq!(outcome.state.get_account(bob), Account::new(175, 1));
    assert_eq!(outcome.state.get_account(carol), Account::new(75, 0));
}

#[test]
fn reject_block_when_any_transaction_is_invalid() {
    let alice_key = signing_key(1);
    let bob_key = signing_key(2);
    let alice = address_for(&alice_key);
    let bob = address_for(&bob_key);
    let previous_state = state_with_accounts(&[(alice, Account::new(1_000, 0))]);
    let previous_block_hash = Hash::new([9; 32]);
    let tx1 = signed_transfer(&alice_key, bob, 100, 0);
    let tx2 = signed_transfer(&alice_key, bob, 50, 0);
    let block = Block::new(
        CURRENT_CHAIN_ID,
        CURRENT_CHAIN_VERSION,
        1,
        previous_block_hash,
        Hash::ZERO,
        vec![tx1, tx2],
    );

    assert_eq!(
        execute_block(&previous_state, previous_block_hash, &block),
        Err(BlockError::Transaction {
            index: 1,
            error: flexnet_core::TransferError::InvalidNonce,
        }),
    );
}
