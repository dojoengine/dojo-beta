pub mod error;
pub mod logger;
pub mod subscription;

use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;

use futures::Stream;
use proto::world::{
    MetadataRequest, MetadataResponse, RetrieveEntitiesRequest, RetrieveEntitiesResponse,
    SubscribeEntitiesRequest, SubscribeEntitiesResponse,
};
use sqlx::{Pool, Sqlite};
use starknet::core::utils::cairo_short_string_to_felt;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;
use starknet_crypto::FieldElement;
use tokio::net::TcpListener;
use tokio::sync::mpsc::Receiver;
use tokio_stream::wrappers::{ReceiverStream, TcpListenerStream};
use tonic::transport::Server;
use tonic::{Request, Response, Status};
use torii_core::cache::ModelCache;
use torii_core::error::{Error, ParseError, QueryError};
use torii_core::model::map_rows_to_tys;

use self::subscription::SubscribeRequest;
use crate::proto::types::clause::ClauseType;
use crate::proto::world::world_server::WorldServer;
use crate::proto::{self};

#[derive(Clone)]
pub struct DojoWorld {
    world_address: FieldElement,
    pool: Pool<Sqlite>,
    subscriber_manager: Arc<subscription::SubscriberManager>,
    model_cache: Arc<ModelCache>,
}

impl DojoWorld {
    pub fn new(
        pool: Pool<Sqlite>,
        block_rx: Receiver<u64>,
        world_address: FieldElement,
        provider: Arc<JsonRpcClient<HttpTransport>>,
    ) -> Self {
        let subscriber_manager = Arc::new(subscription::SubscriberManager::default());

        tokio::task::spawn(subscription::Service::new_with_block_rcv(
            block_rx,
            world_address,
            provider,
            Arc::clone(&subscriber_manager),
        ));

        let model_cache = Arc::new(ModelCache::new(pool.clone()));

        Self { pool, model_cache, world_address, subscriber_manager }
    }
}

impl DojoWorld {
    pub async fn metadata(&self) -> Result<proto::types::WorldMetadata, Error> {
        let (world_address, world_class_hash, executor_address, executor_class_hash): (
            String,
            String,
            String,
            String,
        ) = sqlx::query_as(&format!(
            "SELECT world_address, world_class_hash, executor_address, executor_class_hash FROM \
             worlds WHERE id = '{:#x}'",
            self.world_address
        ))
        .fetch_one(&self.pool)
        .await?;

        let models: Vec<(String, String, u32, u32, String)> = sqlx::query_as(
            "SELECT name, class_hash, packed_size, unpacked_size, layout FROM models",
        )
        .fetch_all(&self.pool)
        .await?;

        let mut models_metadata = Vec::with_capacity(models.len());
        for model in models {
            let schema_data = self.model_cache.schema(&model.0).await?;
            models_metadata.push(proto::types::ModelMetadata {
                name: model.0,
                class_hash: model.1,
                packed_size: model.2,
                unpacked_size: model.3,
                layout: hex::decode(&model.4).unwrap(),
                schema: serde_json::to_vec(&schema_data.ty).unwrap(),
            });
        }

        Ok(proto::types::WorldMetadata {
            world_address,
            world_class_hash,
            executor_address,
            executor_class_hash,
            models: models_metadata,
        })
    }

    async fn entities_by_keys(
        &self,
        _keys: proto::types::KeysClause,
    ) -> Result<Vec<proto::types::Entity>, Error> {
        Ok(vec![])
    }

    async fn entities_by_attribute(
        &self,
        attribute: proto::types::AttributeClause,
    ) -> Result<Vec<proto::types::Entity>, Error> {
        let schema_data = self.model_cache.schema(&attribute.model).await?;
        let results = sqlx::query(&schema_data.sql).fetch_all(&self.pool).await?;
        let tys = map_rows_to_tys(schema_data.ty.as_struct().unwrap(), &results)?;

        let mut entities = Vec::with_capacity(tys.len());

        for ty in tys {
            entities.push(proto::types::Entity {
                key: "".to_string(),
                models: vec![
                    proto::types::Model {
                        name: ty.name(),
                        data: serde_json::to_vec(&ty).unwrap()
                    }
                ]
            })
        }
    
        Ok(vec![])
    }

    async fn entities_by_composite(
        &self,
        _composite: proto::types::CompositeClause,
    ) -> Result<Vec<proto::types::Entity>, Error> {
        Ok(vec![])
    }

