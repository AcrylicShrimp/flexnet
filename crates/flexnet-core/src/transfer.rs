use std::collections::BTreeMap;

use crate::{
    address::Address,
    codec::{EncodeCanonical, append_fixed, append_u16_le, append_u128_le},
    constants::{CURRENT_CHAIN_ID, CURRENT_CHAIN_VERSION, SIGNATURE_LENGTH},
    error::TransferError,
    signature::verify_transfer_signature,
    state::{State, StateDelta},
};

pub type SignatureBytes = [u8; SIGNATURE_LENGTH];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Transfer {
    pub chain_id: u16,
    pub chain_version: u16,
    pub from: Address,
    pub to: Address,
    pub amount: u128,
    pub nonce: u128,
    pub signature: SignatureBytes,
}

impl Transfer {
    pub const fn new(
        chain_id: u16,
        chain_version: u16,
        from: Address,
        to: Address,
        amount: u128,
        nonce: u128,
        signature: SignatureBytes,
    ) -> Self {
        Self {
            chain_id,
            chain_version,
            from,
            to,
            amount,
            nonce,
            signature,
        }
    }

    pub const fn unsigned(
        chain_id: u16,
        chain_version: u16,
        from: Address,
        to: Address,
        amount: u128,
        nonce: u128,
    ) -> Self {
        Self::new(
            chain_id,
            chain_version,
            from,
            to,
            amount,
            nonce,
            [0; SIGNATURE_LENGTH],
        )
    }

    pub const fn with_signature(mut self, signature: SignatureBytes) -> Self {
        self.signature = signature;
        self
    }

    pub fn signing_view(&self) -> TransferSigningView<'_> {
        TransferSigningView { transfer: self }
    }

    pub fn bytes_view(&self) -> TransferBytesView<'_> {
        TransferBytesView { transfer: self }
    }

    pub fn verify(&self, state: &State) -> Result<(), TransferError> {
        if self.chain_id != CURRENT_CHAIN_ID {
            return Err(TransferError::InvalidChainId);
        }
        if self.chain_version != CURRENT_CHAIN_VERSION {
            return Err(TransferError::InvalidChainVersion);
        }
        if self.from == self.to {
            return Err(TransferError::UnableToTransferToSelf);
        }
        if self.amount == 0 {
            return Err(TransferError::InvalidAmount);
        }

        let from_account = state.get_account(self.from);
        let to_account = state.get_account(self.to);

        if from_account.nonce != self.nonce {
            return Err(TransferError::InvalidNonce);
        }
        if from_account.nonce.checked_add(1).is_none() {
            return Err(TransferError::NonceOverflow);
        }
        if from_account.balance.checked_sub(self.amount).is_none() {
            return Err(TransferError::InsufficientBalance);
        }
        if to_account.balance.checked_add(self.amount).is_none() {
            return Err(TransferError::BalanceOverflow);
        }
        if !verify_transfer_signature(self) {
            return Err(TransferError::InvalidSignature);
        }

        Ok(())
    }

    pub fn apply(&self, state: &State) -> Result<StateDelta, TransferError> {
        self.verify(state)?;

        let mut from_account = state.get_account(self.from);
        let mut to_account = state.get_account(self.to);

        from_account.balance -= self.amount;
        from_account.nonce += 1;
        to_account.balance += self.amount;

        Ok(StateDelta::new(BTreeMap::from([
            (self.from, from_account),
            (self.to, to_account),
        ])))
    }
}

pub struct TransferSigningView<'a> {
    transfer: &'a Transfer,
}

impl EncodeCanonical for TransferSigningView<'_> {
    fn encode_into(&self, out: &mut Vec<u8>) {
        append_u16_le(out, self.transfer.chain_id);
        append_u16_le(out, self.transfer.chain_version);
        append_fixed(out, self.transfer.from.as_bytes());
        append_fixed(out, self.transfer.to.as_bytes());
        append_u128_le(out, self.transfer.amount);
        append_u128_le(out, self.transfer.nonce);
    }
}

pub struct TransferBytesView<'a> {
    transfer: &'a Transfer,
}

impl EncodeCanonical for TransferBytesView<'_> {
    fn encode_into(&self, out: &mut Vec<u8>) {
        self.transfer.signing_view().encode_into(out);
        append_fixed(out, &self.transfer.signature);
    }
}
