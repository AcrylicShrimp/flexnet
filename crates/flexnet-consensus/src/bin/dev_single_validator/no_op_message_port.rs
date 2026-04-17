use flexnet_consensus::{message::Message, ports::message_port::MessagePort};
use tokio::sync::mpsc::{Receiver, Sender};

pub struct NoOpMessagePort {
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
