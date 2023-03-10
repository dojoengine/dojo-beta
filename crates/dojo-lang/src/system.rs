use std::collections::{HashMap, HashSet};

use cairo_lang_defs::ids::{ModuleItemId, SubmoduleId};
use cairo_lang_defs::plugin::{DynGeneratedFileAuxData, PluginGeneratedFile, PluginResult};
use cairo_lang_filesystem::ids::CrateId;
use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_semantic::patcher::{PatchBuilder, RewriteNode};
use cairo_lang_semantic::plugin::DynPluginAuxData;
use cairo_lang_syntax::node::ast::FunctionWithBody;
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::{ast, Terminal, TypedSyntaxNode};
use dojo_project::WorldConfig;
use smol_str::SmolStr;

use crate::plugin::DojoAuxData;
use crate::query::Query;

#[cfg(test)]
#[path = "system_test.rs"]
mod test;

/// Represents a declaration of a system.
pub struct SystemDeclaration {
    /// The id of the module that defines the system.
    pub submodule_id: SubmoduleId,
    pub name: SmolStr,
}

pub struct System {
    queries: Vec<Query>,
}

impl System {
    // self.imports
    //             .iter()
    //             .map(|import| {
    //                 RewriteNode::interpolate_patched(
    //                     "use super::$import$;",
    //                     HashMap::from([("import".to_string(), RewriteNode::Text(import.to_string()))]),
    //                 )
    //             })
    //             .collect()
    pub fn from_function(
        db: &dyn SyntaxGroup,
        world_config: WorldConfig,
        function_ast: ast::FunctionWithBody,
    ) -> PluginResult {
        let mut system = System { queries: vec![] };
        let name = function_ast.declaration(db).name(db).text(db);
        let mut rewrite_nodes = vec![];
        let mut query_nodes = vec![];

        let signature = function_ast.declaration(db).signature(db);
        let parameters = signature.parameters(db).elements(db);

        for param_ast in parameters.iter() {
            let type_ast = param_ast.type_clause(db).ty(db);

            if let ast::Expr::Path(path) = type_ast.clone() {
                let binding = path.elements(db);
                let last = binding.last().unwrap();
                match last {
                    ast::PathSegment::WithGenericArgs(segment) => {
                        let ty = segment.ident(db).text(db);
                        if ty == "Query" {
                            let query = Query::from_expr(db, type_ast.clone());
                            system.queries.push(query);
                            // import_nodes.extend(query.imports());
                            // query_nodes.extend(query.rewrite_nodes);
                            query_nodes.push(RewriteNode::interpolate_patched(
                                "
                            let $name$ = QueryTrait::$typename$::new();",
                                HashMap::from([
                                    (
                                        "name".to_string(),
                                        RewriteNode::new_trimmed(
                                            param_ast.name(db).as_syntax_node(),
                                        ),
                                    ),
                                    (
                                        "typename".to_string(),
                                        RewriteNode::new_trimmed(
                                            segment.generic_args(db).as_syntax_node(),
                                        ),
                                    ),
                                ]),
                            ));
                        }
                    }
                    ast::PathSegment::Simple(_segment) => {
                        // TODO: ...
                    }
                };
            }
            // if let Some(res) = handle_param(db, param, world_config) {
            //     import_nodes.extend(res.0);
            //     query_nodes.extend(res.1);
            // }
        }

        let flat_deps: HashSet<SmolStr> = HashSet::from_iter(
            system.queries.iter().map(|query| query.dependencies.clone()).flatten(),
        );
        let import_nodes = flat_deps
            .iter()
            .map(|dep| {
                RewriteNode::interpolate_patched(
                    "use super::$dep$;\n",
                    HashMap::from([("dep".to_string(), RewriteNode::Text(dep.to_string()))]),
                )
            })
            .collect();

        // self.rewrite_nodes.push(RewriteNode::interpolate_patched(
        //     "let $var_prefix$_ids = IWorldDispatcher { contract_address: \
        //      world_address \
        //      }.entities(starknet::contract_address_const::<$component_address$>());\
        //      \n",
        //     HashMap::from([
        //         ("var_prefix".to_string(), RewriteNode::Text(var_prefix)),
        //         ("component_address".to_string(), RewriteNode::Text(component_id)),
        //     ]),
        // ))

        let body_nodes = resolve_function_body_members(db, function_ast);
        rewrite_nodes.push(RewriteNode::interpolate_patched(
            "
                #[external]
                fn execute() {
                    let world_address = starknet::contract_address_const::<$world_address$>();
                    $query$
                    $body$
                }
            ",
            HashMap::from([
                ("body".to_string(), RewriteNode::new_modified(body_nodes)),
                ("query".to_string(), RewriteNode::new_modified(query_nodes)),
                (
                    "world_address".to_string(),
                    RewriteNode::Text(format!("{:#x}", world_config.address.unwrap_or_default())),
                ),
            ]),
        ));

        let mut builder = PatchBuilder::new(db);
        builder.add_modified(RewriteNode::interpolate_patched(
            "
                #[contract]
                mod $name$System {
                    use dojo::world;
                    use dojo::world::IWorldDispatcher;
                    use dojo::world::IWorldDispatcherTrait;
                    use dojo::query::QueryTrait;
                    $imports$
                    $body$
                }
            ",
            HashMap::from([
                ("name".to_string(), RewriteNode::Text(capitalize_first(name.to_string()))),
                ("imports".to_string(), RewriteNode::new_modified(import_nodes)),
                ("body".to_string(), RewriteNode::new_modified(rewrite_nodes)),
            ]),
        ));

        PluginResult {
            code: Some(PluginGeneratedFile {
                name: name.clone(),
                content: builder.code,
                aux_data: DynGeneratedFileAuxData::new(DynPluginAuxData::new(DojoAuxData {
                    patches: builder.patches,
                    components: vec![],
                    systems: vec![format!("{}System", capitalize_first(name.to_string())).into()],
                })),
            }),
            diagnostics: vec![],
            remove_original_item: true,
        }
    }
}

