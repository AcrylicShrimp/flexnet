use flexnet_chain::address::Address;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConsensusConfig {
    pub address: Address,
    pub validators: Vec<Address>,
    pub quorum: usize,
    pub round_timeout_ms: u64,
}
