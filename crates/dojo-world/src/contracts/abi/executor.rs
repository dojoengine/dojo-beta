// AUTOGENERATED FILE, DO NOT EDIT.
// To generate the bindings, please run `cargo run --bin dojo-world-abigen` instead.
use cainome::rs::abigen;

abigen!(
    ExecutorContract,
    r#"[{"type":"impl","name":"Executor","interface_name":"dojo::executor::IExecutor"},{"type":"struct","name":"core::array::Span::<core::felt252>","members":[{"name":"snapshot","type":"@core::array::Array::<core::felt252>"}]},{"type":"interface","name":"dojo::executor::IExecutor","items":[{"type":"function","name":"call","inputs":[{"name":"class_hash","type":"core::starknet::class_hash::ClassHash"},{"name":"entrypoint","type":"core::felt252"},{"name":"calldata","type":"core::array::Span::<core::felt252>"}],"outputs":[{"type":"core::array::Span::<core::felt252>"}],"state_mutability":"view"}]},{"type":"event","name":"dojo::executor::executor::Event","kind":"enum","variants":[]}]"#
);
