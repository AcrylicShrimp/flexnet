use crate::{
    chain_config::ChainConfig,
    codec::{DecodeError, Decoder},
    rules::rule_transfer::{
        TransferExecutionError, TransferVerificationError, execute_transfer,
        verify_transfer_stateless,
    },
    state::{StateDelta, StateView},
    transaction_kind::TransactionKind,
    transactions::tx_transfer::TxTransfer,
};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Transaction {
    Transfer(TxTransfer),
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum TransactionVerificationError {
    #[error("transfer verification failed: {0}")]
    Transfer(#[from] TransferVerificationError),
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum TransactionExecutionError {
    #[error("transfer execution failed: {0}")]
    Transfer(#[from] TransferExecutionError),
}

impl Transaction {
    pub fn kind(&self) -> TransactionKind {
        match self {
            Transaction::Transfer(_) => TransactionKind::Transfer,
        }
    }

    pub fn encoded_len(&self) -> usize {
        let kind = 1;
        let tx = match self {
            Transaction::Transfer(tx) => tx.encoded_len(),
        };

        kind + tx
    }

    pub fn encode_canonical(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.kind().into_u8().to_le_bytes());

        match self {
            Transaction::Transfer(tx) => {
                tx.encode_canonical(out);
            }
        }
    }

    pub fn decode_canonical(input: &[u8]) -> Result<Self, DecodeError> {
        let mut decoder = Decoder::new(input);
        let decoded = Self::decode_from(&mut decoder)?;

        decoder.finish()?;

        Ok(decoded)
    }

    pub fn decode_from(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let kind = TransactionKind::decode_from(decoder)?;
        let transaction = match kind {
            TransactionKind::Transfer => Self::Transfer(TxTransfer::decode_from(decoder)?),
        };

        Ok(transaction)
    }

    pub fn verify_stateless(
        &self,
        config: &ChainConfig,
    ) -> Result<(), TransactionVerificationError> {
        match self {
            Transaction::Transfer(tx) => {
                verify_transfer_stateless(tx, config)?;
            }
        }

        Ok(())
    }

    pub fn execute(
        &self,
        config: &ChainConfig,
        state: &impl StateView,
    ) -> Result<StateDelta, TransactionExecutionError> {
        let delta = match self {
            Transaction::Transfer(tx) => execute_transfer(tx, config, state)?,
        };

        Ok(delta)
    }
}

#[cfg(test)]
mod tests {
    use super::Transaction;
    use crate::{
        address::Address,
        chain_id::ChainId,
        chain_version::ChainVersion,
        crypto::{SecretKey, address_from_secret_key, sign},
        transaction_kind::TransactionKind,
        transactions::tx_transfer::{TransferPayload, TxTransfer},
    };

    #[test]
    fn transaction_roundtrips_and_reports_kind() {
        let secret_key = SecretKey::new([1; 32]);
        let payload = TransferPayload::new(
            ChainId::new(1),
            ChainVersion::new(1),
            address_from_secret_key(&secret_key),
            Address::new([2; 32]),
            5,
            0,
        );
        let mut signing_payload = Vec::with_capacity(payload.signing_payload_len());
        payload.encode_signing_payload(&mut signing_payload);
        let transaction = Transaction::Transfer(TxTransfer::new(
            payload,
            sign(&secret_key, &signing_payload),
        ));
        let mut encoded = Vec::with_capacity(transaction.encoded_len());
        transaction.encode_canonical(&mut encoded);

        let decoded = Transaction::decode_canonical(&encoded).unwrap();

        assert_eq!(decoded, transaction);
        assert_eq!(decoded.kind(), TransactionKind::Transfer);
    }
}
