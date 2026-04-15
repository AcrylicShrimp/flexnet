use flexnet_chain::{
    block::Block,
    chain_config::ChainConfig,
    chain_id::ChainId,
    chain_version::ChainVersion,
    crypto::{SecretKey, address_from_secret_key},
    hash::{Hash, compute_block_hash},
};
use flexnet_consensus::{
    consensus_config::ConsensusConfig,
    consensus_driver::ConsensusDriver,
    message::Message,
    ports::{block_port::BlockPort, chain_port::ChainPort, message_port::MessagePort},
};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    select,
    sync::mpsc::{Receiver, Sender},
};

#[tokio::main]
async fn main() {
    let secret_key = SecretKey::generate_random();
    let address = address_from_secret_key(&secret_key);

    let chain_config = ChainConfig::new(ChainId::new(1), ChainVersion::new(1), 128);
    let consensus_config = ConsensusConfig::new(secret_key, vec![address], 1, 5000);

    println!("Starting consensus driver with a single validator");
    println!("Validator address: {}", address);
    println!("Chain ID: {}", chain_config.chain_id);
    println!("Chain version: {}", chain_config.chain_version);

    let message_port = NoOpMessagePort::new();
    let (block_port, mut wait_for_interaction) =
        InteractiveInfiniteBlockPort::new(chain_config.chain_id, chain_config.chain_version);
    let chain_port = NoOpChainPort;
    let mut driver = ConsensusDriver::new(chain_config, consensus_config);

    println!("Starting consensus driver...");

    driver
        .run(1, message_port, block_port, chain_port)
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
            _ = wait_for_interaction.request_receiver.recv() => {
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

struct NoOpMessagePort {
    sender: Sender<Message>,
    receiver: Receiver<Message>,
    _rx_sender: Sender<Message>,
}

impl NoOpMessagePort {
    pub fn new() -> Self {
        let (tx_sender, mut tx_receiver) = tokio::sync::mpsc::channel(1);
        let (rx_sender, rx_receiver) = tokio::sync::mpsc::channel(1);

        tokio::spawn(async move { while tx_receiver.recv().await.is_some() {} });

        Self {
            sender: tx_sender,
            receiver: rx_receiver,
            _rx_sender: rx_sender,
        }
    }
}

impl MessagePort for NoOpMessagePort {
    fn sender(&self) -> &Sender<Message> {
        &self.sender
    }

    fn receiver(&mut self) -> &mut Receiver<Message> {
        &mut self.receiver
    }
}

struct WaitForInteraction {
    request_receiver: Receiver<()>,
    response_sender: Sender<()>,
}

impl WaitForInteraction {
    pub fn new() -> (Self, Sender<()>, Receiver<()>) {
        let (request_sender, request_receiver) = tokio::sync::mpsc::channel(1);
        let (response_sender, response_receiver) = tokio::sync::mpsc::channel(1);

        (
            Self {
                request_receiver,
                response_sender,
            },
            request_sender,
            response_receiver,
        )
    }
}

struct InteractiveInfiniteBlockPort {
    chain_id: ChainId,
    chain_version: ChainVersion,
    request_sender: Sender<()>,
    response_receiver: Receiver<()>,
}

impl InteractiveInfiniteBlockPort {
    pub fn new(chain_id: ChainId, chain_version: ChainVersion) -> (Self, WaitForInteraction) {
        let (wait_for_interaction, request_sender, response_receiver) = WaitForInteraction::new();

        (
            Self {
                chain_id,
                chain_version,
                request_sender,
                response_receiver,
            },
            wait_for_interaction,
        )
    }
}

impl BlockPort for InteractiveInfiniteBlockPort {
    async fn next_candidate(&mut self) -> Block {
        let _ = self.request_sender.send(()).await;
        let _ = self.response_receiver.recv().await;

        Block::new(
            self.chain_id,
            self.chain_version,
            0,
            Hash::ZERO,
            Hash::ZERO,
            Vec::new(),
        )
    }
}

struct NoOpChainPort;

impl ChainPort for NoOpChainPort {
    fn commit(&self, height: u128, block: Block) {
        let block_hash = compute_block_hash(&block);

        println!(
            "Committed block at height {} with hash {}",
            height, block_hash
        );
    }
}
