use flexnet_chain::hash::Hash;

/// Proposal fact that has already passed external proposal validation.
///
/// The core only sees the proposed block hash and the proposal's safety
/// justification shape. It does not own the block body, proposer signature, or
/// raw proposal message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Proposal {
    /// A proposal that does not claim a previous prevote quorum.
    ///
    /// ## Locking
    /// A locked validator cannot unlock and re-lock on this proposal unless it
    /// is already locked on the same block.
    Fresh {
        /// Proposed block hash.
        block_hash: Hash,
    },
    /// A proposal justified by a verified prevote quorum in a previous round.
    ///
    /// ## Locking
    /// A locked validator may unlock and re-lock on this proposal only when
    /// `valid_round` safely supersedes the validator's current lock.
    Justified {
        /// Previous round in this height where `block` had quorum prevotes.
        valid_round: u64,
        /// Proposed block hash.
        block_hash: Hash,
    },
}
