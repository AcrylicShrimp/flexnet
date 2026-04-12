use crate::{block::Block, error::HexEncodingError, hash::Hash, transfer::Transfer};

pub trait EncodeCanonical {
    fn encode_into(&self, out: &mut Vec<u8>);
}

pub fn encode<T>(value: &T) -> Vec<u8>
where
    T: EncodeCanonical + ?Sized,
{
    let mut out = Vec::new();
    value.encode_into(&mut out);
    out
}

pub fn append_u16_le(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_le_bytes());
}

pub fn append_u128_le(out: &mut Vec<u8>, value: u128) {
    out.extend_from_slice(&value.to_le_bytes());
}

pub fn append_fixed<const N: usize>(out: &mut Vec<u8>, value: &[u8; N]) {
    out.extend_from_slice(value);
}

pub fn encode_transfer_signing_payload(transfer: &Transfer) -> Vec<u8> {
    encode(&transfer.signing_view())
}

pub fn encode_transfer_bytes(transfer: &Transfer) -> Vec<u8> {
    encode(&transfer.bytes_view())
}

pub fn encode_transactions_hash_input(transfers: &[Transfer]) -> Vec<u8> {
    encode(&TransactionsHashView { transfers })
}

pub fn encode_block_hash_input(block: &Block, transactions_hash: &Hash) -> Vec<u8> {
    encode(&block.hash_view(transactions_hash))
}

pub fn decode_hex_array<const N: usize>(value: &str) -> Result<[u8; N], HexEncodingError> {
    let stripped = value
        .strip_prefix("0x")
        .ok_or(HexEncodingError::MissingPrefix)?;
    let bytes = hex::decode(stripped)?;
    let actual = bytes.len();

    bytes
        .try_into()
        .map_err(|_| HexEncodingError::InvalidLength {
            expected: N,
            actual,
        })
}

pub fn encode_hex(bytes: &[u8]) -> String {
    format!("0x{}", hex::encode(bytes))
}

struct TransactionsHashView<'a> {
    transfers: &'a [Transfer],
}

impl EncodeCanonical for TransactionsHashView<'_> {
    fn encode_into(&self, out: &mut Vec<u8>) {
        let count = u16::try_from(self.transfers.len())
            .expect("transaction count exceeds u16 serialization range");

        append_u16_le(out, count);

        for transfer in self.transfers {
            transfer.bytes_view().encode_into(out);
        }
    }
}
