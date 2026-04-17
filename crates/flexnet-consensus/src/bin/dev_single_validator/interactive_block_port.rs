use flexnet_chain::block::Block;
use flexnet_consensus::ports::block_port::BlockPort;
use tokio::sync::{
    Mutex,
    mpsc::{Receiver, Sender},
};

pub struct WaitForInteraction {
    pub request_receiver: Receiver<()>,
    pub response_sender: Sender<()>,
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

pub struct InteractiveInfiniteBlockPort<B>
where
    B: BlockPort,
{
    block_port: B,
    request_sender: Sender<()>,
    response_receiver: Mutex<Receiver<()>>,
}

impl<B> InteractiveInfiniteBlockPort<B>
where
    B: BlockPort,
{
    pub fn new(block_port: B) -> (Self, WaitForInteraction) {
        let (wait_for_interaction, request_sender, response_receiver) = WaitForInteraction::new();

        (
            Self {
                block_port,
                request_sender,
                response_receiver: Mutex::new(response_receiver),
            },
            wait_for_interaction,
        )
    }
}

impl<B> BlockPort for InteractiveInfiniteBlockPort<B>
where
    B: BlockPort,
{
    async fn next_candidate(&self, height: u128) -> Option<Block> {
        let mut response_receiver = self.response_receiver.lock().await;

        let _ = self.request_sender.send(()).await;
        let _ = response_receiver.recv().await;

        self.block_port.next_candidate(height).await
    }
}
