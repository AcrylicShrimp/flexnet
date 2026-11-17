use crate::justification::Justification;
use flexnet_chain::hash::Hash256;

#[derive(Debug, Clone)]
pub struct Polka<P> {
    pub proposal: P,
    pub proposal_hash: Hash256,
    pub justification: Justification,
}
