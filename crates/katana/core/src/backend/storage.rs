use std::path::Path;

use anyhow::{anyhow, Result};
use katana_db::init_db;
use katana_primitives::block::{BlockHash, FinalityStatus, SealedBlockWithStatus};
use katana_primitives::genesis::Genesis;
use katana_primitives::state::StateUpdatesWithDeclaredClasses;
use katana_provider::providers::db::DbProviderFactory;
use katana_provider::traits::block::{BlockHashProvider, BlockWriter};
use katana_provider::traits::{Provider, ProviderFactory, ProviderMut};
use katana_provider::{BlockchainProvider, ProviderResult};

pub struct Blockchain {
    provider_factory: Box<dyn ProviderFactory>,
}

impl Blockchain {
    pub fn new(provider_factory: impl ProviderFactory + 'static) -> Self {
        Self { provider_factory: Box::new(provider_factory) }
    }

    /// Creates a new [Blockchain] with the given [Database] implementation and genesis state.
    pub fn new_with_genesis(
        provider_factory: impl ProviderFactory + 'static,
        genesis: &Genesis,
    ) -> Result<Self> {
        // check whether the genesis block has been initialized
        let genesis_hash = provider_factory.provider()?.block_hash_by_num(genesis.number)?;

        match genesis_hash {
            Some(db_hash) => {
                let genesis_hash = genesis.block().header.compute_hash();
                // check genesis should be the same
                if db_hash == genesis_hash {
                    Ok(Self::new(provider_factory))
                } else {
                    Err(anyhow!(
                        "Genesis block hash mismatch: expected {genesis_hash:#x}, got {db_hash:#}",
                    ))
                }
            }

            None => {
                let block = genesis.block().seal();
                let block = SealedBlockWithStatus { block, status: FinalityStatus::AcceptedOnL1 };
                let state_updates = genesis.state_updates();

                Self::new_with_block_and_state(provider_factory, block, state_updates)
            }
        }
    }

    /// Creates a new [Blockchain] from a database at `path` and `genesis` state.
    pub fn new_with_db(db_path: impl AsRef<Path>, genesis: &Genesis) -> Result<Self> {
        let db = init_db(db_path)?;
        let provider = DbProviderFactory::new(db);
        Self::new_with_genesis(provider, genesis)
    }

    /// Builds a new blockchain with a forked block.
    pub fn new_from_forked(
        provider_factory: impl ProviderFactory + 'static,
        genesis_hash: BlockHash,
        genesis: &Genesis,
        block_status: FinalityStatus,
    ) -> Result<Self> {
        let block = genesis.block().seal_with_hash_and_status(genesis_hash, block_status);
        let state_updates = genesis.state_updates();
        Self::new_with_block_and_state(provider_factory, block, state_updates)
    }

    pub fn provider(&self) -> ProviderResult<BlockchainProvider<Box<dyn Provider>>> {
        Ok(self.provider_factory.provider()?)
    }

    pub fn provider_mut(&self) -> ProviderResult<BlockchainProvider<Box<dyn ProviderMut>>> {
        Ok(self.provider_factory.provider_mut()?)
    }

    fn new_with_block_and_state(
        provider_factory: impl ProviderFactory + 'static,
        block: SealedBlockWithStatus,
        states: StateUpdatesWithDeclaredClasses,
    ) -> Result<Self> {
        BlockWriter::insert_block_with_states_and_receipts(
            &provider_factory.provider_mut()?,
            block,
            states,
            vec![],
            vec![],
        )?;
        Ok(Self::new(provider_factory))
    }
}

