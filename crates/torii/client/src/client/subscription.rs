use std::cell::RefCell;
use std::collections::HashSet;
use std::future::Future;
use std::str::FromStr;
use std::sync::Arc;
use std::task::Poll;

use dojo_types::schema::EntityModel;
use dojo_types::WorldMetadata;
use futures::channel::mpsc::{self, Receiver, Sender};
use futures_util::StreamExt;
use parking_lot::{Mutex, RwLock};
use starknet::core::utils::cairo_short_string_to_felt;
use starknet_crypto::FieldElement;
use torii_grpc::protos;
use torii_grpc::protos::types::EntityDiff;
use torii_grpc::protos::world::SubscribeEntitiesResponse;

use super::error::{Error, ParseError};
use super::ModelStorage;
use crate::utils::compute_all_storage_addresses;

#[derive(Debug)]
pub enum SubscriptionEvent {
    UpdateSubsciptionStream(tonic::Streaming<SubscribeEntitiesResponse>),
}

pub struct SubscribedEntities {
    metadata: Arc<RwLock<WorldMetadata>>,
    pub(super) entities: RwLock<HashSet<EntityModel>>,
    /// All the relevant storage addresses derived from the subscribed entities
    pub(super) subscribed_storage_addresses: RwLock<HashSet<FieldElement>>,
}

impl SubscribedEntities {
    pub(crate) fn new(metadata: Arc<RwLock<WorldMetadata>>) -> Self {
        Self {
            metadata,
            entities: Default::default(),
            subscribed_storage_addresses: Default::default(),
        }
    }

    pub fn add_entities(&self, entities: Vec<EntityModel>) -> Result<(), Error> {
        for entity in entities {
            Self::add_entity(&self, entity)?;
        }
        Ok(())
    }

    pub fn remove_entities(&self, entities: Vec<EntityModel>) -> Result<(), Error> {
        for entity in entities {
            Self::remove_entity(&self, entity)?;
        }
        Ok(())
    }

    pub(crate) fn add_entity(&self, entity: EntityModel) -> Result<(), Error> {
        if !self.entities.write().insert(entity.clone()) {
            return Ok(());
        }

        let model_packed_size = self
            .metadata
            .read()
            .models
            .get(&entity.model)
            .map(|c| c.packed_size)
            .ok_or(Error::UnknownModel(entity.model.clone()))?;

        let storage_addresses = compute_all_storage_addresses(
            cairo_short_string_to_felt(&entity.model)
                .map_err(ParseError::CairoShortStringToFelt)?,
            &entity.keys,
            model_packed_size,
        );

        let storage_lock = &mut self.subscribed_storage_addresses.write();
        storage_addresses.into_iter().for_each(|address| {
            storage_lock.insert(address);
        });

        Ok(())
    }

