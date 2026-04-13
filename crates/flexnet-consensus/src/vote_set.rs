use flexnet_chain::{address::Address, hash::Hash};
use std::collections::{BTreeMap, btree_map::Entry};

#[derive(Default, Debug, Clone, Hash)]
pub struct VoteSet {
    pub votes: BTreeMap<Address, Option<Hash>>,
    pub counts: BTreeMap<Option<Hash>, usize>,
}

impl VoteSet {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_vote(&mut self, address: Address, hash: Option<Hash>) {
        let entry = match self.votes.entry(address) {
            Entry::Vacant(entry) => entry,
            Entry::Occupied(_) => {
                return;
            }
        };

        entry.insert(hash);

        self.counts
            .entry(hash)
            .and_modify(|count| *count += 1)
            .or_insert(1);
    }

    pub fn any_quorum_satisfied(&self, quorum: usize) -> Option<Option<Hash>> {
        self.counts
            .iter()
            .find(|(_, count)| **count >= quorum)
            .map(|(hash, _)| *hash)
    }
}
