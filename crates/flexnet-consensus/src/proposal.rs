use flexnet_chain::hash::Hash;
use std::hash::Hash as StdHash;

pub trait Proposal: Clone + Eq + StdHash {
    fn hash(&self) -> Hash;
}
