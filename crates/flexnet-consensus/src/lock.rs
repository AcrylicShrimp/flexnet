use flexnet_chain::hash::Hash256;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Lock {
    pub round: u32,
    pub proposal_hash: Hash256,
}
