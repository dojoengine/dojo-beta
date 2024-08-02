use starknet::contract_address_const;

use dojo::model::Model;
use dojo::utils::{bytearray_hash, entity_id_from_keys};
use dojo::world::{IWorldDispatcher, IWorldDispatcherTrait, world};

use dojo::tests::helpers::{deploy_world, Foo, foo};

#[test]
fn test_owner() {
    let world = deploy_world();
    world.register_model(foo::TEST_CLASS_HASH.try_into().unwrap());
    let foo_selector = Model::<Foo>::selector();

    let alice = starknet::contract_address_const::<0xa11ce>();
    let bob = starknet::contract_address_const::<0xb0b>();

    assert(!world.is_owner(0, alice), 'should not be owner');
    assert(!world.is_owner(foo_selector, bob), 'should not be owner');

    world.grant_owner(0, alice);
    assert(world.is_owner(0, alice), 'should be owner');

    world.grant_owner(foo_selector, bob);
    assert(world.is_owner(foo_selector, bob), 'should be owner');

    world.revoke_owner(0, alice);
    assert(!world.is_owner(0, alice), 'should not be owner');

    world.revoke_owner(foo_selector, bob);
    assert(!world.is_owner(foo_selector, bob), 'should not be owner');
}


#[test]
#[should_panic(expected: ('resource not registered', 'ENTRYPOINT_FAILED'))]
fn test_grant_owner_not_registered_resource() {
    let world = deploy_world();

    // 42 is not a registered resource ID
    world.grant_owner(42, 69.try_into().unwrap());
}

#[test]
#[should_panic(expected: ('not a direct call', 'ENTRYPOINT_FAILED'))]
fn test_grant_owner_through_malicious_contract() {
    let world = deploy_world();
    world.register_model(foo::TEST_CLASS_HASH.try_into().unwrap());
    let foo_selector = Model::<Foo>::selector();

    let alice = starknet::contract_address_const::<0xa11ce>();
    let bob = starknet::contract_address_const::<0xb0b>();
    let malicious_contract = starknet::contract_address_const::<0xdead>();

    world.grant_owner(foo_selector, alice);

    starknet::testing::set_account_contract_address(alice);
    starknet::testing::set_contract_address(malicious_contract);

    world.grant_owner(foo_selector, bob);
}

#[test]
#[should_panic(expected: ('not owner', 'ENTRYPOINT_FAILED'))]
fn test_grant_owner_fails_for_non_owner() {
    let world = deploy_world();
    world.register_model(foo::TEST_CLASS_HASH.try_into().unwrap());
    let foo_selector = Model::<Foo>::selector();

    let alice = starknet::contract_address_const::<0xa11ce>();
    let bob = starknet::contract_address_const::<0xb0b>();

    starknet::testing::set_account_contract_address(alice);
    starknet::testing::set_contract_address(alice);

    world.grant_owner(foo_selector, bob);
}

#[test]
#[should_panic(expected: ('not a direct call', 'ENTRYPOINT_FAILED'))]
fn test_revoke_owner_through_malicious_contract() {
    let world = deploy_world();
    world.register_model(foo::TEST_CLASS_HASH.try_into().unwrap());
    let foo_selector = Model::<Foo>::selector();

    let alice = starknet::contract_address_const::<0xa11ce>();
    let bob = starknet::contract_address_const::<0xb0b>();
    let malicious_contract = starknet::contract_address_const::<0xdead>();

    world.grant_owner(foo_selector, alice);
    world.grant_owner(foo_selector, bob);

    starknet::testing::set_account_contract_address(alice);
    starknet::testing::set_contract_address(malicious_contract);

    world.revoke_owner(foo_selector, bob);
}

#[test]
#[should_panic(expected: ('not owner', 'ENTRYPOINT_FAILED'))]
fn test_revoke_owner_fails_for_non_owner() {
    let world = deploy_world();
    world.register_model(foo::TEST_CLASS_HASH.try_into().unwrap());
    let foo_selector = Model::<Foo>::selector();

    let alice = starknet::contract_address_const::<0xa11ce>();
    let bob = starknet::contract_address_const::<0xb0b>();

    world.grant_owner(foo_selector, bob);

    starknet::testing::set_account_contract_address(alice);
    starknet::testing::set_contract_address(alice);

    world.revoke_owner(foo_selector, bob);
}

