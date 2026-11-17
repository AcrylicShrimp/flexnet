use crate::decoder::{Decoder, DecoderError};

pub trait CanonicalDecode: Sized {
    fn decode_from(decoder: &mut Decoder) -> Result<Self, DecoderError>;

    fn decode_canonical(input: &[u8]) -> Result<Self, DecoderError> {
        let mut decoder = Decoder::new(input);
        let decoded = Self::decode_from(&mut decoder)?;
        decoder.finish()?;
        Ok(decoded)
    }
}

impl<'a> Decoder<'a> {
    pub fn decode<T>(&mut self) -> Result<T, DecoderError>
    where
        T: CanonicalDecode,
    {
        T::decode_from(self)
    }
}
