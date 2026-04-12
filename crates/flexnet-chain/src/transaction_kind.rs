use crate::codec::{DecodeError, Decoder};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TransactionKind {
    Transfer = 1,
}

impl TransactionKind {
    pub fn new(kind: u8) -> Option<Self> {
        match kind {
            1 => Some(TransactionKind::Transfer),
            _ => None,
        }
    }

    pub fn encoded_len(&self) -> usize {
        match self {
            TransactionKind::Transfer => 1,
        }
    }

    pub fn into_u8(self) -> u8 {
        self as u8
    }

    pub fn encode_canonical(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.into_u8().to_le_bytes());
    }

    pub fn decode_canonical(input: &[u8]) -> Result<Self, DecodeError> {
        let mut decoder = Decoder::new(input);
        let decoded = Self::decode_from(&mut decoder)?;

        decoder.finish()?;

        Ok(decoded)
    }

    pub fn decode_from(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let kind = decoder.read_u8_le()?;

        TransactionKind::new(kind).ok_or(DecodeError::InvalidInput)
    }
}
