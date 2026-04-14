use crate::polka::Polka;
use flexnet_chain::hash::Hash;

pub enum StateOutput<P> {
    StartTimeout {
        height: u128,
        round: u32,
        timeout_ms: u64,
    },
    StartRound {
        height: u128,
        round: u32,
    },
    Propose {
        height: u128,
        round: u32,
        polka: Option<Polka<P>>,
    },
    Prevote {
        height: u128,
        round: u32,
        proposal_hash: Option<Hash>,
    },
    Precommit {
        height: u128,
        round: u32,
        proposal_hash: Option<Hash>,
    },
    Commit {
        height: u128,
        round: u32,
        proposal: P,
    },
    RoundFailure {
        height: u128,
        round: u32,
        reason: RoundFailureReason,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RoundFailureReason {
    Timeout,
    NoDecision,
}
