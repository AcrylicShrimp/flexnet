use flexnet_chain::block::Block;

pub trait ChainPort
where
    Self: 'static + Send + Sync,
{
    fn commit(&self, height: u128, block: Block);
}
