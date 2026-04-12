use crate::{account::Account, address::Address};
use std::collections::BTreeMap;

pub trait StateView {
    fn all_accounts_in_order(&self) -> impl Iterator<Item = (Address, Account)>;
    fn get_account(&self, address: &Address) -> Account;
}

pub trait WritableState: StateView {
    fn apply_delta(&mut self, delta: StateDelta);
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct State {
    accounts: BTreeMap<Address, Account>,
}

impl State {
    pub fn new(accounts: BTreeMap<Address, Account>) -> Self {
        let mut state = Self { accounts };
        state.normalize();
        state
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

    fn normalize(&mut self) {
        self.accounts.retain(|_, account| !account.is_empty());
    }
}

impl StateView for State {
    fn all_accounts_in_order(&self) -> impl Iterator<Item = (Address, Account)> {
        self.accounts
            .iter()
            .map(|(address, account)| (*address, *account))
    }

    fn get_account(&self, address: &Address) -> Account {
        self.accounts.get(address).copied().unwrap_or_default()
    }
}

impl WritableState for State {
    fn apply_delta(&mut self, delta: StateDelta) {
        for (address, account) in delta.into_account_updates() {
            self.set_account(address, account);
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct StateDelta {
    account_updates: BTreeMap<Address, Account>,
}

impl StateDelta {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn account_updates(&self) -> &BTreeMap<Address, Account> {
        &self.account_updates
    }

    pub fn is_empty(&self) -> bool {
        self.account_updates.is_empty()
    }

    pub fn update_account(&mut self, address: Address, account: Account) {
        self.account_updates.insert(address, account);
    }

    pub fn into_account_updates(self) -> BTreeMap<Address, Account> {
        self.account_updates
    }
}

#[derive(Debug)]
pub struct WorkingState<'a, S>
where
    S: StateView,
{
    base: &'a S,
    account_updates: BTreeMap<Address, Account>,
}

impl<'a, S> WorkingState<'a, S>
where
    S: StateView,
{
    pub fn new(base: &'a S) -> Self {
        Self {
            base,
            account_updates: BTreeMap::new(),
        }
    }

    pub fn into_delta(self) -> StateDelta {
        StateDelta {
            account_updates: self.account_updates,
        }
    }
}

impl<S> StateView for WorkingState<'_, S>
where
    S: StateView,
{
    fn all_accounts_in_order(&self) -> impl Iterator<Item = (Address, Account)> {
        let mut merged = BTreeMap::new();

        for (address, account) in self.base.all_accounts_in_order() {
            merged.insert(address, account);
        }

        for (address, account) in &self.account_updates {
            merged.insert(*address, *account);
        }

        merged.into_iter()
    }

    fn get_account(&self, address: &Address) -> Account {
        self.account_updates
            .get(address)
            .copied()
            .unwrap_or_else(|| self.base.get_account(address))
    }
}

impl<S> WritableState for WorkingState<'_, S>
where
    S: StateView,
{
    fn apply_delta(&mut self, delta: StateDelta) {
        for (address, account) in delta.into_account_updates() {
            self.account_updates.insert(address, account);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{State, StateDelta, StateView, WorkingState, WritableState};
    use crate::{account::Account, address::Address};
    use std::collections::BTreeMap;

    fn address(seed: u8) -> Address {
        Address::new([seed; 32])
    }

    #[test]
    fn state_normalizes_empty_accounts() {
        let state = State::new(BTreeMap::from([
            (address(1), Account::new(10, 0)),
            (address(2), Account::new(0, 0)),
        ]));

        assert_eq!(state.accounts().len(), 1);
        assert_eq!(state.get_account(&address(1)), Account::new(10, 0));
        assert_eq!(state.get_account(&address(2)), Account::new(0, 0));
    }

    #[test]
    fn apply_delta_updates_and_removes_accounts() {
        let mut state = State::new(BTreeMap::from([
            (address(1), Account::new(10, 0)),
            (address(2), Account::new(5, 1)),
        ]));
        let mut delta = StateDelta::new();
        delta.update_account(address(1), Account::new(8, 1));
        delta.update_account(address(2), Account::new(0, 0));
        delta.update_account(address(3), Account::new(7, 0));

        state.apply_delta(delta);

        assert_eq!(state.get_account(&address(1)), Account::new(8, 1));
        assert_eq!(state.get_account(&address(2)), Account::new(0, 0));
        assert_eq!(state.get_account(&address(3)), Account::new(7, 0));
        assert_eq!(state.accounts().len(), 2);
    }

    #[test]
    fn working_state_reads_overlay_before_base() {
        let base = State::new(BTreeMap::from([
            (address(1), Account::new(10, 0)),
            (address(3), Account::new(30, 0)),
        ]));
        let mut working = WorkingState::new(&base);
        let mut delta = StateDelta::new();
        delta.update_account(address(1), Account::new(9, 1));
        delta.update_account(address(2), Account::new(20, 0));
        delta.update_account(address(3), Account::new(0, 0));

        working.apply_delta(delta);

        let merged = working.all_accounts_in_order().collect::<Vec<_>>();

        assert_eq!(working.get_account(&address(1)), Account::new(9, 1));
        assert_eq!(working.get_account(&address(2)), Account::new(20, 0));
        assert_eq!(working.get_account(&address(3)), Account::new(0, 0));
        assert_eq!(
            merged,
            vec![
                (address(1), Account::new(9, 1)),
                (address(2), Account::new(20, 0)),
                (address(3), Account::new(0, 0)),
            ]
        );
    }
}
