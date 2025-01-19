use criterion::{criterion_group, criterion_main, Criterion};
use katana_core::backend::UncommittedBlock;
use std::collections::BTreeMap;

use katana_primitives::{
    address,
    block::{GasPrices, PartialHeader},
    da::L1DataAvailabilityMode,
    felt,
    receipt::ReceiptWithTxHash,
    state::StateUpdates,
    transaction::TxWithHash,
    version::CURRENT_STARKNET_VERSION,
    ContractAddress,
};
use katana_provider::providers::db::DbProvider;

pub fn commit() {
    let provider = DbProvider::new_ephemeral();

    let gas_prices = GasPrices { eth: 100 * u128::pow(10, 9), strk: 100 * u128::pow(10, 9) };
    let sequencer_address = ContractAddress(1u64.into());

    let header = PartialHeader {
        protocol_version: CURRENT_STARKNET_VERSION,
        number: 1,
        timestamp: 100,
        sequencer_address,
        parent_hash: 123u64.into(),
        l1_gas_prices: gas_prices.clone(),
        l1_data_gas_prices: gas_prices.clone(),
        l1_da_mode: L1DataAvailabilityMode::Calldata,
    };

    let transactions: Vec<TxWithHash> = vec![];
    let receipts: [ReceiptWithTxHash; 0] = [];

    let address_1 = address!("1337");
    let address_2 = address!("80085");

    let class_hash_1 = felt!("11");
    let compiled_class_hash_1 = felt!("1000");

    let state_updates = StateUpdates {
        nonce_updates: BTreeMap::from([(address_1, 1u8.into()), (address_2, 1u8.into())]),
        storage_updates: BTreeMap::from([
            (address_1, BTreeMap::from([(1u8.into(), 100u32.into()), (2u8.into(), 101u32.into())])),
            (address_2, BTreeMap::from([(1u8.into(), 200u32.into()), (2u8.into(), 201u32.into())])),
        ]),
        declared_classes: BTreeMap::from([(class_hash_1, compiled_class_hash_1)]),
        deployed_contracts: BTreeMap::from([(address_1, class_hash_1), (address_2, class_hash_1)]),
        ..Default::default()
    };

    let block = UncommittedBlock::new(header, transactions, &receipts, &state_updates, provider);

    let _sealed_block = block.commit();
}

pub fn commit_benchmark(c: &mut Criterion) {
    c.bench_function("commit", |b| b.iter(|| commit()));
}

criterion_group!(benches, commit_benchmark);
criterion_main!(benches);
