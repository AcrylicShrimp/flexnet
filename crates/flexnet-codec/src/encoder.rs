use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum EncoderError {
    #[error("dynamic bytes length exceeds canonical u16 range")]
    DynamicBytesLengthExceedsCanonicalU16Range,
}

#[derive(Default, Debug, Clone)]
pub struct Encoder {
    buffer: Vec<u8>,
}

impl Encoder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn finish(self) -> Vec<u8> {
        self.buffer
    }

    pub fn write_u8(&mut self, value: u8) {
        self.buffer.push(value);
    }

    pub fn write_u16_le(&mut self, value: u16) {
        self.buffer.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_u32_le(&mut self, value: u32) {
        self.buffer.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_u64_le(&mut self, value: u64) {
        self.buffer.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_u128_le(&mut self, value: u128) {
        self.buffer.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_fixed_bytes(&mut self, bytes: &[u8]) {
        self.buffer.extend_from_slice(bytes);
    }

    pub fn write_dynamic_bytes(&mut self, bytes: &[u8]) -> Result<(), EncoderError> {
        let len = u16::try_from(bytes.len())
            .map_err(|_| EncoderError::DynamicBytesLengthExceedsCanonicalU16Range)?;
        self.write_u16_le(len);
        self.buffer.extend_from_slice(bytes);
        Ok(())
    }
}
