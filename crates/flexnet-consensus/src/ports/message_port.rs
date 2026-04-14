use crate::message::Message;
use tokio::sync::mpsc::{Receiver, Sender};

pub trait MessagePort {
    fn sender(&self) -> Sender<Message>;
    fn receiver(&self) -> Receiver<Message>;
}
