use flexnet_chain::{
    account::Account,
    address::Address,
    state::{StateDelta, StateView, WritableState},
};
use std::collections::BTreeMap;

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct MemoryState {
    accounts: BTreeMap<Address, Account>,
}

impl MemoryState {
    pub fn new(accounts: BTreeMap<Address, Account>) -> Self {
        let mut state = Self { accounts };
        state.normalize();
        state
    }

    pub fn accounts(&self) -> &BTreeMap<Address, Account> {
        &self.accounts
    }

    fn normalize(&mut self) {
        self.accounts.retain(|_, account| !account.is_empty());
    }
}

impl StateView for MemoryState {
    fn all_accounts_in_order(&self) -> impl Iterator<Item = (Address, Account)> {
        self.accounts
            .iter()
            .map(|(address, account)| (*address, *account))
    }

    fn get_account(&self, address: &Address) -> Account {
        self.accounts.get(address).copied().unwrap_or_default()
    }
}

impl WritableState for MemoryState {
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