#[test]
#[available_gas(6000000)]
fn test_writer() {
    let world = deploy_world();
    world.register_model(foo::TEST_CLASS_HASH.try_into().unwrap());
    let foo_selector = Model::<Foo>::selector();

    assert(!world.is_writer(foo_selector, 69.try_into().unwrap()), 'should not be writer');

    world.grant_writer(foo_selector, 69.try_into().unwrap());
    assert(world.is_writer(foo_selector, 69.try_into().unwrap()), 'should be writer');

    world.revoke_writer(foo_selector, 69.try_into().unwrap());
    assert(!world.is_writer(foo_selector, 69.try_into().unwrap()), 'should not be writer');
}

#[test]
#[should_panic(expected: ('resource not registered', 'ENTRYPOINT_FAILED'))]
fn test_writer_not_registered_resource() {
    let world = deploy_world();

    // 42 is not a registered resource ID
    world.grant_writer(42, 69.try_into().unwrap());
}

#[test]
#[should_panic(expected: ('not a direct call', 'ENTRYPOINT_FAILED'))]
fn test_grant_writer_through_malicious_contract() {
    let world = deploy_world();
    world.register_model(foo::TEST_CLASS_HASH.try_into().unwrap());
    let foo_selector = Model::<Foo>::selector();

    let alice = starknet::contract_address_const::<0xa11ce>();
    let bob = starknet::contract_address_const::<0xb0b>();
    let malicious_contract = starknet::contract_address_const::<0xdead>();

    world.grant_owner(foo_selector, alice);

    starknet::testing::set_account_contract_address(alice);
    starknet::testing::set_contract_address(malicious_contract);

    world.grant_writer(foo_selector, bob);
}

#[test]
#[should_panic(expected: ('not owner', 'ENTRYPOINT_FAILED'))]
fn test_grant_writer_fails_for_non_owner() {
    let world = deploy_world();
    world.register_model(foo::TEST_CLASS_HASH.try_into().unwrap());
    let foo_selector = Model::<Foo>::selector();

    let alice = starknet::contract_address_const::<0xa11ce>();
    let bob = starknet::contract_address_const::<0xb0b>();

    starknet::testing::set_account_contract_address(alice);
    starknet::testing::set_contract_address(alice);

    world.grant_writer(foo_selector, bob);
}

#[test]
#[should_panic(expected: ('not a direct call', 'ENTRYPOINT_FAILED'))]
fn test_revoke_writer_through_malicious_contract() {
    let world = deploy_world();
    world.register_model(foo::TEST_CLASS_HASH.try_into().unwrap());
    let foo_selector = Model::<Foo>::selector();

    let alice = starknet::contract_address_const::<0xa11ce>();
    let bob = starknet::contract_address_const::<0xb0b>();
    let malicious_contract = starknet::contract_address_const::<0xdead>();

    world.grant_owner(foo_selector, alice);
    world.grant_writer(foo_selector, bob);

    starknet::testing::set_account_contract_address(alice);
    starknet::testing::set_contract_address(malicious_contract);

    world.revoke_writer(foo_selector, bob);
}

#[test]
#[should_panic(expected: ('not owner', 'ENTRYPOINT_FAILED'))]
fn test_revoke_writer_fails_for_non_owner() {
    let world = deploy_world();
    world.register_model(foo::TEST_CLASS_HASH.try_into().unwrap());
    let foo_selector = Model::<Foo>::selector();

    let alice = starknet::contract_address_const::<0xa11ce>();
    let bob = starknet::contract_address_const::<0xb0b>();

    world.grant_writer(foo_selector, bob);

    starknet::testing::set_account_contract_address(alice);
    starknet::testing::set_contract_address(alice);

    world.revoke_writer(foo_selector, bob);
}
