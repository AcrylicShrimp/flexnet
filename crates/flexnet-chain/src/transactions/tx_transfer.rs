use crate::{
    address::Address,
    chain_id::ChainId,
    chain_version::ChainVersion,
    codec::{DecodeError, Decoder},
    crypto::{Signature, VerificationError, verify},
    transaction_kind::TransactionKind,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TransferPayload {
    pub chain_id: ChainId,
    pub chain_version: ChainVersion,
    pub from: Address,
    pub to: Address,
    pub amount: u128,
    pub nonce: u128,
}

impl TransferPayload {
    pub fn new(
        chain_id: ChainId,
        chain_version: ChainVersion,
        from: Address,
        to: Address,
        amount: u128,
        nonce: u128,
    ) -> Self {
        Self {
            chain_id,
            chain_version,
            from,
            to,
            amount,
            nonce,
        }
    }

    pub fn signing_payload_len(&self) -> usize {
        let kind = 1;
        let chain_id = 2;
        let chain_version = 2;
        let from = 32;
        let to = 32;
        let amount = 16;
        let nonce = 16;

        kind + chain_id + chain_version + from + to + amount + nonce
    }

    pub fn encode_signing_payload(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&TransactionKind::Transfer.into_u8().to_le_bytes());
        out.extend_from_slice(&self.chain_id.into_u16().to_le_bytes());
        out.extend_from_slice(&self.chain_version.into_u16().to_le_bytes());
        out.extend_from_slice(self.from.as_bytes());
        out.extend_from_slice(self.to.as_bytes());
        out.extend_from_slice(&self.amount.to_le_bytes());
        out.extend_from_slice(&self.nonce.to_le_bytes());
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TxTransfer {
    pub payload: TransferPayload,
    pub signature: Signature,
}

impl TxTransfer {
    pub fn new(payload: TransferPayload, signature: Signature) -> Self {
        Self { payload, signature }
    }

    pub fn verify_signature(&self) -> Result<(), VerificationError> {
        let mut out = Vec::with_capacity(self.payload.signing_payload_len());
        self.payload.encode_signing_payload(&mut out);
        verify(&self.payload.from, &self.signature, &out)
    }
}

impl TxTransfer {
    pub fn encoded_len(&self) -> usize {
        let chain_id = 2;
        let chain_version = 2;
        let from = 32;
        let to = 32;
        let amount = 16;
        let nonce = 16;
        let signature = 64;

        chain_id + chain_version + from + to + amount + nonce + signature
    }

    pub fn encode_canonical(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.payload.chain_id.into_u16().to_le_bytes());
        out.extend_from_slice(&self.payload.chain_version.into_u16().to_le_bytes());
        out.extend_from_slice(self.payload.from.as_bytes());
        out.extend_from_slice(self.payload.to.as_bytes());
        out.extend_from_slice(&self.payload.amount.to_le_bytes());
        out.extend_from_slice(&self.payload.nonce.to_le_bytes());
        out.extend_from_slice(self.signature.as_bytes());
    }

    pub fn decode_canonical(input: &[u8]) -> Result<Self, DecodeError> {
        let mut decoder = Decoder::new(input);
        let decoded = Self::decode_from(&mut decoder)?;

        decoder.finish()?;

        Ok(decoded)
    }

    pub fn decode_from(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let chain_id = ChainId::new(decoder.read_u16_le()?);
        let chain_version = ChainVersion::new(decoder.read_u16_le()?);
        let from = Address::new(decoder.read_fixed::<32>()?);
        let to = Address::new(decoder.read_fixed::<32>()?);
        let amount = decoder.read_u128_le()?;
        let nonce = decoder.read_u128_le()?;
        let signature = Signature::new(decoder.read_fixed::<64>()?);

        Ok(Self::new(
            TransferPayload::new(chain_id, chain_version, from, to, amount, nonce),
            signature,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{TransferPayload, TxTransfer};
    use crate::{
        address::Address,
        chain_id::ChainId,
        chain_version::ChainVersion,
        crypto::{SecretKey, address_from_secret_key, sign},
    };

    #[test]
    fn transfer_roundtrips_and_verifies_signature() {
        let secret_key = SecretKey::new([1; 32]);
        let payload = TransferPayload::new(
            ChainId::new(1),
            ChainVersion::new(1),
            address_from_secret_key(&secret_key),
            Address::new([2; 32]),
            55,
            9,
        );
        let mut signing_payload = Vec::with_capacity(payload.signing_payload_len());
        payload.encode_signing_payload(&mut signing_payload);
        let tx = TxTransfer::new(payload, sign(&secret_key, &signing_payload));
        let mut encoded = Vec::with_capacity(tx.encoded_len());
        tx.encode_canonical(&mut encoded);

        let decoded = TxTransfer::decode_canonical(&encoded).unwrap();

        assert_eq!(decoded, tx);
        assert_eq!(decoded.verify_signature(), Ok(()));
    }
}
