# Flexnet Chain Specification v0

## 1. Overview

This document defines the deterministic chain engine for Flexnet.

The chain engine is responsible for:

- representing chain state
- representing blocks and transactions
- validating transactions
- applying transactions to state
- validating blocks
- applying blocks to state
- computing canonical hashes for state, transactions, and blocks

Consensus, networking, peer discovery, and message transport are out of scope.

---

## 2. Constants

The following protocol constants are fixed for this version of the specification:

- `CURRENT_CHAIN_ID`: `u16`, currently `1`
- `CURRENT_CHAIN_VERSION`: `u16`, currently `1`
- `MAX_TRANSACTIONS_PER_BLOCK`: `u16`, currently `1024`
- `ADDRESS_LENGTH`: `32` bytes
- `SIGNATURE_LENGTH`: `64` bytes
- `HASH_LENGTH`: `32` bytes
- `HASH_FUNCTION`: `SHA-256`
- `SIGNATURE_SCHEME`: `Ed25519`

---

## 3. Serialization Rules

All canonical encodings in this specification follow these rules:

- all integer fields are serialized in little-endian
- all byte array fields are serialized as-is
- all fields are serialized in the exact order defined by this specification
- canonical encodings must not include padding, field names, or optional metadata unless explicitly defined

These rules apply to:

- transaction signing payloads
- transaction canonical encodings
- transaction hash computation
- state hash computation
- block hash computation

---

## 4. Primitive Types

### 4.1 Address

`Address` is the 32-byte Ed25519 public key.

### 4.2 Signature

`Signature` is a 64-byte Ed25519 signature.

### 4.3 Hash

`Hash` is a 32-byte SHA-256 digest.

---

## 5. State Model

### 5.1 Account

An account is defined as:

- `balance: u128`
- `nonce: u128`

`nonce` is the next expected nonce for outgoing transactions from the account.

### 5.2 State

`State` is a map from `Address` to `Account`.

If an address is not present in the map, it is treated as an implicit account with:

- `balance = 0`
- `nonce = 0`

Implicit accounts are not stored in the state map unless explicitly materialized by a state transition.

### 5.3 Empty Accounts

Accounts with:

- `balance = 0`
- `nonce = 0`

are considered empty accounts.

Empty accounts and implicit accounts are treated equivalently for `state_hash` computation.

---

## 6. State Hash

`state_hash` is a canonical digest of the full current state.

It is computed as follows:

1. collect all stored accounts
2. exclude all accounts whose `balance = 0` and `nonce = 0`
3. sort remaining accounts by address byte representation in ascending order
4. initialize:

   `hash = 0x0000000000000000000000000000000000000000000000000000000000000000`

5. for each account in sorted order, update:

   `hash = sha256(hash || address || balance || nonce)`

6. the final value is the `state_hash`

Where:

- `hash` is 32 bytes
- `address` is 32 bytes
- `balance` is 16 bytes
- `nonce` is 16 bytes

---

## 7. Transaction Model

### 7.1 Transaction

A transaction is a chain operation that may be included in a block.

A transaction consists of:

- `kind: u8`
- `body: operation-specific payload`
- `authorization: operation-specific authorization data`

The meaning and validation rules of `body` and `authorization` depend on the transaction kind.

Authorization rules are defined per transaction kind, not globally for all transactions.

### 7.2 Transaction Kinds

The following transaction kinds are currently defined:

- `0x01`: `Transfer`

Transaction kind values are part of canonical transaction encoding and must be included in all relevant signing and hashing rules.

---

## 8. Transfer Transaction

### 8.1 Transfer Kind

A `Transfer` transaction has:

- `kind = 0x01`
- `body = TransferPayload`
- `authorization = one Ed25519 signature by the address in `from``

### 8.2 TransferPayload

`TransferPayload` consists of:

- `chain_id: u16`
- `chain_version: u16`
- `from: Address`
- `to: Address`
- `amount: u128`
- `nonce: u128`

### 8.3 Transfer Authorization

Transfer authorization consists of:

- `signature: Signature`

### 8.4 Transfer Signing Payload

The canonical unsigned payload for a `Transfer` transaction is:

`transaction_kind || chain_id || chain_version || from || to || amount || nonce`

Encoded as:

- `transaction_kind`: 1 byte
- `chain_id`: 2 bytes
- `chain_version`: 2 bytes
- `from`: 32 bytes
- `to`: 32 bytes
- `amount`: 16 bytes
- `nonce`: 16 bytes

Total length: `101` bytes.

The signature must be a valid Ed25519 signature over this exact payload, verified with `from` as the verifying public key.

### 8.5 Canonical Transaction Encoding

The canonical encoding of a `Transfer` transaction is:

`transaction_kind || chain_id || chain_version || from || to || amount || nonce || signature`

Encoded as:

