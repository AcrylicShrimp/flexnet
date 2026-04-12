#[path = "shared/common.rs"]
mod common;

use ed25519_dalek::SigningKey;
use flexnet_core::{
    Account, Address, Block, BlockError, ChainError, Genesis, Hash, Simulation, SimulationError,
    State, hash_state,
};

use self::common::{
    address_for, block_with_transfers, signed_transfer, signing_key, state_with_accounts,
};

fn build_long_scenario_block(
    signing_keys: &[SigningKey],
    addresses: &[Address],
    previous_state: &State,
    previous_block_hash: Hash,
    block_height: u128,
    transfers_per_block: usize,
    expected_nonces: &mut [u128],
) -> (Block, State) {
    let mut next_state = previous_state.clone();
    let mut transfers = Vec::with_capacity(transfers_per_block);

    for transfer_index in 0..transfers_per_block {
        let from_index = ((block_height as usize * 7) + (transfer_index * 3)) % addresses.len();
        let mut to_index =
            (from_index + transfer_index + 1 + (block_height as usize % 3)) % addresses.len();
        if to_index == from_index {
            to_index = (to_index + 1) % addresses.len();
        }

        let from = addresses[from_index];
        let to = addresses[to_index];
        let amount =
            1 + (((block_height as usize * 17) + (transfer_index * 13) + from_index) % 97) as u128;
        let nonce = next_state.get_account(from).nonce;
        let transfer = signed_transfer(&signing_keys[from_index], to, amount, nonce);
        let delta = transfer
            .apply(&next_state)
            .expect("generated long scenario transfer should be valid");

        next_state.apply_delta(delta);
        expected_nonces[from_index] += 1;
        transfers.push(transfer);
    }

    let block = Block::new(
        1,
        1,
        block_height,
        previous_block_hash,
        hash_state(&next_state),
        transfers,
    );

    (block, next_state)
}

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

#[test]
fn replay_long_multi_transaction_multi_block_scenario_across_nodes() {
    const NODE_COUNT: usize = 5;
    const ACCOUNT_COUNT: usize = 6;
    const BLOCK_COUNT: usize = 64;
    const TRANSFERS_PER_BLOCK: usize = 5;
    const INITIAL_BALANCE: u128 = 100_000;

    let signing_keys = (1..=ACCOUNT_COUNT as u8)
        .map(signing_key)
        .collect::<Vec<_>>();
    let addresses = signing_keys.iter().map(address_for).collect::<Vec<_>>();
    let genesis_accounts = addresses
        .iter()
        .copied()
        .map(|address| (address, Account::new(INITIAL_BALANCE, 0)))
        .collect::<Vec<_>>();
    let genesis = Genesis {
        chain_id: 1,
        chain_version: 1,
        block_height: 0,
        previous_block_hash: Hash::ZERO,
        state: state_with_accounts(&genesis_accounts),
    };
    let mut simulation = Simulation::new(genesis, NODE_COUNT).unwrap();
    let mut expected_nonces = vec![0_u128; ACCOUNT_COUNT];
    let total_supply = INITIAL_BALANCE * ACCOUNT_COUNT as u128;

    for _ in 0..BLOCK_COUNT {
        let reference = simulation
            .chain(0)
            .expect("simulation should contain a reference chain");
        let expected_height = reference.next_block_height();
        let (block, expected_state) = build_long_scenario_block(
            &signing_keys,
            &addresses,
            reference.state(),
            reference.tip_hash(),
            expected_height,
            TRANSFERS_PER_BLOCK,
            &mut expected_nonces,
        );

        simulation.append_block(block).unwrap();
        simulation.assert_in_sync().unwrap();

        for chain in simulation.chains() {
            assert_eq!(chain.tip_height(), expected_height);
            assert_eq!(chain.state_hash(), hash_state(&expected_state));
        }
    }

    let reference = simulation
        .chain(0)
        .expect("simulation should contain a reference chain");
    assert_eq!(reference.tip_height(), BLOCK_COUNT as u128);

    for (index, address) in addresses.iter().copied().enumerate() {
        assert_eq!(
            reference.state().get_account(address).nonce,
            expected_nonces[index]
        );
    }

    let final_total_supply = reference
        .state()
        .accounts()
        .values()
        .map(|account| account.balance)
        .sum::<u128>();
    assert_eq!(final_total_supply, total_supply);

    for chain in simulation.chains() {
        assert_eq!(chain, reference);
    }
}
