pub mod db;
pub mod dev;
pub mod execution;
pub mod fork;
pub mod metrics;
pub mod rpc;

use std::collections::{BTreeMap, HashSet};
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;

use db::DbConfig;
use dev::{DevConfig, FixedL1GasPriceConfig};
use execution::ExecutionConfig;
use fork::ForkingConfig;
use katana_core::service::messaging::MessagingConfig;
use katana_primitives::block::{BlockHash, BlockHashOrNumber, BlockNumber, GasPrices};
use katana_primitives::chain::ChainId;
use katana_primitives::chain_spec::ChainSpec;
use katana_primitives::class::ClassHash;
use katana_primitives::genesis::allocation::GenesisAllocation;
use katana_primitives::genesis::GenesisClass;
use katana_primitives::version::ProtocolVersion;
use katana_primitives::{ContractAddress, Felt};
use metrics::MetricsConfig;
use rpc::{ApiKind, RpcConfig};
use starknet::providers::Url;

/// Node configurations.
///
/// List of all possible options that can be used to configure a node.
#[derive(Debug, Clone, Default)]
pub struct Config {
    /// The chain specification.
    pub chain: ChainSpec,

    /// Database options.
    pub db: DbConfig,

    /// Forking options.
    pub forking: Option<ForkingConfig>,

    /// Rpc options.
    pub rpc: RpcConfig,

    /// Metrics options.
    pub metrics: Option<MetricsConfig>,

    /// Execution options.
    pub execution: ExecutionConfig,

    /// Messaging options.
    pub messaging: Option<MessagingConfig>,

    /// Sequencing options.
    pub sequencing: SequencingConfig,

    /// Development options.
    pub dev: DevConfig,
}

/// Configurations related to block production.
#[derive(Debug, Clone, Default)]
pub struct SequencingConfig {
    /// The time in milliseconds for a block to be produced.
    pub block_time: Option<u64>,

    /// Disable automatic block production.
    ///
    /// Allowing block to only be produced manually.
    pub no_mining: bool,
}

