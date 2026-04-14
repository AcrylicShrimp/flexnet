use crate::{justification::Justification, proposal::Proposal};
use flexnet_chain::{address::Address, crypto::Signature, hash::Hash};

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
        /// verified justification of the proposal
        justification: Option<Justification>,
    },
    PrevoteReceived {
        height: u128,
        round: u32,
        address: Address,
        proposal_hash: Option<Hash>,
        /// verified signature of the prevote
        signature: Signature,
    },
    PrecommitReceived {
        height: u128,
        round: u32,
        address: Address,
        proposal_hash: Option<Hash>,
        /// verified signature of the precommit
        signature: Signature,
    },
    RoundTimeout {
        height: u128,
        round: u32,
    },
}
