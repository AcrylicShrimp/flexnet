use crate::{
    account::Account,
    address::Address,
    block::Block,
    state::{StateDelta, StateDeltaOverlay, StateView},
    transaction::Transaction,
};
use sha2::{Digest, Sha256};
use std::fmt::{Debug, Display};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hash([u8; 32]);

impl Hash {
    pub const ZERO: Self = Self([0; 32]);

    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn compute(data: &[u8]) -> Self {
        Self::new(Sha256::digest(data).into())
    }
}

impl Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{}", hex::encode(self.0))
    }
}

impl Debug for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Hash({})", self)
    }
}

pub fn encode_transactions_hash_preimage(transactions: &[Transaction]) -> Vec<u8> {
    let transaction_count =
        u16::try_from(transactions.len()).expect("transaction count exceeds canonical u16 range");
    let mut preimage_size = 2;

    for transaction in transactions {
        preimage_size += transaction.encoded_len();
    }

    let mut preimage = Vec::with_capacity(preimage_size);
    preimage.extend_from_slice(&transaction_count.to_le_bytes());

    for transaction in transactions {
        transaction.encode_canonical(&mut preimage);
    }

    preimage
}

pub fn compute_transactions_hash(transactions: &[Transaction]) -> Hash {
    Hash::compute(&encode_transactions_hash_preimage(transactions))
}

pub fn encode_state_hash_preimage(rolling: &Hash, address: &Address, account: &Account) -> Vec<u8> {
    let preimage_size = 32 + 32 + 16 + 16;

    let mut preimage = Vec::with_capacity(preimage_size);
    preimage.extend_from_slice(rolling.as_bytes());
    preimage.extend_from_slice(address.as_bytes());
    preimage.extend_from_slice(&account.balance.to_le_bytes());
    preimage.extend_from_slice(&account.nonce.to_le_bytes());
    preimage
}

pub fn compute_state_hash<S>(state: &S) -> Hash
where
    S: StateView,
{
    let mut rolling = Hash::ZERO;

    for (address, account) in state
        .all_accounts_in_order()
        .filter(|(_, account)| !account.is_empty())
    {
        rolling = Hash::compute(&encode_state_hash_preimage(&rolling, &address, &account));
    }

    rolling
}

pub fn compute_state_hash_from_delta<S>(state: &S, delta: &StateDelta) -> Hash
where
    S: StateView,
{
    let mut rolling = Hash::ZERO;
    let overlay = StateDeltaOverlay::new(state, delta);

    for (address, account) in overlay
        .all_accounts_in_order()
        .filter(|(_, account)| !account.is_empty())
    {
        rolling = Hash::compute(&encode_state_hash_preimage(&rolling, &address, &account));
    }

    rolling
}

pub fn encode_block_hash_preimage(block: &Block) -> Vec<u8> {
    let preimage_size = 2 + 2 + 16 + 32 + 32 + 32;
    let transactions_hash = compute_transactions_hash(&block.transactions);

    let mut preimage = Vec::with_capacity(preimage_size);
    preimage.extend_from_slice(&block.chain_id.into_u16().to_le_bytes());
    preimage.extend_from_slice(&block.chain_version.into_u16().to_le_bytes());
    preimage.extend_from_slice(&block.block_height.to_le_bytes());
    preimage.extend_from_slice(block.previous_block_hash.as_bytes());
    preimage.extend_from_slice(block.state_hash.as_bytes());
    preimage.extend_from_slice(transactions_hash.as_bytes());
    preimage
}

pub fn compute_block_hash(block: &Block) -> Hash {
    Hash::compute(&encode_block_hash_preimage(block))
}

#[cfg(test)]
mod tests {
    use super::{
        Hash, compute_block_hash, compute_state_hash, compute_transactions_hash,
        encode_transactions_hash_preimage,
    };
    use crate::{
        account::Account,
        address::Address,
        block::Block,
        chain_id::ChainId,
        chain_version::ChainVersion,
        crypto::{SecretKey, address_from_secret_key, sign},
        state::{StateDelta, StateView, WritableState},
        transaction::Transaction,
        transactions::tx_transfer::{TransferPayload, TxTransfer},
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

    fn state_with_accounts(accounts: &[(Address, Account)]) -> TestState {
        TestState::new(BTreeMap::from_iter(accounts.iter().copied()))
    }

    fn transfer(seed: u8, to_seed: u8, amount: u128, nonce: u128) -> Transaction {
        let secret_key = SecretKey::new([seed; 32]);
        let from = address_from_secret_key(&secret_key);
        let to = Address::new([to_seed; 32]);
        let payload = TransferPayload::new(
            ChainId::new(1),
            ChainVersion::new(1),
            from,
            to,
            amount,
            nonce,
        );
        let mut signing_payload = Vec::with_capacity(payload.signing_payload_len());
        payload.encode_signing_payload(&mut signing_payload);
        Transaction::Transfer(TxTransfer::new(
            payload,
            sign(&secret_key, &signing_payload),
        ))
    }

    #[test]
    fn state_hash_ignores_empty_accounts() {
        let a = Address::new([1; 32]);
        let b = Address::new([2; 32]);
        let left = state_with_accounts(&[(a, Account::new(10, 0)), (b, Account::new(0, 0))]);
        let right = state_with_accounts(&[(a, Account::new(10, 0))]);

        assert_eq!(compute_state_hash(&left), compute_state_hash(&right));
    }

    #[test]
    fn transactions_hash_preimage_starts_with_u16_count() {
        let txs = vec![transfer(1, 2, 5, 0), transfer(2, 3, 3, 0)];
        let preimage = encode_transactions_hash_preimage(&txs);

        assert_eq!(&preimage[..2], &2_u16.to_le_bytes());
        assert_eq!(compute_transactions_hash(&txs), Hash::compute(&preimage));
    }

    #[test]
    fn block_hash_changes_when_transactions_change() {
        let base_block = Block::new(
            ChainId::new(1),
            ChainVersion::new(1),
            1,
            Hash::ZERO,
            compute_state_hash(&state_with_accounts(&[(
                Address::new([1; 32]),
                Account::new(10, 0),
            )])),
            vec![transfer(1, 2, 5, 0)],
        );
        let changed_block = Block::new(
            base_block.chain_id,
            base_block.chain_version,
            base_block.block_height,
            base_block.previous_block_hash,
            base_block.state_hash,
            vec![transfer(1, 2, 6, 0)],
        );

        assert_ne!(
            compute_block_hash(&base_block),
            compute_block_hash(&changed_block)
        );
    }
}
