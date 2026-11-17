use crate::{
    address::Address,
    chain_id::ChainId,
    chain_version::ChainVersion,
    crypto::{Ed25519Signature, VerificationError, verify},
    domain::Domain,
    magic::Magic,
    protocol_context::ProtocolContext,
    signing_pre_image::SingingPreImage,
    transaction_kind::TransactionKind,
};
use flexnet_codec::{
    canonical_decode::CanonicalDecode,
    canonical_encode::CanonicalEncode,
    decoder::{Decoder, DecoderError},
    encoder::{Encoder, EncoderError},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TransferOperation {
    pub from: Address,
    pub to: Address,
    pub amount: u128,
    pub nonce: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TransferPayload {
    pub from: Address,
    pub to: Address,
    pub amount: u128,
    pub nonce: u128,
}

impl TransferPayload {
    pub fn new(from: Address, to: Address, amount: u128, nonce: u128) -> Self {
        Self {
            from,
            to,
            amount,
            nonce,
        }
    }

    pub fn signing_pre_image<'a>(
        &'a self,
        context: &'a ProtocolContext,
    ) -> TransferSigningPreimage {
        TransferSigningPreimage {
            context,
            payload: self,
        }
    }
}

impl CanonicalEncode for TransferPayload {
    fn encode_to(&self, encoder: &mut Encoder) -> Result<(), EncoderError> {
        self.from.encode_to(encoder)?;
        self.to.encode_to(encoder)?;
        self.amount.encode_to(encoder)?;
        self.nonce.encode_to(encoder)?;
        Ok(())
    }
}

impl CanonicalDecode for TransferPayload {
    fn decode_from(decoder: &mut Decoder) -> Result<Self, DecoderError> {
        let from = decoder.decode()?;
        let to = decoder.decode()?;
        let amount = decoder.decode()?;
        let nonce = decoder.decode()?;

        Ok(Self {
            from,
            to,
            amount,
            nonce,
        })
    }
}

pub struct TransferSigningPreimage<'a> {
    context: &'a ProtocolContext,
    payload: &'a TransferPayload,
}

impl<'a> SingingPreImage for TransferSigningPreimage<'a> {
    fn encode_signing_pre_image_to(&self, encoder: &mut Encoder) -> Result<(), EncoderError> {
        self.context.encode_to(encoder)?;
        self.payload.encode_to(encoder)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TxTransfer {
    pub payload: TransferPayload,
    pub signature: Ed25519Signature,
}

impl TxTransfer {
    pub fn new(payload: TransferPayload) -> Self {
        Self { payload, signature }
    }

    pub fn verify_signature(&self) -> Result<(), VerificationError> {
        let mut out = Vec::with_capacity(self.payload.signing_payload_len());
        self.payload.encode_signing_payload(&mut out);
        verify(&self.payload.from, &self.signature, &out)
    }
}

impl CanonicalEncode for TxTransfer {
    fn encode_to(&self, encoder: &mut Encoder) -> Result<(), EncoderError> {
        self.payload.encode_to(encoder)?;
        self.signature.encode_to(encoder)?;
        Ok(())
    }
}

impl CanonicalDecode for TxTransfer {
    fn decode_from(decoder: &mut Decoder) -> Result<Self, DecoderError> {
        let payload = decoder.decode()?;
        let signature = decoder.decode()?;

        Ok(Self { payload, signature })
    }
}

#[cfg(test)]
mod tests {
    use super::{TransferPayload, TxTransfer};
    use crate::{
        address::Address,
        chain_id::ChainId,
        chain_version::ChainVersion,
        crypto::{Ed25519SecretKey, address_from_secret_key, sign},
    };

    #[test]
    fn transfer_roundtrips_and_verifies_signature() {
        let secret_key = Ed25519SecretKey::new([1; 32]);
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
