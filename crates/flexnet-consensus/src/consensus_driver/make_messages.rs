use crate::{
    consensus_config::ConsensusConfig,
    consensus_driver::proposal_block::ProposalBlock,
    messages::{
        msg_precommit::{MsgPrecommit, PrecommitPayload},
        msg_prevote::{MsgPrevote, PrevotePayload},
        msg_propose::{
            MsgPropose, ProposeEvidencePayload, ProposeJustificationPayload, ProposePayload,
        },
    },
    polka::Polka,
};
use flexnet_chain::{block::Block, crypto::sign, hash::Hash};

pub fn make_propose_message_using_polka(
    height: u128,
    round: u32,
    polka: Polka<ProposalBlock>,
    consensus_config: &ConsensusConfig,
) -> MsgPropose {
    let payload = ProposePayload {
        height,
        round,
        address: consensus_config.address,
        proposal: polka.proposal.into_block(),
        justification: Some(ProposeJustificationPayload::new(
            polka.justification.height,
            polka.justification.round,
            polka.proposal_hash,
            polka
                .justification
                .evidences
                .into_iter()
                .map(|evidence| ProposeEvidencePayload::new(evidence.address, evidence.signature))
                .collect(),
        )),
    };

    let mut out = Vec::with_capacity(payload.signing_payload_len());
    payload.encode_signing_payload(&mut out);
    let signature = sign(&consensus_config.secret_key, &out);

    MsgPropose { payload, signature }
}

pub fn make_propose_message_using_block(
    height: u128,
    round: u32,
    block: Block,
    consensus_config: &ConsensusConfig,
) -> MsgPropose {
    let payload = ProposePayload {
        height,
        round,
        address: consensus_config.address,
        proposal: block,
        justification: None,
    };

    let mut out = Vec::with_capacity(payload.signing_payload_len());
    payload.encode_signing_payload(&mut out);
    let signature = sign(&consensus_config.secret_key, &out);

    MsgPropose { payload, signature }
}

pub fn make_prevote_message(
    height: u128,
    round: u32,
    proposal_hash: Option<Hash>,
    consensus_config: &ConsensusConfig,
) -> MsgPrevote {
    let payload = PrevotePayload {
        height,
        round,
        address: consensus_config.address,
        proposal_hash,
    };

    let mut out = Vec::with_capacity(payload.signing_payload_len());
    payload.encode_signing_payload(&mut out);
    let signature = sign(&consensus_config.secret_key, &out);

    MsgPrevote { payload, signature }
}

pub fn make_precommit_message(
    height: u128,
    round: u32,
    proposal_hash: Option<Hash>,
    consensus_config: &ConsensusConfig,
) -> MsgPrecommit {
    let payload = PrecommitPayload {
        height,
        round,
        address: consensus_config.address,
        proposal_hash,
    };

    let mut out = Vec::with_capacity(payload.signing_payload_len());
    payload.encode_signing_payload(&mut out);
    let signature = sign(&consensus_config.secret_key, &out);

    MsgPrecommit { payload, signature }
}
