use async_graphql::dynamic::{Field, FieldFuture, FieldValue, InputValue, TypeRef};
use async_graphql::{Name, Value};
use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use serde::Deserialize;
use sqlx::pool::PoolConnection;
use sqlx::{FromRow, Pool, Result, Sqlite};

use super::types::ScalarType;
use super::utils::remove_quotes;
use super::{ObjectTraitInstance, ObjectTraitStatic, TypeMapping, ValueMapping};

#[derive(FromRow, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Component {
    pub id: String,
    pub name: String,
    pub address: String,
    pub class_hash: String,
    pub transaction_hash: String,
    pub storage_definition: String,
    pub created_at: DateTime<Utc>,
}

pub struct ComponentObject {
    pub field_type_mapping: TypeMapping,
}

impl ObjectTraitStatic for ComponentObject {
    fn new() -> Self {
        Self {
            field_type_mapping: IndexMap::from([
                (Name::new("id"), TypeRef::ID.to_string()),
                (Name::new("name"), TypeRef::STRING.to_string()),
                (Name::new("address"), ScalarType::ADDRESS.to_string()),
                (Name::new("classHash"), ScalarType::FELT.to_string()),
                (Name::new("transactionHash"), ScalarType::FELT.to_string()),
                (Name::new("storageDefinition"), TypeRef::STRING.to_string()),
                (Name::new("createdAt"), ScalarType::DATE_TIME.to_string()),
            ]),
        }
    }

    fn from(field_type_mapping: TypeMapping) -> Self {
        Self { field_type_mapping }
    }
}

impl ObjectTraitInstance for ComponentObject {
    fn name(&self) -> &str {
        "component"
    }

    fn type_name(&self) -> &str {
        "Component"
    }

    fn field_type_mapping(&self) -> &TypeMapping {
        &self.field_type_mapping
    }

    fn field_resolvers(&self) -> Vec<Field> {
        vec![
            Field::new(self.name(), TypeRef::named_nn(self.type_name()), |ctx| {
                FieldFuture::new(async move {
                    let mut conn = ctx.data::<Pool<Sqlite>>()?.acquire().await?;
                    let id = remove_quotes(ctx.args.try_get("id")?.string()?);
                    let component_values = component_by_id(&mut conn, &id).await?;
                    Ok(Some(FieldValue::owned_any(component_values)))
                })
            })
            .argument(InputValue::new("id", TypeRef::named_nn(TypeRef::ID))),
        ]
    }
}

async fn component_by_id(conn: &mut PoolConnection<Sqlite>, id: &str) -> Result<ValueMapping> {
    let component = sqlx::query_as!(
        Component,
        r#"
            SELECT 
                id,
                name,
                address,
                class_hash,
                transaction_hash,
                storage_definition,
                created_at as "created_at: _"
            FROM components WHERE id = $1
        "#,
        id
    )
    .fetch_one(conn)
    .await?;

    Ok(value_mapping(component))
}

#[allow(dead_code)]
pub async fn components(conn: &mut PoolConnection<Sqlite>) -> Result<Vec<ValueMapping>> {
    let components = sqlx::query_as!(
        Component,
        r#"
            SELECT 
                id,
                name,
                address,
                class_hash,
                transaction_hash,
                storage_definition,
                created_at as "created_at: _"
            FROM components
        "#
    )
    .fetch_all(conn)
    .await?;

    Ok(components.into_iter().map(value_mapping).collect())
}

fn value_mapping(component: Component) -> ValueMapping {
    IndexMap::from([
        (Name::new("id"), Value::from(component.id)),
        (Name::new("name"), Value::from(component.name)),
        (Name::new("address"), Value::from(component.address)),
        (Name::new("classHash"), Value::from(component.class_hash)),
        (Name::new("transactionHash"), Value::from(component.transaction_hash)),
        (Name::new("storageDefinition"), Value::from(component.storage_definition)),
        (
            Name::new("createdAt"),
            Value::from(component.created_at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)),
        ),
    ])
}
