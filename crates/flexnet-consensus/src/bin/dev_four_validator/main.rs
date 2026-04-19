mod chain_proposal_validator;
mod in_memory_network;
mod in_memory_state;
mod passthrough_message_port;
mod validating_block_port;
mod validating_chain_port;
mod validator_node;

use crate::in_memory_network::InMemoryNetwork;
use flexnet_chain::{
    chain_config::ChainConfig,
    chain_id::ChainId,
    chain_version::ChainVersion,
    crypto::{SecretKey, address_from_secret_key},
};
use flexnet_consensus::consensus_config::ConsensusConfig;

#[tokio::main]
async fn main() {
    println!("Starting 4-validator network");

    let mut network = InMemoryNetwork::new();

    let secrets = vec![
        SecretKey::generate_random(),
        SecretKey::generate_random(),
        SecretKey::generate_random(),
        SecretKey::generate_random(),
    ];
    let addresses = secrets
        .iter()
        .map(address_from_secret_key)
        .collect::<Vec<_>>();

    println!("Addresses: {:?}", addresses);

    let chain_config = ChainConfig::new(ChainId::new(1), ChainVersion::new(1), 128);
    let consensus_configs = secrets
        .into_iter()
        .map(|secret| ConsensusConfig::new(secret, addresses.clone(), 3, 15000))
        .collect::<Vec<_>>();

    for (index, consensus_config) in consensus_configs.into_iter().enumerate() {
        network.add_validator(
            format!("val-{}", index),
            chain_config.clone(),
            consensus_config,
        );
    }

    println!("Starting network...");

    network.run(1).expect("failed to start network");

    println!("Network started");
    println!("Press CTRL+C to stop");

    async fn wait_for_ctrl_c() {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to wait for CTRL+C");
    }

    wait_for_ctrl_c().await;

    println!();
    println!("Received CTRL+C, stopping network...");

    network.stop().await;

    println!("Network stopped");
}
