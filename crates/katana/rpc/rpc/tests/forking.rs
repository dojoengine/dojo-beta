use anyhow::{Context, Result};
use assert_matches::assert_matches;
use cainome::rs::abigen_legacy;
use dojo_test_utils::sequencer::{get_default_test_config, TestSequencer};
use jsonrpsee::http_client::HttpClientBuilder;
use katana_node::config::fork::ForkingConfig;
use katana_node::config::SequencingConfig;
use katana_primitives::block::{BlockHashOrNumber, BlockIdOrTag, BlockNumber};
use katana_primitives::chain::NamedChainId;
use katana_primitives::genesis::constant::DEFAULT_ETH_FEE_TOKEN_ADDRESS;
use katana_primitives::transaction::TxHash;
use katana_primitives::{felt, Felt};
use katana_rpc_api::dev::DevApiClient;
use starknet::core::types::{MaybePendingBlockWithTxs, StarknetError};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Provider, ProviderError};
use url::Url;

mod common;

const SEPOLIA_CHAIN_ID: Felt = NamedChainId::SN_SEPOLIA;
const SEPOLIA_URL: &str = "https://api.cartridge.gg/x/starknet/sepolia";
const FORK_BLOCK_NUMBER: BlockNumber = 268_471;

fn forking_cfg() -> ForkingConfig {
    ForkingConfig {
        url: Url::parse(SEPOLIA_URL).unwrap(),
        block: Some(BlockHashOrNumber::Num(FORK_BLOCK_NUMBER)),
    }
}

fn provider(url: Url) -> JsonRpcClient<HttpTransport> {
    JsonRpcClient::new(HttpTransport::new(url))
}

type LocalTestVector = Vec<(BlockNumber, TxHash)>;

/// A helper function for setting a test environment, forked from the SN_SEPOLIA chain.
/// This function will forked Sepolia at block [`FORK_BLOCK_NUMBER`] and create 10 blocks, each has
/// a single transaction.
///
/// The returned [`TestVector`] is a list of all the locally created blocks and transactions.
async fn setup_test() -> (TestSequencer, impl Provider, LocalTestVector) {
    let mut config = get_default_test_config(SequencingConfig::default());
    config.forking = Some(forking_cfg());

    let sequencer = TestSequencer::start(config).await;
    let provider = provider(sequencer.url());

    let mut txs_vector: LocalTestVector = Vec::new();

    {
        // create some emtpy blocks and dummy transactions
        abigen_legacy!(FeeToken, "crates/katana/rpc/rpc/tests/test_data/erc20.json");
        let contract = FeeToken::new(DEFAULT_ETH_FEE_TOKEN_ADDRESS.into(), sequencer.account());
        let client = HttpClientBuilder::default().build(sequencer.url()).unwrap();

        for i in 0..10 {
            let amount = Uint256 { low: Felt::ONE, high: Felt::ZERO };
            let res = contract.transfer(&Felt::ONE, &amount).send().await.unwrap();
            client.generate_block().await.expect("failed to create block");
            txs_vector.push((FORK_BLOCK_NUMBER + i, res.transaction_hash));
        }
    }

    (sequencer, provider, txs_vector)
}

#[tokio::test]
async fn can_fork() -> Result<()> {
    let (_sequencer, provider, _) = setup_test().await;

    let block = provider.block_number().await?;
    let chain = provider.chain_id().await?;

    assert_eq!(chain, SEPOLIA_CHAIN_ID);
    assert_eq!(block, FORK_BLOCK_NUMBER + 10);

    Ok(())
}

async fn assert_get_block_methods(provider: &impl Provider, num: BlockNumber) -> Result<()> {
    let id = BlockIdOrTag::Number(num);

    let block = provider.get_block_with_txs(id).await?;
    assert_matches!(block, MaybePendingBlockWithTxs::Block(b) if b.block_number == num);

    let block = provider.get_block_with_receipts(id).await?;
    assert_matches!(block, starknet::core::types::MaybePendingBlockWithReceipts::Block(b) if b.block_number == num);

    let block = provider.get_block_with_tx_hashes(id).await?;
    assert_matches!(block, starknet::core::types::MaybePendingBlockWithTxHashes::Block(b) if b.block_number == num);

    let result = provider.get_block_transaction_count(id).await;
    assert!(result.is_ok());

    let state = provider.get_state_update(id).await?;
    assert_matches!(state, starknet::core::types::MaybePendingStateUpdate::Update(_));

    Ok(())
}

