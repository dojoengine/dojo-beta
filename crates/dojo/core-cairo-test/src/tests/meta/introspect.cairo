use dojo::meta::introspect::Introspect;
use dojo::meta::{Layout, FieldLayout};

#[derive(Drop, Introspect)]
struct Base {
    value: u32,
}

#[derive(Drop, Introspect)]
struct WithArray {
    value: u32,
    arr: Array<u8>
}

#[derive(Drop, Introspect)]
struct WithFixedArray {
    value: u32,
    arr: [u8; 3]
}

#[derive(Drop, Introspect)]
struct WithByteArray {
    value: u32,
    arr: ByteArray
}

#[derive(Drop, Introspect)]
struct WithTuple {
    value: u32,
    arr: (u8, u16, u32)
}

#[derive(Drop, Introspect)]
struct WithNestedTuple {
    value: u32,
    arr: (u8, (u16, u128, u256), u32)
}

#[derive(Drop, Introspect)]
struct WithNestedArrayInTuple {
    value: u32,
    arr: (u8, (u16, Array<u128>, u256), u32)
}

#[derive(Drop, IntrospectPacked)]
struct Vec3 {
    x: u32,
    y: u32,
    z: u32
}

#[derive(IntrospectPacked)]
struct Translation {
    from: Vec3,
    to: Vec3
}

#[derive(Drop, IntrospectPacked)]
struct StructInnerNotPacked {
    x: Base
}

#[derive(Drop, Introspect)]
enum EnumNoData {
    One,
    Two,
    Three
}

#[derive(Drop, Introspect)]
enum EnumWithSameData {
    One: u256,
    Two: u256,
    Three: u256
}

#[derive(Drop, Introspect)]
enum EnumWithSameTupleData {
    One: (u256, u32),
    Two: (u256, u32),
    Three: (u256, u32)
}

#[derive(Drop, Introspect)]
enum EnumWithVariousData {
    One: u32,
    Two: (u8, u16),
    Three: Array<u128>,
}


#[derive(Drop, IntrospectPacked)]
enum EnumPacked {
    A: u32,
    B: u32,
}

#[derive(Drop, IntrospectPacked)]
enum EnumInnerPacked {
    A: (EnumPacked, Vec3),
    B: (EnumPacked, Vec3),
}

#[derive(Drop, IntrospectPacked)]
enum EnumInnerNotPacked {
    A: (EnumPacked, Base),
    B: (EnumPacked, Base),
}

#[derive(Drop, Introspect)]
struct StructWithOption {
    x: Option<u16>
}

#[derive(Drop, Introspect)]
struct Generic<T> {
    value: T,
}

#[derive(Drop, Introspect)]
struct StructWithFixedArray {
    x: [u8; 3]
}

#[derive(Drop, Introspect)]
struct StructWithComplexFixedArray {
    x: [[EnumWithSameData; 2]; 3]
}

#[derive(Drop, Introspect)]
enum EnumWithFixedArray {
    A: [u8; 3]
}

#[derive(Drop, IntrospectPacked)]
struct StructWithFixedArrayPacked {
    x: [u8; 3]
}

#[derive(Drop, IntrospectPacked)]
enum EnumWithFixedArrayPacked {
    A: [u8; 3]
}

fn field(selector: felt252, layout: Layout) -> FieldLayout {
    FieldLayout { selector, layout }
}

fn fixed(values: Array<u8>) -> Layout {
    Layout::Fixed(values.span())
}

fn tuple(values: Array<Layout>) -> Layout {
    Layout::Tuple(values.span())
}

fn fixed_array(inner_layout: Layout, size: u32) -> Layout {
    Layout::FixedArray(array![(inner_layout, size)].span())
}

fn _enum(values: Array<Option<Layout>>) -> Layout {
    let mut items = array![];
    let mut i = 0;

    loop {
        if i >= values.len() {
            break;
        }

        let v = *values.at(i);
        match v {
            Option::Some(v) => { items.append(field(i.into(), v)); },
            Option::None => { items.append(field(i.into(), fixed(array![]))) }
        }

        i += 1;
    };

    Layout::Enum(items.span())
}

fn arr(item_layout: Layout) -> Layout {
    Layout::Array([item_layout].span())
}

#[test]
#[available_gas(2000000)]
fn test_generic_introspect() {
    let _generic = Generic { value: Base { value: 123 } };
}

#[test]
fn test_size_basic_struct() {
    let size = Introspect::<Base>::size();
    assert!(size.is_some());
    assert!(size.unwrap() == 1);
}

#[test]
fn test_size_with_array() {
    assert!(Introspect::<WithArray>::size().is_none());
}

// fn test_size_with_fixed_array() {
//     let size = Introspect::<WithFixedArray>::size();
//     assert!(size.is_some());
//     assert!(size.unwrap() == 4);
// }

#[test]
fn test_size_with_byte_array() {
    assert!(Introspect::<WithByteArray>::size().is_none());
}

#[test]
fn test_size_with_tuple() {
    let size = Introspect::<WithTuple>::size();
    assert!(size.is_some());
    assert!(size.unwrap() == 4);
}

#[test]
fn test_size_with_nested_tuple() {
    let size = Introspect::<WithNestedTuple>::size();
    assert!(size.is_some());
    assert!(size.unwrap() == 7);
}

#[test]
fn test_size_with_nested_array_in_tuple() {
    let size = Introspect::<WithNestedArrayInTuple>::size();
    assert!(size.is_none());
}

#[test]
fn test_size_of_enum_without_variant_data() {
    let size = Introspect::<EnumNoData>::size();
    assert!(size.is_some());
    assert!(size.unwrap() == 1);
}

