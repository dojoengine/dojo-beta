use std::collections::HashMap;

use katana_primitives::Felt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ContractChanges {
    pub addr: Felt,
    pub nonce: Felt,
    pub class_hash: Option<Felt>,
    pub storage_changes: HashMap<Felt, Felt>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
pub struct StarknetOsOutput {
    pub initial_root: Felt,
    pub final_root: Felt,
    pub prev_block_number: Felt,
    pub new_block_number: Felt,
    pub prev_block_hash: Felt,
    pub new_block_hash: Felt,
    pub os_program_hash: Felt,
    pub starknet_os_config_hash: Felt,
    pub use_kzg_da: Felt,
    pub full_output: Felt,
    pub messages_to_l1: Vec<Felt>,
    pub messages_to_l2: Vec<Felt>,
    pub contracts: Vec<ContractChanges>,
    pub classes: HashMap<Felt, Felt>,
}
