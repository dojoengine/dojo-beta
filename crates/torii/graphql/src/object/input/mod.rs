use async_graphql::dynamic::InputObject;

use super::TypeMapping;

pub mod r#where;



pub trait InputObjectTrait {
  // Type name of the input graphql object, we don't need a name as this will always be an input object
  fn type_name(&self) -> &str;

  // Type mapping consist of parent object's mapping plus 6 comparators for each component member
  fn type_mapping(&self) -> &TypeMapping;

  // Create a new graphql input object with fields defined from type mapping
  fn create(&self) -> InputObject;
}