use flexnet_chain::hash::Hash;

/// Stage of the fixed-height consensus core.
///
/// Each stage can have a reactor-managed timeout. The core receives only the
/// resulting `TimedOut` fact and never manages timers directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoreStage {
    /// Waiting for a valid proposal or for the propose timeout.
    Propose,
    /// Waiting for prevote quorum evidence after casting or observing prevotes.
    Prevote,
    /// Waiting for precommit quorum evidence after casting or observing precommits.
    Precommit,
    /// A block is committed and the consensus core is waiting for the reactor to apply it and advance to the next height.
    Commit { block_hash: Hash },
}
