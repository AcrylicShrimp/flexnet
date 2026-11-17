use flexnet_chain::{address::Address, crypto::Ed25519Signature};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Evidence {
    pub address: Address,
    pub signature: Ed25519Signature,
}

impl Evidence {
    pub fn new(address: Address, signature: Ed25519Signature) -> Self {
        Self { address, signature }
    }
}
