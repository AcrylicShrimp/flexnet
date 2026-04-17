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

    pub fn merge(lhs: Self, rhs: Self) -> Self {
        let mut result = Self::new();

        result.account_updates.extend(lhs.account_updates);
        result.account_updates.extend(rhs.account_updates);

        result
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StateDeltaOverlay<'s, S>
where
    S: StateView,
{
    base: &'s S,
    delta: &'s StateDelta,
}

impl<'s, S> StateDeltaOverlay<'s, S>
where
    S: StateView,
{
    pub fn new(base: &'s S, delta: &'s StateDelta) -> Self {
        Self { base, delta }
    }
}

impl<'s, S> StateView for StateDeltaOverlay<'s, S>
where
    S: StateView,
{
    fn all_accounts_in_order(&self) -> impl Iterator<Item = (Address, Account)> {
        let mut base = self.base.all_accounts_in_order().peekable();
        let mut delta = self
            .delta
            .account_updates()
            .iter()
            .map(|(address, account)| (*address, *account))
            .peekable();

        std::iter::from_fn(move || {
            match (base.peek().copied(), delta.peek().copied()) {
                (None, None) => None,
                (Some(_), None) => base.next(),
                (None, Some(_)) => delta.next(),
                (Some((base_addr, _)), Some((delta_addr, _))) => {
                    if base_addr < delta_addr {
                        return base.next();
                    }

                    if base_addr > delta_addr {
                        return delta.next();
                    }

                    // same address: delta overrides base
                    let _ = base.next();
                    delta.next()
                }
            }
        })
    }

    fn get_account(&self, address: &Address) -> Account {
        self.delta
            .account_updates()
            .get(address)
            .copied()
            .unwrap_or_else(|| self.base.get_account(address))
    }
}

#[cfg(test)]
mod tests {
    use super::{StateDelta, StateView, WritableState};
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
