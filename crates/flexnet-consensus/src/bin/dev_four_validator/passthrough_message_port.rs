use flexnet_consensus::{message::Message, ports::message_port::MessagePort};
use tokio::sync::mpsc::{Receiver, Sender};

pub struct PassthroughMessagePort {
    tx: Sender<Message>,
    rx: Receiver<Message>,
}

impl PassthroughMessagePort {
    pub fn new(tx: Sender<Message>, rx: Receiver<Message>) -> Self {
        Self { tx, rx }
    }
}

impl MessagePort for PassthroughMessagePort {
    fn sender(&self) -> &Sender<Message> {
        &self.tx
    }

    fn receiver(&mut self) -> &mut Receiver<Message> {
        &mut self.rx
    }
}
