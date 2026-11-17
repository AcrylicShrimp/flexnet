use crate::{
    chain_id::ChainId,
    chain_version::ChainVersion,
    domain::Domain,
    hash::{BlockHash, StateHash},
    magic::Magic,
    transaction::Transaction,
};
use flexnet_codec::{
    canonical_decode::CanonicalDecode,
    canonical_encode::CanonicalEncode,
    decoder::{Decoder, DecoderError},
    encoder::{Encoder, EncoderError},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Block {
    pub previous_block_hash: BlockHash,
    pub block_height: u128,
    pub transactions: Vec<Transaction>,
    pub state_hash: StateHash,
}

impl Block {
    pub fn new(
        previous_block_hash: BlockHash,
        block_height: u128,
        transactions: Vec<Transaction>,
        state_hash: StateHash,
    ) -> Self {
        Self {
            previous_block_hash,
            block_height,
            transactions,
            state_hash,
        }
    }

    pub fn is_genesis(&self) -> bool {
        self.block_height == 0
    }
}

impl Block {
    pub fn encoded_len(&self) -> usize {
        let magic = Magic::BYTES;
        let domain = Domain::BYTES;
        let chain_id = ChainId::BYTES;
        let chain_version = ChainVersion::BYTES;

        let block_height = 16;
        let previous_block_hash = Hash256::BYTES;
        let state_hash = Hash256::BYTES;
        let transactions_len = 2;

        let mut transactions = 0;

        for transaction in &self.transactions {
            transactions += transaction.encoded_len();
        }

        magic
            + domain
            + chain_id
            + chain_version
            + block_height
            + previous_block_hash
            + state_hash
            + transactions_len
            + transactions
    }

    pub fn encode_canonical(&self, chain_id: ChainId, out: &mut Vec<u8>) {
        let transaction_count = u16::try_from(self.transactions.len())
            .expect("transaction count exceeds canonical u16 range");

        out.extend_from_slice(&Magic.as_bytes());
        out.extend_from_slice(&Domain::Block.into_u8().to_le_bytes());
        out.extend_from_slice(&chain_id.into_u16().to_le_bytes());
        out.extend_from_slice(&ChainVersion.as_bytes());

        out.extend_from_slice(&self.block_height.to_le_bytes());
        out.extend_from_slice(self.previous_block_hash.as_bytes());
        out.extend_from_slice(self.state_hash.as_bytes());
        out.extend_from_slice(&transaction_count.to_le_bytes());

        for transaction in &self.transactions {
            transaction.encode_canonical(out);
        }
    }

    pub fn decode_canonical(input: &[u8], chain_id: ChainId) -> Result<Self, DecodeError> {
        let mut decoder = Decoder::new(input);
        let decoded = Self::decode_from(&mut decoder, chain_id)?;

        decoder.finish()?;

        Ok(decoded)
    }

    pub fn decode_from(decoder: &mut Decoder, chain_id: ChainId) -> Result<Self, DecodeError> {
        let _magic = decoder.read_magic()?;
        let _domain = decoder.read_domain(Domain::Block)?;
        let _chain_id = decoder.read_chain_id(chain_id)?;
        let _chain_version = decoder.read_chain_version()?;

        let block_height = decoder.read_u128_le()?;
        let previous_block_hash = Hash256::new(decoder.read_fixed::<32>()?);
        let state_hash = Hash256::new(decoder.read_fixed::<32>()?);
        let transactions_len = decoder.read_u16_le()?;

        let mut transactions = Vec::with_capacity(transactions_len as usize);

        for _ in 0..transactions_len {
            let transaction = Transaction::decode_from(decoder)?;
            transactions.push(transaction);
        }

        Ok(Self::new(
            block_height,
            previous_block_hash,
            state_hash,
            transactions,
        ))
    }
}

impl CanonicalEncode for Block {
    fn encode_to(&self, encoder: &mut Encoder) -> Result<(), EncoderError> {
        self.previous_block_hash.encode_to(encoder)?;
        self.block_height.encode_to(encoder)?;
        todo!();
        Ok(())
    }
}

impl CanonicalDecode for Block {
    fn decode_from(decoder: &mut Decoder) -> Result<Self, DecoderError> {
        Ok(Self::new(*decoder.read_fixed_bytes()?))
    }
}

#[cfg(test)]
mod tests {
    use super::Block;
    use crate::{
        address::Address,
        chain_id::ChainId,
        chain_version::ChainVersion,
        crypto::{Ed25519SecretKey, address_from_secret_key, sign},
        hash::Hash256,
        transaction::Transaction,
        transactions::tx_transfer::{TransferPayload, TxTransfer},
    };

    fn signed_transfer(seed: u8, to_seed: u8, amount: u128, nonce: u128) -> Transaction {
        let secret_key = Ed25519SecretKey::new([seed; 32]);
        let from = address_from_secret_key(&secret_key);
        let to = Address::new([to_seed; 32]);
        let payload = TransferPayload::new(
            ChainId::new(1),
            ChainVersion::new(1),
            from,
            to,
            amount,
            nonce,
        );
        let mut signing_payload = Vec::with_capacity(payload.signing_payload_len());
        payload.encode_signing_payload(&mut signing_payload);
        Transaction::Transfer(TxTransfer::new(
            payload,
            sign(&secret_key, &signing_payload),
        ))
    }

    #[test]
    fn block_roundtrips_through_canonical_codec() {
        let block = Block::new(
            ChainId::new(1),
            ChainVersion::new(1),
            42,
            Hash256::new([7; 32]),
            Hash256::new([9; 32]),
            vec![signed_transfer(1, 2, 5, 0), signed_transfer(2, 3, 2, 0)],
        );
        let mut encoded = Vec::with_capacity(block.encoded_len());
        block.encode_canonical(&mut encoded);

        let decoded = Block::decode_canonical(&encoded).unwrap();

        assert_eq!(decoded, block);
        assert_eq!(encoded.len(), block.encoded_len());
    }
}
