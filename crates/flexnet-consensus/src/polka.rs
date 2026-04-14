use crate::justification::Justification;
use flexnet_chain::hash::Hash;

#[derive(Clone)]
pub struct Polka<P> {
    pub proposal: P,
    pub proposal_hash: Hash,
    pub justification: Justification,
}
