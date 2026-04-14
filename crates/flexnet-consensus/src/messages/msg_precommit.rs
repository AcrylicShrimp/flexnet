use crate::message_kind::MessageKind;
use flexnet_chain::{
    address::Address,
    codec::{DecodeError, Decoder},
    crypto::{Signature, VerificationError, verify},
    hash::Hash,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PrecommitPayload {
    pub height: u128,
    pub round: u32,
    pub address: Address,
    pub proposal_hash: Option<Hash>,
}

impl PrecommitPayload {
    pub fn new(height: u128, round: u32, address: Address, proposal_hash: Option<Hash>) -> Self {
        Self {
            height,
            round,
            address,
            proposal_hash,
        }
    }

    pub fn signing_payload_len(&self) -> usize {
        let kind = 1;
        let height = 16;
        let round = 4;
        let address = 32;
        let proposal_hash_kind = 1;
        let proposal_hash = if self.proposal_hash.is_some() { 32 } else { 0 };

        kind + height + round + address + proposal_hash_kind + proposal_hash
    }

    pub fn encode_signing_payload(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&MessageKind::Precommit.into_u8().to_le_bytes());
        out.extend_from_slice(&self.height.to_le_bytes());
        out.extend_from_slice(&self.round.to_le_bytes());
        out.extend_from_slice(self.address.as_bytes());
        out.extend_from_slice(if self.proposal_hash.is_some() {
            &[1]
        } else {
            &[0]
        });
        if let Some(proposal_hash) = self.proposal_hash {
            out.extend_from_slice(proposal_hash.as_bytes());
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MsgPrecommit {
    pub payload: PrecommitPayload,
    pub signature: Signature,
}

impl MsgPrecommit {
    pub fn new(payload: PrecommitPayload, signature: Signature) -> Self {
        Self { payload, signature }
    }

    pub fn verify_signature(&self) -> Result<(), VerificationError> {
        let mut out = Vec::with_capacity(self.payload.signing_payload_len());
        self.payload.encode_signing_payload(&mut out);
        verify(&self.payload.address, &self.signature, &out)
    }
}

impl MsgPrecommit {
    pub fn encoded_len(&self) -> usize {
        let height = 16;
        let round = 4;
        let address = 32;
        let proposal_hash_kind = 1;
        let proposal_hash = if self.payload.proposal_hash.is_some() {
            32
        } else {
            0
        };
        let signature = 64;

        height + round + address + proposal_hash_kind + proposal_hash + signature
    }

    pub fn encode_canonical(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.payload.height.to_le_bytes());
        out.extend_from_slice(&self.payload.round.to_le_bytes());
        out.extend_from_slice(self.payload.address.as_bytes());
        out.extend_from_slice(if self.payload.proposal_hash.is_some() {
            &[1]
        } else {
            &[0]
        });

        if let Some(proposal_hash) = self.payload.proposal_hash {
            out.extend_from_slice(proposal_hash.as_bytes());
        }

        out.extend_from_slice(self.signature.as_bytes());
    }

    pub fn decode_canonical(input: &[u8]) -> Result<Self, DecodeError> {
        let mut decoder = Decoder::new(input);
        let decoded = Self::decode_from(&mut decoder)?;

        decoder.finish()?;

        Ok(decoded)
    }

    pub fn decode_from(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let height = decoder.read_u128_le()?;
        let round = decoder.read_u32_le()?;
        let address = Address::new(decoder.read_fixed::<32>()?);
        let proposal_hash_kind = decoder.read_u8_le()?;
        let proposal_hash = if proposal_hash_kind == 1 {
            Some(Hash::new(decoder.read_fixed::<32>()?))
        } else {
            None
        };
        let signature = Signature::new(decoder.read_fixed::<64>()?);

        Ok(Self::new(
            PrecommitPayload::new(height, round, address, proposal_hash),
            signature,
        ))
    }
}
