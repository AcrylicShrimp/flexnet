use flexnet_chain::{
    address::Address,
    crypto::{SecretKey, address_from_secret_key},
};

pub struct ConsensusConfig {
    pub secret_key: SecretKey,
    pub address: Address,
    pub validators: Vec<Address>,
    pub quorum: usize,
    pub round_timeout_ms: u64,
}

impl ConsensusConfig {
    pub fn new(
        secret_key: SecretKey,
        validators: Vec<Address>,
        quorum: usize,
        round_timeout_ms: u64,
    ) -> Self {
        let address = address_from_secret_key(&secret_key);

        Self {
            secret_key,
            address,
            validators,
            quorum,
            round_timeout_ms,
        }
    }
}
