use crate::{
    block::Block, error::ChainError, execute::execute_block, genesis::Genesis, hash::Hash,
    state::State,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chain {
    genesis: Genesis,
    blocks: Vec<Block>,
    state: State,
    state_hash: Hash,
    tip_hash: Hash,
}

impl Chain {
    pub fn new(genesis: Genesis) -> Result<Self, ChainError> {
        genesis.validate()?;

        Ok(Self {
            state: genesis.state.clone(),
            state_hash: genesis.state_hash(),
            tip_hash: genesis.block_hash(),
            genesis,
            blocks: Vec::new(),
        })
    }

    pub fn from_genesis_json_str(input: &str) -> Result<Self, ChainError> {
        Self::new(Genesis::from_json_str(input)?)
    }

    pub fn genesis(&self) -> &Genesis {
        &self.genesis
    }

    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }

    pub fn tip_block(&self) -> Option<&Block> {
        self.blocks.last()
    }

    pub fn tip_height(&self) -> u128 {
        self.blocks.len() as u128
    }

    pub fn next_block_height(&self) -> u128 {
        self.tip_height() + 1
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn state_hash(&self) -> Hash {
        self.state_hash
    }

    pub fn tip_hash(&self) -> Hash {
        self.tip_hash
    }

    pub fn append_block(&mut self, block: Block) -> Result<(), ChainError> {
        let expected_height = self.next_block_height();
        if block.block_height != expected_height {
            return Err(ChainError::UnexpectedBlockHeight {
                expected: expected_height,
                actual: block.block_height,
            });
        }

        let outcome = execute_block(&self.state, self.tip_hash, &block)?;

        self.state = outcome.state;
        self.state_hash = outcome.state_hash;
        self.tip_hash = outcome.block_hash;
        self.blocks.push(block);

        Ok(())
    }
}
