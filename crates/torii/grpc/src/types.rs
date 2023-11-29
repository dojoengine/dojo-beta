use std::collections::HashMap;
use std::str::FromStr;

use dojo_types::primitive::Primitive;
use dojo_types::schema::{Enum, EnumOption, Member, Struct, Ty};
use serde::{Deserialize, Serialize};
use starknet::core::types::{
    ContractStorageDiffItem, FromByteSliceError, FromStrError, StateDiff, StateUpdate, StorageEntry,
};
use starknet_crypto::FieldElement;

use crate::client::Error as ClientError;
use crate::proto::{self};

#[derive(Debug, Serialize, Deserialize, PartialEq, Hash, Eq, Clone)]
pub struct Query {
    pub clause: Clause,
    pub limit: u32,
    pub offset: u32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Hash, Eq, Clone)]
pub enum Clause {
    Keys(KeysClause),
    Member(MemberClause),
    Composite(CompositeClause),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Hash, Eq, Clone)]
pub struct KeysClause {
    pub model: String,
    pub keys: Vec<FieldElement>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Hash, Eq, Clone)]
pub struct MemberClause {
    pub model: String,
    pub member: String,
    pub operator: ComparisonOperator,
    pub value: Value,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Hash, Eq, Clone)]
pub struct CompositeClause {
    pub model: String,
    pub operator: LogicalOperator,
    pub clauses: Vec<Clause>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Hash, Eq, Clone)]
pub enum LogicalOperator {
    And,
    Or,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Hash, Eq, Clone)]
pub enum ComparisonOperator {
    Eq,
    Neq,
    Gt,
    Gte,
    Lt,
    Lte,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Hash, Eq, Clone)]
pub struct Value {
    pub primitive_type: Primitive,
    pub value_type: ValueType,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Hash, Eq, Clone)]
pub enum ValueType {
    String(String),
    Int(i64),
    UInt(u64),
    Bool(bool),
    Bytes(Vec<u8>),
}

impl TryFrom<proto::types::ModelMetadata> for dojo_types::schema::ModelMetadata {
    type Error = FromStrError;
    fn try_from(value: proto::types::ModelMetadata) -> Result<Self, Self::Error> {
        let schema: Ty = serde_json::from_slice(&value.schema).unwrap();
        let layout: Vec<FieldElement> = value.layout.into_iter().map(FieldElement::from).collect();
        Ok(Self {
            schema,
            layout,
            name: value.name,
            packed_size: value.packed_size,
            unpacked_size: value.unpacked_size,
            class_hash: FieldElement::from_str(&value.class_hash)?,
        })
    }
}

impl TryFrom<proto::types::WorldMetadata> for dojo_types::WorldMetadata {
    type Error = FromStrError;
    fn try_from(value: proto::types::WorldMetadata) -> Result<Self, Self::Error> {
        let models = value
            .models
            .into_iter()
            .map(|component| Ok((component.name.clone(), component.try_into()?)))
            .collect::<Result<HashMap<_, dojo_types::schema::ModelMetadata>, _>>()?;

        Ok(dojo_types::WorldMetadata {
            models,
            world_address: FieldElement::from_str(&value.world_address)?,
            world_class_hash: FieldElement::from_str(&value.world_class_hash)?,
            executor_address: FieldElement::from_str(&value.executor_address)?,
            executor_class_hash: FieldElement::from_str(&value.executor_class_hash)?,
        })
    }
}

impl From<Query> for proto::types::EntityQuery {
    fn from(value: Query) -> Self {
        Self { clause: Some(value.clause.into()), limit: value.limit, offset: value.offset }
    }
}

impl From<Clause> for proto::types::Clause {
    fn from(value: Clause) -> Self {
        match value {
            Clause::Keys(clause) => {
                Self { clause_type: Some(proto::types::clause::ClauseType::Keys(clause.into())) }
            }
            Clause::Member(clause) => {
                Self { clause_type: Some(proto::types::clause::ClauseType::Member(clause.into())) }
            }
            Clause::Composite(clause) => Self {
                clause_type: Some(proto::types::clause::ClauseType::Composite(clause.into())),
            },
        }
    }
}

impl From<KeysClause> for proto::types::KeysClause {
    fn from(value: KeysClause) -> Self {
        Self {
            model: value.model,
            keys: value.keys.iter().map(|k| k.to_bytes_be().into()).collect(),
        }
    }
}

impl TryFrom<proto::types::KeysClause> for KeysClause {
    type Error = FromByteSliceError;

    fn try_from(value: proto::types::KeysClause) -> Result<Self, Self::Error> {
        let keys = value
            .keys
            .into_iter()
            .map(|k| FieldElement::from_byte_slice_be(&k))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { model: value.model, keys })
    }
}

impl From<MemberClause> for proto::types::MemberClause {
    fn from(value: MemberClause) -> Self {
        Self {
            model: value.model,
            member: value.member,
            operator: value.operator as i32,
            value: Some(value.value.into()),
        }
    }
}

impl From<CompositeClause> for proto::types::CompositeClause {
    fn from(value: CompositeClause) -> Self {
        Self {
            model: value.model,
            operator: value.operator as i32,
            clauses: value.clauses.into_iter().map(|clause| clause.into()).collect(),
        }
    }
}

