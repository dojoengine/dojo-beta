use anyhow::{Error, Result};
use async_trait::async_trait;
use dojo_world::contracts::model::ModelReader;
use dojo_world::contracts::world::WorldContractReader;
use starknet::core::types::{BlockWithTxs, Event, TransactionReceipt};
use starknet::providers::Provider;

use super::EventProcessor;
use crate::processors::MODEL_INDEX;
use crate::sql::Sql;

#[derive(Default)]
pub struct EventMessageProcessor;

#[async_trait]
impl<P> EventProcessor<P> for EventMessageProcessor
where
    P: Provider + Send + Sync,
{
    fn event_key(&self) -> String {
        "".to_string()
    }

    fn validate(&self, _event: &Event) -> bool {
        true
    }

    async fn process(
        &self,
        _world: &WorldContractReader<P>,
        db: &mut Sql,
        _block: &BlockWithTxs,
        _transaction_receipt: &TransactionReceipt,
        event_id: &str,
        event: &Event,
    ) -> Result<(), Error> {
        // silently ignore if the model is not found
        let model = match db.model(&format!("{:#x}", event.keys[MODEL_INDEX])).await {
            Ok(model) => model,
            Err(_) => return Ok(()),
        };

        // skip the first key, as its the event selector
        // and dont include last key as its the system key
        let mut keys_and_unpacked = [
            event.keys.clone().into_iter().skip(1).skip(event.keys.len() - 2).collect(),
            event.data.clone(),
        ]
        .concat();

        let mut entity = model.schema().await?;
        entity.deserialize(&mut keys_and_unpacked)?;

        db.set_event_message(entity, event_id).await?;
        Ok(())
    }
}
