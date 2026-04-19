use crate::validator_node::ValidatorNode;
use flexnet_chain::{address::Address, chain_config::ChainConfig};
use flexnet_consensus::{
    consensus_config::ConsensusConfig, consensus_driver::ConsensusDriverStartError,
    message::Message,
};
use tokio::task::JoinHandle;

pub struct InMemoryNetwork {
    validators: Vec<ValidatorNode>,
    join_handles: Option<Vec<JoinHandle<()>>>,
}

impl InMemoryNetwork {
    pub fn new() -> Self {
        Self {
            validators: vec![],
            join_handles: None,
        }
    }

    pub fn add_validator(
        &mut self,
        name: impl Into<String>,
        chain_config: ChainConfig,
        consensus_config: ConsensusConfig,
    ) {
        self.validators
            .push(ValidatorNode::new(name, chain_config, consensus_config));
    }

    pub fn run(&mut self, height: u128) -> Result<(), ConsensusDriverStartError> {
        let (val2net_tx, mut val2net_rx) = tokio::sync::mpsc::channel::<(Address, Message)>(4);
        let mut net2val_tx_with_addresses = vec![];
        let mut join_handles = vec![];

        for validator in &mut self.validators {
            let address = validator.address();
            let (net2val_tx, net2val_rx) = tokio::sync::mpsc::channel(4);
            net2val_tx_with_addresses.push((address, net2val_tx));

            let val2net_tx = val2net_tx.clone();
            let (val2net_without_addr_tx, mut val2net_without_addr_rx) =
                tokio::sync::mpsc::channel::<Message>(4);

            let join_handle = tokio::spawn(async move {
                while let Some(message) = val2net_without_addr_rx.recv().await {
                    let _ = val2net_tx.send((address, message)).await;
                }
            });
            join_handles.push(join_handle);

            validator.run(height, val2net_without_addr_tx, net2val_rx)?;
        }

        let join_handle = tokio::spawn(async move {
            while let Some((tx_address, message)) = val2net_rx.recv().await {
                for (rx_address, tx) in &net2val_tx_with_addresses {
                    if &tx_address == rx_address {
                        continue;
                    }

                    let _ = tx.send(message.clone()).await;
                }
            }
        });
        join_handles.push(join_handle);

        self.join_handles = Some(join_handles);

        Ok(())
    }

    pub async fn stop(&mut self) {
        for validator in &mut self.validators {
            validator.stop().await;
        }

        if let Some(join_handles) = self.join_handles.take() {
            for join_handle in join_handles {
                let _ = join_handle.await;
            }
        }
    }
}
