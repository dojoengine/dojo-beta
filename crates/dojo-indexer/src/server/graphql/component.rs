use juniper::{graphql_object, GraphQLObject};
use juniper_relay_connection::RelayConnectionNode;
use prisma_client_rust::QueryError;

use crate::prisma::{PrismaClient, component};

use super::Query;

#[derive(GraphQLObject)]
pub struct Component {
    pub id: String,
    pub name: String,
    pub transaction_hash: String,
}

impl RelayConnectionNode for Component {
    type Cursor = String;
    fn cursor(&self) -> Self::Cursor {
        self.id.clone()
    }

    fn connection_type_name() -> &'static str {
        "Component"
    }

    fn edge_type_name() -> &'static str {
        "ComponentEdge"
    }
}

#[graphql_object(context = PrismaClient)]
impl Query {
    async fn component(
        context: &PrismaClient,
        id: String,
    ) -> Option<Component> {
        let component = context
            .component()
            .find_first(vec![component::id::equals(id)])
            .exec()
            .await
            .unwrap();

        match component {
            Some(component) => Some(Component { id: component.id, name: component.name, transaction_hash: component.transaction_hash }),
            None => None,
        }
    }
}