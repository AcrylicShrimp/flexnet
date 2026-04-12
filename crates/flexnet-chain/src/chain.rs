use crate::{
    block::Block,
    chain_config::ChainConfig,
    genesis::Genesis,
    hash::{Hash, compute_block_hash},
    rules::rule_block::{BlockExecuteError, execute_block},
    state::WritableState,
};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chain<S>
where
    S: WritableState,
{
    config: ChainConfig,
    state: S,
    tip_block: Block,
    tip_block_hash: Hash,
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ChainAppendError {
    #[error("block height overflow")]
    BlockHeightOverflow,
    #[error("unexpected block height: expected {expected}, got {actual}")]
    UnexpectedBlockHeight { expected: u128, actual: u128 },
    #[error("previous block hash mismatch: expected {expected}, got {actual}")]
    PreviousBlockHashMismatch { expected: Hash, actual: Hash },
    #[error("block execution failed: {0}")]
    BlockExecutionError(#[from] BlockExecuteError),
}

impl<S> Chain<S>
where
    S: WritableState,
{
    pub fn new(genesis: Genesis<S>) -> Self {
        let (config, state, tip_block) = genesis.into_genesis_block();
        let tip_block_hash = compute_block_hash(&tip_block);

        Self {
            config,
            state,
            tip_block,
            tip_block_hash,
        }
    }

    pub fn config(&self) -> &ChainConfig {
        &self.config
    }

    pub fn state(&self) -> &S {
        &self.state
    }

    pub fn tip_block(&self) -> &Block {
        &self.tip_block
    }

    pub fn tip_height(&self) -> u128 {
        self.tip_block.block_height
    }

    pub fn next_block_height(&self) -> Option<u128> {
        self.tip_height().checked_add(1)
    }

    pub fn tip_block_hash(&self) -> Hash {
        self.tip_block_hash
    }

    pub fn append_block(&mut self, block: Block) -> Result<(), ChainAppendError> {
        let expected_height = self
            .tip_block
            .block_height
            .checked_add(1)
            .ok_or(ChainAppendError::BlockHeightOverflow)?;

        if block.block_height != expected_height {
            return Err(ChainAppendError::UnexpectedBlockHeight {
                expected: expected_height,
                actual: block.block_height,
            });
        }

        if block.previous_block_hash != self.tip_block_hash {
            return Err(ChainAppendError::PreviousBlockHashMismatch {
                expected: self.tip_block_hash,
                actual: block.previous_block_hash,
            });
        }

        let outcome = execute_block(&block, &self.config, &self.state)?;
        let block_hash = compute_block_hash(&block);

        self.state.apply_delta(outcome.state_delta);
        self.tip_block = block;
        self.tip_block_hash = block_hash;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Chain, ChainAppendError};
    use crate::{
        account::Account,
        address::Address,
        block::Block,
        chain_config::ChainConfig,
        chain_id::ChainId,
        chain_version::ChainVersion,
        crypto::{SecretKey, address_from_secret_key, sign},
        genesis::Genesis,
        hash::{Hash, compute_state_hash},
        state::{State, StateView},
        transaction::Transaction,
        transactions::tx_transfer::{TransferPayload, TxTransfer},
    };
    use std::collections::BTreeMap;

    fn state_with_accounts(accounts: &[(Address, Account)]) -> State {
        State::new(BTreeMap::from_iter(accounts.iter().copied()))
    }

    fn signed_transfer(
        secret_key: &SecretKey,
        to: Address,
        amount: u128,
        nonce: u128,
    ) -> Transaction {
        let payload = TransferPayload::new(
            ChainId::new(1),
            ChainVersion::new(1),
            address_from_secret_key(secret_key),
            to,
            amount,
            nonce,
        );
        let mut signing_payload = Vec::with_capacity(payload.signing_payload_len());
        payload.encode_signing_payload(&mut signing_payload);
        Transaction::Transfer(TxTransfer::new(payload, sign(secret_key, &signing_payload)))
    }

    fn config() -> ChainConfig {
        ChainConfig {
            chain_id: ChainId::new(1),
            chain_version: ChainVersion::new(1),
            max_transactions_per_block: 16,
        }
    }

    #[test]
    fn append_block_updates_tip_and_state() {
        let alice_key = SecretKey::new([1; 32]);
        let bob_key = SecretKey::new([2; 32]);
        let alice = address_from_secret_key(&alice_key);
        let bob = address_from_secret_key(&bob_key);
        let mut chain = Chain::new(Genesis::new(
            config(),
            state_with_accounts(&[(alice, Account::new(100, 0)), (bob, Account::new(0, 0))]),
        ));
        let tx = signed_transfer(&alice_key, bob, 40, 0);
        let next_state =
            state_with_accounts(&[(alice, Account::new(60, 1)), (bob, Account::new(40, 0))]);
        let block = Block::new(
            ChainId::new(1),
            ChainVersion::new(1),
            chain.next_block_height().unwrap(),
            chain.tip_block_hash(),
            compute_state_hash(&next_state),
            vec![tx],
        );

        chain.append_block(block.clone()).unwrap();

        assert_eq!(chain.tip_height(), 1);
        assert_eq!(chain.tip_block(), &block);
        assert_eq!(chain.state().get_account(&alice), Account::new(60, 1));
        assert_eq!(chain.state().get_account(&bob), Account::new(40, 0));
    }

    #[test]
    fn reject_block_with_wrong_previous_hash_without_mutating_chain() {
        let alice_key = SecretKey::new([1; 32]);
        let alice = address_from_secret_key(&alice_key);
        let mut chain = Chain::new(Genesis::new(
            config(),
            state_with_accounts(&[(alice, Account::new(100, 0))]),
        ));
        let before = chain.clone();
        let block = Block::new(
            ChainId::new(1),
            ChainVersion::new(1),
            1,
            Hash::new([9; 32]),
            Hash::ZERO,
            Vec::new(),
        );

        assert_eq!(
            chain.append_block(block),
            Err(ChainAppendError::PreviousBlockHashMismatch {
                expected: before.tip_block_hash(),
                actual: Hash::new([9; 32]),
            })
        );
        assert_eq!(chain, before);
    }
}