fn resolve_function_body_members(
    db: &dyn SyntaxGroup,
    function_ast: FunctionWithBody,
) -> Vec<RewriteNode> {
    function_ast
        .body(db)
        .statements(db)
        .elements(db)
        .iter()
        .map(|statement| resolve_statement(db, statement.clone()))
        .into_iter()
        .flatten()
        .collect()
}

fn resolve_statement(db: &dyn SyntaxGroup, statement_ast: ast::Statement) -> Vec<RewriteNode> {
    match statement_ast {
        ast::Statement::Let(statement_let) => {
            let expr = statement_let.rhs(db);
            let expr_nodes = resolve_expr(db, expr);
            vec![RewriteNode::interpolate_patched(
                "let $pattern$ = $expr$
                ",
                HashMap::from([
                    (
                        "pattern".to_string(),
                        RewriteNode::Copied(statement_let.pattern(db).as_syntax_node()),
                    ),
                    ("expr".to_string(), RewriteNode::new_modified(expr_nodes)),
                ]),
            )]
        }
        ast::Statement::Expr(statement_expr) => resolve_expr(db, statement_expr.expr(db)),
        ast::Statement::Return(statement_return) => {
            let expr = statement_return.expr(db);
            let expr_nodes = resolve_expr(db, expr);
            vec![RewriteNode::interpolate_patched(
                "return ($expr$);
                ",
                HashMap::from([("expr".to_string(), RewriteNode::new_modified(expr_nodes))]),
            )]
        }
        ast::Statement::Missing(statement_missing) => {
            vec![RewriteNode::new_trimmed(statement_missing.as_syntax_node())]
        }
    }
}

