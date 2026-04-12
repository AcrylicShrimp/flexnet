use crate::{account::Account, address::Address};
use std::collections::BTreeMap;

pub trait StateView {
    fn all_accounts_in_order(&self) -> impl Iterator<Item = (Address, Account)>;
    fn get_account(&self, address: &Address) -> Account;
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct StateDelta {
    account_updates: BTreeMap<Address, Account>,
}

impl StateDelta {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn update_account(&mut self, address: Address, account: Account) {
        self.account_updates.insert(address, account);
    }
}

pub trait StateTransition {
    fn apply<S>(&self, state: S, delta: StateDelta) -> S
    where
        S: StateView;
}
