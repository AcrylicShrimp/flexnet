use crate::{
    chain_id::ChainId,
    chain_version::ChainVersion,
    codec::{DecodeError, Decoder},
    hash::Hash,
    transaction::Transaction,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Block {
    pub chain_id: ChainId,
    pub chain_version: ChainVersion,
    pub block_height: u128,
    pub previous_block_hash: Hash,
    pub state_hash: Hash,
    pub transactions: Vec<Transaction>,
}

impl Block {
    pub fn new(
        chain_id: ChainId,
        chain_version: ChainVersion,
        block_height: u128,
        previous_block_hash: Hash,
        state_hash: Hash,
        transactions: Vec<Transaction>,
    ) -> Self {
        Self {
            chain_id,
            chain_version,
            block_height,
            previous_block_hash,
            state_hash,
            transactions,
        }
    }

    pub fn is_genesis(&self) -> bool {
        self.block_height == 0
    }
}

impl Block {
    pub fn encoded_len(&self) -> usize {
        let chain_id = 2;
        let chain_version = 2;
        let block_height = 16;
        let previous_block_hash = 32;
        let state_hash = 32;
        let transactions_len = 2;

        let mut transactions = 0;

        for transaction in &self.transactions {
            transactions += transaction.encoded_len();
        }

        chain_id
            + chain_version
            + block_height
            + previous_block_hash
            + state_hash
            + transactions_len
            + transactions
    }

    pub fn encode_canonical(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.chain_id.into_u16().to_le_bytes());
        out.extend_from_slice(&self.chain_version.into_u16().to_le_bytes());
        out.extend_from_slice(&self.block_height.to_le_bytes());
        out.extend_from_slice(self.previous_block_hash.as_bytes());
        out.extend_from_slice(self.state_hash.as_bytes());
        out.extend_from_slice(&(self.transactions.len() as u16).to_le_bytes());

        for transaction in &self.transactions {
            transaction.encode_canonical(out);
        }
    }

    pub fn decode_canonical(input: &[u8]) -> Result<Self, DecodeError> {
        let mut decoder = Decoder::new(input);
        let decoded = Self::decode_from(&mut decoder)?;

        decoder.finish()?;

        Ok(decoded)
    }

    pub fn decode_from(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let chain_id = ChainId::new(decoder.read_u16_le()?);
        let chain_version = ChainVersion::new(decoder.read_u16_le()?);
        let block_height = decoder.read_u128_le()?;
        let previous_block_hash = Hash::new(decoder.read_fixed::<32>()?);
        let state_hash = Hash::new(decoder.read_fixed::<32>()?);
        let transactions_len = decoder.read_u16_le()?;

        let mut transactions = Vec::with_capacity(transactions_len as usize);

        for _ in 0..transactions_len {
            let transaction = Transaction::decode_from(decoder)?;
            transactions.push(transaction);
        }

        Ok(Self::new(
            chain_id,
            chain_version,
            block_height,
            previous_block_hash,
            state_hash,
            transactions,
        ))
    }
}