#[derive(Default, Debug)]
pub struct ConfigBuilder {
    config: Config,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        ConfigBuilder::default()
    }

    pub fn chain_id(&mut self, chain_id: ChainId) -> &mut Self {
        self.config.chain.id = chain_id;
        self
    }

    pub fn genesis_parent_hash(&mut self, parent_hash: BlockHash) -> &mut Self {
        self.config.chain.genesis.parent_hash = parent_hash;
        self
    }

    pub fn genesis_state_root(&mut self, state_root: Felt) -> &mut Self {
        self.config.chain.genesis.state_root = state_root;
        self
    }

    pub fn genesis_number(&mut self, number: BlockNumber) -> &mut Self {
        self.config.chain.genesis.number = number;
        self
    }

    pub fn genesis_timestamp(&mut self, timestamp: u64) -> &mut Self {
        self.config.chain.genesis.timestamp = timestamp;
        self
    }

    pub fn genesis_sequencer_address(&mut self, sequencer_address: ContractAddress) -> &mut Self {
        self.config.chain.genesis.sequencer_address = sequencer_address;
        self
    }

    pub fn genesis_gas_prices(&mut self, gas_prices: GasPrices) -> &mut Self {
        self.config.chain.genesis.gas_prices = gas_prices;
        self
    }

    pub fn genesis_classes(&mut self, classes: BTreeMap<ClassHash, GenesisClass>) -> &mut Self {
        self.config.chain.genesis.classes = classes;
        self
    }

    pub fn genesis_allocations(
        &mut self,
        allocations: BTreeMap<ContractAddress, GenesisAllocation>,
    ) -> &mut Self {
        self.config.chain.genesis.allocations = allocations;
        self
    }

    pub fn fee_contracts_eth(&mut self, eth: ContractAddress) -> &mut Self {
        self.config.chain.fee_contracts.eth = eth;
        self
    }

    pub fn fee_contracts_strk(&mut self, strk: ContractAddress) -> &mut Self {
        self.config.chain.fee_contracts.strk = strk;
        self
    }

    pub fn chain_protocol_version(&mut self, version: ProtocolVersion) -> &mut Self {
        self.config.chain.version = version;
        self
    }

    pub fn db_dir(&mut self, dir: Option<PathBuf>) -> &mut Self {
        self.config.db.dir = dir;
        self
    }

    pub fn forking(&mut self, forking: Option<ForkingConfig>) -> &mut Self {
        self.config.forking = forking;
        self
    }

    pub fn fork_url(&mut self, url: Url) -> &mut Self {
        self.config.forking.get_or_insert(ForkingConfig { url, block: None }).url = url.clone();
        self
    }

    pub fn fork_block(&mut self, block: Option<BlockHashOrNumber>) -> &mut Self {
        self.config
            .forking
            .get_or_insert(ForkingConfig { url: Url::from_str("").unwrap(), block: None })
            .block = block;
        self
    }

    pub fn rpc_port(&mut self, port: u16) -> &mut Self {
        self.config.rpc.port = port;
        self
    }

    pub fn rpc_addr(&mut self, addr: IpAddr) -> &mut Self {
        self.config.rpc.addr = addr;
        self
    }

    pub fn rpc_max_connections(&mut self, max_connections: u32) -> &mut Self {
        self.config.rpc.max_connections = max_connections;
        self
    }

    pub fn rpc_allowed_origins(&mut self, allowed_origins: Option<Vec<String>>) -> &mut Self {
        self.config.rpc.allowed_origins = allowed_origins;
        self
    }

    pub fn rpc_apis(&mut self, apis: HashSet<ApiKind>) -> &mut Self {
        self.config.rpc.apis = apis;
        self
    }

    pub fn metrics_addr(&mut self, addr: SocketAddr) -> &mut Self {
        self.config.metrics.get_or_insert(MetricsConfig { addr }).addr = addr;
        self
    }

    pub fn execution_invocation_max_steps(&mut self, steps: u32) -> &mut Self {
        self.config.execution.invocation_max_steps = steps;
        self
    }

    pub fn execution_validation_max_steps(&mut self, steps: u32) -> &mut Self {
        self.config.execution.validation_max_steps = steps;
        self
    }

    pub fn execution_max_recursion_depth(&mut self, depth: usize) -> &mut Self {
        self.config.execution.max_recursion_depth = depth;
        self
    }

    pub fn messaging_chain(&mut self, chain: String) -> &mut Self {
        self.config
            .messaging
            .get_or_insert(MessagingConfig { chain, ..Default::default() })
            .chain = chain.clone();
        self
    }

    pub fn messaging_rpc_url(&mut self, rpc_url: String) -> &mut Self {
        self.config
            .messaging
            .get_or_insert(MessagingConfig { rpc_url, ..Default::default() })
            .rpc_url = rpc_url.clone();
        self
    }

    pub fn messaging_contract_address(&mut self, contract_address: String) -> &mut Self {
        self.config
            .messaging
            .get_or_insert(MessagingConfig { contract_address, ..Default::default() })
            .contract_address = contract_address.clone();
        self
    }

    pub fn messaging_sender_address(&mut self, sender_address: String) -> &mut Self {
        self.config
            .messaging
            .get_or_insert(MessagingConfig { sender_address, ..Default::default() })
            .sender_address = sender_address.clone();
        self
    }

    pub fn messaging_private_key(&mut self, private_key: String) -> &mut Self {
        self.config
            .messaging
            .get_or_insert(MessagingConfig { private_key, ..Default::default() })
            .private_key = private_key.clone();
        self
    }

    pub fn messaging_interval(&mut self, interval: u64) -> &mut Self {
        self.config
            .messaging
            .get_or_insert(MessagingConfig { interval, ..Default::default() })
            .interval = interval;
        self
    }

    pub fn messaging_from_block(&mut self, from_block: u64) -> &mut Self {
        self.config
            .messaging
            .get_or_insert(MessagingConfig { from_block, ..Default::default() })
            .from_block = from_block;
        self
    }

    pub fn sequencing_block_time(&mut self, block_time: Option<u64>) -> &mut Self {
        self.config.sequencing.block_time = block_time;
        self
    }

    pub fn sequencing_no_mining(&mut self, no_mining: bool) -> &mut Self {
        self.config.sequencing.no_mining = no_mining;
        self
    }

    pub fn dev_fee(&mut self, fee: bool) -> &mut Self {
        self.config.dev.fee = fee;
        self
    }

    pub fn dev_account_validation(&mut self, validation: bool) -> &mut Self {
        self.config.dev.account_validation = validation;
        self
    }

    pub fn dev_fixed_gas_prices(&mut self, gas_prices: Option<FixedL1GasPriceConfig>) -> &mut Self {
        self.config.dev.fixed_gas_prices = gas_prices;
        self
    }

    pub fn chain(&mut self, chain: ChainSpec) -> &mut Self {
        self.config.chain = chain;
        self
    }

    pub fn db(&mut self, db: DbConfig) -> &mut Self {
        self.config.db = db;
        self
    }

    pub fn rpc(&mut self, rpc: RpcConfig) -> &mut Self {
        self.config.rpc = rpc;
        self
    }

    pub fn metrics(&mut self, metrics: Option<MetricsConfig>) -> &mut Self {
        self.config.metrics = metrics;
        self
    }

    pub fn execution(&mut self, execution: ExecutionConfig) -> &mut Self {
        self.config.execution = execution;
        self
    }

    pub fn messaging(&mut self, messaging: Option<MessagingConfig>) -> &mut Self {
        self.config.messaging = messaging;
        self
    }

    pub fn sequencing(&mut self, sequencing: SequencingConfig) -> &mut Self {
        self.config.sequencing = sequencing;
        self
    }

    pub fn dev(&mut self, dev: DevConfig) -> &mut Self {
        self.config.dev = dev;
        self
    }

    pub fn build(&mut self) -> Config {
        self.config.clone()
    }
}
