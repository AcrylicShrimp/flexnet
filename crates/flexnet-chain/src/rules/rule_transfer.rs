use crate::{
    chain_config::ChainConfig,
    chain_id::ChainId,
    chain_version::ChainVersion,
    crypto::VerifyError,
    state::{StateDelta, StateView},
    transactions::tx_transfer::TxTransfer,
};
use thiserror::Error;

#[derive(Error, Debug)]
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

#[derive(Error, Debug)]
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
