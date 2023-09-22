use async_graphql::dynamic::TypeRef;
use async_graphql::Name;

use crate::object::{ObjectTrait, TypeMapping};
use crate::types::{ScalarType, TypeDefinition};

pub struct PageInfoObject {
    pub type_mapping: TypeMapping,
}

impl Default for PageInfoObject {
    fn default() -> Self {
        Self {
            type_mapping: TypeMapping::from([
                (
                    Name::new("hasPreviousPage"),
                    TypeDefinition::Simple(TypeRef::named(TypeRef::BOOLEAN)),
                ),
                (
                    Name::new("hasNextPage"),
                    TypeDefinition::Simple(TypeRef::named(TypeRef::BOOLEAN)),
                ),
                (
                    Name::new("startCursor"),
                    TypeDefinition::Simple(TypeRef::named(ScalarType::Cursor.to_string())),
                ),
                (
                    Name::new("endCursor"),
                    TypeDefinition::Simple(TypeRef::named(ScalarType::Cursor.to_string())),
                ),
            ]),
        }
    }
}

impl ObjectTrait for PageInfoObject {
    fn name(&self) -> &str {
        "pageInfo"
    }

    fn type_name(&self) -> &str {
        "PageInfo"
    }

    fn type_mapping(&self) -> &TypeMapping {
        &self.type_mapping
    }
}
