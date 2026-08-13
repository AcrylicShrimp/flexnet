use crate::core::{proposal::Proposal, timeout_stage::CoreTimeoutStage, vote_target::VoteTarget};

/// Normalized facts that are allowed to drive the safety core.
///
/// Inputs must already be reduced from raw network messages into verified
/// consensus facts by the reactor, validation, and evidence layers. The core
/// must not receive raw messages, peer state, gossip metadata, or unverified
/// votes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CoreInput {
    /// A valid proposal is available for this round.
    ///
    /// The proposal body, proposer identity, signature, height, round, and
    /// optional justification must already have been validated outside the
    /// core. Invalid proposals are not represented here; if no valid proposal
    /// becomes available, the propose timeout path should drive a nil prevote.
    ProposalReady {
        /// Round in which the proposal was made.
        round: u64,
        /// Verified proposal target and optional justification.
        proposal: Proposal,
    },
    /// A quorum of prevotes is ready to drive the core for this round and target.
    ///
    /// For `VoteTarget::Block`, the matching proposal and block must already be
    /// locally available and validated. A verified quorum of prevotes must exist
    /// for the same height, round, and block. The evidence layer must retain
    /// whichever parts arrive first and emit this input only after the proposal,
    /// block, and prevote quorum are all ready.
    ///
    /// For `VoteTarget::Nil`, the verified quorum alone is sufficient. A nil
    /// quorum may advance the voting stage but must not create a block lock.
    PrevoteQuorumReady {
        /// Round whose prevotes reached quorum.
        round: u64,
        /// Block or nil target that received quorum prevotes.
        target: VoteTarget,
    },
    /// A precommit quorum is ready to drive the core for this round and target.
    ///
    /// For `VoteTarget::Block`, the matching proposal and block must already be
    /// locally available and validated. A verified quorum of precommits must exist
    /// for the same height, round, and block. The evidence layer must retain
    /// whichever parts arrive first and emit this input only after the proposal,
    /// block, and precommit quorum are all ready.
    ///
    /// For `VoteTarget::Nil`, the verified quorum alone is sufficient. A nil
    /// quorum is round-progress evidence and must not commit the height.
    PrecommitQuorumReady {
        /// Round whose precommits reached quorum.
        round: u64,
        /// Block or nil target whose readiness conditions have been satisfied.
        target: VoteTarget,
    },
    /// The reactor observed that the timer for this round and stage elapsed.
    ///
    /// Duration, timer handles, backoff policy, and async runtime details stay
    /// outside the core. The core only checks whether this timeout still
    /// matches its current round and stage, then applies the protocol
    /// transition if it is current.
    TimedOut {
        /// Round for which the timeout was armed.
        round: u64,
        /// Stage for which the timeout was armed.
        stage: CoreTimeoutStage,
    },
}
