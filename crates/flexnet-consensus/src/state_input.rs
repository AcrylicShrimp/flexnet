use crate::{justification::Justification, proposal::Proposal};
use flexnet_chain::{address::Address, crypto::Signature, hash::Hash};

#[derive(Debug)]
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
        justification: Option<Justification>,
    },
    PrevoteReceived {
        height: u128,
        round: u32,
        address: Address,
        proposal_hash: Option<Hash>,
        signature: Signature,
    },
    PrecommitReceived {
        height: u128,
        round: u32,
        address: Address,
        proposal_hash: Option<Hash>,
        signature: Signature,
    },
    RoundTimeout {
        height: u128,
        round: u32,
    },
}
