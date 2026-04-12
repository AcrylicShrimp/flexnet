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
    use super::{StateDelta, StateView, WorkingState, WritableState};
    use crate::{account::Account, address::Address};
    use std::collections::BTreeMap;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestState {
        accounts: BTreeMap<Address, Account>,
    }

    impl TestState {
        fn new(accounts: BTreeMap<Address, Account>) -> Self {
            Self { accounts }
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

    fn address(seed: u8) -> Address {
        Address::new([seed; 32])
    }

    #[test]
    fn state_delta_starts_empty_and_tracks_updates() {
        let mut delta = StateDelta::new();

        assert!(delta.is_empty());

        delta.update_account(address(1), Account::new(8, 1));

        assert!(!delta.is_empty());
        assert_eq!(delta.account_updates().len(), 1);
    }

    #[test]
    fn working_state_reads_overlay_before_base() {
        let base = TestState::new(BTreeMap::from([
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

    #[test]
    fn local_test_state_can_apply_delta() {
        let mut state = TestState::new(BTreeMap::from([
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
    }
}
