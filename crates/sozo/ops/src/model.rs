use anyhow::Result;
use dojo_types::schema::deep_print_ty;
use dojo_world::contracts::model::ModelReader;
use dojo_world::contracts::world::WorldContractReader;
use starknet::core::types::{BlockId, BlockTag, FieldElement};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;

pub async fn model_class_hash(
    name: String,
    world_address: FieldElement,
    provider: JsonRpcClient<HttpTransport>,
) -> Result<()> {
    let mut world_reader = WorldContractReader::new(world_address, &provider);
    world_reader.set_block(BlockId::Tag(BlockTag::Pending));

    let model = world_reader.model_reader(&name).await?;

    println!("{:#x}", model.class_hash());

    Ok(())
}

pub async fn model_contract_address(
    name: String,
    world_address: FieldElement,
    provider: JsonRpcClient<HttpTransport>,
) -> Result<()> {
    let mut world_reader = WorldContractReader::new(world_address, &provider);
    world_reader.set_block(BlockId::Tag(BlockTag::Pending));

    let model = world_reader.model_reader(&name).await?;

    println!("{:#x}", model.contract_address());

    Ok(())
}

pub async fn model_layout(
    name: String,
    world_address: FieldElement,
    provider: JsonRpcClient<HttpTransport>,
) -> Result<()> {
    let mut world_reader = WorldContractReader::new(world_address, &provider);
    world_reader.set_block(BlockId::Tag(BlockTag::Pending));

    let model = world_reader.model_reader(&name).await?;
    let layout = model.layout().await?;
    let schema = model.schema().await?;

    deep_print_layout(&layout, &schema);

    Ok(())
}

pub async fn model_schema(
    name: String,
    world_address: FieldElement,
    provider: JsonRpcClient<HttpTransport>,
    to_json: bool,
) -> Result<()> {
    let mut world_reader = WorldContractReader::new(world_address, &provider);
    world_reader.set_block(BlockId::Tag(BlockTag::Pending));

    let model = world_reader.model_reader(&name).await?;
    let schema = model.schema().await?;

    if to_json {
        println!("{}", serde_json::to_string_pretty(&schema)?)
    } else {
        deep_print_ty(schema);
    }

    Ok(())
}

pub async fn model_get(
    name: String,
    keys: Vec<FieldElement>,
    world_address: FieldElement,
    provider: JsonRpcClient<HttpTransport>,
) -> Result<()> {
    let mut world_reader = WorldContractReader::new(world_address, &provider);
    world_reader.set_block(BlockId::Tag(BlockTag::Pending));

    let model = world_reader.model_reader(&name).await?;
    let entity = model.entity(&keys).await?;

    println!("{entity}");

    Ok(())
}

#[derive(Clone, Debug)]
struct LayoutInfo {
    layout_type: LayoutInfoType,
    name: String,
    fields: Vec<FieldLayoutInfo>,
}

#[derive(Clone, Debug)]
enum LayoutInfoType {
    Struct,
    Enum,
    Tuple,
    Array,
}

#[derive(Clone, Debug)]
struct FieldLayoutInfo {
    selector: String,
    name: String,
    layout: String,
}

fn format_fixed(layout: &Vec<u8>) -> String {
    format!("[{}]", layout.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", "))
}

fn format_layout_ref(type_name: &str) -> String {
    format!("layout({type_name})")
}

fn format_selector(selector: String) -> String {
    if selector.starts_with("0x") {
        if selector.len() > 14 {
            format!("[{}...{}]", &selector[0..8], &selector[selector.len() - 7..])
        } else {
            format!("[{}]", selector)
        }
    } else {
        selector
    }
}

fn format_name(name: String) -> String {
    if !name.is_empty() {
        format!(" {} ", name)
    } else {
        name
    }
}

fn format_field(selector: String, name: String, layout: String) -> String {
    format!("    {:<20}{:<18}: {}", format_selector(selector), format_name(name), layout)
}

fn format_field_layout(
    layout: &dojo_world::contracts::model::abigen::model::Layout,
    schema: &dojo_types::schema::Ty,
) -> String {
    match layout {
        dojo_world::contracts::model::abigen::model::Layout::Fixed(x) => format_fixed(&x),
        dojo_world::contracts::model::abigen::model::Layout::ByteArray => {
            "layout(ByteArray)".to_string()
        }
        _ => format_layout_ref(&get_name_from_schema(schema)),
    }
}

