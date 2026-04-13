use crate::{proposal::Proposal, vote_set::VoteSet};
use flexnet_chain::hash::Hash;

#[derive(Debug, Clone, Hash)]
pub enum State<P>
where
    P: Proposal,
{
    Propose {
        prevote_set: VoteSet,
        precommit_set: VoteSet,
    },
    Prevote {
        proposal: Option<P>,
        prevote: Option<Hash>,
        prevote_set: VoteSet,
        precommit_set: VoteSet,
    },
    Precommit {
        proposal: Option<P>,
        prevote: Option<Hash>,
        precommit: Option<Hash>,
        precommit_set: VoteSet,
    },
    Commit,
    Failure,
}
