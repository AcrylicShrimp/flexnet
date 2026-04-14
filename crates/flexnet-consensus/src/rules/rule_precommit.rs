use crate::{consensus_config::ConsensusConfig, messages::msg_precommit::MsgPrecommit};
use flexnet_chain::{address::Address, crypto::VerificationError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum PrecommitVerificationError {
    #[error("signature verification failed: {0}")]
    InvalidSignature(VerificationError),
    #[error("address {address} is not a validator")]
    NotValidator { address: Address },
}

pub fn verify_precommit_stateless(
    msg: &MsgPrecommit,
    consensus_config: &ConsensusConfig,
) -> Result<(), PrecommitVerificationError> {
    if let Err(err) = msg.verify_signature() {
        return Err(PrecommitVerificationError::InvalidSignature(err));
    }

    if !consensus_config.validators.contains(&msg.payload.address) {
        return Err(PrecommitVerificationError::NotValidator {
            address: msg.payload.address,
        });
    }

    Ok(())
}
