use thiserror::Error;

use crate::hash::Hash;

#[derive(Debug, Error)]
pub enum HexEncodingError {
    #[error("hex value must start with 0x")]
    MissingPrefix,
    #[error("invalid hex length: expected {expected} bytes, got {actual}")]
    InvalidLength { expected: usize, actual: usize },
    #[error("invalid hex: {0}")]
    InvalidHex(#[from] hex::FromHexError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum TransferError {
    #[error("invalid chain id")]
    InvalidChainId,
    #[error("invalid chain version")]
    InvalidChainVersion,
    #[error("unable to transfer to self")]
    UnableToTransferToSelf,
    #[error("invalid amount")]
    InvalidAmount,
    #[error("invalid nonce")]
    InvalidNonce,
    #[error("nonce overflow")]
    NonceOverflow,
    #[error("insufficient balance")]
    InsufficientBalance,
    #[error("balance overflow")]
    BalanceOverflow,
    #[error("invalid signature")]
    InvalidSignature,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BlockError {
    #[error("invalid chain id")]
    InvalidChainId,
    #[error("invalid chain version")]
    InvalidChainVersion,
    #[error("too many transactions: received {count}, maximum allowed is {max}")]
    TooManyTransactions { count: usize, max: usize },
    #[error("genesis block must use the zero previous block hash")]
    InvalidGenesisPreviousBlockHash,
    #[error("genesis block must be validated through the genesis flow")]
    UnexpectedGenesisBlock,
    #[error("previous block hash mismatch: expected {expected}, got {actual}")]
    PreviousBlockHashMismatch { expected: Hash, actual: Hash },
    #[error("transaction at index {index} is invalid: {error}")]
    Transaction { index: usize, error: TransferError },
    #[error("state hash mismatch: expected {expected}, got {actual}")]
    InvalidStateHash { expected: Hash, actual: Hash },
}

#[derive(Debug, Error)]
pub enum ChainError {
    #[error("invalid genesis: {0}")]
    Genesis(#[from] GenesisError),
    #[error("unexpected block height: expected {expected}, got {actual}")]
    UnexpectedBlockHeight { expected: u128, actual: u128 },
    #[error(transparent)]
    Block(#[from] BlockError),
}

#[derive(Debug, Error)]
pub enum GenesisError {
    #[error("invalid genesis json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid chain id")]
    InvalidChainId,
    #[error("invalid chain version")]
    InvalidChainVersion,
    #[error("genesis block height must be 0")]
    InvalidBlockHeight,
    #[error("genesis previous block hash must be zero")]
    InvalidPreviousBlockHash,
}
