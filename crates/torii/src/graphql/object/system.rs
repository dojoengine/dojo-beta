use async_graphql::dynamic::{Field, FieldFuture, FieldValue, InputValue, TypeRef};
use async_graphql::{Name, Value};
use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use serde::Deserialize;
use sqlx::{FromRow, Pool, Sqlite};

use super::query::{query_all, query_by_id, ID};
use super::system_call::system_calls_by_system_id;
use super::{ObjectTrait, TypeMapping, ValueMapping};
use crate::graphql::constants::DEFAULT_LIMIT;
use crate::graphql::types::ScalarType;
use crate::graphql::utils::extract_value::extract;

#[derive(FromRow, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct System {
    pub id: String,
    pub name: String,
    pub address: String,
    pub class_hash: String,
    pub transaction_hash: String,
    pub created_at: DateTime<Utc>,
}

pub struct SystemObject {
    pub type_mapping: TypeMapping,
}

impl SystemObject {
    pub fn new() -> Self {
        Self {
            type_mapping: IndexMap::from([
                (Name::new("id"), TypeRef::named(TypeRef::ID)),
                (Name::new("name"), TypeRef::named(TypeRef::STRING)),
                (Name::new("address"), TypeRef::named(ScalarType::Address.to_string())),
                (Name::new("classHash"), TypeRef::named(ScalarType::Felt252.to_string())),
                (Name::new("transactionHash"), TypeRef::named(ScalarType::Felt252.to_string())),
                (Name::new("createdAt"), TypeRef::named(ScalarType::DateTime.to_string())),
            ]),
        }
    }

    pub fn value_mapping(system: System) -> ValueMapping {
        IndexMap::from([
            (Name::new("id"), Value::from(system.id)),
            (Name::new("name"), Value::from(system.name)),
            (Name::new("address"), Value::from(system.address)),
            (Name::new("classHash"), Value::from(system.class_hash)),
            (Name::new("transactionHash"), Value::from(system.transaction_hash)),
            (
                Name::new("createdAt"),
                Value::from(system.created_at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)),
            ),
        ])
    }
}

impl ObjectTrait for SystemObject {
    fn name(&self) -> &str {
        "system"
    }

    fn type_name(&self) -> &str {
        "System"
    }

    fn type_mapping(&self) -> &TypeMapping {
        &self.type_mapping
    }

    fn resolve_one(&self) -> Option<Field> {
        Some(
            Field::new(self.name(), TypeRef::named_nn(self.type_name()), |ctx| {
                FieldFuture::new(async move {
                    let mut conn = ctx.data::<Pool<Sqlite>>()?.acquire().await?;
                    let id = ctx.args.try_get("id")?.string()?.to_string();
                    let system = query_by_id(&mut conn, "systems", ID::Str(id)).await?;
                    let result = SystemObject::value_mapping(system);
                    Ok(Some(FieldValue::owned_any(result)))
                })
            })
            .argument(InputValue::new("id", TypeRef::named_nn(TypeRef::ID))),
        )
    }

    fn resolve_many(&self) -> Option<Field> {
        Some(
            Field::new("systems", TypeRef::named_list(self.type_name()), |ctx| {
                FieldFuture::new(async move {
                    let mut conn = ctx.data::<Pool<Sqlite>>()?.acquire().await?;
                    let limit = ctx
                        .args
                        .try_get("limit")
                        .and_then(|limit| limit.u64())
                        .unwrap_or(DEFAULT_LIMIT);

                    let systems: Vec<System> = query_all(&mut conn, "systems", limit).await?;
                    let result: Vec<FieldValue<'_>> = systems
                        .into_iter()
                        .map(SystemObject::value_mapping)
                        .map(FieldValue::owned_any)
                        .collect();

                    Ok(Some(FieldValue::list(result)))
                })
            })
            .argument(InputValue::new("limit", TypeRef::named(TypeRef::INT))),
        )
    }

    fn nested_fields(&self) -> Option<Vec<Field>> {
        Some(vec![Field::new("systemCalls", TypeRef::named_nn_list_nn("SystemCall"), |ctx| {
            FieldFuture::new(async move {
                let mut conn = ctx.data::<Pool<Sqlite>>()?.acquire().await?;
                let system_values = ctx.parent_value.try_downcast_ref::<ValueMapping>()?;

                let id = extract::<String>(system_values, "id")?;
                let system_calls = system_calls_by_system_id(&mut conn, &id).await?;

                Ok(Some(FieldValue::list(system_calls.into_iter().map(FieldValue::owned_any))))
            })
        })])
    }
}
