use std::collections::HashMap;
use std::path::PathBuf;

use async_trait::async_trait;
use cainome::parser::tokens::Composite;
use generator::function::TsFunctionGenerator;
use generator::interface::TsInterfaceGenerator;
use generator::r#enum::TsEnumGenerator;
use generator::schema::TsSchemaGenerator;
use writer::{TsFileContractWriter, TsFileWriter};

use crate::error::BindgenResult;
use crate::plugins::BuiltinPlugin;
use crate::DojoData;

use super::BindgenWriter;

pub(crate) mod generator;
pub(crate) mod writer;

#[cfg(test)]
mod tests;

pub struct TypescriptPlugin {
    writers: Vec<Box<dyn BindgenWriter>>,
}

impl TypescriptPlugin {
    pub fn new() -> Self {
        Self {
            writers: vec![
                Box::new(TsFileWriter::new(
                    "models.gen.ts",
                    vec![
                        Box::new(TsInterfaceGenerator {}),
                        Box::new(TsEnumGenerator {}),
                        Box::new(TsSchemaGenerator {}),
                    ],
                )),
                Box::new(TsFileContractWriter::new(
                    "contracts.gen.ts",
                    vec![Box::new(TsFunctionGenerator {})],
                )),
            ],
        }
    }

    //     // Maps cairo types to C#/Unity SDK defined types
    //     fn map_type(token: &Token) -> String {
    //         match token.type_name().as_str() {
    //             "bool" => "RecsType.Boolean".to_string(),
    //             "i8" => "RecsType.Number".to_string(),
    //             "i16" => "RecsType.Number".to_string(),
    //             "i32" => "RecsType.Number".to_string(),
    //             "i64" => "RecsType.Number".to_string(),
    //             "i128" => "RecsType.BigInt".to_string(),
    //             "u8" => "RecsType.Number".to_string(),
    //             "u16" => "RecsType.Number".to_string(),
    //             "u32" => "RecsType.Number".to_string(),
    //             "u64" => "RecsType.Number".to_string(),
    //             "u128" => "RecsType.BigInt".to_string(),
    //             "u256" => "RecsType.BigInt".to_string(),
    //             "usize" => "RecsType.Number".to_string(),
    //             "felt252" => "RecsType.BigInt".to_string(),
    //             "bytes31" => "RecsType.String".to_string(),
    //             "ClassHash" => "RecsType.BigInt".to_string(),
    //             "ContractAddress" => "RecsType.BigInt".to_string(),
    //             "ByteArray" => "RecsType.String".to_string(),
    //             "array" => {
    //                 if let Token::Array(array) = token {
    //                     let mut mapped = TypescriptPlugin::map_type(&array.inner);
    //                     if mapped == array.inner.type_name() {
    //                         mapped = "RecsType.String".to_string();
    //                     }
    //
    //                     format!("{}Array", mapped)
    //                 } else {
    //                     panic!("Invalid array token: {:?}", token);
    //                 }
    //             }
    //             "generic_arg" => {
    //                 if let Token::GenericArg(g) = &token {
    //                     g.clone()
    //                 } else {
    //                     panic!("Invalid generic arg token: {:?}", token);
    //                 }
    //             }
    //
    //             // we consider tuples as essentially objects
    //             _ => {
    //                 let mut type_name = token.type_name().to_string();
    //
    //                 if let Token::Composite(composite) = token {
    //                     if !composite.generic_args.is_empty() {
    //                         type_name += &format!(
    //                             "<{}>",
    //                             composite
    //                                 .generic_args
    //                                 .iter()
    //                                 .map(|(_, t)| TypescriptPlugin::map_type(t))
    //                                 .collect::<Vec<_>>()
    //                                 .join(", ")
    //                         )
    //                     }
    //                 }
    //
    //                 type_name
    //             }
    //         }
    //     }
    //
    //     fn generated_header() -> String {
    //         format!(
    //             "
    // // Generated by dojo-bindgen on {}. Do not modify this file manually.
    // // Import the necessary types from the recs SDK
    // // generate again with `sozo build --typescript`
    // ",
    //             chrono::Utc::now().to_rfc2822()
    //         )
    //     }
    //
    //     // Token should be a struct
    //     // This will be formatted into a C# struct
    //     // using C# and unity SDK types
    //     fn format_struct(token: &Composite) -> String {
    //         // we want to avoid creating definition for cairo types that are not models
    //         if TypescriptPlugin::filter_type_path(token) {
    //             return String::new();
    //         }
    //
    //         let mut native_fields = String::new();
    //         let mut fields = String::new();
    //
    //         println!("token: {:?}", token);
    //
    //         for field in &token.inners {
    //             println!("field: {:?}", field);
    //             let mapped = TypescriptPlugin::map_type(&field.token);
    //             println!("mapped: {:?}", mapped);
    //
    //             if mapped == field.token.type_name() {
    //                 native_fields +=
    //                     format!("{}: {};\n    ", field.name, field.token.type_name()).as_str();
    //                 fields += format!("{}: {}Definition,\n    ", field.name, mapped,).as_str();
    //             } else {
    //                 native_fields += format!(
    //                     "{}: {};\n    ",
    //                     field.name,
    //                     mapped.replace("RecsType.", "").replace("Array", "[]")
    //                 )
    //                 .as_str();
    //                 fields += format!("{}: {},\n    ", field.name, mapped,).as_str();
    //             }
    //         }
    //
    //         format!(
    //             "
    // // Type definition for `{path}` struct
    // export interface {name} {{
    //     fieldOrder: string[];
    //     {native_fields}}}
    // ",
    //             path = token.type_path,
    //             name = token.type_name(),
    //             native_fields = native_fields
    //         )
    //     }
    //
    //     // Token should be an enum
    //     // This will be formatted into a C# enum
    //     // Enum is mapped using index of cairo enum
    //     fn format_enum(token: &Composite) -> String {
    //         if TypescriptPlugin::filter_type_path(token) {
    //             return String::new();
    //         }
    //         // filter out common types
    //         // TODO: Make cleaner
    //         if token.type_path == "core::option::Option::<core::integer::u32>"
    //             || token.type_path == "core::option::Option::<core::integer::u8>"
    //             || token.type_path == "core::option::Option::<core::integer::u16>"
    //             || token.type_path == "core::option::Option::<core::integer::u64>"
    //             || token.type_path == "core::option::Option::<core::integer::u128>"
    //             || token.type_path == "core::option::Option::<core::integer::u256>"
    //         {
    //             return String::new(); // Return an empty string for these enums
    //         }
    //
    //         let name = TypescriptPlugin::map_type(&Token::Composite(token.clone()));
    //         let mut variants = "".to_owned();
    //
    //         for field in &token.inners {
    //             let field_type = TypescriptPlugin::map_type(&field.token).replace("()", "");
    //
    //             let variant_definition = if field_type.is_empty() {
    //                 // No associated data
    //                 format!("{},\n", field.name)
    //             } else {
    //                 // With associated data
    //                 format!("{},\n", field.name)
    //             };
    //
    //             variants += variant_definition.as_str();
    //         }
    //         return format!(
    //             "
    // // Type definition for `{}` enum
    // export enum {} {{
    //     {}}}
    // ",
    //             token.type_path, name, variants
    //         );
    //     }
    //
    //     // Token should be a model
    //     // This will be formatted into a C# class inheriting from ModelInstance
    //     // Fields are mapped using C# and unity SDK types
    //     fn format_model(namespace: &str, model: &Composite) -> String {
    //         if TypescriptPlugin::filter_type_path(model) {
    //             return String::new();
    //         }
    //         let mut custom_types = Vec::<String>::new();
    //         let mut types = Vec::<String>::new();
    //         let (fields, _composite_type) =
    //             model.inners.iter().fold((Vec::new(), model.r#type), |(mut fields, _), field| {
    //                 let mapped = TypescriptPlugin::map_type(&field.token);
    //
    //                 let field_str = match field.token {
    //                     Token::Composite(ref c) if c.r#type == CompositeType::Enum => {
    //                         types.push(format!("\"{}\"", field.token.type_name()));
    //                         format!("{}: RecsType.String,", field.name)
    //                     }
    //                     Token::Composite(_) => {
    //                         custom_types.push(format!("\"{}\"", field.token.type_name()));
    //                         format!("{}: {}Definition,", field.name, mapped)
    //                     }
    //                     _ if mapped == field.token.type_name() => {
    //                         custom_types.push(format!("\"{}\"", field.token.type_name()));
    //                         format!("{}: {}Definition,", field.name, mapped)
    //                     }
    //                     _ => {
    //                         types.push(format!("\"{}\"", field.token.type_name()));
    //                         format!("{}: {},", field.name, mapped)
    //                     }
    //                 };
    //
    //                 fields.push(field_str);
    //                 (fields, model.r#type)
    //             });
    //
    //         let fields_str = fields.join("\n                    ");
    //
    //         format!(
    //             "
    //         // Model definition for `{path}` model
    //         {model}: (() => {{
    //             return defineComponent(
    //                 world,
    //                 {{
    //                     {fields_str}
    //                 }},
    //                 {{
    //                     metadata: {{
    //                         namespace: \"{namespace}\",
    //                         name: \"{model}\",
    //                         types: [{types}],
    //                         customTypes: [{custom_types}],
    //                     }},
    //                 }}
    //             );
    //         }})(),
    // ",
    //             path = model.type_path,
    //             model = model.type_name(),
    //             types = types.join(", "),
    //             custom_types = custom_types.join(", ")
    //         )
    //     }
    //
    //     // Handles a model definition and its referenced tokens
    //     // Will map all structs and enums to TS types
    //     // Will format the models into a object
    //     fn handle_model(&self, models: &[&DojoModel], handled_tokens: &mut Vec<Composite>) -> String {
    //         let mut out = String::new();
    //         out += TypescriptPlugin::generated_header().as_str();
    //         out += "\n";
    //
    //         let mut models_structs = Vec::new();
    //         for model in models {
    //             let tokens = &model.tokens;
    //
    //             let mut sorted_structs = tokens.structs.clone();
    //             sorted_structs.sort_by(compare_tokens_by_type_name);
    //
    //             let mut sorted_enums = tokens.enums.clone();
    //             sorted_enums.sort_by(compare_tokens_by_type_name);
    //
    //             for token in &sorted_enums {
    //                 handled_tokens.push(token.to_composite().unwrap().to_owned());
    //             }
    //             for token in &sorted_structs {
    //                 handled_tokens.push(token.to_composite().unwrap().to_owned());
    //             }
    //
    //             for token in &sorted_enums {
    //                 if handled_tokens.iter().filter(|t| t.type_name() == token.type_name()).count() > 1
    //                 {
    //                     continue;
    //                 }
    //                 out += TypescriptPlugin::format_enum(token.to_composite().unwrap()).as_str();
    //             }
    //
    //             for token in &sorted_structs {
    //                 if handled_tokens.iter().filter(|t| t.type_name() == token.type_name()).count() > 1
    //                 {
    //                     continue;
    //                 }
    //
    //                 // first index is our model struct
    //                 if token.type_name() == naming::get_name_from_tag(&model.tag) {
    //                     models_structs.push((
    //                         naming::get_namespace_from_tag(&model.tag),
    //                         token.to_composite().unwrap().clone(),
    //                     ));
    //                 }
    //
    //                 out += TypescriptPlugin::format_struct(token.to_composite().unwrap()).as_str();
    //             }
    //
    //             out += "\n";
    //         }
    //
    //         out += "
    // export function defineContractComponents(world: World) {
    //     return {
    // ";
    //
    //         for (namespace, model) in models_structs {
    //             out += TypescriptPlugin::format_model(&namespace, &model).as_str();
    //         }
    //
    //         out += "    };
    // }\n";
    //
    //         out
    //     }
    //

