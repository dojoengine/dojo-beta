use camino::Utf8PathBuf;
use dojo_test_utils::sequencer::{
    get_default_test_starknet_config, SequencerConfig, TestSequencer,
};
use dojo_types::component::Member;
use starknet::accounts::ConnectedAccount;
use starknet::core::types::{BlockId, BlockTag, FieldElement};

use crate::contract::world::test::deploy_world;
use crate::contract::world::WorldContractReader;

#[tokio::test(flavor = "multi_thread")]
async fn test_component() {
    let sequencer =
        TestSequencer::start(SequencerConfig::default(), get_default_test_starknet_config()).await;
    let account = sequencer.account();
    let provider = account.provider();
    let (world_address, _) = deploy_world(
        &sequencer,
        Utf8PathBuf::from_path_buf("../../../examples/ecs/target/dev".into()).unwrap(),
    )
    .await;

    let block_id = BlockId::Tag(BlockTag::Latest);
    let world = WorldContractReader::new(world_address, provider);
    let component = world.component("Position", block_id).await.unwrap();

    assert_eq!(
        component.class_hash(),
        FieldElement::from_hex_be(
            "0x033bfc7a386f2aa93167ff44561470d1160249f4b0afe7b667cc4fb8ae617ace"
        )
        .unwrap()
    );

    let members = component.schema(block_id).await.unwrap();

    assert_eq!(
        members,
        vec![
            Member { name: "player".into(), ty: "ContractAddress".into(), key: true },
            Member { name: "x".into(), ty: "u32".into(), key: false },
            Member { name: "y".into(), ty: "u32".into(), key: false }
        ]
    )
}
