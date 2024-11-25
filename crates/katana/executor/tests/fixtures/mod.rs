pub mod transaction;

use alloy_primitives::U256;
use katana_executor::implementation::noop::NoopExecutorFactory;
use katana_executor::{ExecutionFlags, ExecutorFactory};
use katana_primitives::block::{
    Block, ExecutableBlock, FinalityStatus, GasPrices, PartialHeader, SealedBlockWithStatus,
};
use katana_primitives::chain::ChainId;
use katana_primitives::chain_spec::{self, ChainSpec};
use katana_primitives::class::{CompiledClass, ContractClass};
use katana_primitives::contract::ContractAddress;
use katana_primitives::da::L1DataAvailabilityMode;
use katana_primitives::env::{CfgEnv, FeeTokenAddressses};
use katana_primitives::genesis::allocation::DevAllocationsGenerator;
use katana_primitives::genesis::constant::{
    DEFAULT_ACCOUNT_CLASS_HASH, DEFAULT_ETH_FEE_TOKEN_ADDRESS, DEFAULT_LEGACY_ERC20_CLASS_HASH,
    DEFAULT_PREFUNDED_ACCOUNT_BALANCE, DEFAULT_STRK_FEE_TOKEN_ADDRESS,
};
use katana_primitives::transaction::{
    DeclareTx, DeclareTxV2, DeclareTxWithClass, DeployAccountTx, DeployAccountTxV1, ExecutableTx,
    ExecutableTxWithHash, InvokeTx, InvokeTxV1,
};
use katana_primitives::utils::class::{parse_compiled_class, parse_sierra_class};
use katana_primitives::version::CURRENT_STARKNET_VERSION;
use katana_primitives::{address, Felt};
use katana_provider::providers::db::DbProvider;
use katana_provider::traits::block::BlockWriter;
use katana_provider::traits::state::{StateFactoryProvider, StateProvider};
use starknet::macros::felt;

// TODO: remove support for legacy contract declaration
#[allow(unused)]
pub fn legacy_contract_class() -> CompiledClass {
    let json = include_str!("legacy_contract.json");
    let artifact = serde_json::from_str(json).unwrap();
    parse_compiled_class(artifact).unwrap()
}

pub fn contract_class() -> (CompiledClass, ContractClass) {
    let json = include_str!("contract.json");
    let artifact = serde_json::from_str(json).unwrap();

    let class = parse_sierra_class(json).unwrap();
    let compiled = parse_compiled_class(artifact).unwrap();

    (compiled, class)
}

#[rstest::fixture]
#[once]
pub fn chain() -> ChainSpec {
    let mut chain = chain_spec::DEV_UNALLOCATED.clone();

    // to generate the exact list of accounts as you would when you just run `katana` w/o
    // any additional flags
    let mut seed = [0u8; 32];
    seed[0] = b'0';

    let accounts = DevAllocationsGenerator::new(10)
        .with_seed(seed)
        .with_balance(U256::from(DEFAULT_PREFUNDED_ACCOUNT_BALANCE))
        .generate();

    chain.genesis.extend_allocations(accounts.into_iter().map(|(k, v)| (k, v.into())));
    chain
}

/// Returns a state provider with some prefilled states.
#[rstest::fixture]
pub fn state_provider(chain: &ChainSpec) -> Box<dyn StateProvider> {
    let states = chain.state_updates();
    let provider = DbProvider::new_ephemeral();

    let block = SealedBlockWithStatus {
        status: FinalityStatus::AcceptedOnL2,
        block: Block::default().seal_with_hash(123u64.into()),
    };

    provider
        .insert_block_with_states_and_receipts(block, states, vec![], vec![])
        .expect("able to insert block");

    provider.latest().unwrap()
}