#[cfg(test)]
mod tests {
    use katana_primitives::block::{
        Block, FinalityStatus, GasPrices, Header, SealedBlockWithStatus,
    };
    use katana_primitives::fee::TxFeeInfo;
    use katana_primitives::genesis::constant::{
        DEFAULT_FEE_TOKEN_ADDRESS, DEFAULT_LEGACY_ERC20_CONTRACT_CASM,
        DEFAULT_LEGACY_ERC20_CONTRACT_CLASS_HASH, DEFAULT_LEGACY_UDC_CASM,
        DEFAULT_LEGACY_UDC_CLASS_HASH, DEFAULT_UDC_ADDRESS,
    };
    use katana_primitives::genesis::Genesis;
    use katana_primitives::receipt::{InvokeTxReceipt, Receipt};
    use katana_primitives::state::StateUpdatesWithDeclaredClasses;
    use katana_primitives::trace::TxExecInfo;
    use katana_primitives::transaction::{InvokeTx, Tx, TxWithHash};
    use katana_primitives::FieldElement;
    use katana_provider::providers::in_memory::InMemoryProvider;
    use katana_provider::traits::block::{
        BlockHashProvider, BlockNumberProvider, BlockProvider, BlockStatusProvider, BlockWriter,
        HeaderProvider,
    };
    use katana_provider::traits::state::StateFactoryProvider;
    use katana_provider::traits::transaction::{TransactionProvider, TransactionTraceProvider};
    use starknet::core::types::PriceUnit;
    use starknet::macros::felt;

    use super::Blockchain;

    #[test]
    fn blockchain_from_genesis_states() {
        let provider = InMemoryProvider::new();

        let blockchain = Blockchain::new_with_genesis(provider, &Genesis::default())
            .expect("failed to create blockchain from genesis block");
        let state = provider.latest().expect("failed to get latest state");

        let latest_number = provider.latest_number().unwrap();
        let fee_token_class_hash =
            state.class_hash_of_contract(DEFAULT_FEE_TOKEN_ADDRESS).unwrap().unwrap();
        let udc_class_hash = state.class_hash_of_contract(DEFAULT_UDC_ADDRESS).unwrap().unwrap();

        assert_eq!(latest_number, 0);
        assert_eq!(udc_class_hash, DEFAULT_LEGACY_UDC_CLASS_HASH);
        assert_eq!(fee_token_class_hash, DEFAULT_LEGACY_ERC20_CONTRACT_CLASS_HASH);
    }

    #[test]
    fn blockchain_from_fork() {
        let provider = InMemoryProvider::new();

        let genesis = Genesis {
            number: 23,
            parent_hash: FieldElement::ZERO,
            state_root: felt!("1334"),
            timestamp: 6868,
            gas_prices: GasPrices { eth: 9090, strk: 8080 },
            ..Default::default()
        };

        let genesis_hash = felt!("1111");

        let blockchain = Blockchain::new_from_forked(
            provider,
            genesis_hash,
            &genesis,
            FinalityStatus::AcceptedOnL1,
        )
        .expect("failed to create fork blockchain");

        let latest_number = provider.latest_number().unwrap();
        let latest_hash = provider.latest_hash().unwrap();
        let header = provider.header(latest_number.into()).unwrap().unwrap();
        let block_status = provider.block_status(latest_number.into()).unwrap().unwrap();

        assert_eq!(latest_number, genesis.number);
        assert_eq!(latest_hash, genesis_hash);

        assert_eq!(header.gas_prices.eth, 9090);
        assert_eq!(header.gas_prices.strk, 8080);
        assert_eq!(header.timestamp, 6868);
        assert_eq!(header.number, latest_number);
        assert_eq!(header.state_root, genesis.state_root);
        assert_eq!(header.parent_hash, genesis.parent_hash);
        assert_eq!(block_status, FinalityStatus::AcceptedOnL1);
    }

