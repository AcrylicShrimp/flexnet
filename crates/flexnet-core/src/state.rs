use std::collections::BTreeMap;

use crate::{account::Account, address::Address};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct State {
    accounts: BTreeMap<Address, Account>,
}

impl State {
    pub fn new(accounts: BTreeMap<Address, Account>) -> Self {
        let mut state = Self { accounts };
        state.normalize();
        state
    }

    pub fn get_account(&self, address: Address) -> Account {
        self.accounts.get(&address).copied().unwrap_or_default()
    }

    pub fn accounts(&self) -> &BTreeMap<Address, Account> {
        &self.accounts
    }

    pub fn set_account(&mut self, address: Address, account: Account) {
        if account.is_empty() {
            self.accounts.remove(&address);
        } else {
            self.accounts.insert(address, account);
        }
    }

    pub fn apply_delta(&mut self, delta: StateDelta) {
        for (address, account) in delta.accounts {
            self.set_account(address, account);
        }
    }

    pub fn applying(&self, delta: StateDelta) -> Self {
        let mut next = self.clone();
        next.apply_delta(delta);
        next
    }

    pub fn iter_non_empty(&self) -> impl Iterator<Item = (&Address, &Account)> {
        self.accounts
            .iter()
            .filter(|(_, account)| !account.is_empty())
    }

    fn normalize(&mut self) {
        self.accounts.retain(|_, account| !account.is_empty());
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StateDelta {
    pub accounts: BTreeMap<Address, Account>,
}

impl StateDelta {
    pub fn new(accounts: BTreeMap<Address, Account>) -> Self {
        Self { accounts }
    }
}
