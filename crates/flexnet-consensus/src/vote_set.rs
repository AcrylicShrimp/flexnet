use crate::justification::Evidence;
use flexnet_chain::{address::Address, crypto::Ed25519Signature, hash::Hash256};
use std::collections::{BTreeMap, btree_map::Entry};

#[derive(Default, Debug, Clone, Hash)]
pub struct VoteSet {
    pub votes: BTreeMap<Address, Option<Hash256>>,
    pub evidences: BTreeMap<Option<Hash256>, Vec<Evidence>>,
}

impl VoteSet {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_vote(
        &mut self,
        address: Address,
        hash: Option<Hash256>,
        signature: Ed25519Signature,
    ) {
        let entry = match self.votes.entry(address) {
            Entry::Vacant(entry) => entry,
            Entry::Occupied(_) => {
                return;
            }
        };

        entry.insert(hash);

        self.evidences
            .entry(hash)
            .and_modify(|evidences| evidences.push(Evidence { address, signature }))
            .or_insert(vec![Evidence { address, signature }]);
    }

    pub fn any_quorum_satisfied(&self, quorum: usize) -> Option<(Option<Hash256>, Vec<Evidence>)> {
        self.evidences
            .iter()
            .find(|(_, evidences)| evidences.len() >= quorum)
            .map(|(hash, evidences)| (*hash, evidences.clone()))
    }
}
