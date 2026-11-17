use crate::{proposal::Proposal, vote_set::VoteSet};
use flexnet_chain::hash::Hash256;

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
        prevote: Option<Hash256>,
        prevote_set: VoteSet,
        precommit_set: VoteSet,
    },
    Precommit {
        proposal: Option<P>,
        prevote: Option<Hash256>,
        prevote_set: VoteSet,
        precommit: Option<Hash256>,
        precommit_set: VoteSet,
    },
    Commit,
    Failure,
}
