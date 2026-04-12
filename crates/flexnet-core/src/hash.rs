use std::{fmt, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest, Sha256};

use crate::{
    codec::{
        append_fixed, append_u128_le, decode_hex_array, encode_block_hash_input, encode_hex,
        encode_transactions_hash_input,
    },
    constants::HASH_LENGTH,
    error::HexEncodingError,
    state::State,
    transfer::Transfer,
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hash([u8; HASH_LENGTH]);

impl Hash {
    pub const ZERO: Self = Self([0; HASH_LENGTH]);

    pub const fn new(bytes: [u8; HASH_LENGTH]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; HASH_LENGTH] {
        &self.0
    }
}

impl From<[u8; HASH_LENGTH]> for Hash {
    fn from(value: [u8; HASH_LENGTH]) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&encode_hex(&self.0))
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl FromStr for Hash {
    type Err = HexEncodingError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(decode_hex_array(value)?))
    }
}

impl Serialize for Hash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Hash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::from_str(&value).map_err(serde::de::Error::custom)
    }
}

pub fn hash_bytes(bytes: &[u8]) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Hash::new(hasher.finalize().into())
}

pub fn hash_state(state: &State) -> Hash {
    let mut rolling = Hash::ZERO;

    for (address, account) in state.iter_non_empty() {
        let mut step = Vec::with_capacity(96);
        append_fixed(&mut step, rolling.as_bytes());
        append_fixed(&mut step, address.as_bytes());
        append_u128_le(&mut step, account.balance);
        append_u128_le(&mut step, account.nonce);
        rolling = hash_bytes(&step);
    }

    rolling
}

pub fn hash_transactions(transfers: &[Transfer]) -> Hash {
    hash_bytes(&encode_transactions_hash_input(transfers))
}

pub fn hash_block(block: &crate::block::Block) -> Hash {
    let transactions_hash = hash_transactions(&block.transactions);
    hash_block_with_transactions_hash(block, transactions_hash)
}

pub fn hash_block_with_transactions_hash(
    block: &crate::block::Block,
    transactions_hash: Hash,
) -> Hash {
    hash_bytes(&encode_block_hash_input(block, &transactions_hash))
}
