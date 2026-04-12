#[path = "shared/common.rs"]
mod common;

use proptest::prelude::*;

use flexnet_core::{
    Account, Block, Genesis, Hash,
    constants::{CURRENT_CHAIN_ID, CURRENT_CHAIN_VERSION},
    execute_block, hash_state,
};

use self::common::{address_for, signed_transfer, signing_key, state_with_accounts};

#[test]
fn replaying_the_same_blocks_produces_identical_results() {
    let alice_key = signing_key(1);
    let bob_key = signing_key(2);
    let carol_key = signing_key(3);
    let alice = address_for(&alice_key);
    let bob = address_for(&bob_key);
    let carol = address_for(&carol_key);
    let state =
        state_with_accounts(&[(alice, Account::new(1_000, 0)), (bob, Account::new(500, 0))]);
    let genesis = Genesis {
        chain_id: CURRENT_CHAIN_ID,
        chain_version: CURRENT_CHAIN_VERSION,
        block_height: 0,
        previous_block_hash: Hash::ZERO,
        state: state.clone(),
    };
    let block1 = {
        let tx = signed_transfer(&alice_key, bob, 200, 0);
        let expected_state = state.applying(tx.apply(&state).unwrap());

        Block::new(
            CURRENT_CHAIN_ID,
            CURRENT_CHAIN_VERSION,
            1,
            genesis.block_hash(),
            hash_state(&expected_state),
            vec![tx],
        )
    };
    let outcome1 = execute_block(&state, genesis.block_hash(), &block1).unwrap();
    let block2 = {
        let tx = signed_transfer(&bob_key, carol, 125, 0);
        let expected_state = outcome1.state.applying(tx.apply(&outcome1.state).unwrap());

        Block::new(
            CURRENT_CHAIN_ID,
            CURRENT_CHAIN_VERSION,
            2,
            outcome1.block_hash,
            hash_state(&expected_state),
            vec![tx],
        )
    };

    let left_block1 = execute_block(&state, genesis.block_hash(), &block1).unwrap();
    let left_block2 = execute_block(&left_block1.state, left_block1.block_hash, &block2).unwrap();

    let right_block1 = execute_block(&state, genesis.block_hash(), &block1).unwrap();
    let right_block2 =
        execute_block(&right_block1.state, right_block1.block_hash, &block2).unwrap();

    assert_eq!(left_block1, right_block1);
    assert_eq!(left_block2, right_block2);
    assert_eq!(left_block2.state.get_account(alice), Account::new(800, 1));
    assert_eq!(left_block2.state.get_account(bob), Account::new(575, 1));
    assert_eq!(left_block2.state.get_account(carol), Account::new(125, 0));
}

proptest! {
    #[test]
    fn identical_input_replay_keeps_state_hashes_identical(
        amount in 1_u64..900_u64
    ) {
        let alice_key = signing_key(11);
        let bob_key = signing_key(12);
        let alice = address_for(&alice_key);
        let bob = address_for(&bob_key);
        let state = state_with_accounts(&[
            (alice, Account::new(1_000, 0)),
            (bob, Account::new(0, 0)),
        ]);
        let transfer = signed_transfer(&alice_key, bob, u128::from(amount), 0);

        let next_a = state.applying(transfer.apply(&state).unwrap());
        let next_b = state.applying(transfer.apply(&state).unwrap());

        prop_assert_eq!(next_a.clone(), next_b.clone());
        prop_assert_eq!(hash_state(&next_a), hash_state(&next_b));
    }
}
