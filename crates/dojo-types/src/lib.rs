use std::collections::HashMap;

use component::ModelMetadata;
use serde::Serialize;
use starknet::core::types::FieldElement;
use system::SystemMetadata;

pub mod core;
pub mod event;
pub mod model;
pub mod storage;
pub mod system;

/// Represents the metadata of a World
#[derive(Debug, Clone, Serialize, Default)]
pub struct WorldMetadata {
    pub world_address: FieldElement,
    pub world_class_hash: FieldElement,
    pub executor_address: FieldElement,
    pub executor_class_hash: FieldElement,
    pub systems: HashMap<String, SystemMetadata>,
    pub components: HashMap<String, ModelMetadata>,
}
