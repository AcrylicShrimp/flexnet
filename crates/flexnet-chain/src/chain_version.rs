use flexnet_codec::{
    canonical_decode::CanonicalDecode,
    canonical_encode::CanonicalEncode,
    decoder::{Decoder, DecoderError},
    encoder::{Encoder, EncoderError},
};
use std::fmt::Display;

const CHAIN_VERSION: u64 = 0x0000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChainVersion;

impl ChainVersion {
    pub const fn into_u64(self) -> u64 {
        CHAIN_VERSION
    }
}

impl CanonicalEncode for ChainVersion {
    fn encode_to(&self, encoder: &mut Encoder) -> Result<(), EncoderError> {
        encoder.write_u64_le(CHAIN_VERSION);
        Ok(())
    }
}

impl CanonicalDecode for ChainVersion {
    fn decode_from(decoder: &mut Decoder) -> Result<Self, DecoderError> {
        let chain_version = decoder.read_u64_le()?;

        if chain_version != CHAIN_VERSION {
            return Err(DecoderError::InvalidDiscriminant {
                type_name: "CHAIN_VERSION",
                discriminant: CHAIN_VERSION,
            });
        }

        Ok(Self)
    }
}

impl Display for ChainVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:016x}", CHAIN_VERSION)
    }
}
