use std::collections::HashMap;

use cairo_lang_defs::ids::{ModuleItemId, SubmoduleId};
use cairo_lang_defs::plugin::{
    DynGeneratedFileAuxData, PluginDiagnostic, PluginGeneratedFile, PluginResult,
};
use cairo_lang_filesystem::ids::CrateId;
use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_semantic::patcher::{PatchBuilder, RewriteNode};
use cairo_lang_semantic::plugin::DynPluginAuxData;
use cairo_lang_syntax::node::ast::MaybeModuleBody;
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::{ast, Terminal, TypedSyntaxNode};
use dojo_project::WorldConfig;
use smol_str::SmolStr;

use crate::commands::Command;
use crate::plugin::DojoAuxData;

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
    world_config: WorldConfig,
    diagnostics: Vec<PluginDiagnostic>,
}

impl System {
    pub fn from_module(
        db: &dyn SyntaxGroup,
        world_config: WorldConfig,
        module_ast: ast::ItemModule,
    ) -> PluginResult {
        let name = module_ast.name(db).text(db);
        let mut system = System { world_config, diagnostics: vec![] };

        if let MaybeModuleBody::Some(body) = module_ast.body(db) {
            let body_nodes = body
                .items(db)
                .elements(db)
                .iter()
                .flat_map(|el| {
                    if let ast::Item::FreeFunction(fn_ast) = el {
                        if fn_ast.declaration(db).name(db).text(db).to_string() == "execute" {
                            return system.from_function(db, fn_ast.clone());
                        }
                    }

                    vec![RewriteNode::Copied(el.as_syntax_node())]
                })
                .collect();

            let mut builder = PatchBuilder::new(db);
            builder.add_modified(RewriteNode::interpolate_patched(
                "
                #[contract]
                mod $name$ {
                    use dojo::world;
                    use dojo::interfaces::IWorldDispatcher;
                    use dojo::interfaces::IWorldDispatcherTrait;
                    use dojo::storage::key::StorageKey;
                    use dojo::storage::key::StorageKeyTrait;
                    use dojo::storage::key::Felt252IntoStorageKey;
                    use dojo::storage::key::TupleSize1IntoStorageKey;
                    use dojo::storage::key::TupleSize2IntoStorageKey;
                    use dojo::storage::key::TupleSize3IntoStorageKey;
                    use dojo::storage::key::TupleSize1IntoPartitionedStorageKey;
                    use dojo::storage::key::TupleSize2IntoPartitionedStorageKey;
                    use dojo::storage::key::ContractAddressIntoStorageKey;

                    #[view]
                    fn name() -> felt252 {
                        '$name$'
                    }

                    $body$
                }
                ",
                HashMap::from([
                    ("name".to_string(), RewriteNode::Text(name.to_string())),
                    ("body".to_string(), RewriteNode::new_modified(body_nodes)),
                ]),
            ));

            return PluginResult {
                code: Some(PluginGeneratedFile {
                    name: name.clone(),
                    content: builder.code,
                    aux_data: DynGeneratedFileAuxData::new(DynPluginAuxData::new(DojoAuxData {
                        patches: builder.patches,
                        components: vec![],
                        systems: vec![name],
                    })),
                }),
                diagnostics: system.diagnostics,
                remove_original_item: true,
            };
        }

        PluginResult::default()
    }

    pub fn from_function(
        &mut self,
        db: &dyn SyntaxGroup,
        function_ast: ast::FunctionWithBody,
    ) -> Vec<RewriteNode> {
        let mut rewrite_nodes = vec![];

        let signature = function_ast.declaration(db).signature(db);

        let body_nodes: Vec<RewriteNode> = function_ast
            .body(db)
            .statements(db)
            .elements(db)
            .iter()
            .flat_map(|statement| self.handle_statement(db, statement.clone()))
            .collect();

        rewrite_nodes.push(RewriteNode::interpolate_patched(
            "
                #[external]
                fn execute(world_address: starknet::ContractAddress, $parameters$) {
                    $body$
                }
            ",
            HashMap::from([
                (
                    "parameters".to_string(),
                    RewriteNode::new_trimmed(signature.parameters(db).as_syntax_node()),
                ),
                (
                    "world_address".to_string(),
                    RewriteNode::Text(format!(
                        "{:#x}",
                        self.world_config.address.unwrap_or_default()
                    )),
                ),
                ("body".to_string(), RewriteNode::new_modified(body_nodes)),
            ]),
        ));

        rewrite_nodes
    }

    fn handle_statement(
        &mut self,
        db: &dyn SyntaxGroup,
        statement_ast: ast::Statement,
    ) -> Vec<RewriteNode> {
        match statement_ast.clone() {
            ast::Statement::Let(statement_let) => {
                if let ast::Expr::FunctionCall(expr_fn) = statement_let.rhs(db) {
                    if let Some(rewrite_nodes) =
                        self.handle_expr(db, Some(statement_let.pattern(db)), expr_fn)
                    {
                        return rewrite_nodes;
                    }
                }
            }
            ast::Statement::Expr(expr) => {
                if let ast::Expr::FunctionCall(expr_fn) = expr.expr(db) {
                    if let Some(rewrite_nodes) = self.handle_expr(db, None, expr_fn) {
                        return rewrite_nodes;
                    }
                }
            }
            _ => {}
        }

        vec![RewriteNode::Copied(statement_ast.as_syntax_node())]
    }

    fn handle_expr(
        &mut self,
        db: &dyn SyntaxGroup,
        var_name: Option<ast::Pattern>,
        expr_fn: ast::ExprFunctionCall,
    ) -> Option<Vec<RewriteNode>> {
        let elements = expr_fn.path(db).elements(db);
        let segment = elements.first().unwrap();
        match segment {
            ast::PathSegment::WithGenericArgs(segment_genric) => {
                if segment_genric.ident(db).text(db).as_str() == "commands" {
                    let command = Command::from_ast(db, var_name, expr_fn);
                    self.diagnostics.extend(command.diagnostics);
                    return Some(command.rewrite_nodes);
                }
            }
            ast::PathSegment::Simple(segment_simple) => {
                if segment_simple.ident(db).text(db).as_str() == "commands" {
                    let command = Command::from_ast(db, var_name, expr_fn);
                    self.diagnostics.extend(command.diagnostics);
                    return Some(command.rewrite_nodes);
                }
            }
        }

        None
    }
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
