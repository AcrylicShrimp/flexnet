use flexnet_chain::hash::Hash;

/// Hint that should be attached when the local validator proposes a block.
///
/// This is a snapshot of the core's current valid value. The reactor may use it
/// after `CoreEffect::EnteredPropose` if the local validator is the proposer
/// for that round.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProposalHint {
    /// Previous round in this height where `block_hash` had quorum prevotes.
    pub valid_round: u64,
    /// Block hash that should be re-proposed with `valid_round`.
    pub block_hash: Hash,
}