// TODO: update the txs to include valid signatures
/// Returns an array of blocks with transaction that are valid against the state by
/// [state_provider].
#[rstest::fixture]
pub fn valid_blocks() -> [ExecutableBlock; 3] {
    let protocol_version = CURRENT_STARKNET_VERSION;
    let chain_id = ChainId::parse("KATANA").unwrap();
    let sequencer_address = ContractAddress(1u64.into());

    // one of the accounts that is generated by the `genesis` fixture. also one of the
    // accounts generated by katana.
    let sender_address =
        address!("0x2af9427c5a277474c079a1283c880ee8a6f0f8fbf73ce969c08d88befec1bba");

    let gas_prices = GasPrices { eth: 100 * u128::pow(10, 9), strk: 100 * u128::pow(10, 9) };

    [
        ExecutableBlock {
            header: PartialHeader {
                protocol_version: protocol_version.clone(),
                number: 1,
                timestamp: 100,
                sequencer_address,
                parent_hash: 123u64.into(),
                l1_gas_prices: gas_prices.clone(),
                l1_data_gas_prices: gas_prices.clone(),
                l1_da_mode: L1DataAvailabilityMode::Calldata,
            },
            body: vec![
                // fund the account to be deployed, sending 0x9999999999999 amount
                ExecutableTxWithHash::new(ExecutableTx::Invoke(InvokeTx::V1(InvokeTxV1 {
                    chain_id,
                    sender_address,
                    calldata: vec![
                        felt!("0x1"),
                        felt!("0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7"),
                        felt!("0x83afd3f4caedc6eebf44246fe54e38c95e3179a5ec9ea81740eca5b482d12e"),
                        felt!("0x3"),
                        felt!("0x3ddfa445a70b927497249f94ff7431fc2e2abc761a34417fd4891beb7c2db85"),
                        felt!("0x9999999999999999"),
                        felt!("0x0"),
                    ],
                    max_fee: 4367000000000000,
                    signature: vec![],
                    nonce: Felt::ZERO,
                }))),
                // declare contract
                ExecutableTxWithHash::new(ExecutableTx::Declare({
                    let (.., class) = contract_class();
                    DeclareTxWithClass {
                        class: class.into(),
                        transaction: DeclareTx::V2(DeclareTxV2 {
                            nonce: Felt::ONE,
                            max_fee: 27092100000000000,
                            chain_id,
                            signature: vec![],
                            sender_address,
                            class_hash: felt!("0x420"),
                            compiled_class_hash: felt!(
                                "0x16c6081eb34ad1e0c5513234ed0c025b3c7f305902d291bad534cd6474c85bc"
                            ),
                        }),
                    }
                })),
            ],
        },
        ExecutableBlock {
            header: PartialHeader {
                protocol_version: protocol_version.clone(),
                number: 2,
                timestamp: 200,
                sequencer_address,
                parent_hash: 1234u64.into(),
                l1_gas_prices: gas_prices.clone(),
                l1_data_gas_prices: gas_prices.clone(),
                l1_da_mode: L1DataAvailabilityMode::Calldata,
            },
            body: vec![
                // deploy account tx with the default account class
                ExecutableTxWithHash::new(ExecutableTx::DeployAccount(DeployAccountTx::V1(
                    DeployAccountTxV1 {
                        chain_id,
                        max_fee: 1443900000000000,
                        signature: vec![],
                        nonce: 0u64.into(),
                        contract_address_salt: felt!(
                            "0x2ce091f544a799160324295e62da74d194eda204682b5b8fd0dd4d2f8f5ab18"
                        ),
                        constructor_calldata: vec![felt!(
                            "0x4c339f18b9d1b95b64a6d378abd1480b2e0d5d5bd33cd0828cbce4d65c27284"
                        )],
                        class_hash: DEFAULT_ACCOUNT_CLASS_HASH,
                        contract_address: address!(
                            "0x3ddfa445a70b927497249f94ff7431fc2e2abc761a34417fd4891beb7c2db85"
                        ),
                    },
                ))),
            ],
        },
        ExecutableBlock {
            header: PartialHeader {
                protocol_version,
                number: 3,
                timestamp: 300,
                sequencer_address,
                parent_hash: 12345u64.into(),
                l1_gas_prices: gas_prices.clone(),
                l1_data_gas_prices: gas_prices.clone(),
                l1_da_mode: L1DataAvailabilityMode::Calldata,
            },
            body: vec![
                // deploy a erc20 contract using UDC
                ExecutableTxWithHash::new(ExecutableTx::Invoke(InvokeTx::V1(InvokeTxV1 {
                    chain_id,
                    sender_address,
                    // the calldata is encoded based on the standard account call encoding
                    calldata: vec![
                        felt!("0x1"),
                        felt!("0x41a78e741e5af2fec34b695679bc6891742439f7afb8484ecd7766661ad02bf"),
                        felt!("0x1987cbd17808b9a23693d4de7e246a443cfe37e6e7fbaeabd7d7e6532b07c3d"),
                        felt!("10"), // the # of felts after this point
                        // --- udc::deployContract arguments
                        DEFAULT_LEGACY_ERC20_CLASS_HASH, // class hash
                        felt!("0x6ea2ff5aa6f633708e69f5c61d2ac5f860d2435b46ddbd016aa065bce25100a"), /* salt */
                        felt!("0x1"), // uniquness
                        felt!("6"),   // ctor calldata length
                        // ---- ctor calldata of the erc20 class
                        felt!("0x4b415249"),   // erc20 name
                        felt!("0x4b415249"),   // erc20 symbol
                        felt!("0x12"),         // erc20 decimals
                        felt!("0x1b39"),       // erc20 total supply (low)
                        felt!("0x0"),          // erc20 total supply (high)
                        sender_address.into(), // recipient
                    ],
                    max_fee: 2700700000000000,
                    signature: vec![],
                    nonce: Felt::TWO,
                }))),
            ],
        },
    ]
}

#[rstest::fixture]
pub fn cfg() -> CfgEnv {
    let fee_token_addresses = FeeTokenAddressses {
        eth: DEFAULT_ETH_FEE_TOKEN_ADDRESS,
        strk: DEFAULT_STRK_FEE_TOKEN_ADDRESS,
    };

    CfgEnv {
        fee_token_addresses,
        max_recursion_depth: 100,
        validate_max_n_steps: 1_000_000,
        invoke_tx_max_n_steps: 1_000_000,
        chain_id: ChainId::parse("KATANA").unwrap(),
    }
}

// TODO: test both with and without the flags turned on
#[rstest::fixture]
pub fn flags(
    #[default(false)] skip_validate: bool,
    #[default(false)] skip_fee_transfer: bool,
) -> ExecutionFlags {
    ExecutionFlags::new().with_account_validation(!skip_validate).with_fee(!skip_fee_transfer)
}

/// A fixture that provides a default `ExecutorFactory` implementation.
#[rstest::fixture]
#[default(NoopExecutorFactory)]
pub fn executor_factory<EF: ExecutorFactory>(
    #[default(NoopExecutorFactory::new())] factory: EF,
) -> EF {
    factory
}

#[cfg(feature = "blockifier")]
pub mod blockifier {
    use katana_executor::implementation::blockifier::BlockifierFactory;
    use katana_executor::ExecutionFlags;

    use super::{cfg, flags, CfgEnv};

    #[rstest::fixture]
    pub fn factory(cfg: CfgEnv, #[with(true)] flags: ExecutionFlags) -> BlockifierFactory {
        BlockifierFactory::new(cfg, flags)
    }
}
