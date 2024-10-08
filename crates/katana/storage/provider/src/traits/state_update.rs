use katana_primitives::block::BlockHashOrNumber;
use katana_primitives::block::BlockNumber;
use katana_primitives::state::StateUpdates;
use katana_primitives::state::StateUpdatesWithDeclaredClasses;

use crate::ProviderResult;

#[auto_impl::auto_impl(&, Box, Arc)]
pub trait StateUpdateProvider: Send + Sync {
    /// Returns the state update at the given block.
    fn state_update(&self, block_id: BlockHashOrNumber) -> ProviderResult<Option<StateUpdates>>;
}

#[auto_impl::auto_impl(&, Box, Arc)]
pub trait StateUpdateWriter: Send + Sync {
    /// Apply the given state updates to the storage.
    fn apply_state_updates(
        &self,
        block_number: BlockNumber,
        states: StateUpdatesWithDeclaredClasses,
    ) -> ProviderResult<()>;
}
