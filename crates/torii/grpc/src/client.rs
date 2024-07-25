//! Client implementation for the gRPC service.
use std::num::ParseIntError;

use futures_util::stream::MapOk;
use futures_util::{Stream, StreamExt, TryStreamExt};
use starknet::core::types::{Felt, FromStrError, StateDiff, StateUpdate};

use crate::proto::world::{
    world_client, MetadataRequest, RetrieveEntitiesRequest, RetrieveEntitiesResponse,
    RetrieveEventsRequest, RetrieveEventsResponse, SubscribeEntitiesRequest,
    SubscribeEntityResponse, SubscribeEventsRequest, SubscribeEventsResponse,
    SubscribeModelsRequest, SubscribeModelsResponse, UpdateEntitiesSubscriptionRequest,
};
use crate::types::schema::{self, Entity, SchemaError};
use crate::types::{EntityKeysClause, Event, EventQuery, KeysClause, ModelKeysClause, Query};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Grpc(tonic::Status),
    #[error(transparent)]
    ParseStr(FromStrError),
    #[error(transparent)]
    ParseInt(ParseIntError),
    #[cfg(not(target_arch = "wasm32"))]
    #[error(transparent)]
    Transport(tonic::transport::Error),
    #[error(transparent)]
    Schema(#[from] schema::SchemaError),
}

#[derive(Debug)]
/// A lightweight wrapper around the grpc client.
pub struct WorldClient {
    _world_address: Felt,
    #[cfg(not(target_arch = "wasm32"))]
    inner: world_client::WorldClient<tonic::transport::Channel>,
    #[cfg(target_arch = "wasm32")]
    inner: world_client::WorldClient<tonic_web_wasm_client::Client>,
}

