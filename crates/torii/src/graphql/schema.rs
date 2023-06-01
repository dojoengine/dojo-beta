use anyhow::{anyhow, Result};
use async_graphql::dynamic::{Object, Scalar, Schema};
use async_graphql::Name;
use dojo_world::manifest::Member;
use sqlx::SqlitePool;

use super::component::{Component, ComponentObject};
use super::entity::EntityObject;
use super::event::EventObject;
use super::storage::StorageObject;
use super::system::SystemObject;
use super::system_call::SystemCallObject;
use super::types::ScalarType;
use super::utils::format_name;
use super::{ObjectTrait, TypeMapping};

pub async fn build_schema(pool: &SqlitePool) -> Result<Schema> {
    let mut schema_builder = Schema::build("Query", None, None);

    // base objects + storage objects (component instances)
    let mut objects = base_objects();
    objects.extend(storage_objects(pool).await?);

    // collect field resolvers
    let mut fields = Vec::new();
    for object in &objects {
        fields.extend(object.field_resolvers());
    }

    // add field resolvers to query root
    let mut query_root = Object::new("Query");
    for field in fields {
        query_root = query_root.field(field);
    }

    // register custom scalars
    for scalar_type in ScalarType::types().iter() {
        schema_builder = schema_builder.register(Scalar::new(*scalar_type));
    }

    // register gql objects
    for object in &objects {
        schema_builder = schema_builder.register(object.create());
    }

    schema_builder.register(query_root).data(pool.clone()).finish().map_err(|e| e.into())
}

// predefined base objects
fn base_objects() -> Vec<Box<dyn ObjectTrait>> {
    vec![
        Box::new(EntityObject::new()),
        Box::new(ComponentObject::new()),
        Box::new(SystemObject::new()),
        Box::new(EventObject::new()),
        Box::new(SystemCallObject::new()),
    ]
}

async fn storage_objects(pool: &SqlitePool) -> Result<Vec<Box<dyn ObjectTrait>>> {
    let mut conn = pool.acquire().await?;
    let mut objects = Vec::new();

    let components: Vec<Component> =
        sqlx::query_as("SELECT * FROM components").fetch_all(&mut conn).await?;

    for component in components {
        let storage_object = process_component(component)?;
        objects.push(storage_object);
    }

    Ok(objects)
}

fn process_component(component: Component) -> Result<Box<dyn ObjectTrait>> {
    let members: Vec<Member> = serde_json::from_str(&component.storage_definition)?;

    let field_type_mapping = members.iter().fold(TypeMapping::new(), |mut mapping, member| {
        // TODO: check if member type exists in scalar types
        mapping.insert(Name::new(&member.name), member.ty.to_string());
        mapping
    });

    let (name, type_name) = format_name(component.name.as_str());

    Ok(Box::new(StorageObject::new(name, type_name, field_type_mapping)))
}
