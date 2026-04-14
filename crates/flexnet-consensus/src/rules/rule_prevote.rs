use crate::{consensus_config::ConsensusConfig, messages::msg_prevote::MsgPrevote};
use flexnet_chain::{address::Address, crypto::VerificationError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum PrevoteVerificationError {
    #[error("signature verification failed: {0}")]
    InvalidSignature(VerificationError),
    #[error("address {address} is not a validator")]
    NotValidator { address: Address },
}

pub fn verify_prevote_stateless(
    msg: &MsgPrevote,
    consensus_config: &ConsensusConfig,
) -> Result<(), PrevoteVerificationError> {
    if let Err(err) = msg.verify_signature() {
        return Err(PrevoteVerificationError::InvalidSignature(err));
    }

    if !consensus_config.validators.contains(&msg.payload.address) {
        return Err(PrevoteVerificationError::NotValidator {
            address: msg.payload.address,
        });
    }

    Ok(())
}