    // // Formats a system into a C# method used by the contract class
    // // Handled tokens should be a list of all structs and enums used by the contract
    // // Such as a set of referenced tokens from a model
    // fn format_system(system: &Function, handled_tokens: &[Composite], namespace: String) -> String {
    //     if [
    //         "contract_name",
    //         "namespace",
    //         "tag",
    //         "name_hash",
    //         "selector",
    //         "dojo_init",
    //         "namespace_hash",
    //     ]
    //     .contains(&system.name.as_str())
    //     {
    //         return String::new();
    //     }
    //     fn map_type(token: &Token) -> String {
    //         match token {
    //             Token::CoreBasic(_) => TypescriptPlugin::map_type(token)
    //             .replace("RecsType.", "").replace("Array", "[]")
    //             // types should be lowercased
    //             .to_lowercase(),
    //             Token::Composite(t) => format!("models.{}", t.type_name()),
    //             Token::Array(_) => TypescriptPlugin::map_type(token),
    //             _ => panic!("Unsupported token type: {:?}", token),
    //         }
    //     }
    //
    //     let args = system
    //         .inputs
    //         .iter()
    //         .map(|arg| format!("{}: {}", arg.0, map_type(&arg.1)))
    //         .collect::<Vec<String>>()
    //         .join(", ");
    //
    //     fn handle_arg_recursive(
    //         arg_name: &str,
    //         arg: &Token,
    //         handled_tokens: &[Composite],
    //     ) -> String {
    //         match arg {
    //             Token::Composite(_) => {
    //                 match handled_tokens.iter().find(|t| t.type_name() == arg.type_name()) {
    //                     Some(t) => {
    //                         // Need to flatten the struct members.
    //                         match t.r#type {
    //                             CompositeType::Struct if t.type_name() == "ByteArray" => {
    //                                 format!("byteArray.byteArrayFromString(props.{})", arg_name)
    //                             }
    //                             CompositeType::Struct => t
    //                                 .inners
    //                                 .iter()
    //                                 .map(|field| format!("props.{}.{}", arg_name, field.name))
    //                                 .collect::<Vec<String>>()
    //                                 .join(",\n                    "),
    //                             CompositeType::Enum => format!(
    //                                 "[{}].indexOf(props.{}.type)",
    //                                 t.inners
    //                                     .iter()
    //                                     .map(|field| format!("\"{}\"", field.name))
    //                                     .collect::<Vec<String>>()
    //                                     .join(", "),
    //                                 arg_name
    //                             ),
    //                             _ => {
    //                                 format!("props.{}", arg_name)
    //                             }
    //                         }
    //                     }
    //                     None => format!("props.{}", arg_name),
    //                 }
    //             }
    //             Token::Array(_) => format!("...props.{}", arg_name),
    //             Token::Tuple(t) => format!(
    //                 "...[{}]",
    //                 t.inners
    //                     .iter()
    //                     .enumerate()
    //                     .map(|(idx, t)| handle_arg_recursive(
    //                         &format!("props.{arg_name}[{idx}]"),
    //                         t,
    //                         handled_tokens
    //                     ))
    //                     .collect::<Vec<String>>()
    //                     .join(", ")
    //             ),
    //             _ => format!("props.{}", arg_name),
    //         }
    //     }
    //
    //     let calldata = system
    //         .inputs
    //         .iter()
    //         .map(|arg| handle_arg_recursive(&arg.0, &arg.1, handled_tokens))
    //         .collect::<Vec<String>>()
    //         .join(",\n                ");
    //
    //     format!(
    //         "
    //     // Call the `{system_name}` system with the specified Account and calldata
    //     const {system_name} = async (props: {{ account: Account{arg_sep}{args} }}) => {{
    //         try {{
    //             return await provider.execute(
    //                 props.account,
    //                 {{
    //                     contractName: contract_name,
    //                     entrypoint: \"{system_name}\",
    //                     calldata: [{calldata}],
    //                 }},
    //                 \"{namespace}\"
    //             );
    //         }} catch (error) {{
    //             console.error(\"Error executing {system_name}:\", error);
    //             throw error;
    //         }}
    //     }};
    //         ",
    //         // selector for execute
    //         system_name = system.name,
    //         // add comma if we have args
    //         arg_sep = if !args.is_empty() { ", " } else { "" },
    //         // formatted args to use our mapped types
    //         args = args,
    //         // calldata for execute
    //         calldata = calldata,
    //         namespace = TypescriptPlugin::get_namespace_from_tag(&namespace)
    //     )
    // }
    // //
    // //     // Formats a contract tag into a pretty contract name
    // //     // eg. dojo_examples-actions -> actions
    // //     fn formatted_contract_name(tag: &str) -> String {
    // //         naming::get_name_from_tag(tag)
    // //     }
    // //
    // //     fn get_namespace_from_tag(tag: &str) -> String {
    // //         tag.split('-').next().unwrap_or(tag).to_string()
    // //     }
    // //
    // // Handles a contract definition and its underlying systems
    // // Will format the contract into a C# class and
    // // all systems into C# methods
    // // Handled tokens should be a list of all structs and enums used by the contract
    // fn handle_contracts(
    //     &self,
    //     contracts: &[&DojoContract],
    //     handled_tokens: &[Composite],
    // ) -> String {
    //     let mut out = String::new();
    //     out += TypescriptPlugin::generated_header().as_str();
    //     out += "import { Account, byteArray } from \"starknet\";\n";
    //     out += "import { DojoProvider } from \"@dojoengine/core\";\n";
    //     out += "import * as models from \"./models.gen\";\n";
    //     out += "\n";
    //     out += "export type IWorld = Awaited<ReturnType<typeof setupWorld>>;";
    //
    //     out += "\n\n";
    //
    //     out += "export async function setupWorld(provider: DojoProvider) {";
    //
    //     for contract in contracts {
    //         let systems = contract
    //             .systems
    //             .iter()
    //             .filter(|system| {
    //                 let name = system.to_function().unwrap().name.as_str();
    //                 ![
    //                     "contract_name",
    //                     "namespace",
    //                     "tag",
    //                     "name_hash",
    //                     "selector",
    //                     "dojo_init",
    //                     "namespace_hash",
    //                 ]
    //                 .contains(&name)
    //             })
    //             .map(|system| {
    //                 TypescriptPlugin::format_system(
    //                     system.to_function().unwrap(),
    //                     handled_tokens,
    //                     contract.tag.clone(),
    //                 )
    //             })
    //             .collect::<Vec<String>>()
    //             .join("\n\n    ");
    //
    //         out += &format!(
    //             "
    // // System definitions for `{}` contract
    // function {}() {{
    //     const contract_name = \"{}\";
    //
    //     {}
    //
    //     return {{
    //         {}
    //     }};
    // }}
    //  ",
    //             contract.tag,
    //             // capitalize contract name
    //             TypescriptPlugin::formatted_contract_name(&contract.tag),
    //             TypescriptPlugin::formatted_contract_name(&contract.tag),
    //             systems,
    //             contract
    //                 .systems
    //                 .iter()
    //                 .filter(|system| {
    //                     let name = system.to_function().unwrap().name.as_str();
    //                     ![
    //                         "contract_name",
    //                         "namespace",
    //                         "tag",
    //                         "name_hash",
    //                         "selector",
    //                         "dojo_init",
    //                         "namespace_hash",
    //                     ]
    //                     .contains(&name)
    //                 })
    //                 .map(|system| { system.to_function().unwrap().name.to_string() })
    //                 .collect::<Vec<String>>()
    //                 .join(", ")
    //         );
    //     }
    //
    //     out += "
    // return {
    //     ";
    //
    //     out += &contracts
    //         .iter()
    //         .map(|c| {
    //             format!(
    //                 "{}: {}()",
    //                 TypescriptPlugin::formatted_contract_name(&c.tag),
    //                 TypescriptPlugin::formatted_contract_name(&c.tag)
    //             )
    //         })
    //         .collect::<Vec<String>>()
    //         .join(",\n        ");
    //
    //     out += "
    // };
    //  }\n";
    //
    //     out
    // }
}

#[async_trait]
impl BuiltinPlugin for TypescriptPlugin {
    async fn generate_code(&self, data: &DojoData) -> BindgenResult<HashMap<PathBuf, Vec<u8>>> {
        let mut out: HashMap<PathBuf, Vec<u8>> = HashMap::new();

        let code = self
            .writers
            .iter()
            .map(|writer| match writer.write(writer.get_path(), data) {
                Ok(c) => c,
                Err(e) => {
                    log::error!("Failed to generate code for typescript plugin: {e}");
                    ("".into(), Vec::new())
                }
            })
            .collect::<Vec<_>>();

        code.iter().for_each(|(path, code)| {
            if code.is_empty() {
                return;
            }
            out.insert(PathBuf::from(path), code.clone());
        });

        Ok(out)
    }
}
