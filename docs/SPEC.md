# Flexnet Specification

## Overview

This document describes the Flexnet specification.

## Constants

- `CURRENT_CHAIN_ID`: a chain id (2 bytes), currently 1
- `CURRENT_CHAIN_VERSION`: a chain version (2 bytes), currently 1
- `MAX_TRANSACTIONS_PER_BLOCK`: a maximum number of transactions per block (2 bytes), currently 1024
- `ADDRESS_LENGTH`: 32 bytes
- `SIGNATURE_LENGTH`: 64 bytes
- `HASH_FUNCTION`: sha256
- `SIGNATURE_SCHEME`: ed25519

## Data Models

### Address

- `Address` is the 32-byte Ed25519 public key.

### Account

- `balance`: `u128`
- `nonce`: `u128`
- `nonce` is the next expected nonce for outgoing transactions from the account.

### State

- `State` is a map from `Address` to `Account`.
- An address not present in the map is treated as an implicit account with:
  - `balance` = 0
  - `nonce` = 0

### Block

A block is a collection of transactions, represented as:

1. `chain_id`: a chain id (2 bytes)
2. `chain_version`: a chain version (2 bytes)
3. `block_height`: a height (16 bytes)
4. `previous_block_hash`: a hash of the previous block (32 bytes)
5. `state_hash`: a hash of the state after all transactions are executed (32 bytes)
6. `transactions`: a list of transactions (Transaction[], up to `MAX_TRANSACTIONS_PER_BLOCK` transactions)

- `chain_id` and `chain_version` must be `CURRENT_CHAIN_ID` and `CURRENT_CHAIN_VERSION` respectively
- if the `block_height` is 0 (genesis block), the `previous_block_hash` must be 0x0000000000000000000000000000000000000000000000000000000000000000
- if the `block_height` is greater than 0, the `previous_block_hash` must be the hash of the canonical previous block
- transactions are executed in order, and the result of each transaction is the input of the next transaction
- if any transaction is invalid, the block is invalid and the transactions are not executed

The `state_hash` is computed as follows:

1. sort all addresses in the state by their byte representation
2. filter out addresses with balance = 0 and nonce = 0 (implicit accounts and empty accounts are explicitly excluded)
3. prepare initial hash value: `hash = 0x0000000000000000000000000000000000000000000000000000000000000000 (32 bytes)`
4. for each address, compute the hash of the account: `hash = sha256(hash (32 bytes) || address (32 bytes) || balance (16 bytes) || nonce (16 bytes))`
5. the final hash is the `state_hash`

The `state_hash` must be equal to the state hash after applying all transactions in the block in that order from the state of the canonical previous block.

A block hash is computed as follows:

```
transactions_hash = sha256(number of transactions (2 bytes) || for each transaction: chain_id (2 bytes) || chain_version (2 bytes) || from (32 bytes) || to (32 bytes) || amount (16 bytes) || nonce (16 bytes) || signature (64 bytes) concatenated in order of appearance)
hash = sha256(chain_id (2 bytes) || chain_version (2 bytes) || block_height (16 bytes) || previous_block_hash (32 bytes) || state_hash (32 bytes) || transactions_hash (32 bytes))
```

- All integer fields are serialized in little-endian.
- Byte array fields (Address, hashes, signatures) are serialized as-is, in field order.

### Initial State

The initial state is an ordered map of addresses to accounts.

## Operations

### Transfer

A transfer operation is a message that transfers a amount of tokens from one address to another, represented as:

1. `chain_id`: a chain id (2 bytes)
2. `chain_version`: a chain version (2 bytes)
3. `from`: a from address of Address (32 bytes)
4. `to`: a to address of Address (32 bytes)
5. `amount`: a amount of tokens (16 bytes)
6. `nonce`: a nonce (16 bytes)
7. `signature`: a signature of the transaction of ed25519 (64 bytes)

