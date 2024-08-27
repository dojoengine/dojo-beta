use anyhow::{Error, Ok, Result};
use async_trait::async_trait;
use dojo_world::contracts::naming;
use dojo_world::contracts::world::WorldContractReader;
use starknet::core::types::{Event, TransactionReceiptWithBlockInfo};
use starknet::providers::Provider;
use tracing::info;

use super::EventProcessor;
use crate::processors::{ENTITY_ID_INDEX, MODEL_INDEX};
use crate::sql::Sql;

pub(crate) const LOG_TARGET: &str = "torii_core::processors::store_update_record";

#[derive(Default, Debug)]
pub struct StoreUpdateRecordProcessor;

#[async_trait]
impl<P> EventProcessor<P> for StoreUpdateRecordProcessor
where
    P: Provider + Send + Sync + std::fmt::Debug,
{
    fn event_key(&self) -> String {
        "StoreUpdateRecord".to_string()
    }

    fn validate(&self, event: &Event) -> bool {
        if event.keys.len() > 1 {
            info!(
                target: LOG_TARGET,
                event_key = %<StoreUpdateRecordProcessor as EventProcessor<P>>::event_key(self),
                invalid_keys = %<StoreUpdateRecordProcessor as EventProcessor<P>>::event_keys_as_string(self, event),
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
        _transaction_receipt: &TransactionReceiptWithBlockInfo,
        event_id: &str,
        event: &Event,
    ) -> Result<(), Error> {
        let selector = event.keys[MODEL_INDEX];
        let entity_id = event.keys[ENTITY_ID_INDEX];

        let model = db.model(selector).await?;

        info!(
            target: LOG_TARGET,
            name = %model.name,
            entity_id = format!("{:#x}", entity_id),
            "Store update record.",
        );

        // Skip the length to only get the values as they will be deserialized.
        let values = event.data[1..].to_vec();

        let tag = naming::get_tag(&model.namespace, &model.name);

        // Keys are read from the db, since we don't have access to them when only
        // the entity id is passed.
        let keys = db.get_entity_keys(entity_id, &tag).await?;
        let mut keys_and_unpacked = [keys, values].concat();

        let mut entity = model.schema;
        entity.deserialize(&mut keys_and_unpacked)?;

        db.set_entity(entity, event_id, block_timestamp).await?;
        Ok(())
    }
}
