use crate::{address::Address, signing_pre_image::SingingPreImage};
use ed25519_dalek::{
    PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH, SIGNATURE_LENGTH, Signer, SigningKey, VerifyingKey,
};
use flexnet_codec::{
    canonical_decode::CanonicalDecode,
    canonical_encode::CanonicalEncode,
    decoder::{Decoder, DecoderError},
    encoder::{Encoder, EncoderError},
};
use rand::rngs::ThreadRng;
use std::fmt::{Debug, Display};
use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ed25519PublicKey([u8; PUBLIC_KEY_LENGTH]);

impl Ed25519PublicKey {
    pub const fn new(bytes: [u8; PUBLIC_KEY_LENGTH]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; PUBLIC_KEY_LENGTH] {
        &self.0
    }
}

impl Display for Ed25519PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{}", hex::encode(self.0))
    }
}

impl Debug for Ed25519PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PublicKey({})", self)
    }
}

impl CanonicalEncode for Ed25519PublicKey {
    fn encode_to(&self, encoder: &mut Encoder) -> Result<(), EncoderError> {
        encoder.write_fixed_bytes(&self.0);
        Ok(())
    }
}

impl CanonicalDecode for Ed25519PublicKey {
    fn decode_from(decoder: &mut Decoder) -> Result<Self, DecoderError> {
        Ok(Self::new(*decoder.read_fixed_bytes()?))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ed25519Signature([u8; SIGNATURE_LENGTH]);

impl Ed25519Signature {
    pub const fn new(bytes: [u8; SIGNATURE_LENGTH]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; SIGNATURE_LENGTH] {
        &self.0
    }
}

impl Display for Ed25519Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{}", hex::encode(self.0))
    }
}

impl Debug for Ed25519Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Signature({})", self)
    }
}

impl CanonicalEncode for Ed25519Signature {
    fn encode_to(&self, encoder: &mut Encoder) -> Result<(), EncoderError> {
        encoder.write_fixed_bytes(&self.0);
        Ok(())
    }
}

impl CanonicalDecode for Ed25519Signature {
    fn decode_from(decoder: &mut Decoder) -> Result<Self, DecoderError> {
        Ok(Self::new(*decoder.read_fixed_bytes()?))
    }
}

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct Ed25519SecretKey([u8; SECRET_KEY_LENGTH]);

impl Ed25519SecretKey {
    pub fn generate_random() -> Self {
        Self(SigningKey::generate(&mut ThreadRng::default()).to_bytes())
    }

    pub const fn new(bytes: [u8; SECRET_KEY_LENGTH]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; SECRET_KEY_LENGTH] {
        &self.0
    }

    pub fn public_key(&self) -> Ed25519PublicKey {
        Ed25519PublicKey(*SigningKey::from_bytes(&self.0).verifying_key().as_bytes())
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum SigningError {
    #[error("encoder error: {0}")]
    EncoderError(#[from] EncoderError),
}

fn sign_bytes(secret_key: &Ed25519SecretKey, message: &[u8]) -> Ed25519Signature {
    Ed25519Signature::new(
        SigningKey::from_bytes(secret_key.as_bytes())
            .sign(message)
            .to_bytes(),
    )
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum VerificationError {
    #[error("invalid verifying key")]
    InvalidVerifyingKey,
    #[error("failed to verify signature")]
    FailedToVerify,
    #[error("encoder error: {0}")]
    EncoderError(#[from] EncoderError),
}

fn verify_bytes(
    public_key: &Ed25519PublicKey,
    signature: &Ed25519Signature,
    message: &[u8],
) -> Result<(), VerificationError> {
    VerifyingKey::from_bytes(public_key.as_bytes())
        .map_err(|_| VerificationError::InvalidVerifyingKey)?
        .verify_strict(
            message,
            &ed25519_dalek::Signature::from_bytes(signature.as_bytes()),
        )
        .map_err(|_| VerificationError::FailedToVerify)?;

    Ok(())
}

pub fn sign<T>(
    secret_key: &Ed25519SecretKey,
    pre_image: &T,
) -> Result<Ed25519Signature, SigningError>
where
    T: SingingPreImage,
{
    Ok(sign_bytes(
        secret_key,
        &pre_image.encode_signing_pre_image()?,
    ))
}

pub fn verify<T>(
    public_key: &Ed25519PublicKey,
    signature: &Ed25519Signature,
    pre_image: &T,
) -> Result<(), VerificationError>
where
    T: SingingPreImage,
{
    VerifyingKey::from_bytes(public_key.as_bytes())
        .map_err(|_| VerificationError::InvalidVerifyingKey)?
        .verify_strict(
            &pre_image.encode_signing_pre_image()?,
            &ed25519_dalek::Signature::from_bytes(signature.as_bytes()),
        )
        .map_err(|_| VerificationError::FailedToVerify)?;

    Ok(())
}

pub fn address_from_secret_key(secret_key: &Ed25519SecretKey) -> Address {
    Address::new(
        SigningKey::from_bytes(secret_key.as_bytes())
            .verifying_key()
            .to_bytes(),
    )
}

#[cfg(test)]
mod tests {
    use super::{Ed25519SecretKey, VerificationError, sign_bytes, verify_bytes};

    #[test]
    fn sign_and_verify_roundtrip() {
        let secret_key = Ed25519SecretKey::new([7; 32]);
        let public_key = secret_key.public_key();
        let message = b"flexnet";
        let signature = sign_bytes(&secret_key, message);

        assert_eq!(verify_bytes(&public_key, &signature, message), Ok(()));
        assert_eq!(
            verify_bytes(&public_key, &signature, b"other-message"),
            Err(VerificationError::FailedToVerify)
        );
    }
}
