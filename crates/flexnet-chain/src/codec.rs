use thiserror::Error;

#[derive(Error, Debug)]
pub enum DecodeError {
    #[error("input too long")]
    InputTooLong,
    #[error("insufficient input")]
    InsufficientInput,
    #[error("length exceeded; expected {expected}, got {actual}")]
    LengthExceeded { expected: usize, actual: usize },
    #[error("invalid input")]
    InvalidInput,
}

pub struct Decoder<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> Decoder<'a> {
    pub fn new(input: &'a [u8]) -> Self {
        Self { input, pos: 0 }
    }

    pub fn read_u8_le(&mut self) -> Result<u8, DecodeError> {
        let bytes = self.read_fixed::<1>()?;
        Ok(u8::from_le_bytes(bytes))
    }

    pub fn read_u16_le(&mut self) -> Result<u16, DecodeError> {
        let bytes = self.read_fixed::<2>()?;
        Ok(u16::from_le_bytes(bytes))
    }

    pub fn read_u128_le(&mut self) -> Result<u128, DecodeError> {
        let bytes = self.read_fixed::<16>()?;
        Ok(u128::from_le_bytes(bytes))
    }

    pub fn read_fixed<const N: usize>(&mut self) -> Result<[u8; N], DecodeError> {
        let end = self.pos.checked_add(N).ok_or(DecodeError::InputTooLong)?;
        let slice = self
            .input
            .get(self.pos..end)
            .ok_or(DecodeError::InsufficientInput)?;

        self.pos = end;

        let mut out = [0u8; N];
        out.copy_from_slice(slice);
        Ok(out)
    }

    pub fn read_dynamic(&mut self, size: usize) -> Result<Vec<u8>, DecodeError> {
        let end = self
            .pos
            .checked_add(size)
            .ok_or(DecodeError::InputTooLong)?;
        let slice = self
            .input
            .get(self.pos..end)
            .ok_or(DecodeError::InsufficientInput)?;

        self.pos = end;

        Ok(slice.to_vec())
    }

    pub fn finish(self) -> Result<(), DecodeError> {
        if self.pos != self.input.len() {
            return Err(DecodeError::LengthExceeded {
                expected: self.pos,
                actual: self.input.len(),
            });
        }

        Ok(())
    }
}