#[tokio::test]
async fn forked_blocks() -> Result<()> {
    let (_sequencer, provider, _) = setup_test().await;

    let block_num = FORK_BLOCK_NUMBER;
    assert_get_block_methods(&provider, block_num).await?;

    let block_num = FORK_BLOCK_NUMBER - 5;
    assert_get_block_methods(&provider, block_num).await?;

    let block_num = FORK_BLOCK_NUMBER + 5;
    assert_get_block_methods(&provider, block_num).await?;

    // -----------------------------------------------------------------------
    // Get block that doesn't exist on the both the forked and local chain

    let id = BlockIdOrTag::Number(BlockNumber::MAX);
    let result = provider.get_block_with_txs(id).await.unwrap_err();
    assert_provider_starknet_err!(result, StarknetError::BlockNotFound);

    let result = provider.get_block_with_receipts(id).await.unwrap_err();
    assert_provider_starknet_err!(result, StarknetError::BlockNotFound);

    let result = provider.get_block_with_tx_hashes(id).await.unwrap_err();
    assert_provider_starknet_err!(result, StarknetError::BlockNotFound);

    let result = provider.get_block_transaction_count(id).await.unwrap_err();
    assert_provider_starknet_err!(result, StarknetError::BlockNotFound);

    let result = provider.get_state_update(id).await.unwrap_err();
    assert_provider_starknet_err!(result, StarknetError::BlockNotFound);

    Ok(())
}

async fn assert_get_transaction_methods(provider: &impl Provider, tx_hash: TxHash) -> Result<()> {
    let tx = provider
        .get_transaction_by_hash(tx_hash)
        .await
        .with_context(|| format!("failed to get tx {tx_hash:#x}"))?;
    assert_eq!(*tx.transaction_hash(), tx_hash);

    let tx = provider
        .get_transaction_receipt(tx_hash)
        .await
        .with_context(|| format!("failed to get receipt {tx_hash:#x}"))?;
    assert_eq!(*tx.receipt.transaction_hash(), tx_hash);

    let result = provider.get_transaction_status(tx_hash).await;
    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn forked_transactions() -> Result<()> {
    let (_sequencer, provider, local_only_data) = setup_test().await;

    // -----------------------------------------------------------------------
    // Get txs before the forked block.

    // https://sepolia.voyager.online/tx/0x81207d4244596678e186f6ab9c833fe40a4b35291e8a90b9a163f7f643df9f
    // Transaction in block num FORK_BLOCK_NUMBER - 1
    let tx_hash = felt!("0x81207d4244596678e186f6ab9c833fe40a4b35291e8a90b9a163f7f643df9f");
    assert_get_transaction_methods(&provider, tx_hash).await?;

    // https://sepolia.voyager.online/tx/0x1b18d62544d4ef749befadabcec019d83218d3905abd321b4c1b1fc948d5710
    // Transaction in block num FORK_BLOCK_NUMBER - 2
    let tx_hash = felt!("0x1b18d62544d4ef749befadabcec019d83218d3905abd321b4c1b1fc948d5710");
    assert_get_transaction_methods(&provider, tx_hash).await?;

    // -----------------------------------------------------------------------
    // Get the locally created transactions.

    for (_, tx_hash) in local_only_data {
        assert_get_transaction_methods(&provider, tx_hash).await?;
    }

    // -----------------------------------------------------------------------
    // Get a tx that exists in the forked chain but is included in a block past the forked block.

    // https://sepolia.voyager.online/block/0x335a605f2c91873f8f830a6e5285e704caec18503ca28c18485ea6f682eb65e
    // transaction in block num 268,474 (FORK_BLOCK_NUMBER + 3)
    let tx_hash = felt!("0x335a605f2c91873f8f830a6e5285e704caec18503ca28c18485ea6f682eb65e");
    let result = provider.get_transaction_by_hash(tx_hash).await.unwrap_err();
    assert_provider_starknet_err!(result, StarknetError::TransactionHashNotFound);

    let result = provider.get_transaction_receipt(tx_hash).await.unwrap_err();
    assert_provider_starknet_err!(result, StarknetError::TransactionHashNotFound);

    let result = provider.get_transaction_status(tx_hash).await.unwrap_err();
    assert_provider_starknet_err!(result, StarknetError::TransactionHashNotFound);

    Ok(())
}
