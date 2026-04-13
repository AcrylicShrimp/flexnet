use crate::proposal::Proposal;
use flexnet_chain::{address::Address, hash::Hash};

pub enum StateInput<P>
where
    P: Proposal,
{
    StartHeight {
        height: u128,
    },
    StartRound {
        height: u128,
        round: u32,
    },
    ProposalReceived {
        height: u128,
        round: u32,
        proposal: P,
    },
    PrevoteReceived {
        height: u128,
        round: u32,
        address: Address,
        proposal_hash: Option<Hash>,
    },
    PrecommitReceived {
        height: u128,
        round: u32,
        address: Address,
        proposal_hash: Option<Hash>,
    },
    RoundTimeout {
        height: u128,
        round: u32,
    },
}
