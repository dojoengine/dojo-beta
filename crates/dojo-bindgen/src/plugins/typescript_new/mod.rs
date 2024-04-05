use std::collections::HashMap;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use cainome::parser::tokens::{Composite, CompositeType, Function};
use convert_case::Casing;

use crate::error::BindgenResult;
use crate::plugins::BuiltinPlugin;
use crate::{DojoContract, DojoData, DojoModel};

pub struct TypeScriptV2Plugin {}

impl TypeScriptV2Plugin {
    pub fn new() -> Self {
        Self {}
    }

    // Maps cairo types to TypeScript defined types
    fn map_type(type_name: &str) -> String {
        match type_name {
            "bool" => "boolean".to_string(),
            "u8" => "number".to_string(),
            "u16" => "number".to_string(),
            "u32" => "number".to_string(),
            "u64" => "bigint".to_string(),
            "u128" => "bigint".to_string(),
            "u256" => "bigint".to_string(),
            "usize" => "number".to_string(),
            "felt252" => "string".to_string(),
            "ClassHash" => "string".to_string(),
            "ContractAddress" => "string".to_string(),

            _ => type_name.to_string(),
        }
    }

    fn generate_header() -> String {
        format!(
            "// Generated by dojo-bindgen on {}. Do not modify this file manually.\n",
            chrono::Utc::now().to_rfc2822()
        )
    }

    fn generate_imports() -> String {
        "import { Account } from \"starknet\";
import {
    Clause,
    Client,
    ModelClause,
    createClient,
    extractQueryFromResult,
    valueToToriiValueAndOperator,
} from \"@dojoengine/torii-client\";
import { LOCAL_KATANA, createManifestFromJson } from \"@dojoengine/core\";"
            .to_string()
    }

