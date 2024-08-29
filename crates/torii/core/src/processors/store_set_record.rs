use anyhow::{Error, Ok, Result};
use async_trait::async_trait;
use dojo_world::contracts::world::WorldContractReader;
use starknet::core::types::Event;
use starknet::providers::Provider;
use tracing::{info, warn};

use super::EventProcessor;
use crate::processors::MODEL_INDEX;
use crate::sql::Sql;

pub(crate) const LOG_TARGET: &str = "torii_core::processors::store_set_record";

#[derive(Default, Debug)]
pub struct StoreSetRecordProcessor;

#[async_trait]
impl<P> EventProcessor<P> for StoreSetRecordProcessor
where
    P: Provider + Send + Sync + std::fmt::Debug,
{
    fn event_key(&self) -> String {
        "StoreSetRecord".to_string()
    }

    fn validate(&self, event: &Event) -> bool {
        // At least 3:
        // 0: Event selector
        // 1: table
        // 2: keys (span, at least 1 felt)
        if event.keys.len() < 3 {
            warn!(
                target: LOG_TARGET,
                event_key = %<StoreSetRecordProcessor as EventProcessor<P>>::event_key(self),
                invalid_keys = %<StoreSetRecordProcessor as EventProcessor<P>>::event_keys_as_string(self, event),
                "Invalid event keys."
            );
            return false;
        }
        true
    }

    async fn process(
        &self,
        _world: &WorldContractReader<P>,
        db: &mut Sql,
        _block_number: u64,
        block_timestamp: u64,
        event_id: &str,
        event: &Event,
    ) -> Result<(), Error> {
        let mut offset = MODEL_INDEX;
        let model_selector = event.keys[offset];
        offset += 1;

        let model = db.model(model_selector).await?;

        info!(
            target: LOG_TARGET,
            name = %model.name,
            "Store set record.",
        );

        // Skip the length to only get the keys as they will be deserialized.
        offset += 1;
        let keys = event.keys[offset..].to_vec();

        // Values are serialized as a span<felt252>. We need to skip the first key which is the
        // length of the serialized span to only consider the data.
        let data = event.data[1..].to_vec();
        let mut keys_and_unpacked = [keys, data].concat();

        let mut entity = model.schema;
        entity.deserialize(&mut keys_and_unpacked)?;

        db.set_entity(entity, event_id, block_timestamp).await?;
        Ok(())
    }
}
