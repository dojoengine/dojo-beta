use array::ArrayTrait;
use core::debug::PrintTrait;
use starknet::ContractAddress;
use dojo::database::schema::{Member, MemberType, SchemaIntrospection, serialize_member};

#[derive(Serde, Copy, Drop)]
enum Direction {
    None: (),
    Left: (),
    Right: (),
    Up: (),
    Down: (),
}

impl DirectionSchemaIntrospectionImpl of SchemaIntrospection<Direction> {
    #[inline(always)]
    fn size() -> usize {
        1
    }

    #[inline(always)]
    fn layout(ref layout: Array<u8>) {
        layout.append(8);
    }

    #[inline(always)]
    fn ty() -> MemberType {
        MemberType::Complex(array![
            serialize_member(@Member {
                name: 'Direction',
                ty: MemberType::Enum(array![].span()),
                attrs: array![].span(),
            })
        ].span())
    }
}

impl DirectionPrintImpl of PrintTrait<Direction> {
    fn print(self: Direction) {
        match self {
            Direction::None(()) => 0.print(),
            Direction::Left(()) => 1.print(),
            Direction::Right(()) => 2.print(),
            Direction::Up(()) => 3.print(),
            Direction::Down(()) => 4.print(),
        }
    }
}

impl DirectionIntoFelt252 of Into<Direction, felt252> {
    fn into(self: Direction) -> felt252 {
        match self {
            Direction::None(()) => 0,
            Direction::Left(()) => 1,
            Direction::Right(()) => 2,
            Direction::Up(()) => 3,
            Direction::Down(()) => 4,
        }
    }
}

#[derive(Component, Copy, Drop, Serde)]
struct Moves {
    #[key]
    player: ContractAddress,
    remaining: u8,
    last_direction: Direction
}

#[derive(Copy, Drop, Serde)]
struct Vec2 {
    x: u32,
    y: u32
}

impl Vec2PrintImpl of PrintTrait<Vec2> {
    fn print(self: Vec2) {
        self.x.print();
    }
}

impl Vec2SchemaIntrospectionImpl of SchemaIntrospection<Vec2> {
    #[inline(always)]
    fn size() -> usize {
        2
    }

    #[inline(always)]
    fn layout(ref layout: Array<u8>) {
        layout.append(32);
        layout.append(32);
    }

    #[inline(always)]
    fn ty() -> MemberType {
        MemberType::Complex(array![
            serialize_member(@Member {
                name: 'x',
                ty: SchemaIntrospection::<u32>::ty(),
                attrs: array![].span(),
            }),
            serialize_member(@Member {
                name: 'y',
                ty: SchemaIntrospection::<u32>::ty(),
                attrs: array![].span(),
            })
        ].span())
    }
}

#[derive(Component, Copy, Drop, Serde)]
struct Position {
    #[key]
    player: ContractAddress,
    vec: Vec2,
}

trait Vec2Trait {
    fn is_zero(self: Vec2) -> bool;
    fn is_equal(self: Vec2, b: Vec2) -> bool;
}

impl Vec2Impl of Vec2Trait {
    fn is_zero(self: Vec2) -> bool {
        if self.x - self.y == 0 {
            return true;
        }
        false
    }

    fn is_equal(self: Vec2, b: Vec2) -> bool {
        self.x == b.x && self.y == b.y
    }
}

#[starknet::contract]
mod reproduce {
    #[storage]
    struct Storage {}

    #[derive(Copy, Drop, Serde)]
    struct Member {
        name: felt252,
        ty: MemberType,
    }

    #[derive(Copy, Drop, Serde)]
    enum MemberType {
        Simple: felt252,
        Complex: Member,
    }

    trait SchemaIntrospection<T> {
        fn ty() -> MemberType;
    }

    struct Position {
        x: felt252,
    }

    impl PositionSchemaIntrospection of SchemaIntrospection<Position> {
        #[inline(always)]
        fn ty() -> MemberType {
            MemberType::Simple('ty')
        }
    }

    #[external(v0)]
    fn schema(self: @ContractState) -> MemberType {
        SchemaIntrospection::<Position>::ty()
    }
}

// #[cfg(test)]
// mod tests {
//     use debug::PrintTrait;
//     use super::{Position, Vec2, Vec2Trait};

//     #[test]
//     #[available_gas(100000)]
//     fn test_vec_is_zero() {
//         assert(Vec2Trait::is_zero(Vec2 { x: 0, y: 0 }), 'not zero');
//     }

//     #[test]
//     #[available_gas(100000)]
//     fn test_vec_is_equal() {
//         let position = Vec2 { x: 420, y: 0 };
//         position.print();
//         assert(position.is_equal(Vec2 { x: 420, y: 0 }), 'not equal');
//     }
// }