    fn generate_query_types(models: &[&DojoModel]) -> String {
        let mut query_fields = Vec::new();
        let mut result_mapping = Vec::new();
        let mut name_map = Vec::new();

        for model in models {
            query_fields.push(format!(
                "{}: ModelClause<{}>;",
                model.name.to_case(convert_case::Case::Camel),
                model.name
            ));

            result_mapping.push(format!(
                "{}: {};",
                model.name.to_case(convert_case::Case::Camel),
                model.name
            ));

            name_map.push(format!(
                "{}: \"{}\"",
                model.name.to_case(convert_case::Case::Camel),
                model.name
            ));
        }

        format!(
            "type Query = Partial<{{
    {query_fields}
}}>;

type ResultMapping = {{
    {result_mapping}
}};

const nameMap = {{
    {name_map}
}};

type QueryResult<T extends Query> = {{
    [K in keyof T]: K extends keyof ResultMapping ? ResultMapping[K] : never;
}};

// Only supports a single model for now, since torii doesn't support multiple models
// And inside that single model, there's only support for a single query.
function convertQueryToToriiClause(query: Query): Clause | undefined {{
    const [model, clause] = Object.entries(query)[0];

    if (Object.keys(clause).length === 0) {{
        return undefined;
    }}

    const clauses: Clause[] = Object.entries(clause).map(([key, value]) => {{
        return {{
            Member: {{
                model: nameMap[model as keyof typeof nameMap],
                member: key,
                ...valueToToriiValueAndOperator(value),
            }},
        }} satisfies Clause;
    }});

    return clauses[0];
}}",
            query_fields = query_fields.join("\n    "),
            result_mapping = result_mapping.join("\n    "),
            name_map = name_map.join(",\n    ")
        )
    }

    fn generate_model_types(models: &[&DojoModel], handled_tokens: &mut Vec<Composite>) -> String {
        let mut out = String::new();

        for model in models {
            let tokens = &model.tokens;

            for token in &tokens.enums {
                handled_tokens.push(token.to_composite().unwrap().to_owned());
            }
            for token in &tokens.structs {
                handled_tokens.push(token.to_composite().unwrap().to_owned());
            }

            let mut structs = tokens.structs.to_owned();
            structs.sort_by(|a, b| {
                if a.to_composite()
                    .unwrap()
                    .inners
                    .iter()
                    .any(|field| field.token.type_name() == b.type_name())
                {
                    std::cmp::Ordering::Greater
                } else {
                    std::cmp::Ordering::Less
                }
            });

            for token in &structs {
                out += TypeScriptV2Plugin::format_struct(
                    token.to_composite().unwrap(),
                    handled_tokens,
                )
                .as_str();
            }

            for token in &tokens.enums {
                out += TypeScriptV2Plugin::format_enum(token.to_composite().unwrap()).as_str();
            }

            out += "\n";
        }

        out
    }

    fn generate_base_calls_class() -> String {
        "class BaseCalls {
    contractAddress: string;
    account?: Account;

    constructor(contractAddress: string, account?: Account) {
        this.account = account;
        this.contractAddress = contractAddress;
    }

    async execute(entrypoint: string, calldata: any[] = []): Promise<void> {
        if (!this.account) {
            throw new Error(\"No account set to interact with dojo_starter\");
        }

        await this.account.execute(
            {
                contractAddress: this.contractAddress,
                entrypoint,
                calldata,
            },
            undefined,
            {
                maxFee: 0,
            }
        );
    }
}
"
        .to_string()
    }

    fn generate_contracts(contracts: &[&DojoContract], handled_tokens: &[Composite]) -> String {
        let mut out = String::new();

        for contract in contracts {
            let systems = contract
                .systems
                .iter()
                .map(|system| {
                    TypeScriptV2Plugin::format_system(system.to_function().unwrap(), handled_tokens)
                })
                .collect::<Vec<String>>()
                .join("\n\n    ");

            out += &format!(
                "class {}Calls extends BaseCalls {{
    constructor(contractAddress: string, account?: Account) {{
        super(contractAddress, account);
    }}
    
    {}
}}
",
                TypeScriptV2Plugin::formatted_contract_name(&contract.qualified_path)
                    .to_case(convert_case::Case::Pascal),
                systems,
            );
        }

        out
    }

    fn generate_initial_params(contracts: &[&DojoContract]) -> String {
        let system_addresses = contracts
            .iter()
            .map(|contract| {
                format!(
                    "{}Address: string;",
                    TypeScriptV2Plugin::formatted_contract_name(&contract.qualified_path)
                        .to_case(convert_case::Case::Camel)
                )
            })
            .collect::<Vec<String>>()
            .join("\n    ");

        format!(
            "type InitialParams = GeneralParams &
    (
        | {{
                rpcUrl: string;
                worldAddress: string;
                {system_addresses}
            }}
        | {{
                manifest: any;
            }}
    );"
        )
    }

    fn generate_world_class(world_name: &String, contracts: &[&DojoContract]) -> String {
        let mut out = String::new();

        out += "type GeneralParams = {
    toriiUrl: string;
    relayUrl: string;
    account?: Account;
};";

        out += "\n\n";

        out += TypeScriptV2Plugin::generate_initial_params(contracts).as_str();

        out += "\n\n";

        let system_properties = contracts
            .iter()
            .map(|contract| {
                format!(
                    "{camel_case_name}: {pascal_case_name}Calls;
    {camel_case_name}Address: string;",
                    camel_case_name =
                        TypeScriptV2Plugin::formatted_contract_name(&contract.qualified_path)
                            .to_case(convert_case::Case::Camel),
                    pascal_case_name =
                        TypeScriptV2Plugin::formatted_contract_name(&contract.qualified_path)
                            .to_case(convert_case::Case::Pascal)
                )
            })
            .collect::<Vec<String>>()
            .join("\n    ");

        let system_address_initializations = contracts
            .iter()
            .map(|contract| {
                format!(
                    "const {contract_name}Address = config.contracts.find(
                (contract) =>
                    contract.name === \"dojo_starter::systems::{contract_name}::{contract_name}\"
            )?.address;

            if (!{contract_name}Address) {{
                throw new Error(\"No {contract_name} contract found in the manifest\");
            }}

            this.{contract_name}Address = {contract_name}Address;",
                    contract_name =
                        TypeScriptV2Plugin::formatted_contract_name(&contract.qualified_path)
                            .to_case(convert_case::Case::Camel)
                )
            })
            .collect::<Vec<String>>()
            .join("\n    ");

        let system_address_initializations_from_params = contracts
            .iter()
            .map(|contract| {
                format!(
                    "this.{camel_case_name}Address = params.{camel_case_name}Address;",
                    camel_case_name =
                        TypeScriptV2Plugin::formatted_contract_name(&contract.qualified_path)
                            .to_case(convert_case::Case::Camel),
                )
            })
            .collect::<Vec<String>>()
            .join("\n    ");

        let system_initializations = contracts
            .iter()
            .map(|contract| {
                format!(
                    "this.{camel_case_name} = new {pascal_case_name}Calls(this.{camel_case_name}Address, this._account);",
                    camel_case_name =
                        TypeScriptV2Plugin::formatted_contract_name(&contract.qualified_path)
                            .to_case(convert_case::Case::Camel),
                    pascal_case_name =
                        TypeScriptV2Plugin::formatted_contract_name(&contract.qualified_path)
                            .to_case(convert_case::Case::Pascal)
                )
            })
            .collect::<Vec<String>>()
            .join("\n    ");

        let formatted_world_name = world_name.to_case(convert_case::Case::Pascal);

        out += &format!(
            "export class {formatted_world_name} {{
    rpcUrl: string;
    toriiUrl: string;
    toriiPromise: Promise<Client>;
    relayUrl: string;
    worldAddress: string;
    private _account?: Account;
    {system_properties}

    constructor(params: InitialParams) {{
        this.rpcUrl = LOCAL_KATANA;
        if (\"manifest\" in params) {{
            const config = createManifestFromJson(params.manifest);
            this.worldAddress = config.world.address;

            {system_address_initializations}
        }} else {{
            this.rpcUrl = params.rpcUrl;
            this.worldAddress = params.worldAddress;
            {system_address_initializations_from_params}
        }}
        this.toriiUrl = params.toriiUrl;
        this.relayUrl = params.relayUrl;
        this._account = params.account;
        {system_initializations}

        this.toriiPromise = createClient([], {{
            rpcUrl: this.rpcUrl,
            toriiUrl: this.toriiUrl,
            worldAddress: this.worldAddress,
            relayUrl: this.relayUrl,
        }});
    }}

    get account(): Account | undefined {{
        return this._account;
    }}

    set account(account: Account) {{
        this._account = account;
        {system_initializations}
    }}

    async findEntities<T extends Query>(query: T, limit = 10, offset = 0) {{
        const torii = await this.toriiPromise;

        const clause = convertQueryToToriiClause(query);

        const toriiResult = await torii.getEntities({{
            limit,
            offset,
            clause,
        }});

        const result = Object.values(toriiResult).map((entity: any) => {{
            return extractQueryFromResult<Query>(
                query,
                entity
            ) as QueryResult<T>;
        }});

        return result as QueryResult<T>[];
    }}

    async findEntity<T extends Query>(query: T) {{
        const result = await this.findEntities(query, 1);

        if (result.length === 0) {{
            return undefined;
        }}

        return result[0] as QueryResult<T>;
    }}
}}"
        );

        out
    }

    // Token should be a struct
    // This will be formatted into a TypeScript interface
    // using TypeScript defined types
    fn format_struct(token: &Composite, handled_tokens: &[Composite]) -> String {
        let mut native_fields = String::new();

        for field in &token.inners {
            let mapped = TypeScriptV2Plugin::map_type(field.token.type_name().as_str());
            if mapped == field.token.type_name() {
                let token = handled_tokens
                    .iter()
                    .find(|t| t.type_name() == field.token.type_name())
                    .unwrap_or_else(|| panic!("Token not found: {}", field.token.type_name()));
                if token.r#type == CompositeType::Enum {
                    native_fields += format!("{}: {};\n    ", field.name, mapped).as_str();
                } else {
                    native_fields +=
                        format!("{}: {};\n    ", field.name, field.token.type_name()).as_str();
                }
            } else {
                native_fields += format!("{}: {};\n    ", field.name, mapped).as_str();
            }
        }

        format!(
            "
// Type definition for `{path}` struct
export interface {name} {{
    {native_fields}
}}
",
            path = token.type_path,
            name = token.type_name(),
            native_fields = native_fields
        )
    }

    // Token should be an enum
    // This will be formatted into a C# enum
    // Enum is mapped using index of cairo enum
    fn format_enum(token: &Composite) -> String {
        let fields = token
            .inners
            .iter()
            .map(|field| format!("{},", field.name,))
            .collect::<Vec<String>>()
            .join("\n    ");

        format!(
            "
// Type definition for `{}` enum
export enum {} {{
    {}
}}
",
            token.type_path,
            token.type_name(),
            fields
        )
    }

    // Formats a system into a C# method used by the contract class
    // Handled tokens should be a list of all structs and enums used by the contract
    // Such as a set of referenced tokens from a model
    fn format_system(system: &Function, handled_tokens: &[Composite]) -> String {
        let args = system
            .inputs
            .iter()
            .map(|arg| {
                format!(
                    "{}: {}",
                    arg.0,
                    if TypeScriptV2Plugin::map_type(&arg.1.type_name()) == arg.1.type_name() {
                        arg.1.type_name()
                    } else {
                        TypeScriptV2Plugin::map_type(&arg.1.type_name())
                    }
                )
            })
            .collect::<Vec<String>>()
            .join(", ");

        let calldata = system
            .inputs
            .iter()
            .map(|arg| {
                let token = &arg.1;
                let type_name = &arg.0;

                match handled_tokens.iter().find(|t| t.type_name() == token.type_name()) {
                    Some(t) => {
                        // Need to flatten the struct members.
                        match t.r#type {
                            CompositeType::Struct => t
                                .inners
                                .iter()
                                .map(|field| format!("props.{}.{}", type_name, field.name))
                                .collect::<Vec<String>>()
                                .join(",\n                    "),
                            _ => type_name.to_string(),
                        }
                    }
                    None => type_name.to_string(),
                }
            })
            .collect::<Vec<String>>()
            .join(",\n                ");

        format!(
            "async {pretty_system_name}({args}): Promise<void> {{
        try {{
            await this.execute(\"{system_name}\", [{calldata}])
        }} catch (error) {{
            console.error(\"Error executing {pretty_system_name}:\", error);
            throw error;
        }}
    }}",
            pretty_system_name = system.name.to_case(convert_case::Case::Camel),
            // formatted args to use our mapped types
            args = args,
            system_name = system.name,
            // calldata for execute
            calldata = calldata
        )
    }

    // Formats a contract file path into a pretty contract name
    // eg. dojo_examples::actions::actions.json -> Actions
    fn formatted_contract_name(contract_file_name: &str) -> String {
        let contract_name =
            contract_file_name.split("::").last().unwrap().trim_end_matches(".json");
        contract_name.to_string()
    }
}

