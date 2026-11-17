use crate::core::{proposal_hint::ProposalHint, vote_target::VoteTarget};
use flexnet_chain::hash::Hash256;

/// Deterministic intents emitted by the safety core.
///
/// Effects describe what the runtime should do next, but they do not perform
/// I/O. Signing, broadcasting, timer arming, block application, and evidence
/// insertion remain reactor responsibilities.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CoreEffect {
    /// The core entered the propose stage for a round.
    ///
    /// If the local validator is the proposer for this round, the reactor may
    /// build and broadcast a proposal. `proposal_hint` tells the reactor to
    /// re-propose a previously valid block when the core has one.
    EnteredPropose {
        /// Round now in the propose stage.
        round: u64,
        /// Optional valid value that should be included in a local proposal.
        proposal_hint: Option<ProposalHint>,
    },
    /// The core entered the prevote stage for a round.
    EnteredPrevote {
        /// Round now in the prevote stage.
        round: u64,
    },
    /// The core entered the precommit stage for a round.
    EnteredPrecommit {
        /// Round now in the precommit stage.
        round: u64,
    },
    /// The local validator should sign and publish a prevote.
    ///
    /// The reactor is responsible for signing the vote, inserting the local
    /// vote into the evidence layer, and broadcasting it.
    VotePrevote {
        /// Round for the prevote.
        round: u64,
        /// Block or nil target chosen by the core.
        target: VoteTarget,
    },
    /// The local validator should sign and publish a precommit.
    ///
    /// The reactor is responsible for signing the vote, inserting the local
    /// vote into the evidence layer, and broadcasting it.
    VotePrecommit {
        /// Round for the precommit.
        round: u64,
        /// Block or nil target chosen by the core.
        target: VoteTarget,
    },
    /// The core decided a block from a complete commit-ready fact.
    ///
    /// The matching proposal and block are already retained and validated outside
    /// the core, and the matching precommit quorum has been verified. The reactor
    /// must apply the retained block through the normal chain execution path and
    /// advance to the next height only after application succeeds.
    Commit {
        /// Round whose precommit quorum committed the block.
        round: u64,
        /// Hash of the retained and validated block to commit.
        block_hash: Hash256,
    },
}
