use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;
use std::task::{Context, Poll};

use dojo_types::packing::ParseError;
use futures::Stream;
use futures_util::StreamExt;
use rand::Rng;
use sqlx::{Pool, Sqlite};
use starknet_crypto::FieldElement;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::RwLock;
use torii_core::cache::ModelCache;
use torii_core::error::Error;
use torii_core::model::{build_sql_query, map_row_to_ty};
use torii_core::simple_broker::SimpleBroker;
use torii_core::types::Entity;
use tracing::trace;

use crate::proto;

pub struct EntitiesSubscriber {
    /// Entity ids that the subscriber is interested in
    ids: HashSet<FieldElement>,
    /// The channel to send the response back to the subscriber.
    sender: Sender<Result<proto::world::SubscribeEntityResponse, tonic::Status>>,
}

#[derive(Default)]
pub struct EntitySubscriberManager {
    subscribers: RwLock<HashMap<usize, EntitiesSubscriber>>,
}

impl EntitySubscriberManager {
    pub async fn add_subscriber(
        &self,
        ids: Vec<FieldElement>,
    ) -> Result<Receiver<Result<proto::world::SubscribeEntityResponse, tonic::Status>>, Error> {
        let id = rand::thread_rng().gen::<usize>();
        let (sender, receiver) = channel(1);

        self.subscribers
            .write()
            .await
            .insert(id, EntitiesSubscriber { ids: ids.iter().cloned().collect(), sender });

        Ok(receiver)
    }

    pub(super) async fn remove_subscriber(&self, id: usize) {
        self.subscribers.write().await.remove(&id);
    }
}

#[must_use = "Service does nothing unless polled"]
pub struct Service {
    pool: Pool<Sqlite>,
    subs_manager: Arc<EntitySubscriberManager>,
    model_cache: Arc<ModelCache>,
    simple_broker: Pin<Box<dyn Stream<Item = Entity> + Send>>,
}

impl Service {
    pub fn new(
        pool: Pool<Sqlite>,
        subs_manager: Arc<EntitySubscriberManager>,
        model_cache: Arc<ModelCache>,
    ) -> Self {
        Self {
            pool,
            subs_manager,
            model_cache,
            simple_broker: Box::pin(SimpleBroker::<Entity>::subscribe()),
        }
    }

    async fn publish_updates(
        subs: Arc<EntitySubscriberManager>,
        cache: Arc<ModelCache>,
        pool: Pool<Sqlite>,
        id: &str,
    ) -> Result<(), Error> {
        let mut closed_stream = Vec::new();

        for (idx, sub) in subs.subscribers.read().await.iter() {
            let query = r#"
                SELECT group_concat(entity_model.model_id) as model_names
                FROM entities
                JOIN entity_model ON entities.id = entity_model.entity_id
                WHERE entities.id = ?
                GROUP BY entities.id
            "#;
            let result: (String,) = sqlx::query_as(query).bind(id).fetch_one(&pool).await?;
            let model_names: Vec<&str> = result.0.split(',').collect();
            let schemas = cache.schemas(model_names).await?;

            let entity_query = format!("{} WHERE entities.id = ?", build_sql_query(&schemas)?);
            let row = sqlx::query(&entity_query).bind(&id).fetch_one(&pool).await?;

            let models = schemas
                .iter()
                .map(|s| {
                    let mut struct_ty = s.as_struct().expect("schema should be struct").to_owned();
                    map_row_to_ty(&s.name(), &mut struct_ty, &row)?;

                    Ok(struct_ty.try_into().unwrap())
                })
                .collect::<Result<Vec<_>, Error>>()?;

            let resp = proto::world::SubscribeEntityResponse {
                entity: Some(proto::types::Entity {
                    id: FieldElement::from_str(&id).unwrap().to_bytes_be().to_vec(),
                    models,
                }),
            };

            if sub.sender.send(Ok(resp)).await.is_err() {
                closed_stream.push(*idx);
            }
        }

        for id in closed_stream {
            trace!(target = "subscription", "closing stream idx: {id}");
            subs.remove_subscriber(id).await
        }

        Ok(())
    }
}

impl Future for Service {
    type Output = ();

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> std::task::Poll<Self::Output> {
        let pin = self.get_mut();

        while let Poll::Ready(Some(entity)) = pin.simple_broker.poll_next_unpin(cx) {
            let subs = Arc::clone(&pin.subs_manager);
            let cache = Arc::clone(&pin.model_cache);
            let pool = pin.pool.clone();
            tokio::spawn(
                async move { Service::publish_updates(subs, cache, pool, &entity.id).await },
            );
        }

        Poll::Pending
    }
}
