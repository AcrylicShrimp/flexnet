use flexnet_chain::hash::Hash256;
use std::hash::Hash as StdHash;

pub trait Proposal: Clone + Eq + StdHash {
    fn hash(&self) -> Hash256;
}
