#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Account {
    pub balance: u128,
    pub nonce: u128,
}

impl Account {
    pub const fn new(balance: u128, nonce: u128) -> Self {
        Self { balance, nonce }
    }

    pub const fn is_empty(&self) -> bool {
        self.balance == 0 && self.nonce == 0
    }
}

#[cfg(test)]
mod tests {
    use super::Account;

    #[test]
    fn empty_account_detection_matches_balance_and_nonce() {
        assert!(Account::new(0, 0).is_empty());
        assert!(!Account::new(1, 0).is_empty());
        assert!(!Account::new(0, 1).is_empty());
    }
}
