use flexnet_chain::{address::Address, crypto::Signature};

#[derive(Debug, Clone, Hash)]
pub struct Justification {
    pub height: u128,
    pub round: u32,
    pub evidences: Vec<Evidence>,
}

impl Justification {
    pub fn new(height: u128, round: u32, evidences: Vec<Evidence>) -> Self {
        Self {
            height,
            round,
            evidences,
        }
    }
}

#[derive(Debug, Clone, Hash)]
pub struct Evidence {
    pub address: Address,
    pub signature: Signature,
}

impl Evidence {
    pub fn new(address: Address, signature: Signature) -> Self {
        Self { address, signature }
    }
}
