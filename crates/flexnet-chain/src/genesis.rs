use crate::{
    address::Address,
    block::Block,
    chain_config::ChainConfig,
    hash::{Hash, compute_state_hash},
    state::StateView,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Genesis<S>
where
    S: StateView,
{
    pub config: ChainConfig,
    pub initial_state: S,
    pub validators: Vec<Address>,
}

impl<S> Genesis<S>
where
    S: StateView,
{
    pub fn new(config: ChainConfig, initial_state: S, validators: Vec<Address>) -> Self {
        Self {
            config,
            initial_state,
            validators,
        }
    }

    pub fn state_hash(&self) -> Hash {
        compute_state_hash(&self.initial_state)
    }

    pub fn block(&self) -> Block {
        Block::new(
            self.config.chain_id,
            self.config.chain_version,
            0,
            Hash::ZERO,
            self.state_hash(),
            Vec::new(),
        )
    }

    pub fn into_genesis_block(self) -> (ChainConfig, S, Vec<Address>, Block) {
        let block = self.block();
        (self.config, self.initial_state, self.validators, block)
    }
}

#[cfg(test)]
mod tests {
    use super::Genesis;
    use crate::{
        account::Account,
        address::Address,
        chain_config::ChainConfig,
        chain_id::ChainId,
        chain_version::ChainVersion,
        hash::{Hash, compute_state_hash},
        state::{StateDelta, StateView, WritableState},
    };
    use std::collections::BTreeMap;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestState {
        accounts: BTreeMap<Address, Account>,
    }

    impl TestState {
        fn new(accounts: BTreeMap<Address, Account>) -> Self {
            let mut state = Self { accounts };
            state.accounts.retain(|_, account| !account.is_empty());
            state
        }
    }

    impl StateView for TestState {
        fn all_accounts_in_order(&self) -> impl Iterator<Item = (Address, Account)> {
            self.accounts
                .iter()
                .map(|(address, account)| (*address, *account))
        }

        fn get_account(&self, address: &Address) -> Account {
            self.accounts.get(address).copied().unwrap_or_default()
        }
    }

    impl WritableState for TestState {
        fn apply_delta(&mut self, delta: StateDelta) {
            for (address, account) in delta.into_account_updates() {
                if account.is_empty() {
                    self.accounts.remove(&address);
                } else {
                    self.accounts.insert(address, account);
                }
            }
        }
    }

    #[test]
    fn genesis_builds_empty_genesis_block_with_state_hash() {
        let state = TestState::new(BTreeMap::from([(
            Address::new([1; 32]),
            Account::new(10, 0),
        )]));
        let genesis = Genesis::new(
            ChainConfig {
                chain_id: ChainId::new(1),
                chain_version: ChainVersion::new(1),
                max_transactions_per_block: 16,
            },
            state.clone(),
            vec![],
        );
        let block = genesis.block();

        assert_eq!(block.block_height, 0);
        assert_eq!(block.previous_block_hash, Hash::ZERO);
        assert!(block.transactions.is_empty());
        assert_eq!(block.state_hash, compute_state_hash(&state));
    }
}
