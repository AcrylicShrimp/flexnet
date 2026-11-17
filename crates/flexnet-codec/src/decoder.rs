use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum DecoderError {
    #[error(
        "position overflow; position {position} + bytes requested {bytes_requested} exceeds usize::MAX"
    )]
    PositionOverflow {
        position: usize,
        bytes_requested: usize,
    },
    #[error("input exhausted after consuming {bytes_consumed} bytes")]
    InputExhausted { bytes_consumed: usize },
    #[error(
        "input not fully consumed; {bytes_consumed} bytes consumed, {bytes_remaining} bytes remaining"
    )]
    InputNotFullyConsumed {
        bytes_consumed: usize,
        bytes_remaining: usize,
    },
    #[error("invalid discriminant {discriminant} for {type_name}")]
    InvalidDiscriminant {
        type_name: &'static str,
        discriminant: u64,
    },
}

#[derive(Debug, Clone)]
pub struct Decoder<'a> {
    input: &'a [u8],
    position: usize,
}

impl<'a> Decoder<'a> {
    pub fn new(input: &'a [u8]) -> Self {
        Self { input, position: 0 }
    }

    pub fn finish(self) -> Result<(), DecoderError> {
        if self.position != self.input.len() {
            return Err(DecoderError::InputNotFullyConsumed {
                bytes_consumed: self.position,
                bytes_remaining: self.input.len() - self.position,
            });
        }

        Ok(())
    }

    fn read_exact_dynamic(&mut self, n: usize) -> Result<&'a [u8], DecoderError> {
        let start = self.position;
        let end = self
            .position
            .checked_add(n)
            .ok_or(DecoderError::PositionOverflow {
                position: self.position,
                bytes_requested: n,
            })?;

        let bytes = self
            .input
            .get(start..end)
            .ok_or(DecoderError::InputExhausted {
                bytes_consumed: self.position,
            })?;

        self.position = end;
        Ok(bytes)
    }

    fn read_exact_fixed<const N: usize>(&mut self) -> Result<&[u8; N], DecoderError> {
        Ok(self
            .read_exact_dynamic(N)?
            .try_into()
            .expect("read_exact_dynamic returned N bytes"))
    }

    pub fn read_u8(&mut self) -> Result<u8, DecoderError> {
        Ok(u8::from_le_bytes(*self.read_exact_fixed()?))
    }

    pub fn read_u16_le(&mut self) -> Result<u16, DecoderError> {
        Ok(u16::from_le_bytes(*self.read_exact_fixed()?))
    }

    pub fn read_u32_le(&mut self) -> Result<u32, DecoderError> {
        Ok(u32::from_le_bytes(*self.read_exact_fixed()?))
    }

    pub fn read_u64_le(&mut self) -> Result<u64, DecoderError> {
        Ok(u64::from_le_bytes(*self.read_exact_fixed()?))
    }

    pub fn read_u128_le(&mut self) -> Result<u128, DecoderError> {
        Ok(u128::from_le_bytes(*self.read_exact_fixed()?))
    }

    pub fn read_fixed_bytes<const N: usize>(&mut self) -> Result<&[u8; N], DecoderError> {
        self.read_exact_fixed()
    }

    pub fn read_dynamic_bytes(&mut self) -> Result<&[u8], DecoderError> {
        let len = self.read_u16_le()? as usize;
        self.read_exact_dynamic(len)
    }
}
