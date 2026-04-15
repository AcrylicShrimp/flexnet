use flexnet_chain::block::Block;

pub trait BlockPort
where
    Self: 'static + Send + Sync,
{
    fn next_candidate(&mut self) -> impl Future<Output = Block> + Send;
}
