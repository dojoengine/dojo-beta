use async_graphql::dynamic::{Field, FieldFuture, InputValue, TypeRef};
use async_graphql::Value;
use sqlx::{Pool, Sqlite};

use super::connection::{connection_arguments, connection_output, parse_connection_arguments};
use super::{ObjectTrait, TypeMapping, ValueMapping};
use crate::mapping::{SYSTEM_CALL_TYPE_MAPPING, SYSTEM_TYPE_MAPPING};
use crate::query::constants::SYSTEM_CALL_TABLE;
use crate::query::data::{count_rows, fetch_multiple_rows, fetch_single_row};
use crate::query::value_mapping_from_row;
use crate::utils::extract_value::extract;

pub struct SystemCallObject;

impl ObjectTrait for SystemCallObject {
    fn name(&self) -> (&str, &str) {
        ("systemCall", "systemCalls")
    }

    fn type_name(&self) -> &str {
        "SystemCall"
    }

    fn type_mapping(&self) -> &TypeMapping {
        &SYSTEM_CALL_TYPE_MAPPING
    }

    fn resolve_one(&self) -> Option<Field> {
        Some(
            Field::new(self.name().0, TypeRef::named_nn(self.type_name()), |ctx| {
                FieldFuture::new(async move {
                    let mut conn = ctx.data::<Pool<Sqlite>>()?.acquire().await?;
                    let id = ctx.args.try_get("id")?.i64()?;
                    let data =
                        fetch_single_row(&mut conn, SYSTEM_CALL_TABLE, "id", &id.to_string())
                            .await?;
                    let system_call =
                        value_mapping_from_row(&data, &SYSTEM_CALL_TYPE_MAPPING, false)?;

                    Ok(Some(Value::Object(system_call)))
                })
            })
            .argument(InputValue::new("id", TypeRef::named_nn(TypeRef::INT))),
        )
    }

    fn resolve_many(&self) -> Option<Field> {
        let mut field = Field::new(
            self.name().1,
            TypeRef::named(format!("{}Connection", self.type_name())),
            |ctx| {
                FieldFuture::new(async move {
                    let mut conn = ctx.data::<Pool<Sqlite>>()?.acquire().await?;
                    let connection = parse_connection_arguments(&ctx)?;

                    let total_count =
                        count_rows(&mut conn, SYSTEM_CALL_TABLE, &None, &Vec::new()).await?;
                    let data = fetch_multiple_rows(
                        &mut conn,
                        SYSTEM_CALL_TABLE,
                        "id",
                        &None,
                        &None,
                        &Vec::new(),
                        &connection,
                    )
                    .await?;
                    let results = connection_output(
                        &data,
                        &SYSTEM_CALL_TYPE_MAPPING,
                        &None,
                        "id",
                        total_count,
                        false,
                    )?;

                    Ok(Some(Value::Object(results)))
                })
            },
        );

        field = connection_arguments(field);

        Some(field)
    }

    fn related_fields(&self) -> Option<Vec<Field>> {
        Some(vec![Field::new("system", TypeRef::named_nn("System"), |ctx| {
            FieldFuture::new(async move {
                let mut conn = ctx.data::<Pool<Sqlite>>()?.acquire().await?;
                let syscall_values = ctx.parent_value.try_downcast_ref::<ValueMapping>()?;
                let system_id = extract::<String>(syscall_values, "system_id")?;
                let system = fetch_single_row(&mut conn, "systems", "id", &system_id).await?;
                let result = value_mapping_from_row(&system, &SYSTEM_TYPE_MAPPING, false)?;

                Ok(Some(Value::Object(result)))
            })
        })])
    }
}
