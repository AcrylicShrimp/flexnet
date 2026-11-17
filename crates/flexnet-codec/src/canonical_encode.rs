use crate::encoder::{Encoder, EncoderError};

pub trait CanonicalEncode {
    fn encode_to(&self, encoder: &mut Encoder) -> Result<(), EncoderError>;

    fn encode_canonical(&self) -> Result<Vec<u8>, EncoderError> {
        let mut encoder = Encoder::new();
        self.encode_to(&mut encoder)?;
        Ok(encoder.finish())
    }
}
