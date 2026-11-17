use crate::{chain_id::ChainId, chain_version::ChainVersion};
use flexnet_codec::{
    canonical_decode::CanonicalDecode,
    canonical_encode::CanonicalEncode,
    decoder::{Decoder, DecoderError},
    encoder::{Encoder, EncoderError},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProtocolContext {
    pub chain_id: ChainId,
    pub chain_version: ChainVersion,
}

impl CanonicalEncode for ProtocolContext {
    fn encode_to(&self, encoder: &mut Encoder) -> Result<(), EncoderError> {
        self.chain_id.encode_to(encoder)?;
        self.chain_version.encode_to(encoder)?;
        Ok(())
    }
}

impl CanonicalDecode for ProtocolContext {
    fn decode_from(decoder: &mut Decoder) -> Result<Self, DecoderError> {
        let chain_id = decoder.decode()?;
        let chain_version = decoder.decode()?;

        Ok(Self {
            chain_id,
            chain_version,
        })
    }
}