fn is_layout_in_list(list: &Vec<LayoutInfo>, name: &String) -> bool {
    list.iter().any(|x| x.name.eq(name))
}

fn get_name_from_schema(schema: &dojo_types::schema::Ty) -> String {
    match schema {
        dojo_types::schema::Ty::Struct(s) => s.name.clone(),
        dojo_types::schema::Ty::Enum(e) => e.name.clone(),
        dojo_types::schema::Ty::Primitive(p) => match p {
            dojo_types::primitive::Primitive::U8(_) => "u8".to_string(),
            dojo_types::primitive::Primitive::U16(_) => "u16".to_string(),
            dojo_types::primitive::Primitive::U32(_) => "u32".to_string(),
            dojo_types::primitive::Primitive::U64(_) => "u64".to_string(),
            dojo_types::primitive::Primitive::U128(_) => "u128".to_string(),
            dojo_types::primitive::Primitive::U256(_) => "u256".to_string(),
            dojo_types::primitive::Primitive::USize(_) => "usize".to_string(),
            dojo_types::primitive::Primitive::Bool(_) => "bool".to_string(),
            dojo_types::primitive::Primitive::Felt252(_) => "felt252".to_string(),
            dojo_types::primitive::Primitive::ClassHash(_) => "ClassHash".to_string(),
            dojo_types::primitive::Primitive::ContractAddress(_) => "ContractAddress".to_string(),
        },
        dojo_types::schema::Ty::Tuple(t) => {
            format!(
                "({})",
                t.iter().map(|x| get_name_from_schema(x)).collect::<Vec<_>>().join(", ")
            )
        }
        dojo_types::schema::Ty::Array(a) => format!("Array<{}>", get_name_from_schema(&a[0])),
        _ => "".to_string(),
    }
}

fn get_printable_layout_list_from_struct(
    field_layouts: &Vec<dojo_world::contracts::model::abigen::model::FieldLayout>,
    schema: &dojo_types::schema::Ty,
    layout_list: &mut Vec<LayoutInfo>,
) {
    match schema {
        dojo_types::schema::Ty::Struct(ss) => {
            let name = get_name_from_schema(&schema);

            // proces main struct
            if !is_layout_in_list(layout_list, &name) {
                layout_list.push(LayoutInfo {
                    layout_type: LayoutInfoType::Struct,
                    name,
                    fields: field_layouts
                        .iter()
                        .zip(ss.children.iter().filter(|x| !x.key))
                        .map(|(l, m)| FieldLayoutInfo {
                            selector: format!("{:#x}", l.selector),
                            name: m.name.clone(),
                            layout: format_field_layout(&l.layout, &m.ty),
                        })
                        .collect::<Vec<_>>(),
                });
            }

            // process members
            for (member_layout, member) in
                field_layouts.iter().zip(ss.children.iter().filter(|x| !x.key))
            {
                get_printable_layout_list(&member_layout.layout, &member.ty, layout_list);
            }
        }
        _ => {}
    };
}

fn get_printable_layout_list_from_enum(
    field_layouts: &Vec<dojo_world::contracts::model::abigen::model::FieldLayout>,
    schema: &dojo_types::schema::Ty,
    layout_list: &mut Vec<LayoutInfo>,
) {
    match schema {
        dojo_types::schema::Ty::Enum(se) => {
            let name = get_name_from_schema(&schema);

            // proces main enum
            if !is_layout_in_list(layout_list, &name) {
                layout_list.push(LayoutInfo {
                    layout_type: LayoutInfoType::Enum,
                    name,
                    fields: field_layouts
                        .iter()
                        .zip(se.options.iter())
                        .map(|(l, o)| FieldLayoutInfo {
                            selector: format!("{:#x}", l.selector),
                            name: o.name.to_string(),
                            layout: format_field_layout(&l.layout, &o.ty),
                        })
                        .collect::<Vec<_>>(),
                });
            }

            // process variants
            for (variant_layout, variant) in field_layouts.iter().zip(se.options.iter()) {
                get_printable_layout_list(&variant_layout.layout, &variant.ty, layout_list);
            }
        }
        _ => {}
    }
}