impl From<Value> for proto::types::Value {
    fn from(value: Value) -> Self {
        let value_type = match value.value_type {
            ValueType::String(val) => Some(proto::types::value::ValueType::StringValue(val)),
            ValueType::Int(val) => Some(proto::types::value::ValueType::IntValue(val)),
            ValueType::UInt(val) => Some(proto::types::value::ValueType::UintValue(val)),
            ValueType::Bool(val) => Some(proto::types::value::ValueType::BoolValue(val)),
            ValueType::Bytes(val) => Some(proto::types::value::ValueType::ByteValue(val)),
        };

        Self { primitive_type: value.primitive_type.to_numeric() as i32, value_type }
    }
}

impl TryFrom<proto::types::Value> for Primitive {
    type Error = ClientError;
    fn try_from(value: proto::types::Value) -> Result<Self, Self::Error> {
        let value_type = value.value_type.as_ref().ok_or(ClientError::MissingExpectedData)?;

        let primitive = match value_type {
            proto::types::value::ValueType::BoolValue(bool) => Primitive::Bool(Some(*bool)),
            proto::types::value::ValueType::UintValue(int) => {
                match proto::types::PrimitiveType::try_from(value.primitive_type)
                    .map_err(ClientError::Decode)?
                {
                    proto::types::PrimitiveType::U8 => Primitive::U8(Some(*int as u8)),
                    proto::types::PrimitiveType::U16 => Primitive::U16(Some(*int as u16)),
                    proto::types::PrimitiveType::U32 => Primitive::U32(Some(*int as u32)),
                    proto::types::PrimitiveType::U64 => Primitive::U64(Some(*int as u64)),
                    proto::types::PrimitiveType::Usize => Primitive::USize(Some(*int as u32)),
                    _ => return Err(ClientError::UnsupportedType),
                }
            }
            proto::types::value::ValueType::ByteValue(bytes) => {
                match proto::types::PrimitiveType::try_from(value.primitive_type)
                    .map_err(ClientError::Decode)?
                {
                    proto::types::PrimitiveType::U128
                    | proto::types::PrimitiveType::Felt252
                    | proto::types::PrimitiveType::ClassHash
                    | proto::types::PrimitiveType::ContractAddress => Primitive::Felt252(Some(
                        FieldElement::from_byte_slice_be(bytes).map_err(ClientError::SliceError)?,
                    )),
                    _ => return Err(ClientError::UnsupportedType),
                }
            }
            proto::types::value::ValueType::StringValue(_string) => {
                match proto::types::PrimitiveType::try_from(value.primitive_type)
                    .map_err(ClientError::Decode)?
                {
                    proto::types::PrimitiveType::U256 => {
                        // TODO: Handle u256
                        Primitive::U256(None)
                    }
                    _ => return Err(ClientError::UnsupportedType),
                }
            }
            _ => {
                return Err(ClientError::UnsupportedType);
            }
        };

        Ok(primitive)
    }
}

impl From<proto::types::Enum> for Enum {
    fn from(enum_val: proto::types::Enum) -> Self {
        let options = enum_val
            .options
            .iter()
            .map(|s| EnumOption { name: s.to_owned(), ty: Ty::Tuple(vec![]) })
            .collect::<Vec<_>>();

        Enum { name: enum_val.name.clone(), option: Some(enum_val.option as u8), options }
    }
}

impl TryFrom<proto::types::Model> for Ty {
    type Error = ClientError;
    fn try_from(model: proto::types::Model) -> Result<Self, Self::Error> {
        let mut struct_ty = Struct { name: model.name, children: Vec::new() };

        for member in &model.members {
            let member_type =
                member.member_type.as_ref().ok_or(ClientError::MissingExpectedData)?;

            let ty = match member_type {
                proto::types::member::MemberType::Value(value) => {
                    Ty::Primitive(Primitive::try_from(value.clone())?)
                }
                proto::types::member::MemberType::Enum(enum_val) => {
                    Ty::Enum(Enum::from(enum_val.clone()))
                }
                proto::types::member::MemberType::Struct(nested) => Self::try_from(nested.clone())?,
            };

            struct_ty.children.push(Member { key: false, name: member.name.clone(), ty });
        }

        Ok(Ty::Struct(struct_ty))
    }
}

impl TryFrom<proto::types::StorageEntry> for StorageEntry {
    type Error = FromStrError;
    fn try_from(value: proto::types::StorageEntry) -> Result<Self, Self::Error> {
        Ok(Self {
            key: FieldElement::from_str(&value.key)?,
            value: FieldElement::from_str(&value.value)?,
        })
    }
}

impl TryFrom<proto::types::StorageDiff> for ContractStorageDiffItem {
    type Error = FromStrError;
    fn try_from(value: proto::types::StorageDiff) -> Result<Self, Self::Error> {
        Ok(Self {
            address: FieldElement::from_str(&value.address)?,
            storage_entries: value
                .storage_entries
                .into_iter()
                .map(|entry| entry.try_into())
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl TryFrom<proto::types::EntityDiff> for StateDiff {
    type Error = FromStrError;
    fn try_from(value: proto::types::EntityDiff) -> Result<Self, Self::Error> {
        Ok(Self {
            nonces: vec![],
            declared_classes: vec![],
            replaced_classes: vec![],
            deployed_contracts: vec![],
            deprecated_declared_classes: vec![],
            storage_diffs: value
                .storage_diffs
                .into_iter()
                .map(|diff| diff.try_into())
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl TryFrom<proto::types::EntityUpdate> for StateUpdate {
    type Error = FromStrError;
    fn try_from(value: proto::types::EntityUpdate) -> Result<Self, Self::Error> {
        Ok(Self {
            new_root: FieldElement::ZERO,
            old_root: FieldElement::ZERO,
            block_hash: FieldElement::from_str(&value.block_hash)?,
            state_diff: value.entity_diff.expect("must have").try_into()?,
        })
    }
}
