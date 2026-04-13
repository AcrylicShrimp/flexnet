mod shared;

use flexnet_chain::{
    account::Account,
    address::Address,
    block::Block,
    chain::Chain,
    genesis::Genesis,
    hash::{Hash, compute_state_hash},
    state::{StateView, WritableState},
};
use shared::{
    common::{
        address_for, block_with_transactions, config, secret_key, signed_transfer,
        state_with_accounts,
    },
    memory_state::MemoryState,
};

fn build_long_scenario_block(
    addresses: &[Address],
    previous_state: &MemoryState,
    previous_block_hash: Hash,
    block_height: u128,
    transfers_per_block: usize,
    expected_nonces: &mut [u128],
) -> (Block, MemoryState) {
    let mut next_state = previous_state.clone();
    let mut transactions = Vec::with_capacity(transfers_per_block);

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
        let nonce = next_state.get_account(&from).nonce;
        let transfer = signed_transfer(&secret_key((from_index + 1) as u8), to, amount, nonce);
        let delta = transfer.execute(&config(), &next_state).unwrap();

        WritableState::apply_delta(&mut next_state, delta);
        expected_nonces[from_index] += 1;
        transactions.push(transfer);
    }

    block_with_transactions(
        previous_state,
        previous_block_hash,
        block_height,
        transactions,
    )
}

#[test]
fn replay_long_multi_transaction_multi_block_scenario_across_independent_chains() {
    const NODE_COUNT: usize = 5;
    const ACCOUNT_COUNT: usize = 6;
    const BLOCK_COUNT: usize = 64;
    const TRANSFERS_PER_BLOCK: usize = 5;
    const INITIAL_BALANCE: u128 = 100_000;

    let secret_keys = (1..=ACCOUNT_COUNT as u8)
        .map(secret_key)
        .collect::<Vec<_>>();
    let addresses = secret_keys.iter().map(address_for).collect::<Vec<_>>();
    let genesis_accounts = addresses
        .iter()
        .copied()
        .map(|address| (address, Account::new(INITIAL_BALANCE, 0)))
        .collect::<Vec<_>>();
    let genesis = Genesis::new(config(), state_with_accounts(&genesis_accounts), vec![]);
    let mut chains = (0..NODE_COUNT)
        .map(|_| Chain::new(genesis.clone()))
        .collect::<Vec<_>>();
    let mut expected_nonces = vec![0_u128; ACCOUNT_COUNT];
    let total_supply = INITIAL_BALANCE * ACCOUNT_COUNT as u128;

    for _ in 0..BLOCK_COUNT {
        let reference = &chains[0];
        let expected_height = reference.next_block_height().unwrap();
        let (block, expected_state) = build_long_scenario_block(
            &addresses,
            reference.state(),
            reference.tip_block_hash(),
            expected_height,
            TRANSFERS_PER_BLOCK,
            &mut expected_nonces,
        );

        for chain in &mut chains {
            chain.append_block(block.clone()).unwrap();
            assert_eq!(chain.tip_height(), expected_height);
            assert_eq!(
                compute_state_hash(chain.state()),
                compute_state_hash(&expected_state)
            );
        }
    }

    let reference = &chains[0];
    assert_eq!(reference.tip_height(), BLOCK_COUNT as u128);

    for (index, address) in addresses.iter().copied().enumerate() {
        assert_eq!(
            reference.state().get_account(&address).nonce,
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

    for chain in &chains[1..] {
        assert_eq!(chain.tip_height(), reference.tip_height());
        assert_eq!(chain.tip_block_hash(), reference.tip_block_hash());
        assert_eq!(
            compute_state_hash(chain.state()),
            compute_state_hash(reference.state())
        );
        assert_eq!(chain.state(), reference.state());
    }
}
