use flexnet_chain::hash::Hash256;

/// The matching validated block must be retained outside the core for the
/// remainder of this height.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ValidProposal {
    pub round: u64,
    pub block_hash: Hash256,
}