    pub(crate) fn remove_entity(&self, entity: EntityModel) -> Result<(), Error> {
        if !self.entities.write().remove(&entity) {
            return Ok(());
        }

        let model_packed_size = self
            .metadata
            .read()
            .models
            .get(&entity.model)
            .map(|c| c.packed_size)
            .ok_or(Error::UnknownModel(entity.model.clone()))?;

        let storage_addresses = compute_all_storage_addresses(
            cairo_short_string_to_felt(&entity.model)
                .map_err(ParseError::CairoShortStringToFelt)?,
            &entity.keys,
            model_packed_size,
        );

        let storage_lock = &mut self.subscribed_storage_addresses.write();
        storage_addresses.iter().for_each(|address| {
            storage_lock.remove(address);
        });

        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct SubscriptionClientHandle(Mutex<Sender<SubscriptionEvent>>);

impl SubscriptionClientHandle {
    fn new(sender: Sender<SubscriptionEvent>) -> Self {
        Self(Mutex::new(sender))
    }

    pub(crate) fn update_subscription_stream(
        &self,
        stream: tonic::Streaming<SubscribeEntitiesResponse>,
    ) {
        let _ = self.0.lock().try_send(SubscriptionEvent::UpdateSubsciptionStream(stream));
    }
}

#[must_use = "SubscriptionClient does nothing unless polled"]
pub struct SubscriptionService {
    req_rcv: Receiver<SubscriptionEvent>,
    /// The stream returned by the subscription server to receive the response
    sub_res_stream: RefCell<Option<tonic::Streaming<SubscribeEntitiesResponse>>>,
    /// Callback to be called on error
    err_callback: Option<Box<dyn Fn(tonic::Status) + Send + Sync>>,

    // for processing the entity diff and updating the storage
    storage: Arc<ModelStorage>,
    world_metadata: Arc<RwLock<WorldMetadata>>,
    subscribed_entities: Arc<SubscribedEntities>,
}

impl SubscriptionService {
    pub(super) fn new(
        storage: Arc<ModelStorage>,
        world_metadata: Arc<RwLock<WorldMetadata>>,
        subscribed_entities: Arc<SubscribedEntities>,
        sub_res_stream: tonic::Streaming<SubscribeEntitiesResponse>,
    ) -> (Self, SubscriptionClientHandle) {
        let (req_sender, req_rcv) = mpsc::channel(128);

        let handle = SubscriptionClientHandle::new(req_sender);
        let sub_res_stream = RefCell::new(Some(sub_res_stream));

        let client = Self {
            req_rcv,
            storage,
            world_metadata,
            sub_res_stream,
            err_callback: None,
            subscribed_entities,
        };

        (client, handle)
    }

    // TODO: handle the subscription events properly
    fn handle_event(&self, event: SubscriptionEvent) -> Result<(), Error> {
        match event {
            SubscriptionEvent::UpdateSubsciptionStream(stream) => {
                self.sub_res_stream.replace(Some(stream));
            }
        }
        Ok(())
    }

    // handle the response from the subscription stream
    fn handle_response(&self, response: Result<SubscribeEntitiesResponse, tonic::Status>) {
        match response {
            Ok(res) => {
                let entity_diff = res
                    .entity_update
                    .and_then(|e| e.update)
                    .and_then(|update| match update {
                        protos::types::maybe_pending_entity_update::Update::EntityUpdate(
                            update,
                        ) => update.entity_diff,
                        protos::types::maybe_pending_entity_update::Update::PendingEntityUpdate(
                            update,
                        ) => update.entity_diff,
                    })
                    .expect("have entity update");

                self.process_entity_diff(entity_diff);
            }

            Err(err) => {
                if let Some(ref callback) = self.err_callback {
                    callback(err)
                }
            }
        }
    }

    fn process_entity_diff(&self, diff: EntityDiff) {
        let storage_entries = diff.storage_diffs.into_iter().find_map(|d| {
            let expected = self.world_metadata.read().world_address;
            let current = FieldElement::from_str(&d.address).expect("valid FieldElement value");
            if current == expected { Some(d.storage_entries) } else { None }
        });

        let Some(entries) = storage_entries else {
            return;
        };

        entries.into_iter().enumerate().for_each(|(i, entry)| {
            let key = FieldElement::from_str(&entry.key).expect("valid FieldElement value");
            let value = FieldElement::from_str(&entry.value).expect("valid FieldElement value");

            println!("[{i}] key: {key:#x} value: {value:#x}", key = key, value = value);

            if self.subscribed_entities.subscribed_storage_addresses.read().contains(&key) {
                self.storage.storage.write().insert(key, value);
            } else {
                panic!("unknown storage address");
            }
        })
    }
}

impl Future for SubscriptionService {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let pin = self.get_mut();

        loop {
            while let Poll::Ready(Some(req)) = pin.req_rcv.poll_next_unpin(cx) {
                let _ = pin.handle_event(req);
            }

            if let Some(stream) = pin.sub_res_stream.get_mut() {
                match stream.poll_next_unpin(cx) {
                    Poll::Ready(Some(res)) => pin.handle_response(res),
                    Poll::Ready(None) => return Poll::Ready(()),
                    Poll::Pending => return Poll::Pending,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use dojo_types::schema::{EntityModel, Ty};
    use dojo_types::WorldMetadata;
    use parking_lot::RwLock;
    use starknet::core::utils::cairo_short_string_to_felt;
    use starknet::macros::felt;

    use crate::utils::compute_all_storage_addresses;

    fn create_dummy_metadata() -> WorldMetadata {
        let components = HashMap::from([(
            "Position".into(),
            dojo_types::schema::ModelMetadata {
                name: "Position".into(),
                class_hash: felt!("1"),
                packed_size: 1,
                unpacked_size: 2,
                layout: vec![],
                schema: Ty::Primitive(dojo_types::primitive::Primitive::Bool(None)),
            },
        )]);

        WorldMetadata { models: components, ..Default::default() }
    }

    #[test]
    fn add_and_remove_subscribed_entity() {
        let model_name = String::from("Position");
        let keys = vec![felt!("0x12345")];
        let packed_size: u32 = 1;

        let mut expected_storage_addresses = compute_all_storage_addresses(
            cairo_short_string_to_felt(&model_name).unwrap(),
            &keys,
            packed_size,
        )
        .into_iter();

        let metadata = self::create_dummy_metadata();
        let entity = EntityModel { model: model_name, keys };

        let subscribed_entities = super::SubscribedEntities::new(Arc::new(RwLock::new(metadata)));
        subscribed_entities.add_entities(vec![entity.clone()]).expect("able to add entity");

        let actual_storage_addresses_count =
            subscribed_entities.subscribed_storage_addresses.read().len();
        let actual_storage_addresses =
            subscribed_entities.subscribed_storage_addresses.read().clone();

        assert!(subscribed_entities.entities.read().contains(&entity));
        assert_eq!(actual_storage_addresses_count, expected_storage_addresses.len());
        assert!(expected_storage_addresses.all(|addr| actual_storage_addresses.contains(&addr)));

        subscribed_entities.remove_entities(vec![entity.clone()]).expect("able to remove entities");

        let actual_storage_addresses_count_after =
            subscribed_entities.subscribed_storage_addresses.read().len();

        assert_eq!(actual_storage_addresses_count_after, 0);
        assert!(!subscribed_entities.entities.read().contains(&entity));
    }
}
