mod shared;

use flexnet_chain::{
    account::Account,
    block::Block,
    chain::{Chain, ChainAppendError},
    genesis::Genesis,
    hash::{Hash, compute_block_hash},
    rules::rule_block::BlockExecuteError,
    state::StateView,
};
use shared::common::{
    address_for, block_with_transactions, config, secret_key, signed_transfer, state_with_accounts,
};

#[test]
fn initialize_chain_from_genesis_and_append_blocks() {
    let alice_key = secret_key(1);
    let bob_key = secret_key(2);
    let carol_key = secret_key(3);
    let alice = address_for(&alice_key);
    let bob = address_for(&bob_key);
    let carol = address_for(&carol_key);
    let genesis = Genesis::new(
        config(),
        state_with_accounts(&[(alice, Account::new(1_000, 0)), (bob, Account::new(250, 0))]),
        vec![],
    );
    let mut chain = Chain::new(genesis.clone());

    assert_eq!(chain.tip_height(), 0);
    assert_eq!(chain.tip_block_hash(), compute_block_hash(&genesis.block()));
    assert_eq!(chain.tip_block(), &genesis.block());

    let tx1 = signed_transfer(&alice_key, bob, 150, 0);
    let (block1, _) = block_with_transactions(
        chain.state(),
        chain.tip_block_hash(),
        chain.next_block_height().unwrap(),
        vec![tx1],
    );
    chain.append_block(block1.clone()).unwrap();

    assert_eq!(chain.tip_height(), 1);
    assert_eq!(chain.tip_block(), &block1);
    assert_eq!(chain.state().get_account(&alice), Account::new(850, 1));
    assert_eq!(chain.state().get_account(&bob), Account::new(400, 0));

    let tx2 = signed_transfer(&bob_key, carol, 125, 0);
    let (block2, _) = block_with_transactions(
        chain.state(),
        chain.tip_block_hash(),
        chain.next_block_height().unwrap(),
        vec![tx2],
    );
    chain.append_block(block2.clone()).unwrap();

    assert_eq!(chain.tip_height(), 2);
    assert_eq!(chain.tip_block(), &block2);
    assert_eq!(chain.state().get_account(&alice), Account::new(850, 1));
    assert_eq!(chain.state().get_account(&bob), Account::new(275, 1));
    assert_eq!(chain.state().get_account(&carol), Account::new(125, 0));
}

#[test]
fn reject_out_of_sequence_block_height_without_mutating_chain() {
    let alice_key = secret_key(1);
    let bob_key = secret_key(2);
    let alice = address_for(&alice_key);
    let bob = address_for(&bob_key);
    let mut chain = Chain::new(Genesis::new(
        config(),
        state_with_accounts(&[(alice, Account::new(1_000, 0))]),
        vec![],
    ));
    let before = chain.clone();
    let tx = signed_transfer(&alice_key, bob, 100, 0);
    let (block, _) = block_with_transactions(chain.state(), chain.tip_block_hash(), 2, vec![tx]);

    let error = chain.append_block(block).unwrap_err();

    assert_eq!(
        error,
        ChainAppendError::UnexpectedBlockHeight {
            expected: 1,
            actual: 2,
        }
    );
    assert_eq!(chain, before);
}

#[test]
fn reject_invalid_block_without_mutating_chain() {
    let alice_key = secret_key(1);
    let bob_key = secret_key(2);
    let alice = address_for(&alice_key);
    let bob = address_for(&bob_key);
    let mut chain = Chain::new(Genesis::new(
        config(),
        state_with_accounts(&[(alice, Account::new(1_000, 0))]),
        vec![],
    ));
    let before = chain.clone();
    let tx = signed_transfer(&alice_key, bob, 100, 0);
    let (mut block, _) = block_with_transactions(
        chain.state(),
        chain.tip_block_hash(),
        chain.next_block_height().unwrap(),
        vec![tx],
    );
    block.state_hash = Hash::ZERO;

    let error = chain.append_block(block).unwrap_err();

    assert!(matches!(
        error,
        ChainAppendError::BlockExecutionError(BlockExecuteError::InvalidStateHash { .. })
    ));
    assert_eq!(chain, before);
}

#[test]
fn reject_previous_hash_mismatch_without_mutating_chain() {
    let alice_key = secret_key(1);
    let alice = address_for(&alice_key);
    let mut chain = Chain::new(Genesis::new(
        config(),
        state_with_accounts(&[(alice, Account::new(1_000, 0))]),
        vec![],
    ));
    let before = chain.clone();
    let block = Block::new(
        config().chain_id,
        config().chain_version,
        1,
        Hash::new([9; 32]),
        Hash::ZERO,
        Vec::new(),
    );

    let error = chain.append_block(block).unwrap_err();

    assert_eq!(
        error,
        ChainAppendError::PreviousBlockHashMismatch {
            expected: before.tip_block_hash(),
            actual: Hash::new([9; 32]),
        }
    );
    assert_eq!(chain, before);
}
