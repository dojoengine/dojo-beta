use dojo_world::contracts::world::WorldContract;
use dojo_world::contracts::WorldContractReader;
use katana_runner::KatanaRunner;
use starknet::accounts::ConnectedAccount;
use starknet::core::types::{BlockId, BlockTag, FieldElement};

use super::setup;
use crate::utils;

const ACTION_CONTRACT_TAG: &str = "dojo_examples:actions";

#[tokio::test(flavor = "multi_thread")]
async fn get_contract_address_from_world() {
    let sequencer = KatanaRunner::new().expect("Failed to start runner.");

    let world = setup::setup(&sequencer).await.unwrap();

    let contract_address =
        utils::get_contract_address(&world, ACTION_CONTRACT_TAG.to_string()).await.unwrap();

    assert!(contract_address != FieldElement::ZERO);
}

#[tokio::test(flavor = "multi_thread")]
async fn get_contract_address_from_string() {
    let sequencer = KatanaRunner::new().expect("Failed to start runner.");

    let account = sequencer.account(0);
    let world = WorldContract::new(FieldElement::ZERO, account);

    let contract_address = utils::get_contract_address(&world, "0x1234".to_string()).await.unwrap();

    assert_eq!(contract_address, FieldElement::from_hex_be("0x1234").unwrap());
}

#[tokio::test(flavor = "multi_thread")]
async fn get_contract_address_from_world_with_world_reader() {
    let sequencer = KatanaRunner::new().expect("Failed to start runner.");

    let world = setup::setup(&sequencer).await.unwrap();
    let account = sequencer.account(0);
    let provider = account.provider();
    let world_reader = WorldContractReader::new(world.address, provider);

    let contract_address =
        utils::get_contract_address_from_reader(&world_reader, ACTION_CONTRACT_TAG.to_string())
            .await
            .unwrap();

    assert!(contract_address != FieldElement::ZERO);
}

#[tokio::test(flavor = "multi_thread")]
async fn get_contract_address_from_string_with_world_reader() {
    let sequencer = KatanaRunner::new().expect("Failed to start runner.");

    let account = sequencer.account(0);
    let provider = account.provider();
    let world_reader = WorldContractReader::new(FieldElement::ZERO, provider);

    let contract_address =
        utils::get_contract_address_from_reader(&world_reader, "0x1234".to_string()).await.unwrap();

    assert_eq!(contract_address, FieldElement::from_hex_be("0x1234").unwrap());
}

#[test]
fn parse_block_id_bad_hash() {
    assert!(utils::parse_block_id("0xBadHash".to_string()).is_err());
}

#[test]
fn parse_block_id_bad_string() {
    assert!(utils::parse_block_id("BadString".to_string()).is_err());
}

#[test]
fn parse_block_id_hash() {
    assert!(
        utils::parse_block_id("0x1234".to_string()).unwrap()
            == BlockId::Hash(FieldElement::from_hex_be("0x1234").unwrap())
    );
}

#[test]
fn parse_block_id_pending() {
    assert!(
        utils::parse_block_id("pending".to_string()).unwrap() == BlockId::Tag(BlockTag::Pending)
    );
}

#[test]
fn parse_block_id_latest() {
    assert!(utils::parse_block_id("latest".to_string()).unwrap() == BlockId::Tag(BlockTag::Latest));
}

#[test]
fn parse_block_id_number() {
    assert!(utils::parse_block_id("42".to_string()).unwrap() == BlockId::Number(42));
}