    pub async fn model_metadata(&self, model: &str) -> Result<proto::types::ModelMetadata, Error> {
        let (name, class_hash, packed_size, unpacked_size, layout): (
            String,
            String,
            u32,
            u32,
            String,
        ) = sqlx::query_as(
            "SELECT name, class_hash, packed_size, unpacked_size, layout FROM models WHERE id = ?",
        )
        .bind(model)
        .fetch_one(&self.pool)
        .await?;

        let schema = self.model_cache.schema(model).await?;
        let layout = hex::decode(&layout).unwrap();

        Ok(proto::types::ModelMetadata {
            name,
            layout,
            class_hash,
            packed_size,
            unpacked_size,
            schema: serde_json::to_vec(&schema.ty).unwrap(),
        })
    }

    async fn subscribe_entities(
        &self,
        entities_keys: Vec<proto::types::KeysClause>,
    ) -> Result<Receiver<Result<proto::world::SubscribeEntitiesResponse, tonic::Status>>, Error>
    {
        let mut subs = Vec::with_capacity(entities_keys.len());
        for keys in entities_keys {
            let model = cairo_short_string_to_felt(&keys.model)
                .map_err(ParseError::CairoShortStringToFelt)?;

            let proto::types::ModelMetadata { packed_size, .. } =
                self.model_metadata(&keys.model).await?;

            subs.push(SubscribeRequest {
                keys,
                model: subscription::ModelMetadata {
                    name: model,
                    packed_size: packed_size as usize,
                },
            });
        }

        self.subscriber_manager.add_subscriber(subs).await
    }

    async fn retrieve_entities(
        &self,
        query: proto::types::EntityQuery,
    ) -> Result<proto::world::RetrieveEntitiesResponse, Error> {
        let clause_type = query
            .clause
            .ok_or(QueryError::UnsupportedQuery)?
            .clause_type
            .ok_or(QueryError::UnsupportedQuery)?;

        let entities = match clause_type {
            ClauseType::Keys(keys) => self.entities_by_keys(keys).await?,
            ClauseType::Attribute(attribute) => self.entities_by_attribute(attribute).await?,
            ClauseType::Composite(composite) => self.entities_by_composite(composite).await?,
        };

        Ok(RetrieveEntitiesResponse { entities })
    }
}

type ServiceResult<T> = Result<Response<T>, Status>;
type SubscribeEntitiesResponseStream =
    Pin<Box<dyn Stream<Item = Result<SubscribeEntitiesResponse, Status>> + Send>>;

#[tonic::async_trait]
impl proto::world::world_server::World for DojoWorld {
    async fn world_metadata(
        &self,
        _request: Request<MetadataRequest>,
    ) -> Result<Response<MetadataResponse>, Status> {
        let metadata = self.metadata().await.map_err(|e| match e {
            Error::Sql(sqlx::Error::RowNotFound) => Status::not_found("World not found"),
            e => Status::internal(e.to_string()),
        })?;

        Ok(Response::new(MetadataResponse { metadata: Some(metadata) }))
    }

    type SubscribeEntitiesStream = SubscribeEntitiesResponseStream;

    async fn subscribe_entities(
        &self,
        request: Request<SubscribeEntitiesRequest>,
    ) -> ServiceResult<Self::SubscribeEntitiesStream> {
        let SubscribeEntitiesRequest { entities_keys } = request.into_inner();
        let rx = self
            .subscribe_entities(entities_keys)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(Box::pin(ReceiverStream::new(rx)) as Self::SubscribeEntitiesStream))
    }

    async fn retrieve_entities(
        &self,
        request: Request<RetrieveEntitiesRequest>,
    ) -> Result<Response<RetrieveEntitiesResponse>, Status> {
        let query = request
            .into_inner()
            .query
            .ok_or_else(|| Status::invalid_argument("Missing query argument"))?;

        let entities =
            self.retrieve_entities(query).await.map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(entities))
    }
}

pub async fn new(
    mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
    pool: &Pool<Sqlite>,
    block_rx: Receiver<u64>,
    world_address: FieldElement,
    provider: Arc<JsonRpcClient<HttpTransport>>,
) -> Result<
    (SocketAddr, impl Future<Output = Result<(), tonic::transport::Error>> + 'static),
    std::io::Error,
> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;

    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::world::FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();

    let world = DojoWorld::new(pool.clone(), block_rx, world_address, provider);
    let server = WorldServer::new(world);

    let server_future = Server::builder()
        // GrpcWeb is over http1 so we must enable it.
        .accept_http1(true)
        .add_service(reflection)
        .add_service(tonic_web::enable(server))
        .serve_with_incoming_shutdown(TcpListenerStream::new(listener), async move {
            shutdown_rx.recv().await.map_or((), |_| ())
        });

    Ok((addr, server_future))
}