#[test]
fn test_size_of_enum_with_same_variant_data() {
    let size = Introspect::<EnumWithSameData>::size();
    assert!(size.is_some());
    assert!(size.unwrap() == 3);
}

#[test]
fn test_size_of_enum_with_same_tuple_variant_data() {
    let size = Introspect::<EnumWithSameTupleData>::size();
    assert!(size.is_some());
    assert!(size.unwrap() == 4);
}

#[test]
fn test_size_of_struct_with_option() {
    let size = Introspect::<StructWithOption>::size();
    assert!(size.is_none());
}

#[test]
fn test_size_of_enum_with_variant_data() {
    let size = Introspect::<EnumWithVariousData>::size();
    assert!(size.is_none());
}

#[test]
fn test_size_of_struct_with_fixed_array() {
    let size = Introspect::<StructWithFixedArray>::size();
    assert!(size.is_some());
    assert!(size.unwrap() == 3);
}

#[test]
fn test_size_of_struct_with_complex_fixed_array() {
    let size = Introspect::<StructWithComplexFixedArray>::size();
    assert!(size.is_some());
    assert!(size.unwrap() == 18);
}

#[test]
fn test_size_of_packed_struct_with_fixed_array() {
    let size = Introspect::<StructWithFixedArrayPacked>::size();
    assert!(size.is_some());
    assert!(size.unwrap() == 3);
}

#[test]
fn test_size_of_enum_with_fixed_array() {
    let size = Introspect::<EnumWithFixedArray>::size();
    assert!(size.is_some());
    assert!(size.unwrap() == 4);
}

#[test]
fn test_size_of_packed_enum_with_fixed_array() {
    let size = Introspect::<EnumWithFixedArrayPacked>::size();
    assert!(size.is_some());
    assert!(size.unwrap() == 4);
}

#[test]
fn test_layout_of_enum_without_variant_data() {
    let layout = Introspect::<EnumNoData>::layout();
    let expected = _enum(array![ // One
    Option::None, // Two
     Option::None, // Three
     Option::None,]);

    assert!(layout == expected);
}

#[test]
fn test_layout_of_enum_with_variant_data() {
    let layout = Introspect::<EnumWithVariousData>::layout();
    let expected = _enum(
        array![
            // One
            Option::Some(fixed(array![32])),
            // Two
            Option::Some(tuple(array![fixed(array![8]), fixed(array![16])])),
            // Three
            Option::Some(arr(fixed(array![128]))),
        ]
    );

    assert!(layout == expected);
}

#[test]
fn test_layout_of_struct_with_option() {
    let layout = Introspect::<StructWithOption>::layout();
    let expected = Layout::Struct(
        array![field(selector!("x"), _enum(array![Option::Some(fixed(array![16])), Option::None]))]
            .span()
    );

    assert!(layout == expected);
}

#[test]
fn test_layout_of_struct_with_fixed_array() {
    let layout = Introspect::<StructWithFixedArray>::layout();
    let expected = Layout::Struct(
        array![field(selector!("x"), fixed_array(Introspect::<u8>::layout(), 3))].span()
    );

    assert!(layout == expected);
}

#[test]
fn test_layout_of_struct_with_complex_fixed_array() {
    let layout = Introspect::<StructWithComplexFixedArray>::layout();
    let expected = Layout::Struct(
        array![
            field(
                selector!("x"),
                fixed_array(fixed_array(Introspect::<EnumWithSameData>::layout(), 2), 3)
            )
        ]
            .span()
    );

    assert!(layout == expected);
}

#[test]
fn test_layout_of_packed_struct_with_fixed_array() {
    let layout = Introspect::<StructWithFixedArrayPacked>::layout();
    let expected = Layout::Fixed([8, 8, 8].span());

    assert!(layout == expected);
}

#[test]
fn test_layout_of_enum_with_fixed_array() {
    let layout = Introspect::<EnumWithFixedArray>::layout();
    let expected = _enum(array![Option::Some(fixed_array(Introspect::<u8>::layout(), 3))]);

    assert!(layout == expected);
}

#[test]
fn test_layout_of_packed_enum_with_fixed_array() {
    let layout = Introspect::<EnumWithFixedArrayPacked>::layout();
    let expected = Layout::Fixed([8, 8, 8, 8].span());

    assert!(layout == expected);
}

#[test]
fn test_layout_of_packed_struct() {
    let layout = Introspect::<Vec3>::layout();
    let expected = Layout::Fixed([32, 32, 32].span());

    assert!(layout == expected);
}

#[test]
fn test_layout_of_inner_packed_struct() {
    let layout = Introspect::<Translation>::layout();
    let expected = Layout::Fixed([32, 32, 32, 32, 32, 32].span());

    assert!(layout == expected);
}

#[test]
#[should_panic(expected: ("A packed model layout must contain Fixed layouts only.",))]
fn test_layout_of_not_packed_inner_struct() {
    let _ = Introspect::<StructInnerNotPacked>::layout();
}

#[test]
fn test_layout_of_packed_enum() {
    let layout = Introspect::<EnumPacked>::layout();
    let expected = Layout::Fixed([8, 32].span());

    assert!(layout == expected);
}

#[test]
fn test_layout_of_inner_packed_enum() {
    let layout = Introspect::<EnumInnerPacked>::layout();
    let expected = Layout::Fixed([8, 8, 32, 32, 32, 32].span());

    assert!(layout == expected);
}

#[test]
#[should_panic(expected: ("A packed model layout must contain Fixed layouts only.",))]
fn test_layout_of_not_packed_inner_enum() {
    let _ = Introspect::<EnumInnerNotPacked>::layout();
}
