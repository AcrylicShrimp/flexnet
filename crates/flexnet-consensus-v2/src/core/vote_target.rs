use flexnet_chain::hash::Hash;

/// Value targeted by a prevote or precommit.
///
/// `Nil` is a vote target, not a proposal. It represents refusal or inability
/// to vote for a block at the current protocol point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VoteTarget {
    /// Vote for no block.
    Nil,
    /// Vote for the block identified by this hash.
    Block(Hash),
}