fn resolve_expr(db: &dyn SyntaxGroup, expr_ast: ast::Expr) -> Vec<RewriteNode> {
    match expr_ast {
        ast::Expr::Path(_path) => {
            unimplemented!("path expressions are not supported yet")
        }
        ast::Expr::Parenthesized(expr_paren) => resolve_expr(db, expr_paren.expr(db)),
        ast::Expr::Tuple(expr_tuple) => expr_tuple
            .expressions(db)
            .elements(db)
            .iter()
            .map(|expr| resolve_expr(db, expr.clone()))
            .into_iter()
            .flatten()
            .collect(),
        ast::Expr::FunctionCall(expr_fn) => {
            vec![RewriteNode::interpolate_patched(
                "super::$pattern$($args$);
                ",
                HashMap::from([
                    (
                        "pattern".to_string(),
                        RewriteNode::new_trimmed(expr_fn.path(db).as_syntax_node()),
                    ),
                    (
                        "args".to_string(),
                        RewriteNode::Copied(expr_fn.arguments(db).args(db).as_syntax_node()),
                    ),
                ]),
            )]
        }
        ast::Expr::StructCtorCall(_expr_struct) => {
            unimplemented!("match struct constructor are not yet supported")
        }
        ast::Expr::Block(_expr_block) => {
            unimplemented!("match block are not yet supported")
        }
        ast::Expr::Match(_expr_match) => {
            unimplemented!("match expressions are not yet supported")
        }
        ast::Expr::If(expr_if) => resolve_if(db, expr_if),
        _ => vec![RewriteNode::interpolate_patched(
            "$pattern$;
            ",
            HashMap::from([(
                "pattern".to_string(),
                RewriteNode::Copied(expr_ast.as_syntax_node()),
            )]),
        )],
    }
}

fn resolve_if(db: &dyn SyntaxGroup, expr_if: ast::ExprIf) -> Vec<RewriteNode> {
    let body_nodes = expr_if
        .if_block(db)
        .statements(db)
        .elements(db)
        .iter()
        .map(|statement| resolve_statement(db, statement.clone()))
        .into_iter()
        .flatten()
        .collect();
    let else_nodes = match expr_if.else_clause(db) {
        ast::OptionElseClause::ElseClause(else_clause) => match else_clause.else_block_or_if(db) {
            ast::BlockOrIf::If(else_if) => resolve_if(db, else_if),
            ast::BlockOrIf::Block(else_block) => else_block
                .statements(db)
                .elements(db)
                .iter()
                .map(|statement| resolve_statement(db, statement.clone()))
                .into_iter()
                .flatten()
                .collect(),
        },
        ast::OptionElseClause::Empty(_) => vec![],
    };
    vec![RewriteNode::interpolate_patched(
        "if $condition$ {
            $body$
        } else {
            $else$
        }
        ",
        HashMap::from([
            ("condition".to_string(), RewriteNode::Copied(expr_if.condition(db).as_syntax_node())),
            ("body".to_string(), RewriteNode::new_modified(body_nodes)),
            ("else".to_string(), RewriteNode::new_modified(else_nodes)),
        ]),
    )]
}

fn capitalize_first(s: String) -> String {
    let mut chars = s.chars();
    let mut capitalized = chars.next().unwrap().to_uppercase().to_string();
    capitalized.extend(chars);
    capitalized
}

/// Finds the inline modules annotated as systems in the given crate_ids and
/// returns the corresponding SystemDeclarations.
pub fn find_systems(db: &dyn SemanticGroup, crate_ids: &[CrateId]) -> Vec<SystemDeclaration> {
    let mut systems = vec![];
    for crate_id in crate_ids {
        let modules = db.crate_modules(*crate_id);
        for module_id in modules.iter() {
            let generated_file_infos =
                db.module_generated_file_infos(*module_id).unwrap_or_default();

            for generated_file_info in generated_file_infos.iter().skip(1) {
                let Some(generated_file_info) = generated_file_info else { continue; };
                let Some(mapper) = generated_file_info.aux_data.0.as_any(
                ).downcast_ref::<DynPluginAuxData>() else { continue; };
                let Some(aux_data) = mapper.0.as_any(
                ).downcast_ref::<DojoAuxData>() else { continue; };

                for name in &aux_data.systems {
                    if let Ok(Some(ModuleItemId::Submodule(submodule_id))) =
                        db.module_item_by_name(*module_id, name.clone())
                    {
                        systems.push(SystemDeclaration { name: name.clone(), submodule_id });
                    } else {
                        panic!("System `{name}` was not found.");
                    }
                }
            }
        }
    }
    systems
}
