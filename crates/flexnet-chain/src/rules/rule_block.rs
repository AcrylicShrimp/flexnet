use crate::{
    block::Block,
    chain_config::ChainConfig,
    chain_id::ChainId,
    chain_version::ChainVersion,
    hash::{Hash, compute_state_hash},
    state::{StateDelta, StateView, WorkingState, WritableState},
    transaction::{TransactionExecutionError, TransactionVerificationError},
};
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum BlockVerificationError {
    #[error("invalid chain id: expected {expected}, got {actual}")]
    InvalidChainId { expected: ChainId, actual: ChainId },
    #[error("invalid chain version: expected {expected}, got {actual}")]
    InvalidChainVersion {
        expected: ChainVersion,
        actual: ChainVersion,
    },
    #[error("genesis block must use the zero previous block hash")]
    InvalidGenesisPreviousBlockHash,
    #[error("genesis block must not contain any transactions")]
    NonEmptyTransactionsInGenesisBlock,
    #[error("too many transactions: received {count}, maximum allowed is {max}")]
    TooManyTransactions { count: usize, max: usize },
    #[error("transaction at index {index} verification failed: {error}")]
    TxVerifyError {
        index: usize,
        error: TransactionVerificationError,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionOutcome {
    pub state_delta: StateDelta,
    pub state_hash: Hash,
}

pub fn verify_block_stateless(
    block: &Block,
    config: &ChainConfig,
) -> Result<(), BlockVerificationError> {
    if block.chain_id != config.chain_id {
        return Err(BlockVerificationError::InvalidChainId {
            expected: config.chain_id,
            actual: block.chain_id,
        });
    }

    if block.chain_version != config.chain_version {
        return Err(BlockVerificationError::InvalidChainVersion {
            expected: config.chain_version,
            actual: block.chain_version,
        });
    }

    if block.is_genesis() {
        if block.previous_block_hash != Hash::ZERO {
            return Err(BlockVerificationError::InvalidGenesisPreviousBlockHash);
        }

        if !block.transactions.is_empty() {
            return Err(BlockVerificationError::NonEmptyTransactionsInGenesisBlock);
        }
    }

    let max_transactions = config.max_transactions_per_block.min(u16::MAX as usize);
    if block.transactions.len() > max_transactions {
        return Err(BlockVerificationError::TooManyTransactions {
            count: block.transactions.len(),
            max: max_transactions,
        });
    }

    for (index, transaction) in block.transactions.iter().enumerate() {
        transaction
            .verify_stateless(config)
            .map_err(|error| BlockVerificationError::TxVerifyError { index, error })?;
    }

    Ok(())
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum BlockExecutionError {
    #[error("block verification failed: {0}")]
    VerifyError(#[from] BlockVerificationError),
    #[error("transaction at index {index} execution failed: {error}")]
    TxExecuteError {
        index: usize,
        error: TransactionExecutionError,
    },
    #[error("invalid state hash: expected {expected}, got {actual}")]
    InvalidStateHash { expected: Hash, actual: Hash },
}

pub fn execute_block<S>(
    block: &Block,
    config: &ChainConfig,
    state: &S,
) -> Result<ExecutionOutcome, BlockExecutionError>
where
    S: StateView,
{
    verify_block_stateless(block, config)?;

    let mut working_state = WorkingState::new(state);

    for (index, transaction) in block.transactions.iter().enumerate() {
        let delta = transaction
            .execute(config, &working_state)
            .map_err(|error| BlockExecutionError::TxExecuteError { index, error })?;
        working_state.apply_delta(delta);
    }

    let state_hash = compute_state_hash(&working_state);

    if block.state_hash != state_hash {
        return Err(BlockExecutionError::InvalidStateHash {
            expected: block.state_hash,
            actual: state_hash,
        });
    }

    Ok(ExecutionOutcome {
        state_delta: working_state.into_delta(),
        state_hash,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        BlockExecutionError, BlockVerificationError, execute_block, verify_block_stateless,
    };
    use crate::{
        account::Account,
        address::Address,
        block::Block,
        chain_config::ChainConfig,
        chain_id::ChainId,
        chain_version::ChainVersion,
        crypto::{SecretKey, address_from_secret_key, sign},
        hash::{Hash, compute_state_hash},
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

    fn secret_key(seed: u8) -> SecretKey {
        SecretKey::new([seed; 32])
    }

    fn config() -> ChainConfig {
        ChainConfig {
            chain_id: ChainId::new(1),
            chain_version: ChainVersion::new(1),
            max_transactions_per_block: 16,
        }
    }

    fn state_with_accounts(accounts: &[(Address, Account)]) -> TestState {
        TestState::new(BTreeMap::from_iter(accounts.iter().copied()))
    }

    fn signed_transfer(
        secret_key: &SecretKey,
        to: Address,
        amount: u128,
        nonce: u128,
    ) -> Transaction {
        let from = address_from_secret_key(secret_key);
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
        Transaction::Transfer(TxTransfer::new(payload, sign(secret_key, &signing_payload)))
    }

    #[test]
    fn reject_block_when_transaction_count_exceeds_canonical_limit() {
        let repeated = signed_transfer(
            &secret_key(1),
            address_from_secret_key(&secret_key(2)),
            1,
            0,
        );
        let mut block = Block::new(
            ChainId::new(1),
            ChainVersion::new(1),
            1,
            Hash::ZERO,
            Hash::ZERO,
            Vec::new(),
        );
        block.transactions = vec![repeated; u16::MAX as usize + 1];

        assert_eq!(
            verify_block_stateless(
                &block,
                &ChainConfig {
                    max_transactions_per_block: usize::MAX,
                    ..config()
                }
            ),
            Err(BlockVerificationError::TooManyTransactions {
                count: u16::MAX as usize + 1,
                max: u16::MAX as usize,
            })
        );
    }

    #[test]
    fn execute_block_returns_delta_for_valid_block() {
        let alice_key = secret_key(1);
        let bob_key = secret_key(2);
        let alice = address_from_secret_key(&alice_key);
        let bob = address_from_secret_key(&bob_key);
        let state =
            state_with_accounts(&[(alice, Account::new(100, 0)), (bob, Account::new(5, 0))]);
        let tx = signed_transfer(&alice_key, bob, 40, 0);
        let expected_state = {
            let mut next = state.clone();
            let delta = crate::rules::rule_transfer::execute_transfer(
                match &tx {
                    Transaction::Transfer(tx) => tx,
                },
                &config(),
                &state,
            )
            .unwrap();
            next.apply_delta(delta);
            next
        };
        let block = Block::new(
            ChainId::new(1),
            ChainVersion::new(1),
            1,
            Hash::ZERO,
            compute_state_hash(&expected_state),
            vec![tx],
        );

        let outcome = execute_block(&block, &config(), &state).unwrap();
        let mut next_state = state.clone();
        next_state.apply_delta(outcome.state_delta);

        assert_eq!(outcome.state_hash, compute_state_hash(&expected_state));
        assert_eq!(next_state, expected_state);
    }

    #[test]
    fn reject_block_when_state_hash_is_wrong() {
        let alice_key = secret_key(1);
        let bob = address_from_secret_key(&secret_key(2));
        let state =
            state_with_accounts(&[(address_from_secret_key(&alice_key), Account::new(100, 0))]);
        let tx = signed_transfer(&alice_key, bob, 40, 0);
        let block = Block::new(
            ChainId::new(1),
            ChainVersion::new(1),
            1,
            Hash::ZERO,
            Hash::ZERO,
            vec![tx],
        );

        assert!(matches!(
            execute_block(&block, &config(), &state),
            Err(BlockExecutionError::InvalidStateHash { .. })
        ));
    }
}
