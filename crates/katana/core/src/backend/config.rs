use blockifier::block_context::BlockContext;
use starknet_api::block::{BlockNumber, BlockTimestamp};
use starknet_api::core::ChainId;
use url::Url;

use crate::constants::{
    DEFAULT_GAS_PRICE, DEFAULT_INVOKE_MAX_STEPS, DEFAULT_VALIDATE_MAX_STEPS, FEE_TOKEN_ADDRESS,
    SEQUENCER_ADDRESS,
};
use crate::env::{get_default_vm_resource_fee_cost, BlockContextGenerator};

#[derive(Debug, Clone)]
pub struct StarknetConfig {
    pub seed: [u8; 32],
    pub total_accounts: u8,
    pub disable_fee: bool,
    pub env: Environment,
    pub fork_rpc_url: Option<Url>,
    pub fork_block_number: Option<u64>,
}

impl StarknetConfig {
    pub fn block_context(&self) -> BlockContext {
        BlockContext {
            block_number: BlockNumber::default(),
            chain_id: ChainId(self.env.chain_id.clone()),
            block_timestamp: BlockTimestamp::default(),
            sequencer_address: (*SEQUENCER_ADDRESS).into(),
            fee_token_address: (*FEE_TOKEN_ADDRESS).into(),
            vm_resource_fee_cost: get_default_vm_resource_fee_cost().into(),
            gas_price: self.env.gas_price,
            validate_max_n_steps: self.env.validate_max_steps,
            invoke_tx_max_n_steps: self.env.invoke_max_steps,
            max_recursion_depth: 1000,
        }
    }

    pub fn block_context_generator(&self) -> BlockContextGenerator {
        BlockContextGenerator::default()
    }
}

impl Default for StarknetConfig {
    fn default() -> Self {
        Self {
            seed: [0; 32],
            total_accounts: 10,
            disable_fee: false,
            fork_rpc_url: None,
            fork_block_number: None,
            env: Environment::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Environment {
    pub chain_id: String,
    pub gas_price: u128,
    pub invoke_max_steps: u32,
    pub validate_max_steps: u32,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            gas_price: DEFAULT_GAS_PRICE,
            chain_id: "KATANA".to_string(),
            invoke_max_steps: DEFAULT_INVOKE_MAX_STEPS,
            validate_max_steps: DEFAULT_VALIDATE_MAX_STEPS,
        }
    }
}
