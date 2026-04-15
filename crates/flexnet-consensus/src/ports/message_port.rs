use crate::message::Message;
use tokio::sync::mpsc::{Receiver, Sender};

pub trait MessagePort
where
    Self: 'static + Send + Sync,
{
    fn sender(&self) -> &Sender<Message>;
    fn receiver(&mut self) -> &mut Receiver<Message>;
}
