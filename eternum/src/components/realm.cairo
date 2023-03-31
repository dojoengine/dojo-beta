// Realm specific Metadata

// use eternum::constants::TICK_TIME;
// use starknet;

#[derive(Component)]
struct Realm {
    id: felt252, // OG Realm Id
    founder: felt252, // address of founder
    armies: felt252, // ??? TODO - We need a way to attach Armies to the Realm. This needs to be an array
    resource_ids: felt252, // hash of ids
    cities: felt252,
    harbors: felt252,
    rivers: felt252,
    regions: felt252,
}

// trait RealmTrait {
//     // calculates happiness on the Realm
//     fn happiness(self: Realm, population: felt252, food: felt252) -> felt252; 
// }

// impl RealmImpl of RealmTrait {
//     fn happiness(self: Realm, population: felt252, food: felt252) -> felt252 {
//         // calculate happiness
//         // return happiness
//         0
//     }

//     // fn population(self: Realm, building_population: felt252, army_population: felt252) -> felt252 {
//     //     // calculate building population
//     //     // calculate army population
//     //     // return population
//     //     0
//     // }
// }

