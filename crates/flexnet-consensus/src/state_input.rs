use crate::{justification::Justification, proposal::Proposal};
use flexnet_chain::{address::Address, crypto::Ed25519Signature, hash::Hash256};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
        proposal_hash: Option<Hash256>,
        signature: Ed25519Signature,
    },
    PrecommitReceived {
        height: u128,
        round: u32,
        address: Address,
        proposal_hash: Option<Hash256>,
        signature: Ed25519Signature,
    },
    RoundTimeout {
        height: u128,
        round: u32,
    },
}
