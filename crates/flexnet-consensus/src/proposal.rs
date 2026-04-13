use flexnet_chain::hash::Hash;

pub trait Proposal {
    fn hash(&self) -> Hash;
}
