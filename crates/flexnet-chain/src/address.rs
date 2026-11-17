use flexnet_codec::{
    canonical_decode::CanonicalDecode,
    canonical_encode::CanonicalEncode,
    decoder::{Decoder, DecoderError},
    encoder::{Encoder, EncoderError},
};
use std::fmt::{Debug, Display};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Address([u8; 32]);

impl Address {
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{}", hex::encode(self.as_bytes()))
    }
}

impl Debug for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Address({})", self)
    }
}

impl CanonicalEncode for Address {
    fn encode_to(&self, encoder: &mut Encoder) -> Result<(), EncoderError> {
        encoder.write_fixed_bytes(&self.0);
        Ok(())
    }
}

impl CanonicalDecode for Address {
    fn decode_from(decoder: &mut Decoder) -> Result<Self, DecoderError> {
        Ok(Self::new(*decoder.read_fixed_bytes()?))
    }
}

#[cfg(test)]
mod tests {
    use super::Address;

    #[test]
    fn exposes_underlying_bytes() {
        let address = Address::new([3; 32]);

        assert_eq!(address.as_bytes(), &[3; 32]);
    }
}
