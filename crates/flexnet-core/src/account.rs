use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
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