#[async_trait]
impl BuiltinPlugin for TypeScriptV2Plugin {
    async fn generate_code(&self, data: &DojoData) -> BindgenResult<HashMap<PathBuf, Vec<u8>>> {
        let mut out: HashMap<PathBuf, Vec<u8>> = HashMap::new();
        let mut handled_tokens = Vec::<Composite>::new();
        let models = data.models.values().collect::<Vec<_>>();
        let contracts = data.contracts.values().collect::<Vec<_>>();

        let output_path = Path::new(&format!("{}.ts", data.world.name)).to_owned();

        let mut code = String::new();
        code += TypeScriptV2Plugin::generate_header().as_str();
        code += TypeScriptV2Plugin::generate_imports().as_str();
        code += "\n";
        code += TypeScriptV2Plugin::generate_model_types(models.as_slice(), &mut handled_tokens)
            .as_str();
        code += "\n";
        code += TypeScriptV2Plugin::generate_base_calls_class().as_str();
        code += "\n";
        code +=
            TypeScriptV2Plugin::generate_contracts(contracts.as_slice(), &handled_tokens).as_str();
        code += "\n";
        code += TypeScriptV2Plugin::generate_query_types(models.as_slice()).as_str();
        code += "\n";
        code += TypeScriptV2Plugin::generate_world_class(&data.world.name, contracts.as_slice())
            .as_str();

        out.insert(output_path, code.as_bytes().to_vec());

        Ok(out)
    }
}