    #[test]
    fn blockchain_from_db() {
        let db_path = tempfile::TempDir::new().expect("Failed to create temp dir.").into_path();

        let dummy_tx = TxWithHash {
            hash: felt!("0xbad"),
            transaction: Tx::Invoke(InvokeTx::V1(Default::default())),
        };

        let dummy_block = SealedBlockWithStatus {
            status: FinalityStatus::AcceptedOnL1,
            block: Block {
                header: Header {
                    parent_hash: FieldElement::ZERO,
                    number: 1,
                    gas_prices: GasPrices::default(),
                    timestamp: 123456,
                    ..Default::default()
                },
                body: vec![dummy_tx.clone()],
            }
            .seal(),
        };

        let genesis = Genesis::default();

        {
            let blockchain = Blockchain::new_with_db(&db_path, &genesis)
                .expect("Failed to create db-backed blockchain storage");

            let provider = blockchain.provider_mut().expect("failed to get mutable provider");

            provider
                .insert_block_with_states_and_receipts(
                    dummy_block.clone(),
                    StateUpdatesWithDeclaredClasses::default(),
                    vec![Receipt::Invoke(InvokeTxReceipt {
                        revert_error: None,
                        events: Vec::new(),
                        messages_sent: Vec::new(),
                        execution_resources: Default::default(),
                        fee: TxFeeInfo {
                            gas_price: 0,
                            overall_fee: 0,
                            gas_consumed: 0,
                            unit: PriceUnit::Wei,
                        },
                    })],
                    vec![TxExecInfo::default()],
                )
                .unwrap();

            // assert genesis state is correct

            let state = provider.latest().expect("failed to get latest state");

            let actual_udc_class_hash =
                state.class_hash_of_contract(DEFAULT_UDC_ADDRESS).unwrap().unwrap();
            let actual_udc_class = state.class(actual_udc_class_hash).unwrap().unwrap();

            let actual_fee_token_class_hash =
                state.class_hash_of_contract(DEFAULT_FEE_TOKEN_ADDRESS).unwrap().unwrap();
            let actual_fee_token_class = state.class(actual_fee_token_class_hash).unwrap().unwrap();

            assert_eq!(actual_udc_class_hash, DEFAULT_LEGACY_UDC_CLASS_HASH);
            assert_eq!(actual_udc_class, DEFAULT_LEGACY_UDC_CASM.clone());

            assert_eq!(actual_fee_token_class_hash, DEFAULT_LEGACY_ERC20_CONTRACT_CLASS_HASH);
            assert_eq!(actual_fee_token_class, DEFAULT_LEGACY_ERC20_CONTRACT_CASM.clone());
        }

        // re open the db and assert the state is the same and not overwritten

        {
            let blockchain = Blockchain::new_with_db(&db_path, &genesis)
                .expect("Failed to create db-backed blockchain storage");

            // assert genesis state is correct
            let provider = blockchain.provider().expect("failed to get provider");

            let state = provider.latest().expect("failed to get latest state");

            let actual_udc_class_hash =
                state.class_hash_of_contract(DEFAULT_UDC_ADDRESS).unwrap().unwrap();
            let actual_udc_class = state.class(actual_udc_class_hash).unwrap().unwrap();

            let actual_fee_token_class_hash =
                state.class_hash_of_contract(DEFAULT_FEE_TOKEN_ADDRESS).unwrap().unwrap();
            let actual_fee_token_class = state.class(actual_fee_token_class_hash).unwrap().unwrap();

            assert_eq!(actual_udc_class_hash, DEFAULT_LEGACY_UDC_CLASS_HASH);
            assert_eq!(actual_udc_class, DEFAULT_LEGACY_UDC_CASM.clone());

            assert_eq!(actual_fee_token_class_hash, DEFAULT_LEGACY_ERC20_CONTRACT_CLASS_HASH);
            assert_eq!(actual_fee_token_class, DEFAULT_LEGACY_ERC20_CONTRACT_CASM.clone());

            let block_number = provider.latest_number().unwrap();
            let block_hash = provider.latest_hash().unwrap();
            let block = provider.block_by_hash(dummy_block.block.header.hash).unwrap().unwrap();

            let tx = provider.transaction_by_hash(dummy_tx.hash).unwrap().unwrap();
            let tx_exec = provider.transaction_execution(dummy_tx.hash).unwrap().unwrap();

            assert_eq!(block_hash, dummy_block.block.header.hash);
            assert_eq!(block_number, dummy_block.block.header.header.number);
            assert_eq!(block, dummy_block.block.unseal());
            assert_eq!(tx, dummy_tx);
            assert_eq!(tx_exec, TxExecInfo::default());
        }
    }
}
