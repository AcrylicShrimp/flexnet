use crate::address::Address;
use ed25519_dalek::{SECRET_KEY_LENGTH, SIGNATURE_LENGTH, Signer, SigningKey, VerifyingKey};
use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecretKey([u8; SECRET_KEY_LENGTH]);

impl SecretKey {
    pub const fn new(bytes: [u8; SECRET_KEY_LENGTH]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; SECRET_KEY_LENGTH] {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Signature([u8; SIGNATURE_LENGTH]);

impl Signature {
    pub const fn new(bytes: [u8; SIGNATURE_LENGTH]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; SIGNATURE_LENGTH] {
        &self.0
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum VerificationError {
    #[error("invalid verifying key")]
    InvalidVerifyingKey,
    #[error("failed to verify signature")]
    FailedToVerify,
}

pub fn sign(secret_key: &SecretKey, message: &[u8]) -> Signature {
    Signature::new(
        SigningKey::from_bytes(secret_key.as_bytes())
            .sign(message)
            .to_bytes(),
    )
}

pub fn verify(
    address: &Address,
    signature: &Signature,
    message: &[u8],
) -> Result<(), VerificationError> {
    VerifyingKey::from_bytes(address.as_bytes())
        .map_err(|_| VerificationError::InvalidVerifyingKey)?
        .verify_strict(
            message,
            &ed25519_dalek::Signature::from_bytes(signature.as_bytes()),
        )
        .map_err(|_| VerificationError::FailedToVerify)?;

    Ok(())
}

pub fn address_from_secret_key(secret_key: &SecretKey) -> Address {
    Address::new(
        SigningKey::from_bytes(secret_key.as_bytes())
            .verifying_key()
            .to_bytes(),
    )
}

#[cfg(test)]
mod tests {
    use super::{SecretKey, VerificationError, address_from_secret_key, sign, verify};

    #[test]
    fn sign_and_verify_roundtrip() {
        let secret_key = SecretKey::new([7; 32]);
        let address = address_from_secret_key(&secret_key);
        let message = b"flexnet";
        let signature = sign(&secret_key, message);

        assert_eq!(verify(&address, &signature, message), Ok(()));
        assert_eq!(
            verify(&address, &signature, b"other-message"),
            Err(VerificationError::FailedToVerify)
        );
    }
}
