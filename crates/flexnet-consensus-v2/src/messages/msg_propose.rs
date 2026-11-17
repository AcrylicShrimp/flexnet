use crate::message_kind::MessageKind;
use flexnet_chain::{
    address::Address,
    block::Block,
    codec::{DecodeError, Decoder},
    crypto::{Ed25519Signature, VerificationError, verify},
    hash::{Hash256, compute_block_hash},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProposePayload {
    pub height: u128,
    pub round: u64,
    pub address: Address,
    pub proposal: Block,
    pub justification: Option<ProposeJustificationPayload>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProposeJustificationPayload {
    pub height: u128,
    pub round: u64,
    pub proposal_hash: Hash256,
    pub evidences: Vec<ProposeEvidencePayload>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProposeEvidencePayload {
    pub address: Address,
    pub signature: Ed25519Signature,
}

impl ProposePayload {
    pub fn new(
        height: u128,
        round: u64,
        address: Address,
        proposal: Block,
        justification: Option<ProposeJustificationPayload>,
    ) -> Self {
        Self {
            height,
            round,
            address,
            proposal,
            justification,
        }
    }

    pub fn signing_payload_len(&self, chain_id: &str) -> usize {
        let magic = crate::consts::MAGIC_BYTES.len();
        let protocol_version = 4;
        let chain_id_len = 4;
        let chain_id = chain_id.len();

        let kind = 1;
        let height = 16;
        let round = 8;
        let address = 32;
        let proposal_hash = 32;
        let justification_option = 1;
        let justification = if let Some(justification) = &self.justification {
            justification.signing_payload_len()
        } else {
            0
        };

        magic
            + protocol_version
            + chain_id_len
            + chain_id
            + kind
            + height
            + round
            + address
            + proposal_hash
            + justification_option
            + justification
    }

    pub fn encode_signing_payload(&self, chain_id: &str, out: &mut Vec<u8>) {
        debug_assert!(chain_id.len() <= u32::MAX as usize, "chain_id is too long");

        out.extend_from_slice(crate::consts::MAGIC_BYTES);
        out.extend_from_slice(&crate::consts::PROTOCOL_VERSION.to_le_bytes());
        out.extend_from_slice(&(chain_id.len() as u32).to_le_bytes());
        out.extend_from_slice(chain_id.as_bytes());

        out.extend_from_slice(&MessageKind::Propose.into_u8().to_le_bytes());
        out.extend_from_slice(&self.height.to_le_bytes());
        out.extend_from_slice(&self.round.to_le_bytes());
        out.extend_from_slice(self.address.as_bytes());
        out.extend_from_slice(compute_block_hash(&self.proposal).as_bytes());
        out.extend_from_slice(if self.justification.is_some() {
            &[1]
        } else {
            &[0]
        });

        if let Some(justification) = &self.justification {
            justification.encode_signing_payload(out);
        }
    }
}

impl ProposeJustificationPayload {
    pub fn new(
        height: u128,
        round: u64,
        proposal_hash: Hash256,
        evidences: Vec<ProposeEvidencePayload>,
    ) -> Self {
        Self {
            height,
            round,
            proposal_hash,
            evidences,
        }
    }

    pub fn signing_payload_len(&self) -> usize {
        let height = 16;
        let round = 8;
        let proposal_hash = 32;
        let evidences_len = 2;

        let mut evidences = 0;

        for evidence in &self.evidences {
            evidences += evidence.signing_payload_len();
        }

        height + round + proposal_hash + evidences_len + evidences
    }

    pub fn encode_signing_payload(&self, out: &mut Vec<u8>) {
        let evidence_count = u16::try_from(self.evidences.len())
            .expect("evidence count exceeds canonical u16 range");

        out.extend_from_slice(&self.height.to_le_bytes());
        out.extend_from_slice(&self.round.to_le_bytes());
        out.extend_from_slice(self.proposal_hash.as_bytes());
        out.extend_from_slice(&evidence_count.to_le_bytes());

        for evidence in &self.evidences {
            evidence.encode_signing_payload(out);
        }
    }
}

impl ProposeEvidencePayload {
    pub fn new(address: Address, signature: Ed25519Signature) -> Self {
        Self { address, signature }
    }

    pub fn signing_payload_len(&self) -> usize {
        let address = 32;
        let signature = 64;

        address + signature
    }

    pub fn encode_signing_payload(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(self.address.as_bytes());
        out.extend_from_slice(self.signature.as_bytes());
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MsgPropose {
    pub payload: ProposePayload,
    pub signature: Ed25519Signature,
}

impl MsgPropose {
    pub fn new(payload: ProposePayload, signature: Ed25519Signature) -> Self {
        Self { payload, signature }
    }

    pub fn verify_signature(&self, chain_id: &str) -> Result<(), VerificationError> {
        let mut out = Vec::with_capacity(self.payload.signing_payload_len(chain_id));
        self.payload.encode_signing_payload(chain_id, &mut out);
        verify(&self.payload.address, &self.signature, &out)
    }
}

impl MsgPropose {
    pub fn encoded_len(&self) -> usize {
        let height = 16;
        let round = 8;
        let address = 32;
        let proposal = self.payload.proposal.encoded_len();
        let justification_option = 1;
        let justification = if let Some(justification) = &self.payload.justification {
            let height = 16;
            let round = 8;
            let proposal_hash = 32;
            let evidences_len = 2;
            let evidences = (32 + 64) * justification.evidences.len();

            height + round + proposal_hash + evidences_len + evidences
        } else {
            0
        };
        let signature = 64;

        height + round + address + proposal + justification_option + justification + signature
    }

    pub fn encode_canonical(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.payload.height.to_le_bytes());
        out.extend_from_slice(&self.payload.round.to_le_bytes());
        out.extend_from_slice(self.payload.address.as_bytes());

        self.payload.proposal.encode_canonical(out);

        out.extend_from_slice(if self.payload.justification.is_some() {
            &[1]
        } else {
            &[0]
        });

        if let Some(justification) = &self.payload.justification {
            let evidence_count = u16::try_from(justification.evidences.len())
                .expect("evidence count exceeds canonical u16 range");

            out.extend_from_slice(&justification.height.to_le_bytes());
            out.extend_from_slice(&justification.round.to_le_bytes());
            out.extend_from_slice(justification.proposal_hash.as_bytes());
            out.extend_from_slice(&evidence_count.to_le_bytes());

            for evidence in &justification.evidences {
                out.extend_from_slice(evidence.address.as_bytes());
                out.extend_from_slice(evidence.signature.as_bytes());
            }
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
        let round = decoder.read_u64_le()?;
        let address = Address::new(decoder.read_fixed::<32>()?);
        let proposal = Block::decode_from(decoder)?;
        let justification_option = decoder.read_u8_le()?;
        let justification = if justification_option == 1 {
            let height = decoder.read_u128_le()?;
            let round = decoder.read_u64_le()?;
            let proposal_hash = decoder.read_fixed::<32>()?;
            let evidence_count = decoder.read_u16_le()?;

            let mut evidences = Vec::with_capacity(evidence_count as usize);

            for _ in 0..evidence_count {
                let address = Address::new(decoder.read_fixed::<32>()?);
                let signature = Ed25519Signature::new(decoder.read_fixed::<64>()?);
                evidences.push(ProposeEvidencePayload::new(address, signature));
            }

            Some(ProposeJustificationPayload::new(
                height,
                round,
                Hash256::new(proposal_hash),
                evidences,
            ))
        } else if justification_option == 0 {
            None
        } else {
            return Err(DecodeError::InvalidInput);
        };

        let signature = Ed25519Signature::new(decoder.read_fixed::<64>()?);

        Ok(Self::new(
            ProposePayload::new(height, round, address, proposal, justification),
            signature,
        ))
    }
}
