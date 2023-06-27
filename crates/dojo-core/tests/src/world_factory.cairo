use core::traits::Into;
use core::result::ResultTrait;
use array::{ArrayTrait, SpanTrait};
use clone::Clone;
use option::OptionTrait;
use traits::TryInto;
use serde::Serde;
use debug::PrintTrait;
use starknet::syscalls::deploy_syscall;
use starknet::get_caller_address;
use starknet::class_hash::ClassHash;
use starknet::class_hash::Felt252TryIntoClassHash;
use dojo::interfaces::IWorldFactoryDispatcher;
use dojo::interfaces::IWorldFactoryDispatcherTrait;
use dojo::interfaces::IWorldDispatcher;
use dojo::interfaces::IWorldDispatcherTrait;
use dojo::executor::Executor;
use dojo::world::World;
use dojo::world_factory::WorldFactory;

use dojo::auth::components::AuthRoleComponent;
use dojo::auth::systems::{Route, RouteTrait, GrantAuthRole};
use dojo::test_utils::{build_world_factory_calldata, mock_auth_components_systems};

#[derive(Component, Copy, Drop, Serde)]
struct Foo {
    a: felt252,
    b: u128,
}

#[system]
mod Bar {
    use super::Foo;

    fn execute(foo: Foo) -> Foo {
        foo
    }
}

#[test]
#[available_gas(40000000)]
fn test_constructor() {
    let (auth_components, auth_systems) = mock_auth_components_systems();
    let calldata = build_world_factory_calldata(
        starknet::class_hash_const::<0x420>(),
        starknet::contract_address_const::<0x69>(),
        auth_components.span(),
        auth_systems.span()
    );

    let (factory_address, _) = deploy_syscall(
        WorldFactory::TEST_CLASS_HASH.try_into().unwrap(), 0, calldata, false
    )
        .unwrap();

    let factory = IWorldFactoryDispatcher { contract_address: factory_address };

    assert(factory.world() == starknet::class_hash_const::<0x420>(), 'wrong world class hash');
    assert(
        factory.executor() == starknet::contract_address_const::<0x69>(), 'wrong executor contract'
    );

    assert(
        factory.default_auth_components().len() == auth_components.len(), 'wrong components length'
    );
    assert(factory.default_auth_systems().len() == auth_systems.len(), 'wrong systems length');
}

#[test]
#[available_gas(90000000)]
fn test_spawn_world() {
    // Deploy Executor
    let constructor_calldata = array::ArrayTrait::new();
    let (executor_address, _) = deploy_syscall(
        Executor::TEST_CLASS_HASH.try_into().unwrap(), 0, constructor_calldata.span(), false
    )
        .unwrap();

    // WorldFactory constructor
    let (auth_components, auth_systems) = mock_auth_components_systems();
    let calldata = build_world_factory_calldata(
        World::TEST_CLASS_HASH.try_into().unwrap(),
        executor_address,
        auth_components.span(),
        auth_systems.span()
    );

    let (factory_address, _) = deploy_syscall(
        WorldFactory::TEST_CLASS_HASH.try_into().unwrap(), 0, calldata, false
    )
        .unwrap();

    let factory = IWorldFactoryDispatcher { contract_address: factory_address };

    assert(factory.executor() == executor_address, 'wrong executor address');
    assert(
        factory.world() == World::TEST_CLASS_HASH.try_into().unwrap(), 'wrong world class hash'
    );

    // Prepare components and systems and routes
    let mut systems: Array<ClassHash> = array::ArrayTrait::new();
    systems.append(Bar::TEST_CLASS_HASH.try_into().unwrap());

    let mut components: Array<ClassHash> = array::ArrayTrait::new();
    components.append(FooComponent::TEST_CLASS_HASH.try_into().unwrap());

    let mut routes: Array<Route> = array::ArrayTrait::new();
    routes.append(RouteTrait::new('Bar'.into(), 'FooWriter'.into(), 'Foo'.into(), ));

    // Spawn World from WorldFactory
    let world_address = factory.spawn(components, systems, routes);
    let world = IWorldDispatcher { contract_address: world_address };

    // Check Admin role is set
    let caller = get_caller_address();
    let role = world.entity('AuthRole'.into(), caller.into(), 0, 0);
    assert(*role[0] == World::ADMIN, 'admin role not set');

    // Check AuthRole component and GrantAuthRole system are registered
    let role_hash = world.component('AuthRole'.into());
    assert(
        role_hash == AuthRoleComponent::TEST_CLASS_HASH.try_into().unwrap(),
        'component not registered'
    );

    let grant_role_hash = world.system('GrantAuthRole'.into());
    assert(
        grant_role_hash == GrantAuthRole::TEST_CLASS_HASH.try_into().unwrap(),
        'system not registered'
    );

    // Check Foo component and Bar system are registered
    let foo_hash = world.component('Foo'.into());
    assert(
        foo_hash == FooComponent::TEST_CLASS_HASH.try_into().unwrap(), 'component not registered'
    );

    let bar_hash = world.system('Bar'.into());
    assert(bar_hash == Bar::TEST_CLASS_HASH.try_into().unwrap(), 'system not registered');

    // Check that the auth routes are registered
    let role = world.entity('AuthRole'.into(), ('Bar', 'Foo').into(), 0, 0);
    assert(*role[0] == 'FooWriter', 'role not set');

    let status = world.entity('AuthStatus'.into(), (*role[0], 'Foo').into(), 0, 0);
    assert(*status[0] == 1, 'role not set');
}
