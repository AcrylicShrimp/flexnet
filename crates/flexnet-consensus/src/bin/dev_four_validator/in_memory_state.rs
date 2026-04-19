use flexnet_chain::{
    account::Account,
    address::Address,
    state::{StateDelta, StateView, WritableState},
};
use std::collections::BTreeMap;

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct InMemoryState {
    accounts: BTreeMap<Address, Account>,
}

impl InMemoryState {
    pub fn new() -> Self {
        Self {
            accounts: BTreeMap::new(),
        }
    }
}

impl StateView for InMemoryState {
    fn all_accounts_in_order(&self) -> impl Iterator<Item = (Address, Account)> {
        self.accounts
            .iter()
            .map(|(address, account)| (*address, *account))
    }

    fn get_account(&self, address: &Address) -> Account {
        self.accounts.get(address).copied().unwrap_or_default()
    }
}

impl WritableState for InMemoryState {
    fn apply_delta(&mut self, delta: StateDelta) {
        for (address, account) in delta.into_account_updates() {
            self.accounts.insert(address, account);
        }
    }
}
