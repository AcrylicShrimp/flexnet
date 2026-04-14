use flexnet_chain::block::Block;

pub trait BlockPort {
    fn commit(&self, block: Block);
}