And this operation works as follows:

```rust
pub fn verify_transfer(chain_id: u16, chain_version: u16, from: Address, to: Address, amount: u128, nonce: u128, signature: [u8; 64]) -> Result<(), TransferError> {
  if chain_id != CURRENT_CHAIN_ID {
    return Err(TransferError::InvalidChainId);
  }
  if chain_version != CURRENT_CHAIN_VERSION {
    return Err(TransferError::InvalidChainVersion);
  }

  if from == to {
    return Err(TransferError::UnableToTransferToSelf);
  }
  if amount == 0 {
    return Err(TransferError::InvalidAmount);
  }

  let from_account = get_account(from);
  let to_account = get_account(to);

  if from_account.nonce != nonce {
    return Err(TransferError::InvalidNonce);
  }

  if from_account.nonce.checked_add(1).is_none() {
    return Err(TransferError::NonceOverflow);
  }

  if from_account.balance.checked_sub(amount).is_none() {
    return Err(TransferError::InsufficientBalance);
  }

  if to_account.balance.checked_add(amount).is_none() {
    return Err(TransferError::BalanceOverflow);
  }

  if verify_signature(chain_id, chain_version, from, to, amount, nonce, signature) != true {
    return Err(TransferError::InvalidSignature);
  }

  return Ok(());
}
```

```rust
/// No need to call `verify_transfer` in order to call this function; `verify_transfer` is called internally in this function
pub fn apply_transfer(chain_id: u16, chain_version: u16, from: Address, to: Address, amount: u128, nonce: u128, signature: [u8; 64]) -> Result<StateDelta, TransferError> {
  verify_transfer(chain_id, chain_version, from, to, amount, nonce, signature)?;

  let mut from_account = get_account(from);
  let mut to_account = get_account(to);

  from_account.balance -= amount;
  from_account.nonce += 1;
  to_account.balance += amount;

  return StateDelta {
    accounts: BTreeMap::from_iter([(from, from_account), (to, to_account)]),
  }
}
```

Notes:

- `from` and `to` must be different addresses
- `amount` must be greater than 0 (zero amount is not allowed)
- `get_account` is a function that returns the account of an address; returns empty account (but not saved to the state) if the address is not found (balance = 0, nonce = 0)
- `checked_sub` and `checked_add` are functions that return `Some(value)` if the operation is successful, `None` if the operation would overflow
- `verify_signature` is a function that verifies the signature of a transaction; returns true if the signature is valid, false otherwise
- `StateDelta` overrides the accounts of the state, and the rest of the state is not changed

### Signature Verification

Signature targets are:

```
unsigned_payload = chain_id || chain_version || from || to || amount || nonce
```

- All integer fields are serialized in little-endian.
- Byte array fields (Address, hashes, signatures) are serialized as-is, in field order.

signature must be a valid Ed25519 signature over the unsigned payload, verified with `from` as the public key.

## Genesis State

All node must start with the same genesis state, as represented by the following JSON:

```json
{
  "chain_id": 1,
  "chain_version": 1,
  "block_height": 0,
  "previous_block_hash": "0x0000000000000000000000000000000000000000000000000000000000000000",
  "state": {
    "accounts": {
      "<alice_address>": {
        "balance": 1000,
        "nonce": 0
      },
      "<bob_address>": {
        "balance": 9999,
        "nonce": 0
      }
    }
  }
}
```

### Genesis Block

- A genesis block is a block with:
  - `chain_id` = `CURRENT_CHAIN_ID`
  - `chain_version` = `CURRENT_CHAIN_VERSION`
  - `block_height` = 0
  - `previous_block_hash` = 0x0000000000000000000000000000000000000000000000000000000000000000
  - `transactions` = empty list

- The genesis block must assume that the state is initialized by the genesis state
- The `transactions` MUST be empty
- The `state_hash` of the genesis block must be the same as the `state_hash` of the genesis state
