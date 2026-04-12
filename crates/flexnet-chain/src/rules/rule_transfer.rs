use crate::{
    chain_config::ChainConfig,
    chain_id::ChainId,
    chain_version::ChainVersion,
    crypto::VerifyError,
    state::{StateDelta, StateView},
    transactions::tx_transfer::TxTransfer,
};
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum TransferVerificationError {
    #[error("invalid chain id; expected {expected}, got {actual}")]
    InvalidChainId { expected: ChainId, actual: ChainId },
    #[error("invalid chain version; expected {expected}, got {actual}")]
    InvalidChainVersion {
        expected: ChainVersion,
        actual: ChainVersion,
    },
    #[error("unable to transfer to self")]
    UnableToTransferToSelf,
    #[error("amount must be greater than zero")]
    ZeroAmount,
    #[error("invalid signature: {0}")]
    InvalidSignature(VerifyError),
}

pub fn verify_transfer_stateless(
    tx: &TxTransfer,
    config: &ChainConfig,
) -> Result<(), TransferVerificationError> {
    if tx.payload.chain_id != config.chain_id {
        return Err(TransferVerificationError::InvalidChainId {
            expected: config.chain_id,
            actual: tx.payload.chain_id,
        });
    }

    if tx.payload.chain_version != config.chain_version {
        return Err(TransferVerificationError::InvalidChainVersion {
            expected: config.chain_version,
            actual: tx.payload.chain_version,
        });
    }

    if tx.payload.from == tx.payload.to {
        return Err(TransferVerificationError::UnableToTransferToSelf);
    }

    if tx.payload.amount == 0 {
        return Err(TransferVerificationError::ZeroAmount);
    }

    if let Err(err) = tx.verify_signature() {
        return Err(TransferVerificationError::InvalidSignature(err));
    }

    Ok(())
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum TransferExecutionError {
    #[error("transfer verification failed: {0}")]
    VerificationError(#[from] TransferVerificationError),
    #[error("invalid nonce; expected {expected}, got {actual}")]
    InvalidNonce { expected: u128, actual: u128 },
    #[error("nonce overflow")]
    NonceOverflow,
    #[error("insufficient balance; expected at least {amount}, got {balance}")]
    InsufficientBalance { balance: u128, amount: u128 },
    #[error("balance overflow")]
    BalanceOverflow,
}

pub fn execute_transfer(
    tx: &TxTransfer,
    config: &ChainConfig,
    state: &impl StateView,
) -> Result<StateDelta, TransferExecutionError> {
    verify_transfer_stateless(tx, config)?;

    let mut from_account = state.get_account(&tx.payload.from);
    let mut to_account = state.get_account(&tx.payload.to);

    if from_account.nonce != tx.payload.nonce {
        return Err(TransferExecutionError::InvalidNonce {
            expected: from_account.nonce,
            actual: tx.payload.nonce,
        });
    }

    let new_nonce = from_account
        .nonce
        .checked_add(1)
        .ok_or(TransferExecutionError::NonceOverflow)?;

    let new_from_balance = from_account.balance.checked_sub(tx.payload.amount).ok_or(
        TransferExecutionError::InsufficientBalance {
            balance: from_account.balance,
            amount: tx.payload.amount,
        },
    )?;

    let new_to_balance = to_account
        .balance
        .checked_add(tx.payload.amount)
        .ok_or(TransferExecutionError::BalanceOverflow)?;

    from_account.nonce = new_nonce;
    from_account.balance = new_from_balance;
    to_account.balance = new_to_balance;

    let mut state_delta = StateDelta::default();

    state_delta.update_account(tx.payload.from, from_account);
    state_delta.update_account(tx.payload.to, to_account);

    Ok(state_delta)
}

#[cfg(test)]
mod tests {
    use super::{
        TransferExecutionError, TransferVerificationError, execute_transfer,
        verify_transfer_stateless,
    };
    use crate::{
        account::Account,
        address::Address,
        chain_config::ChainConfig,
        chain_id::ChainId,
        chain_version::ChainVersion,
        crypto::{SecretKey, address_from_secret_key, sign},
        state::State,
        transactions::tx_transfer::{TransferPayload, TxTransfer},
    };
    use std::collections::BTreeMap;

    fn config() -> ChainConfig {
        ChainConfig {
            chain_id: ChainId::new(1),
            chain_version: ChainVersion::new(1),
            max_transactions_per_block: 16,
        }
    }

    fn state_with_accounts(accounts: &[(Address, Account)]) -> State {
        State::new(BTreeMap::from_iter(accounts.iter().copied()))
    }

    fn signed_transfer(
        secret_key: &SecretKey,
        to: Address,
        amount: u128,
        nonce: u128,
    ) -> TxTransfer {
        let payload = TransferPayload::new(
            ChainId::new(1),
            ChainVersion::new(1),
            address_from_secret_key(secret_key),
            to,
            amount,
            nonce,
        );
        let mut signing_payload = Vec::with_capacity(payload.signing_payload_len());
        payload.encode_signing_payload(&mut signing_payload);
        TxTransfer::new(payload, sign(secret_key, &signing_payload))
    }

    #[test]
    fn reject_transfer_to_self() {
        let secret_key = SecretKey::new([1; 32]);
        let address = address_from_secret_key(&secret_key);
        let tx = signed_transfer(&secret_key, address, 5, 0);

        assert_eq!(
            verify_transfer_stateless(&tx, &config()),
            Err(TransferVerificationError::UnableToTransferToSelf)
        );
    }

    #[test]
    fn reject_transfer_with_wrong_nonce() {
        let alice_key = SecretKey::new([1; 32]);
        let bob = Address::new([2; 32]);
        let alice = address_from_secret_key(&alice_key);
        let tx = signed_transfer(&alice_key, bob, 5, 3);
        let state = state_with_accounts(&[(alice, Account::new(10, 2))]);

        assert_eq!(
            execute_transfer(&tx, &config(), &state),
            Err(TransferExecutionError::InvalidNonce {
                expected: 2,
                actual: 3,
            })
        );
    }

    #[test]
    fn reject_transfer_with_insufficient_balance() {
        let alice_key = SecretKey::new([1; 32]);
        let bob = Address::new([2; 32]);
        let alice = address_from_secret_key(&alice_key);
        let tx = signed_transfer(&alice_key, bob, 11, 0);
        let state = state_with_accounts(&[(alice, Account::new(10, 0))]);

        assert_eq!(
            execute_transfer(&tx, &config(), &state),
            Err(TransferExecutionError::InsufficientBalance {
                balance: 10,
                amount: 11,
            })
        );
    }
}
