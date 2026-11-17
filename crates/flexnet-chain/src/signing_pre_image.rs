use flexnet_codec::encoder::{Encoder, EncoderError};

pub trait SingingPreImage {
    fn encode_signing_pre_image_to(&self, encoder: &mut Encoder) -> Result<(), EncoderError>;

    fn encode_signing_pre_image(&self) -> Result<Vec<u8>, EncoderError> {
        let mut encoder = Encoder::new();
        self.encode_signing_pre_image_to(&mut encoder)?;
        Ok(encoder.finish())
    }
}
