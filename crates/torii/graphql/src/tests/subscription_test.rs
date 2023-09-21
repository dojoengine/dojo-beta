#[cfg(test)]
mod tests {
    use std::time::Duration;
    use std::sync::Arc;

    use async_graphql::value;
    use lazy_static::lazy_static;
    use dojo_world::manifest::{Component, Member};
    use serial_test::serial;
    use sqlx::SqlitePool;
    use starknet_crypto::{poseidon_hash_many, FieldElement};
    use tokio::sync::{mpsc, Semaphore};
    use torii_core::sql::Sql;
    use torii_core::State;

    use crate::tests::common::{init, run_graphql_subscription};

    lazy_static! {
        static ref SEMAPHORE: Arc<Semaphore> = Arc::new(Semaphore::new(1));
    }

    #[sqlx::test(migrations = "../migrations")]
    #[serial]
    async fn test_entity_subscription(pool: SqlitePool) {
        let permit = SEMAPHORE.clone().acquire_owned().await.unwrap();
        let state = init(&pool).await;
        // 0. Preprocess expected entity value
        let key = vec![FieldElement::ONE];
        let entity_id = format!("{:#x}", poseidon_hash_many(&key));
        let keys_str = key.iter().map(|k| format!("{:#x}", k)).collect::<Vec<String>>().join(",");
        let expected_value: async_graphql::Value = value!({
                            "entityUpdated": { "id": entity_id.clone(), "keys":vec![keys_str.clone()], "componentNames": "Moves" }
        });
        let (tx, mut rx) = mpsc::channel(10);

        tokio::spawn(async move {
            // 1. Open process and sleep.Go to execute subscription
            tokio::time::sleep(Duration::from_secs(1)).await;

            // Set entity with one moves component
            // remaining: 10, last_direction: 0
            let moves_values = vec![FieldElement::from_hex_be("0xa").unwrap(), FieldElement::ZERO];
            state.set_entity("Moves".to_string(), key, moves_values).await.unwrap();
            // 3. fn publish() is called from state.set_entity()

            tx.send(()).await.unwrap();
            drop(permit);
        });

        // 2. The subscription is executed and it is listening, waiting for publish() to be executed
        let response_value = run_graphql_subscription(
            &pool,
            r#"
          subscription {
              entityUpdated {
                  id, keys, componentNames
              }
          }"#,
        )
        .await;
        // 4. The subcription has received the message from publish()
        // 5. Compare values
        assert_eq!(expected_value, response_value);
        rx.recv().await.unwrap();
    }

    #[sqlx::test(migrations = "../migrations")]
    #[serial]
    async fn test_entity_subscription_with_id(pool: SqlitePool) {
        let permit = SEMAPHORE.clone().acquire_owned().await.unwrap();
        let state = init(&pool).await;
        // 0. Preprocess expected entity value
        let key = vec![FieldElement::ONE];
        let entity_id = format!("{:#x}", poseidon_hash_many(&key));
        let keys_str = key.iter().map(|k| format!("{:#x}", k)).collect::<Vec<String>>().join(",");
        let expected_value: async_graphql::Value = value!({
                                                "entityUpdated": { "id": entity_id.clone(), "keys":vec![keys_str.clone()], "componentNames": "Moves" }
        });
        let (tx, mut rx) = mpsc::channel(1);

        tokio::spawn(async move {
            // 1. Open process and sleep.Go to execute subscription
            tokio::time::sleep(Duration::from_secs(1)).await;

            // Set entity with one moves component
            // remaining: 10, last_direction: 0
            let moves_values = vec![FieldElement::from_hex_be("0xa").unwrap(), FieldElement::ZERO];
            state.set_entity("Moves".to_string(), key, moves_values).await.unwrap();
            // 3. fn publish() is called from state.set_entity()

            tx.send(()).await.unwrap();
            drop(permit);
        });

        // 2. The subscription is executed and it is listeing, waiting for publish() to be executed
        let response_value = run_graphql_subscription(
            &pool,
            r#"
				subscription {
						entityUpdated(id: "0x579e8877c7755365d5ec1ec7d3a94a457eff5d1f40482bbe9729c064cdead2") {
								id, keys, componentNames
						}
				}"#,
        )
        .await;
        // 4. The subcription has received the message from publish()
        // 5. Compare values
        assert_eq!(expected_value, response_value);
        rx.recv().await.unwrap();
    }


    #[sqlx::test(migrations = "../migrations")]
    #[serial]
    async fn test_component_subscription_with_id(pool: SqlitePool) {
        let permit = SEMAPHORE.clone().acquire_owned().await.unwrap();
        let state = Sql::new(pool.clone(), FieldElement::ZERO).await.unwrap();
        // 0. Preprocess component value
        let name = "Test".to_string();
        let component_id = name.to_lowercase();
        let class_hash = FieldElement::TWO;
        let hex_class_hash = format!("{:#x}", class_hash);
        let expected_value: async_graphql::Value = value!({
         "componentRegistered": { "id": component_id.clone(), "name":name, "classHash": hex_class_hash }
        });
        let (tx, mut rx) = mpsc::channel(17);

        tokio::spawn(async move {
            // 1. Open process and sleep.Go to execute subscription
            tokio::time::sleep(Duration::from_secs(1)).await;

            let component = Component {
                name,
                members: vec![Member { name: "test".into(), ty: "u32".into(), key: false }],
                class_hash,
                ..Default::default()
            };
            state.register_component(component).await.unwrap();
            // 3. fn publish() is called from state.set_entity()

            tx.send(()).await.unwrap();
            drop(permit);
        });

        // 2. The subscription is executed and it is listeing, waiting for publish() to be executed
        let response_value = run_graphql_subscription(
            &pool,
            r#"
            subscription {
                componentRegistered(id: "test") {
                        id, name, classHash
                    }
            }"#,
        )
            .await;
        // 4. The subcription has received the message from publish()
        // 5. Compare values
        assert_eq!(expected_value, response_value);
        rx.recv().await.unwrap();
    }

    #[sqlx::test(migrations = "../migrations")]
    #[serial]
    async fn test_component_subscription(pool: SqlitePool) {
        let permit = SEMAPHORE.clone().acquire_owned().await.unwrap();
        let state = Sql::new(pool.clone(), FieldElement::ZERO).await.unwrap();
        // 0. Preprocess component value
        let name = "Test".to_string();
        let component_id = name.to_lowercase();
        let class_hash = FieldElement::TWO;
        let hex_class_hash = format!("{:#x}", class_hash);
        let expected_value: async_graphql::Value = value!({
         "componentRegistered": { "id": component_id.clone(), "name":name, "classHash": hex_class_hash }
        });
        let (tx, mut rx) = mpsc::channel(7);

        tokio::spawn(async move {
            // 1. Open process and sleep.Go to execute subscription
            tokio::time::sleep(Duration::from_secs(1)).await;

            let component = Component {
                name,
                members: vec![Member { name: "test".into(), ty: "u32".into(), key: false }],
                class_hash,
                ..Default::default()
            };
            state.register_component(component).await.unwrap();
            // 3. fn publish() is called from state.set_entity()

            tx.send(()).await.unwrap();
            drop(permit);
        });

        // 2. The subscription is executed and it is listeing, waiting for publish() to be executed
        let response_value = run_graphql_subscription(
            &pool,
            r#"
            subscription {
                componentRegistered {
                        id, name, classHash
                    }
            }"#,
        )
        .await;
        // 4. The subcription has received the message from publish()
        // 5. Compare values
        assert_eq!(expected_value, response_value);
        rx.recv().await.unwrap();
    }
}
