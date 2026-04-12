use crate::{block::Block, chain::Chain, error::SimulationError, genesis::Genesis};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Simulation {
    chains: Vec<Chain>,
}

impl Simulation {
    pub fn new(genesis: Genesis, node_count: usize) -> Result<Self, SimulationError> {
        if node_count == 0 {
            return Err(SimulationError::EmptySimulation);
        }

        let mut chains = Vec::with_capacity(node_count);
        for index in 0..node_count {
            let chain = Chain::new(genesis.clone())
                .map_err(|error| SimulationError::ChainInitialization { index, error })?;
            chains.push(chain);
        }

        let simulation = Self { chains };
        simulation.assert_in_sync()?;

        Ok(simulation)
    }

    pub fn from_genesis_json_str(input: &str, node_count: usize) -> Result<Self, SimulationError> {
        Self::new(Genesis::from_json_str(input)?, node_count)
    }

    pub fn node_count(&self) -> usize {
        self.chains.len()
    }

    pub fn chains(&self) -> &[Chain] {
        &self.chains
    }

    pub fn chain(&self, index: usize) -> Option<&Chain> {
        self.chains.get(index)
    }

    pub fn append_block(&mut self, block: Block) -> Result<(), SimulationError> {
        let mut next_chains = self.chains.clone();

        for (index, chain) in next_chains.iter_mut().enumerate() {
            chain
                .append_block(block.clone())
                .map_err(|error| SimulationError::ChainAppend { index, error })?;
        }

        let simulation = Self {
            chains: next_chains,
        };
        simulation.assert_in_sync()?;
        self.chains = simulation.chains;

        Ok(())
    }

    pub fn assert_in_sync(&self) -> Result<(), SimulationError> {
        let Some(reference) = self.chains.first() else {
            return Err(SimulationError::EmptySimulation);
        };

        for (index, chain) in self.chains.iter().enumerate().skip(1) {
            if chain.tip_height() != reference.tip_height() {
                return Err(SimulationError::DivergedTipHeight {
                    index,
                    expected: reference.tip_height(),
                    actual: chain.tip_height(),
                });
            }
            if chain.state_hash() != reference.state_hash() {
                return Err(SimulationError::DivergedStateHash {
                    index,
                    expected: reference.state_hash(),
                    actual: chain.state_hash(),
                });
            }
            if chain.tip_hash() != reference.tip_hash() {
                return Err(SimulationError::DivergedTipHash {
                    index,
                    expected: reference.tip_hash(),
                    actual: chain.tip_hash(),
                });
            }
        }

        Ok(())
    }
}
