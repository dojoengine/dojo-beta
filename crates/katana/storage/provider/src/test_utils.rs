use crate::providers::db::DbProvider;
use crate::{providers::in_memory::InMemoryProvider, traits::block::BlockWriter};

use katana_db::mdbx::{test_utils, DbEnvKind};
use katana_primitives::block::{BlockHash, FinalityStatus};
use katana_primitives::genesis::{Genesis, GenesisBuilder};

const GENESIS_BUILD_ERROR: &str = "Failed to build genesis block.";
const GENESIS_INIT_ERROR: &str =
    "Failed to initialize test provider with genesis block and states.";

/// Creates an in-memory provider for testing.
pub fn create_test_in_memory_provider() -> InMemoryProvider {
    let provider = InMemoryProvider::new();
    initialize_test_provider(&provider);
    provider
}

/// Creates a persistent storage provider for testing.
pub fn create_test_db_provider() -> DbProvider {
    let provider = DbProvider::new(test_utils::create_test_db(DbEnvKind::RW));
    initialize_test_provider(&provider);
    provider
}

/// Initializes the provider with a genesis block and states.
fn initialize_test_provider<P: BlockWriter>(provider: &P) {
    let genesis = create_genesis_for_testing();

    let hash = BlockHash::ZERO;
    let status = FinalityStatus::AcceptedOnL2;

    let block = genesis.block().seal_with_hash_and_status(hash, status);
    let states = genesis.state_updates();

    provider
        .insert_block_with_states_and_receipts(block, states, Vec::new(), Vec::new())
        .expect(GENESIS_INIT_ERROR);
}

/// Creates a genesis config specifically for testing purposes.
fn create_genesis_for_testing() -> Genesis {
    // TODO(kariy): add test account here
    GenesisBuilder::from(Genesis::default()).build().expect(GENESIS_BUILD_ERROR)
}
