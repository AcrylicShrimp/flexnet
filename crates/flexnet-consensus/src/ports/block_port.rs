use flexnet_chain::block::Block;

pub trait BlockPort
where
    Self: 'static + Send + Sync,
{
    fn next_candidate(&self, height: u128) -> impl Future<Output = Option<Block>> + Send;
}
