use std::collections::HashSet;
use std::fmt;
use std::str::FromStr;

use async_graphql::dynamic::TypeRef;
use async_graphql::{Name, Value};
use indexmap::IndexMap;

// ValueMapping is used to map the values of the fields of a model and TypeMapping their
// correpsonding types. Both are used at runtime to dynamically build/resolve graphql
// queries/schema. `Value` from async-graphql supports nesting, but TypeRef does not. TypeData is
// used to support nesting.
pub type ValueMapping = IndexMap<Name, Value>;
pub type TypeMapping = IndexMap<Name, TypeData>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeData {
    Simple(TypeRef),
    Nested((TypeRef, IndexMap<Name, TypeData>)),
    // TODO: Enum, could be combined with Simple
}

impl TypeData {
    pub fn type_ref(&self) -> TypeRef {
        match self {
            TypeData::Simple(ty) | TypeData::Nested((ty, _)) => ty.clone(),
        }
    }

    pub fn is_simple(&self) -> bool {
        matches!(self, TypeData::Simple(_))
    }

    pub fn is_nested(&self) -> bool {
        matches!(self, TypeData::Nested(_))
    }

    pub fn type_mapping(&self) -> Option<&IndexMap<Name, TypeData>> {
        match self {
            TypeData::Simple(_) => None,
            TypeData::Nested((_, type_mapping)) => Some(type_mapping),
        }
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum ScalarType {
    U8,
    U16,
    U32,
    U64,
    U128,
    U256,
    USize,
    Bool,
    Cursor,
    Address,
    ClassHash,
    DateTime,
    Felt252,
    Enum,
    Custom(String),
}

impl fmt::Display for ScalarType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ScalarType::U8 => write!(f, "u8"),
            ScalarType::U16 => write!(f, "u16"),
            ScalarType::U32 => write!(f, "u32"),
            ScalarType::U64 => write!(f, "u64"),
            ScalarType::U128 => write!(f, "u128"),
            ScalarType::U256 => write!(f, "u256"),
            ScalarType::USize => write!(f, "usize"),
            ScalarType::Bool => write!(f, "bool"),
            ScalarType::Cursor => write!(f, "Cursor"),
            ScalarType::Address => write!(f, "ContractAddress"),
            ScalarType::ClassHash => write!(f, "ClassHash"),
            ScalarType::DateTime => write!(f, "DateTime"),
            ScalarType::Felt252 => write!(f, "felt252"),
            ScalarType::Enum => write!(f, "Enum"),
            ScalarType::Custom(ref type_name) => write!(f, "{}", type_name),
        }
    }
}

impl ScalarType {
    pub fn default_types() -> HashSet<ScalarType> {
        vec![
            ScalarType::U8,
            ScalarType::U16,
            ScalarType::U32,
            ScalarType::U64,
            ScalarType::U128,
            ScalarType::U256,
            ScalarType::USize,
            ScalarType::Bool,
            ScalarType::Cursor,
            ScalarType::Address,
            ScalarType::ClassHash,
            ScalarType::DateTime,
            ScalarType::Felt252,
            ScalarType::Enum,
        ]
        .into_iter()
        .collect()
    }

    pub fn numeric_types() -> HashSet<ScalarType> {
        vec![
            ScalarType::U8,
            ScalarType::U16,
            ScalarType::U32,
            ScalarType::U64,
            ScalarType::USize,
            ScalarType::Bool,
            ScalarType::Enum,
        ]
        .into_iter()
        .collect()
    }

    // u128 and u256 are non numeric here due to
    // sqlite constraint on integer columns
    pub fn _non_numeric_types() -> HashSet<ScalarType> {
        vec![
            ScalarType::U128,
            ScalarType::U256,
            ScalarType::Cursor,
            ScalarType::Address,
            ScalarType::ClassHash,
            ScalarType::DateTime,
            ScalarType::Felt252,
        ]
        .into_iter()
        .collect()
    }

    pub fn is_numeric_type(&self) -> bool {
        ScalarType::numeric_types().contains(self)
    }
}

impl FromStr for ScalarType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "u8" => Ok(ScalarType::U8),
            "u16" => Ok(ScalarType::U16),
            "u32" => Ok(ScalarType::U32),
            "u64" => Ok(ScalarType::U64),
            "u128" => Ok(ScalarType::U128),
            "u256" => Ok(ScalarType::U256),
            "usize" => Ok(ScalarType::USize),
            "bool" => Ok(ScalarType::Bool),
            "Cursor" => Ok(ScalarType::Cursor),
            "ContractAddress" => Ok(ScalarType::Address),
            "ClassHash" => Ok(ScalarType::ClassHash),
            "DateTime" => Ok(ScalarType::DateTime),
            "felt252" => Ok(ScalarType::Felt252),
            "Enum" => Ok(ScalarType::Enum),
            _ => Ok(ScalarType::Custom(s.to_string())),
        }
    }
}