impl WorldClient {
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn new<D>(dst: D, _world_address: Felt) -> Result<Self, Error>
    where
        D: TryInto<tonic::transport::Endpoint>,
        D::Error: Into<Box<(dyn std::error::Error + Send + Sync + 'static)>>,
    {
        Ok(Self {
            _world_address,
            inner: world_client::WorldClient::connect(dst).await.map_err(Error::Transport)?,
        })
    }

    // we make this function async so that we can keep the function signature similar
    #[cfg(target_arch = "wasm32")]
    pub async fn new(endpoint: String, _world_address: Felt) -> Result<Self, Error> {
        Ok(Self {
            _world_address,
            inner: world_client::WorldClient::new(tonic_web_wasm_client::Client::new(endpoint)),
        })
    }

    /// Retrieve the metadata of the World.
    pub async fn metadata(&mut self) -> Result<dojo_types::WorldMetadata, Error> {
        self.inner
            .world_metadata(MetadataRequest {})
            .await
            .map_err(Error::Grpc)
            .and_then(|res| {
                res.into_inner().metadata.ok_or(Error::Schema(SchemaError::MissingExpectedData))
            })
            .and_then(|metadata| metadata.try_into().map_err(Error::ParseStr))
    }

    pub async fn retrieve_entities(
        &mut self,
        query: Query,
    ) -> Result<RetrieveEntitiesResponse, Error> {
        let request = RetrieveEntitiesRequest { query: Some(query.into()) };
        self.inner.retrieve_entities(request).await.map_err(Error::Grpc).map(|res| res.into_inner())
    }

    pub async fn retrieve_event_messages(
        &mut self,
        query: Query,
    ) -> Result<RetrieveEntitiesResponse, Error> {
        let request = RetrieveEntitiesRequest { query: Some(query.into()) };
        self.inner
            .retrieve_event_messages(request)
            .await
            .map_err(Error::Grpc)
            .map(|res| res.into_inner())
    }

    pub async fn retrieve_events(
        &mut self,
        query: EventQuery,
    ) -> Result<RetrieveEventsResponse, Error> {
        let request = RetrieveEventsRequest { query: Some(query.into()) };
        self.inner.retrieve_events(request).await.map_err(Error::Grpc).map(|res| res.into_inner())
    }

    /// Subscribe to entities updates of a World.
    pub async fn subscribe_entities(
        &mut self,
        clauses: Vec<EntityKeysClause>,
    ) -> Result<EntityUpdateStreaming, Error> {
        let clauses = clauses.into_iter().map(|c| c.into()).collect();
        let stream = self
            .inner
            .subscribe_entities(SubscribeEntitiesRequest { clauses })
            .await
            .map_err(Error::Grpc)
            .map(|res| res.into_inner())?;

        Ok(EntityUpdateStreaming(stream.map_ok(Box::new(|res| {
            res.entity.map_or(
                (res.subscription_id, Entity { hashed_keys: Felt::ZERO, models: vec![] }),
                |entity| (res.subscription_id, entity.try_into().expect("must able to serialize")),
            )
        }))))
    }

    /// Update an entities subscription.
    pub async fn update_entities_subscription(
        &mut self,
        subscription_id: u64,
        clauses: Vec<EntityKeysClause>,
    ) -> Result<(), Error> {
        let clauses = clauses.into_iter().map(|c| c.into()).collect();

        self.inner
            .update_entities_subscription(UpdateEntitiesSubscriptionRequest {
                subscription_id,
                clauses,
            })
            .await
            .map_err(Error::Grpc)
            .map(|res| res.into_inner())
    }

    /// Subscribe to event messages of a World.
    pub async fn subscribe_event_messages(
        &mut self,
        clauses: Vec<EntityKeysClause>,
    ) -> Result<EntityUpdateStreaming, Error> {
        let clauses = clauses.into_iter().map(|c| c.into()).collect();
        let stream = self
            .inner
            .subscribe_event_messages(SubscribeEntitiesRequest { clauses })
            .await
            .map_err(Error::Grpc)
            .map(|res| res.into_inner())?;

        Ok(EntityUpdateStreaming(stream.map_ok(Box::new(|res| {
            res.entity.map_or(
                (res.subscription_id, Entity { hashed_keys: Felt::ZERO, models: vec![] }),
                |entity| (res.subscription_id, entity.try_into().expect("must able to serialize")),
            )
        }))))
    }

    /// Update an event messages subscription.
    pub async fn update_event_messages_subscription(
        &mut self,
        subscription_id: u64,
        clauses: Vec<EntityKeysClause>,
    ) -> Result<(), Error> {
        let clauses = clauses.into_iter().map(|c| c.into()).collect();
        self.inner
            .update_event_messages_subscription(UpdateEntitiesSubscriptionRequest {
                subscription_id,
                clauses,
            })
            .await
            .map_err(Error::Grpc)
            .map(|res| res.into_inner())
    }

    /// Subscribe to the events of a World.
    pub async fn subscribe_events(
        &mut self,
        keys: Option<KeysClause>,
    ) -> Result<EventUpdateStreaming, Error> {
        let keys = keys.map(|c| c.into());

        let stream = self
            .inner
            .subscribe_events(SubscribeEventsRequest { keys })
            .await
            .map_err(Error::Grpc)
            .map(|res| res.into_inner())?;

        Ok(EventUpdateStreaming(stream.map_ok(Box::new(|res| match res.event {
            Some(event) => event.into(),
            None => Event { keys: vec![], data: vec![], transaction_hash: Felt::ZERO },
        }))))
    }

    /// Subscribe to the model diff for a set of models of a World.
    pub async fn subscribe_model_diffs(
        &mut self,
        models_keys: Vec<ModelKeysClause>,
    ) -> Result<ModelDiffsStreaming, Error> {
        let stream = self
            .inner
            .subscribe_models(SubscribeModelsRequest {
                models_keys: models_keys.into_iter().map(|e| e.into()).collect(),
            })
            .await
            .map_err(Error::Grpc)
            .map(|res| res.into_inner())?;

        Ok(ModelDiffsStreaming(stream.map_ok(Box::new(|res| match res.model_update {
            Some(update) => {
                TryInto::<StateUpdate>::try_into(update).expect("must able to serialize")
            }
            None => empty_state_update(),
        }))))
    }
}

type ModelDiffMappedStream = MapOk<
    tonic::Streaming<SubscribeModelsResponse>,
    Box<dyn Fn(SubscribeModelsResponse) -> StateUpdate + Send>,
>;

#[derive(Debug)]
pub struct ModelDiffsStreaming(ModelDiffMappedStream);

impl Stream for ModelDiffsStreaming {
    type Item = <ModelDiffMappedStream as Stream>::Item;
    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.0.poll_next_unpin(cx)
    }
}

type SubscriptionId = u64;
type EntityMappedStream = MapOk<
    tonic::Streaming<SubscribeEntityResponse>,
    Box<dyn Fn(SubscribeEntityResponse) -> (SubscriptionId, Entity) + Send>,
>;

#[derive(Debug)]
pub struct EntityUpdateStreaming(EntityMappedStream);

impl Stream for EntityUpdateStreaming {
    type Item = <EntityMappedStream as Stream>::Item;
    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.0.poll_next_unpin(cx)
    }
}

type EventMappedStream = MapOk<
    tonic::Streaming<SubscribeEventsResponse>,
    Box<dyn Fn(SubscribeEventsResponse) -> Event + Send>,
>;

#[derive(Debug)]
pub struct EventUpdateStreaming(EventMappedStream);

impl Stream for EventUpdateStreaming {
    type Item = <EventMappedStream as Stream>::Item;
    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.0.poll_next_unpin(cx)
    }
}

fn empty_state_update() -> StateUpdate {
    StateUpdate {
        block_hash: Felt::ZERO,
        new_root: Felt::ZERO,
        old_root: Felt::ZERO,
        state_diff: StateDiff {
            declared_classes: vec![],
            deployed_contracts: vec![],
            deprecated_declared_classes: vec![],
            nonces: vec![],
            replaced_classes: vec![],
            storage_diffs: vec![],
        },
    }
}