use crate::{
    consensus_config::ConsensusConfig,
    message_kind::MessageKind,
    messages::{msg_precommit::MsgPrecommit, msg_prevote::MsgPrevote, msg_propose::MsgPropose},
    rules::{
        rule_precommit::{PrecommitVerificationError, verify_precommit_stateless},
        rule_prevote::{PrevoteVerificationError, verify_prevote_stateless},
        rule_propose::{ProposeVerificationError, verify_propose_stateless},
    },
};
use flexnet_chain::{
    address::Address,
    chain_config::ChainConfig,
    codec::{DecodeError, Decoder},
};
use thiserror::Error;

#[derive(Debug, Clone)]
pub enum Message {
    Propose(MsgPropose),
    Prevote(MsgPrevote),
    Precommit(MsgPrecommit),
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum MessageVerificationError {
    #[error("proposal verification failed: {0}")]
    Proposal(#[from] ProposeVerificationError),
    #[error("prevote verification failed: {0}")]
    Prevote(#[from] PrevoteVerificationError),
    #[error("precommit verification failed: {0}")]
    Precommit(#[from] PrecommitVerificationError),
}

impl Message {
    pub fn kind(&self) -> MessageKind {
        match self {
            Message::Propose(_) => MessageKind::Propose,
            Message::Prevote(_) => MessageKind::Prevote,
            Message::Precommit(_) => MessageKind::Precommit,
        }
    }

    pub fn encoded_len(&self) -> usize {
        let kind = 1;
        let msg = match self {
            Message::Propose(msg) => msg.encoded_len(),
            Message::Prevote(msg) => msg.encoded_len(),
            Message::Precommit(msg) => msg.encoded_len(),
        };

        kind + msg
    }

    pub fn encode_canonical(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.kind().into_u8().to_le_bytes());

        match self {
            Message::Propose(msg) => {
                msg.encode_canonical(out);
            }
            Message::Prevote(msg) => {
                msg.encode_canonical(out);
            }
            Message::Precommit(msg) => {
                msg.encode_canonical(out);
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
        let kind = MessageKind::decode_from(decoder)?;
        let message = match kind {
            MessageKind::Propose => Self::Propose(MsgPropose::decode_from(decoder)?),
            MessageKind::Prevote => Self::Prevote(MsgPrevote::decode_from(decoder)?),
            MessageKind::Precommit => Self::Precommit(MsgPrecommit::decode_from(decoder)?),
        };

        Ok(message)
    }

    pub fn verify_stateless(
        &self,
        current_proposer: &Address,
        chain_config: &ChainConfig,
        consensus_config: &ConsensusConfig,
    ) -> Result<(), MessageVerificationError> {
        match self {
            Message::Propose(msg) => {
                verify_propose_stateless(msg, current_proposer, chain_config, consensus_config)?;
            }
            Message::Prevote(msg) => {
                verify_prevote_stateless(msg, consensus_config)?;
            }
            Message::Precommit(msg) => {
                verify_precommit_stateless(msg, consensus_config)?;
            }
        }

        Ok(())
    }
}
