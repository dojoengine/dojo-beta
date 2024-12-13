use bonsai_trie::{BonsaiDatabase, BonsaiPersistentDatabase, MultiProof};
use katana_primitives::block::BlockNumber;
use katana_primitives::{ContractAddress, Felt};

use crate::id::CommitId;

pub struct ContractsTrie<DB: BonsaiDatabase> {
    pub trie: crate::BonsaiTrie<DB>,
}

impl<DB: BonsaiDatabase> ContractsTrie<DB> {
    /// NOTE: The identifier value is only relevant if the underlying [`BonsaiDatabase`]
    /// implementation is shared across other tries.
    const BONSAI_IDENTIFIER: &'static [u8] = b"contracts";

    pub fn root(&self, address: ContractAddress) -> Felt {
        self.trie.root(&address.to_bytes_be())
    }

    pub fn multiproof(&mut self, addresses: Vec<ContractAddress>) -> MultiProof {
        let keys = addresses.into_iter().map(Felt::from).collect::<Vec<Felt>>();
        self.trie.multiproof(Self::BONSAI_IDENTIFIER, keys)
    }
}

impl<DB> ContractsTrie<DB>
where
    DB: BonsaiDatabase + BonsaiPersistentDatabase<CommitId>,
{
    pub fn insert(&mut self, address: ContractAddress, state_hash: Felt) {
        self.trie.insert(Self::BONSAI_IDENTIFIER, *address, state_hash)
    }

    pub fn commit(&mut self, block: BlockNumber) {
        self.trie.commit(block.into())
    }
}
