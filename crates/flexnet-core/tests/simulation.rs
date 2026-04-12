#[path = "shared/common.rs"]
mod common;

use flexnet_core::{Account, BlockError, ChainError, Genesis, Hash, Simulation, SimulationError};

use self::common::{
    address_for, block_with_transfers, signed_transfer, signing_key, state_with_accounts,
};

#[test]
fn replay_blocks_across_nodes_and_keep_them_in_sync() {
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
    let mut simulation = Simulation::new(genesis.clone(), 3).unwrap();

    assert_eq!(simulation.node_count(), 3);
    simulation.assert_in_sync().unwrap();

    let tx1 = signed_transfer(&alice_key, bob, 150, 0);
    let (block1, _) = block_with_transfers(
        simulation.chain(0).unwrap().state(),
        simulation.chain(0).unwrap().tip_hash(),
        simulation.chain(0).unwrap().next_block_height(),
        vec![tx1],
    );
    simulation.append_block(block1).unwrap();
    simulation.assert_in_sync().unwrap();

    let tx2 = signed_transfer(&bob_key, carol, 125, 0);
    let (block2, _) = block_with_transfers(
        simulation.chain(0).unwrap().state(),
        simulation.chain(0).unwrap().tip_hash(),
        simulation.chain(0).unwrap().next_block_height(),
        vec![tx2],
    );
    simulation.append_block(block2).unwrap();
    simulation.assert_in_sync().unwrap();

    for chain in simulation.chains() {
        assert_eq!(chain.tip_height(), 2);
        assert_eq!(chain.state().get_account(alice), Account::new(850, 1));
        assert_eq!(chain.state().get_account(bob), Account::new(275, 1));
        assert_eq!(chain.state().get_account(carol), Account::new(125, 0));
    }
}

#[test]
fn reject_zero_node_simulation() {
    let genesis = Genesis {
        chain_id: 1,
        chain_version: 1,
        block_height: 0,
        previous_block_hash: Hash::ZERO,
        state: state_with_accounts(&[]),
    };

    assert!(matches!(
        Simulation::new(genesis, 0),
        Err(SimulationError::EmptySimulation)
    ));
}

#[test]
fn reject_invalid_block_without_mutating_any_node() {
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
    let mut simulation = Simulation::new(genesis.clone(), 3).unwrap();
    let before = simulation.chains().to_vec();
    let tx = signed_transfer(&alice_key, bob, 100, 0);
    let (mut block, _) = block_with_transfers(
        simulation.chain(0).unwrap().state(),
        simulation.chain(0).unwrap().tip_hash(),
        simulation.chain(0).unwrap().next_block_height(),
        vec![tx],
    );
    block.state_hash = Hash::ZERO;

    let error = simulation.append_block(block).unwrap_err();

    assert!(matches!(
        error,
        SimulationError::ChainAppend {
            index: 0,
            error: ChainError::Block(BlockError::InvalidStateHash { .. }),
        }
    ));
    assert_eq!(simulation.chains(), before.as_slice());
    simulation.assert_in_sync().unwrap();
    for chain in simulation.chains() {
        assert_eq!(chain.tip_height(), 0);
        assert_eq!(chain.tip_hash(), genesis.block_hash());
        assert_eq!(chain.state_hash(), genesis.state_hash());
        assert_eq!(chain.state().get_account(alice), Account::new(1_000, 0));
        assert_eq!(chain.state().get_account(bob), Account::new(0, 0));
    }
}
