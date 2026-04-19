use crate::{
    chain_proposal_validator::ChainProposalValidator, in_memory_state::InMemoryState,
    passthrough_message_port::PassthroughMessagePort, validating_block_port::ValidatingBlockPort,
    validating_chain_port::ValidatingChainPort,
};
use flexnet_chain::{
    address::Address, chain::Chain, chain_config::ChainConfig, crypto::address_from_secret_key,
    genesis::Genesis,
};
use flexnet_consensus::{
    consensus_config::ConsensusConfig,
    consensus_driver::{ConsensusDriver, ConsensusDriverStartError},
    message::Message,
};
use parking_lot::Mutex;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};

pub struct ValidatorNode {
    address: Address,
    chain: Arc<Mutex<Chain<InMemoryState>>>,
    driver: ConsensusDriver,
}

impl ValidatorNode {
    pub fn new(
        name: impl Into<String>,
        chain_config: ChainConfig,
        consensus_config: ConsensusConfig,
    ) -> Self {
        let genesis = Genesis::new(
            chain_config.clone(),
            InMemoryState::new(),
            consensus_config.validators.clone(),
        );

        let address = address_from_secret_key(&consensus_config.secret_key);
        let chain = Arc::new(Mutex::new(Chain::new(genesis)));
        let driver = ConsensusDriver::new(name, chain_config, consensus_config);

        Self {
            address,
            chain,
            driver,
        }
    }

    pub fn address(&self) -> Address {
        self.address
    }

    pub fn run(
        &mut self,
        height: u128,
        message_tx: Sender<Message>,
        message_rx: Receiver<Message>,
    ) -> Result<(), ConsensusDriverStartError> {
        let proposal_validator = ChainProposalValidator::new(self.chain.clone());
        let message_port = PassthroughMessagePort::new(message_tx, message_rx);
        let block_port = ValidatingBlockPort::new(self.chain.clone());
        let chain_port = ValidatingChainPort::new(self.chain.clone());

        self.driver.run(
            height,
            proposal_validator,
            message_port,
            block_port,
            chain_port,
        )
    }

    pub async fn stop(&mut self) {
        self.driver.stop().await;
    }
}
