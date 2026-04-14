use flexnet_chain::{address::Address, crypto::Signature};

#[derive(Debug, Clone, Hash)]
pub struct Justification {
    pub height: u128,
    pub round: u32,
    pub evidences: Vec<Evidence>,
}

#[derive(Debug, Clone, Hash)]
pub struct Evidence {
    pub address: Address,
    pub signature: Signature,
}
