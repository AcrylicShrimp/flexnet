mod chain_proposal_validator;
mod in_memory_state;
mod interactive_block_port;
mod no_op_message_port;
mod validating_block_port;
mod validating_chain_port;

use crate::{
    chain_proposal_validator::ChainProposalValidator, in_memory_state::InMemoryState,
    interactive_block_port::InteractiveInfiniteBlockPort, no_op_message_port::NoOpMessagePort,
    validating_block_port::ValidatingBlockPort, validating_chain_port::ValidatingChainPort,
};
use flexnet_chain::{
    chain::Chain,
    chain_config::ChainConfig,
    chain_id::ChainId,
    chain_version::ChainVersion,
    crypto::{SecretKey, address_from_secret_key},
    genesis::Genesis,
};
use flexnet_consensus::{consensus_config::ConsensusConfig, consensus_driver::ConsensusDriver};
use parking_lot::Mutex;
use std::sync::Arc;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    select,
};

#[tokio::main]
async fn main() {
    let secret_key = SecretKey::generate_random();
    let address = address_from_secret_key(&secret_key);

    let chain_config = ChainConfig::new(ChainId::new(1), ChainVersion::new(1), 128);
    let consensus_config = ConsensusConfig::new(secret_key, vec![address], 1, 50000000);

    println!("Starting consensus driver with a single validator");
    println!("Validator address: {}", address);
    println!("Chain ID: {}", chain_config.chain_id);
    println!("Chain version: {}", chain_config.chain_version);

    let genesis = Genesis::new(chain_config.clone(), InMemoryState::new(), vec![address]);
    let chain = Arc::new(Mutex::new(Chain::new(genesis)));

    let proposal_validator = ChainProposalValidator::new(chain.clone());
    let message_port = NoOpMessagePort::new();
    let block_port = ValidatingBlockPort::new(chain.clone());
    let chain_port = ValidatingChainPort::new(chain);

    let (interactive_block_port, mut wait_for_interaction) =
        InteractiveInfiniteBlockPort::new(block_port);
    let mut driver = ConsensusDriver::new("val", chain_config, consensus_config);

    println!("Starting consensus driver...");

    driver
        .run(
            1,
            proposal_validator,
            message_port,
            interactive_block_port,
            chain_port,
        )
        .expect("failed to start consensus driver");

    println!("Consensus driver started");
    println!("Press CTRL+C to stop");

    async fn wait_for_enter() {
        let mut reader = BufReader::new(tokio::io::stdin()).lines();
        reader.next_line().await.expect("failed to read line");
    }

    async fn wait_for_ctrl_c() {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to wait for CTRL+C");
    }

    loop {
        select! {
            _ = wait_for_interaction.request_receiver   .recv() => {
                println!("[ACTION REQUIRED] Press ENTER to feed the consensus driver a block");

                select! {
                    _ = wait_for_enter() => {
                        println!("Block fed to the consensus driver");
                        let _ = wait_for_interaction.response_sender.send(()).await;
                    }
                    _ = wait_for_ctrl_c() => {
                        break;
                    }
                }
            }
            _ = wait_for_ctrl_c() => {
                break;
            }
        }
    }

    println!();
    println!("Received CTRL+C, stopping consensus driver...");

    driver.stop().await;

    println!("Consensus driver stopped");
}
