use crate::{
    consensus_config::ConsensusConfig,
    messages::{
        msg_prevote::{MsgPrevote, PrevotePayload},
        msg_propose::MsgPropose,
    },
};
use flexnet_chain::{
    address::Address,
    chain_config::ChainConfig,
    crypto::VerificationError,
    hash::{Hash, compute_block_hash},
    rules::rule_block::{BlockVerificationError, verify_block_stateless},
};
use std::collections::BTreeSet;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ProposeVerificationError {
    #[error("address {actual} is not the current proposer {expected}")]
    NotCurrentProposer { expected: Address, actual: Address },
    #[error("block verification failed: {0}")]
    InvalidBlock(BlockVerificationError),
    #[error("signature verification failed: {0}")]
    InvalidSignature(VerificationError),
    #[error("address {address} is not a validator")]
    NotValidator { address: Address },
    #[error("justification height mismatch: expected {expected}, got {actual}")]
    InvalidJustificationHeight { expected: u128, actual: u128 },
    #[error(
        "justification round is not less than the proposal round: expected < {expected}, got {actual}"
    )]
    InvalidJustificationRound { expected: u32, actual: u32 },
    #[error("not enough evidence: expected at least {quorum}, got {evidence_count}")]
    NotEnoughEvidence {
        quorum: usize,
        evidence_count: usize,
    },
    #[error("justification proposal hash mismatch: expected {expected}, got {actual}")]
    InvalidJustificationProposalHash { expected: Hash, actual: Hash },
    #[error("evidence at {index} address {address} is duplicated")]
    DuplicateEvidence { index: usize, address: Address },
    #[error("evidence at {index} address {address} is not a validator")]
    EvidenceNotValidator { index: usize, address: Address },
    #[error("evidence at {index} verification failed: {err:?}")]
    InvalidEvidenceSignature {
        index: usize,
        err: VerificationError,
    },
}

pub fn verify_propose_stateless(
    msg: &MsgPropose,
    current_proposer: &Address,
    chain_config: &ChainConfig,
    consensus_config: &ConsensusConfig,
) -> Result<(), ProposeVerificationError> {
    if &msg.payload.address != current_proposer {
        return Err(ProposeVerificationError::NotCurrentProposer {
            actual: msg.payload.address,
            expected: *current_proposer,
        });
    }

    if let Err(err) = verify_block_stateless(&msg.payload.proposal, chain_config) {
        return Err(ProposeVerificationError::InvalidBlock(err));
    }

    if let Err(err) = msg.verify_signature() {
        return Err(ProposeVerificationError::InvalidSignature(err));
    }

    if !consensus_config.validators.contains(&msg.payload.address) {
        return Err(ProposeVerificationError::NotValidator {
            address: msg.payload.address,
        });
    }

    if let Some(justification) = &msg.payload.justification {
        if justification.height != msg.payload.height {
            return Err(ProposeVerificationError::InvalidJustificationHeight {
                expected: msg.payload.height,
                actual: justification.height,
            });
        }

        if justification.round >= msg.payload.round {
            return Err(ProposeVerificationError::InvalidJustificationRound {
                expected: msg.payload.round,
                actual: justification.round,
            });
        }

        if justification.evidences.len() < consensus_config.quorum {
            return Err(ProposeVerificationError::NotEnoughEvidence {
                quorum: consensus_config.quorum,
                evidence_count: justification.evidences.len(),
            });
        }

        let proposal_hash = compute_block_hash(&msg.payload.proposal);

        if justification.proposal_hash != proposal_hash {
            return Err(ProposeVerificationError::InvalidJustificationProposalHash {
                expected: proposal_hash,
                actual: justification.proposal_hash,
            });
        }

        let mut address_set = BTreeSet::new();

        for (index, evidence) in justification.evidences.iter().enumerate() {
            if !address_set.insert(evidence.address) {
                return Err(ProposeVerificationError::DuplicateEvidence {
                    index,
                    address: evidence.address,
                });
            }

            if !consensus_config.validators.contains(&evidence.address) {
                return Err(ProposeVerificationError::EvidenceNotValidator {
                    index,
                    address: evidence.address,
                });
            }

            let reconstructed_prevote = MsgPrevote::new(
                PrevotePayload::new(
                    justification.height,
                    justification.round,
                    evidence.address,
                    Some(justification.proposal_hash),
                ),
                evidence.signature,
            );

            if let Err(err) = reconstructed_prevote.verify_signature() {
                return Err(ProposeVerificationError::InvalidEvidenceSignature { index, err });
            }
        }
    }

    Ok(())
}
