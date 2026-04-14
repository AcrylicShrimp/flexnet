use flexnet_chain::hash::Hash;

pub trait Proposal: Clone {
    fn hash(&self) -> Hash;
}
