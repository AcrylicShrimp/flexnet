#[path = "shared/common.rs"]
mod common;

use flexnet_core::{Account, BlockError, Chain, ChainError, Genesis, Hash};

use self::common::{
    address_for, block_with_transfers, signed_transfer, signing_key, state_with_accounts,
};

#[test]
fn initialize_chain_from_genesis_and_append_blocks() {
    let alice_key = signing_key(1);
    let bob_key = signing_key(2);
    let carol_key = signing_key(3);
    let alice = address_for(&alice_key);
    let bob = address_for(&bob_key);
    let carol = address_for(&carol_key);
    let genesis = Genesis {
        chain_id: 1,
        chain_version: 1,
        block_height: 0,
        previous_block_hash: Hash::ZERO,
        state: state_with_accounts(&[(alice, Account::new(1_000, 0)), (bob, Account::new(250, 0))]),
    };
    let mut chain = Chain::new(genesis.clone()).unwrap();

    assert_eq!(chain.tip_height(), 0);
    assert_eq!(chain.tip_hash(), genesis.block_hash());
    assert_eq!(chain.state_hash(), genesis.state_hash());
    assert!(chain.tip_block().is_none());

    let tx1 = signed_transfer(&alice_key, bob, 150, 0);
    let (block1, _) = block_with_transfers(
        chain.state(),
        chain.tip_hash(),
        chain.next_block_height(),
        vec![tx1],
    );
    chain.append_block(block1.clone()).unwrap();

    assert_eq!(chain.tip_height(), 1);
    assert_eq!(chain.tip_block(), Some(&block1));
    assert_eq!(chain.blocks(), std::slice::from_ref(&block1));
    assert_eq!(chain.state().get_account(alice), Account::new(850, 1));
    assert_eq!(chain.state().get_account(bob), Account::new(400, 0));

    let tx2 = signed_transfer(&bob_key, carol, 125, 0);
    let (block2, _) = block_with_transfers(
        chain.state(),
        chain.tip_hash(),
        chain.next_block_height(),
        vec![tx2],
    );
    chain.append_block(block2.clone()).unwrap();

    assert_eq!(chain.tip_height(), 2);
    assert_eq!(chain.tip_block(), Some(&block2));
    assert_eq!(chain.blocks(), &[block1, block2]);
    assert_eq!(chain.state().get_account(alice), Account::new(850, 1));
    assert_eq!(chain.state().get_account(bob), Account::new(275, 1));
    assert_eq!(chain.state().get_account(carol), Account::new(125, 0));
}

#[test]
fn reject_out_of_sequence_block_height_without_mutating_chain() {
    let alice_key = signing_key(1);
    let bob_key = signing_key(2);
    let alice = address_for(&alice_key);
    let bob = address_for(&bob_key);
    let genesis = Genesis {
        chain_id: 1,
        chain_version: 1,
        block_height: 0,
        previous_block_hash: Hash::ZERO,
        state: state_with_accounts(&[(alice, Account::new(1_000, 0))]),
    };
    let mut chain = Chain::new(genesis.clone()).unwrap();
    let tx = signed_transfer(&alice_key, bob, 100, 0);
    let (block, _) = block_with_transfers(chain.state(), chain.tip_hash(), 2, vec![tx]);

    let error = chain.append_block(block).unwrap_err();

    assert!(matches!(
        error,
        ChainError::UnexpectedBlockHeight {
            expected: 1,
            actual: 2,
        }
    ));
    assert_eq!(chain.tip_height(), 0);
    assert_eq!(chain.tip_hash(), genesis.block_hash());
    assert_eq!(chain.state_hash(), genesis.state_hash());
    assert_eq!(chain.state().get_account(alice), Account::new(1_000, 0));
    assert_eq!(chain.state().get_account(bob), Account::new(0, 0));
}

#[test]
fn reject_invalid_block_without_mutating_chain() {
    let alice_key = signing_key(1);
    let bob_key = signing_key(2);
    let alice = address_for(&alice_key);
    let bob = address_for(&bob_key);
    let genesis = Genesis {
        chain_id: 1,
        chain_version: 1,
        block_height: 0,
        previous_block_hash: Hash::ZERO,
        state: state_with_accounts(&[(alice, Account::new(1_000, 0))]),
    };
    let mut chain = Chain::new(genesis.clone()).unwrap();
    let tx = signed_transfer(&alice_key, bob, 100, 0);
    let (mut block, _) = block_with_transfers(chain.state(), chain.tip_hash(), 1, vec![tx]);

    block.state_hash = Hash::ZERO;

    let error = chain.append_block(block).unwrap_err();

    assert!(matches!(
        error,
        ChainError::Block(BlockError::InvalidStateHash { .. })
    ));
    assert_eq!(chain.tip_height(), 0);
    assert_eq!(chain.tip_hash(), genesis.block_hash());
    assert_eq!(chain.state_hash(), genesis.state_hash());
    assert_eq!(chain.state().get_account(alice), Account::new(1_000, 0));
    assert_eq!(chain.state().get_account(bob), Account::new(0, 0));
}
