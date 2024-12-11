use std::collections::BTreeMap;

use katana_primitives::block::BlockNumber;
use katana_primitives::class::{ClassHash, CompiledClassHash};
use katana_primitives::state::StateUpdates;
use katana_primitives::Felt;
use katana_trie::MultiProof;

use crate::ProviderResult;

#[auto_impl::auto_impl(&, Box, Arc)]
pub trait ClassTrieProvider: Send + Sync {
    fn proofs(
        &self,
        block_number: BlockNumber,
        class_hashes: &[ClassHash],
    ) -> ProviderResult<MultiProof>;

    fn root(&self) -> ProviderResult<Felt>;
}

#[auto_impl::auto_impl(&, Box, Arc)]
pub trait ContractTrieProvider: Send + Sync {
    fn proofs(
        &self,
        block_number: BlockNumber,
        class_hashes: &[ClassHash],
    ) -> ProviderResult<MultiProof>;

    fn root(&self) -> ProviderResult<Felt>;
}

#[auto_impl::auto_impl(&, Box, Arc)]
pub trait ClassTrieWriter: Send + Sync {
    fn insert_updates(
        &self,
        block_number: BlockNumber,
        updates: &BTreeMap<ClassHash, CompiledClassHash>,
    ) -> ProviderResult<Felt>;
}

#[auto_impl::auto_impl(&, Box, Arc)]
pub trait ContractTrieWriter: Send + Sync {
    fn insert_updates(
        &self,
        block_number: BlockNumber,
        state_updates: &StateUpdates,
    ) -> ProviderResult<Felt>;
}