- `transaction_kind`: 1 byte
- `chain_id`: 2 bytes
- `chain_version`: 2 bytes
- `from`: 32 bytes
- `to`: 32 bytes
- `amount`: 16 bytes
- `nonce`: 16 bytes
- `signature`: 64 bytes

Total length: `165` bytes.

---

## 9. Transfer Validation

A `Transfer` transaction is valid if and only if all of the following conditions hold:

1. `chain_id == CURRENT_CHAIN_ID`
2. `chain_version == CURRENT_CHAIN_VERSION`
3. `from != to`
4. `amount > 0`
5. the signature is a valid Ed25519 signature by `from` over the canonical transfer signing payload
6. `from_account.nonce == nonce`
7. `from_account.nonce + 1` does not overflow `u128`
8. `from_account.balance - amount` does not underflow `u128`
9. `to_account.balance + amount` does not overflow `u128`

Where:

- `from_account` is the stored account for `from`, or the implicit empty account if not present
- `to_account` is the stored account for `to`, or the implicit empty account if not present

---

## 10. Transfer State Transition

Applying a valid `Transfer` transaction produces the following state transition:

- `from.balance = from.balance - amount`
- `from.nonce = from.nonce + 1`
- `to.balance = to.balance + amount`

No other state entries are modified.

If `to` was previously implicit, it becomes materialized in state after the transfer.

If any validation rule fails, no state changes are applied.

---

## 11. Transaction Hashing

A block does not hash raw transaction lists directly.

Instead, it first computes a `transactions_hash`:

`transactions_hash = sha256(transaction_count || tx_0 || tx_1 || ... || tx_n)`

Where:

- `transaction_count` is `u16`
- each `tx_i` is the canonical transaction encoding of the i-th transaction
- transactions are concatenated in order of appearance in the block

---

## 12. Block Model

A block consists of:

- `chain_id: u16`
- `chain_version: u16`
- `block_height: u128`
- `previous_block_hash: Hash`
- `state_hash: Hash`
- `transactions: Vec<Transaction>`

### 12.1 Block Validity Rules

A block is valid if and only if all of the following conditions hold:

1. `chain_id == CURRENT_CHAIN_ID`
2. `chain_version == CURRENT_CHAIN_VERSION`
3. `transactions.len() <= MAX_TRANSACTIONS_PER_BLOCK`
4. if `block_height == 0`, then `previous_block_hash == 0x00..00` (32 zero bytes)
5. if `block_height > 0`, then `previous_block_hash` equals the canonical hash of the canonical previous block
6. all transactions are valid when executed sequentially from the previous canonical state
7. transactions are executed in order
8. if any transaction is invalid, the block is invalid and no transaction in the block is applied
9. the block's `state_hash` must equal the `state_hash` of the resulting state after all transactions are applied in order

### 12.2 Block Hash

Block hashing is performed in two steps.

First:

`transactions_hash = sha256(transaction_count || canonical_transaction_0 || canonical_transaction_1 || ... )`

Then:

`block_hash = sha256(chain_id || chain_version || block_height || previous_block_hash || state_hash || transactions_hash)`

Encoded as:

- `chain_id`: 2 bytes
- `chain_version`: 2 bytes
- `block_height`: 16 bytes
- `previous_block_hash`: 32 bytes
- `state_hash`: 32 bytes
- `transactions_hash`: 32 bytes

Total preimage length: `116` bytes.

---

## 13. Initial State

The initial state is a protocol-defined ordered map of addresses to accounts.

All nodes must start from the same initial state.

---

## 14. Genesis State

All nodes must be initialized with the same genesis state.

Example:

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

---

## 15. Genesis Block

The genesis block is defined as:

- `chain_id = CURRENT_CHAIN_ID`
- `chain_version = CURRENT_CHAIN_VERSION`
- `block_height = 0`
- `previous_block_hash = 0x0000000000000000000000000000000000000000000000000000000000000000`
- `transactions = empty list`
- `state_hash = state_hash(genesis_state)`

The genesis block assumes that the state before block execution is exactly the genesis state.

The genesis block contains no transactions.

---

## 16. Execution Rules

### 16.1 Transaction Execution

Transactions in a block must be executed strictly in order.

The output state of transaction `i` is the input state of transaction `i + 1`.

### 16.2 Atomic Block Application

Block application is atomic.

If any transaction in a block is invalid, then:

- the block is invalid
- no transaction in the block is applied
- the resulting state is unchanged

### 16.3 Canonical Chain State

The canonical chain state at height `H` is defined as:

- the genesis state, followed by
- sequential application of all canonical blocks from height `0` through height `H`

---

## 17. Out of Scope

The following are out of scope for this specification version:

- consensus protocol
- peer-to-peer networking
- mempool policy
- persistent storage
- validator set changes
- minting, burning, and system transactions
- multisignature authorization
- time-based operations
- smart contracts
