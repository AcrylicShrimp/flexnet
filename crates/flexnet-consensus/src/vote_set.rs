use crate::justification::Evidence;
use flexnet_chain::{address::Address, crypto::Signature, hash::Hash};
use std::collections::{BTreeMap, btree_map::Entry};

#[derive(Default, Debug, Clone, Hash)]
pub struct VoteSet {
    pub votes: BTreeMap<Address, Option<Hash>>,
    pub evidences: BTreeMap<Option<Hash>, Vec<Evidence>>,
}

impl VoteSet {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_vote(&mut self, address: Address, hash: Option<Hash>, signature: Signature) {
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

    pub fn any_quorum_satisfied(&self, quorum: usize) -> Option<(Option<Hash>, Vec<Evidence>)> {
        self.evidences
            .iter()
            .find(|(_, evidences)| evidences.len() >= quorum)
            .map(|(hash, evidences)| (*hash, evidences.clone()))
    }
}
