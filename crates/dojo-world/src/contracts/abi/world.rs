// AUTOGENERATED FILE, DO NOT EDIT.
// To generate the bindings, please run `cargo run --bin dojo-world-abigen` instead.
use cainome::rs::abigen;

abigen!(
    WorldContract,
    r#"[{"type":"impl","name":"World","interface_name":"dojo::world::IWorld"},{"type":"struct","name":"core::array::Span::<core::felt252>","members":[{"name":"snapshot","type":"@core::array::Array::<core::felt252>"}]},{"type":"struct","name":"core::array::Span::<core::integer::u8>","members":[{"name":"snapshot","type":"@core::array::Array::<core::integer::u8>"}]},{"type":"enum","name":"core::option::Option::<core::felt252>","variants":[{"name":"Some","type":"core::felt252"},{"name":"None","type":"()"}]},{"type":"struct","name":"core::array::Span::<core::array::Span::<core::felt252>>","members":[{"name":"snapshot","type":"@core::array::Array::<core::array::Span::<core::felt252>>"}]},{"type":"enum","name":"core::bool","variants":[{"name":"False","type":"()"},{"name":"True","type":"()"}]},{"type":"interface","name":"dojo::world::IWorld","items":[{"type":"function","name":"metadata_uri","inputs":[{"name":"resource","type":"core::felt252"}],"outputs":[{"type":"core::array::Span::<core::felt252>"}],"state_mutability":"view"},{"type":"function","name":"set_metadata_uri","inputs":[{"name":"resource","type":"core::felt252"},{"name":"uri","type":"core::array::Span::<core::felt252>"}],"outputs":[],"state_mutability":"external"},{"type":"function","name":"model","inputs":[{"name":"name","type":"core::felt252"}],"outputs":[{"type":"core::starknet::class_hash::ClassHash"}],"state_mutability":"view"},{"type":"function","name":"register_model","inputs":[{"name":"class_hash","type":"core::starknet::class_hash::ClassHash"}],"outputs":[],"state_mutability":"external"},{"type":"function","name":"deploy_contract","inputs":[{"name":"salt","type":"core::felt252"},{"name":"class_hash","type":"core::starknet::class_hash::ClassHash"}],"outputs":[{"type":"core::starknet::contract_address::ContractAddress"}],"state_mutability":"external"},{"type":"function","name":"upgrade_contract","inputs":[{"name":"address","type":"core::starknet::contract_address::ContractAddress"},{"name":"class_hash","type":"core::starknet::class_hash::ClassHash"}],"outputs":[{"type":"core::starknet::class_hash::ClassHash"}],"state_mutability":"external"},{"type":"function","name":"uuid","inputs":[],"outputs":[{"type":"core::integer::u32"}],"state_mutability":"external"},{"type":"function","name":"emit","inputs":[{"name":"keys","type":"core::array::Array::<core::felt252>"},{"name":"values","type":"core::array::Span::<core::felt252>"}],"outputs":[],"state_mutability":"view"},{"type":"function","name":"entity","inputs":[{"name":"model","type":"core::felt252"},{"name":"keys","type":"core::array::Span::<core::felt252>"},{"name":"offset","type":"core::integer::u8"},{"name":"length","type":"core::integer::u32"},{"name":"layout","type":"core::array::Span::<core::integer::u8>"}],"outputs":[{"type":"core::array::Span::<core::felt252>"}],"state_mutability":"view"},{"type":"function","name":"set_entity","inputs":[{"name":"model","type":"core::felt252"},{"name":"keys","type":"core::array::Span::<core::felt252>"},{"name":"offset","type":"core::integer::u8"},{"name":"values","type":"core::array::Span::<core::felt252>"},{"name":"layout","type":"core::array::Span::<core::integer::u8>"}],"outputs":[],"state_mutability":"external"},{"type":"function","name":"entities","inputs":[{"name":"model","type":"core::felt252"},{"name":"index","type":"core::option::Option::<core::felt252>"},{"name":"values","type":"core::array::Span::<core::felt252>"},{"name":"values_length","type":"core::integer::u32"},{"name":"values_layout","type":"core::array::Span::<core::integer::u8>"}],"outputs":[{"type":"(core::array::Span::<core::felt252>, core::array::Span::<core::array::Span::<core::felt252>>)"}],"state_mutability":"view"},{"type":"function","name":"entity_ids","inputs":[{"name":"model","type":"core::felt252"}],"outputs":[{"type":"core::array::Span::<core::felt252>"}],"state_mutability":"view"},{"type":"function","name":"set_executor","inputs":[{"name":"contract_address","type":"core::starknet::contract_address::ContractAddress"}],"outputs":[],"state_mutability":"external"},{"type":"function","name":"executor","inputs":[],"outputs":[{"type":"core::starknet::contract_address::ContractAddress"}],"state_mutability":"view"},{"type":"function","name":"base","inputs":[],"outputs":[{"type":"core::starknet::class_hash::ClassHash"}],"state_mutability":"view"},{"type":"function","name":"delete_entity","inputs":[{"name":"model","type":"core::felt252"},{"name":"keys","type":"core::array::Span::<core::felt252>"},{"name":"layout","type":"core::array::Span::<core::integer::u8>"}],"outputs":[],"state_mutability":"external"},{"type":"function","name":"is_owner","inputs":[{"name":"address","type":"core::starknet::contract_address::ContractAddress"},{"name":"resource","type":"core::felt252"}],"outputs":[{"type":"core::bool"}],"state_mutability":"view"},{"type":"function","name":"grant_owner","inputs":[{"name":"address","type":"core::starknet::contract_address::ContractAddress"},{"name":"resource","type":"core::felt252"}],"outputs":[],"state_mutability":"external"},{"type":"function","name":"revoke_owner","inputs":[{"name":"address","type":"core::starknet::contract_address::ContractAddress"},{"name":"resource","type":"core::felt252"}],"outputs":[],"state_mutability":"external"},{"type":"function","name":"is_writer","inputs":[{"name":"model","type":"core::felt252"},{"name":"system","type":"core::starknet::contract_address::ContractAddress"}],"outputs":[{"type":"core::bool"}],"state_mutability":"view"},{"type":"function","name":"grant_writer","inputs":[{"name":"model","type":"core::felt252"},{"name":"system","type":"core::starknet::contract_address::ContractAddress"}],"outputs":[],"state_mutability":"external"},{"type":"function","name":"revoke_writer","inputs":[{"name":"model","type":"core::felt252"},{"name":"system","type":"core::starknet::contract_address::ContractAddress"}],"outputs":[],"state_mutability":"external"}]},{"type":"impl","name":"UpgradeableWorld","interface_name":"dojo::world::IUpgradeableWorld"},{"type":"interface","name":"dojo::world::IUpgradeableWorld","items":[{"type":"function","name":"upgrade","inputs":[{"name":"new_class_hash","type":"core::starknet::class_hash::ClassHash"}],"outputs":[],"state_mutability":"external"}]},{"type":"constructor","name":"constructor","inputs":[{"name":"executor","type":"core::starknet::contract_address::ContractAddress"},{"name":"contract_base","type":"core::starknet::class_hash::ClassHash"}]},{"type":"event","name":"dojo::world::world::WorldSpawned","kind":"struct","members":[{"name":"address","type":"core::starknet::contract_address::ContractAddress","kind":"data"},{"name":"creator","type":"core::starknet::contract_address::ContractAddress","kind":"data"}]},{"type":"event","name":"dojo::world::world::ContractDeployed","kind":"struct","members":[{"name":"salt","type":"core::felt252","kind":"data"},{"name":"class_hash","type":"core::starknet::class_hash::ClassHash","kind":"data"},{"name":"address","type":"core::starknet::contract_address::ContractAddress","kind":"data"}]},{"type":"event","name":"dojo::world::world::ContractUpgraded","kind":"struct","members":[{"name":"class_hash","type":"core::starknet::class_hash::ClassHash","kind":"data"},{"name":"address","type":"core::starknet::contract_address::ContractAddress","kind":"data"}]},{"type":"event","name":"dojo::world::world::WorldUpgraded","kind":"struct","members":[{"name":"class_hash","type":"core::starknet::class_hash::ClassHash","kind":"data"}]},{"type":"event","name":"dojo::world::world::MetadataUpdate","kind":"struct","members":[{"name":"resource","type":"core::felt252","kind":"data"},{"name":"uri","type":"core::array::Span::<core::felt252>","kind":"data"}]},{"type":"event","name":"dojo::world::world::ModelRegistered","kind":"struct","members":[{"name":"name","type":"core::felt252","kind":"data"},{"name":"class_hash","type":"core::starknet::class_hash::ClassHash","kind":"data"},{"name":"prev_class_hash","type":"core::starknet::class_hash::ClassHash","kind":"data"}]},{"type":"event","name":"dojo::world::world::StoreSetRecord","kind":"struct","members":[{"name":"table","type":"core::felt252","kind":"data"},{"name":"keys","type":"core::array::Span::<core::felt252>","kind":"data"},{"name":"offset","type":"core::integer::u8","kind":"data"},{"name":"values","type":"core::array::Span::<core::felt252>","kind":"data"}]},{"type":"event","name":"dojo::world::world::StoreDelRecord","kind":"struct","members":[{"name":"table","type":"core::felt252","kind":"data"},{"name":"keys","type":"core::array::Span::<core::felt252>","kind":"data"}]},{"type":"event","name":"dojo::world::world::WriterUpdated","kind":"struct","members":[{"name":"model","type":"core::felt252","kind":"data"},{"name":"system","type":"core::starknet::contract_address::ContractAddress","kind":"data"},{"name":"value","type":"core::bool","kind":"data"}]},{"type":"event","name":"dojo::world::world::OwnerUpdated","kind":"struct","members":[{"name":"address","type":"core::starknet::contract_address::ContractAddress","kind":"data"},{"name":"resource","type":"core::felt252","kind":"data"},{"name":"value","type":"core::bool","kind":"data"}]},{"type":"event","name":"dojo::world::world::ExecutorUpdated","kind":"struct","members":[{"name":"address","type":"core::starknet::contract_address::ContractAddress","kind":"data"},{"name":"prev_address","type":"core::starknet::contract_address::ContractAddress","kind":"data"}]},{"type":"event","name":"dojo::world::world::Event","kind":"enum","variants":[{"name":"WorldSpawned","type":"dojo::world::world::WorldSpawned","kind":"nested"},{"name":"ContractDeployed","type":"dojo::world::world::ContractDeployed","kind":"nested"},{"name":"ContractUpgraded","type":"dojo::world::world::ContractUpgraded","kind":"nested"},{"name":"WorldUpgraded","type":"dojo::world::world::WorldUpgraded","kind":"nested"},{"name":"MetadataUpdate","type":"dojo::world::world::MetadataUpdate","kind":"nested"},{"name":"ModelRegistered","type":"dojo::world::world::ModelRegistered","kind":"nested"},{"name":"StoreSetRecord","type":"dojo::world::world::StoreSetRecord","kind":"nested"},{"name":"StoreDelRecord","type":"dojo::world::world::StoreDelRecord","kind":"nested"},{"name":"WriterUpdated","type":"dojo::world::world::WriterUpdated","kind":"nested"},{"name":"OwnerUpdated","type":"dojo::world::world::OwnerUpdated","kind":"nested"},{"name":"ExecutorUpdated","type":"dojo::world::world::ExecutorUpdated","kind":"nested"}]}]"#
);