fn get_printable_layout_list_from_tuple(
    item_layouts: &Vec<dojo_world::contracts::model::abigen::model::Layout>,
    schema: &dojo_types::schema::Ty,
    layout_list: &mut Vec<LayoutInfo>,
) {
    match schema {
        dojo_types::schema::Ty::Tuple(st) => {
            let name = get_name_from_schema(&schema);

            // process tuple
            if !is_layout_in_list(layout_list, &name) {
                layout_list.push(LayoutInfo {
                    layout_type: LayoutInfoType::Tuple,
                    name,
                    fields: item_layouts
                        .iter()
                        .enumerate()
                        .zip(st.iter())
                        .map(|((i, l), s)| FieldLayoutInfo {
                            selector: format!("{:#x}", i),
                            name: "".to_string(),
                            layout: format_field_layout(l, s),
                        })
                        .collect::<Vec<_>>(),
                });
            }

            // process tuple items
            for (item_layout, item_schema) in item_layouts.iter().zip(st.iter()) {
                get_printable_layout_list(&item_layout, &item_schema, layout_list);
            }
        }
        _ => {}
    }
}

fn get_printable_layout_list_from_array(
    item_layout: &dojo_world::contracts::model::abigen::model::Layout,
    schema: &dojo_types::schema::Ty,
    layout_list: &mut Vec<LayoutInfo>,
) {
    match schema {
        dojo_types::schema::Ty::Array(sa) => {
            let name = get_name_from_schema(&schema);

            // process array
            if !is_layout_in_list(layout_list, &name) {
                layout_list.push(LayoutInfo {
                    layout_type: LayoutInfoType::Array,
                    name,
                    fields: vec![
                        FieldLayoutInfo {
                            selector: "Length".to_string(),
                            name: "".to_string(),
                            layout: "[32]".to_string(),
                        },
                        FieldLayoutInfo {
                            selector: "[ItemIndex]".to_string(),
                            name: "".to_string(),
                            layout: format_field_layout(item_layout, &sa[0]),
                        },
                    ],
                });
            }

            // process array item
            get_printable_layout_list(&item_layout, &sa[0], layout_list);
        }
        _ => {}
    }
}

fn get_printable_layout_list(
    root_layout: &dojo_world::contracts::model::abigen::model::Layout,
    schema: &dojo_types::schema::Ty,
    layout_list: &mut Vec<LayoutInfo>,
) {
    match root_layout {
        dojo_world::contracts::model::abigen::model::Layout::Struct(ls) => {
            get_printable_layout_list_from_struct(ls, schema, layout_list);
        }
        dojo_world::contracts::model::abigen::model::Layout::Enum(le) => {
            get_printable_layout_list_from_enum(le, schema, layout_list);
        }
        dojo_world::contracts::model::abigen::model::Layout::Tuple(lt) => {
            get_printable_layout_list_from_tuple(lt, schema, layout_list);
        }
        dojo_world::contracts::model::abigen::model::Layout::Array(la) => {
            get_printable_layout_list_from_array(&la[0], schema, layout_list);
        }
        _ => {}
    };
}

fn print_layout_info(layout_info: LayoutInfo) {
    let fields = layout_info
        .fields
        .into_iter()
        .map(|f| format_field(f.selector, f.name, f.layout))
        .collect::<Vec<_>>();
    let layout_title = match layout_info.layout_type {
        LayoutInfoType::Struct => format!("Struct {} {{", layout_info.name),
        LayoutInfoType::Enum => format!("Enum {} {{", layout_info.name),
        LayoutInfoType::Tuple => format!("{} (", layout_info.name),
        LayoutInfoType::Array => format!("{} (", layout_info.name),
    };
    let end_token = match layout_info.layout_type {
        LayoutInfoType::Struct => '}',
        LayoutInfoType::Enum => '}',
        LayoutInfoType::Tuple => ')',
        LayoutInfoType::Array => ')',
    };

    println!(
        "{layout_title}
{}
{end_token}\n",
        fields.join("\n")
    );
}

// print the full Layout tree
fn deep_print_layout(
    layout: &dojo_world::contracts::model::abigen::model::Layout,
    schema: &dojo_types::schema::Ty,
) {
    let mut layout_list = vec![];
    get_printable_layout_list(&layout, &schema, &mut layout_list);

    for l in layout_list {
        print_layout_info(l);
    }
}

/*
    Struct S1 {
        [0x123...345] (field_name): layout(S2),
        [0x123...345] (field_name): layout((u8, u32))
        [0x456...768]: [8, 16, ..., 251]
        [0x456...768]: layout(Array<u32>)
    }

    (u8, u32)
        [0]: [8]
        [1]: [32]

    Array<u32>
        []: array length
        [item_index]: [32]


    Enum E1 {
        [0]:
    }
*/
