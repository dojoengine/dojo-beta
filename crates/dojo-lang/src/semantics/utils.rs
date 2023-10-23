use std::collections::HashMap;

use cairo_lang_compiler::db::RootDatabase;
use cairo_lang_defs::db::DefsGroup;
use cairo_lang_defs::ids::{FunctionWithBodyId, LookupItemId, ModuleId, ModuleItemId};
use cairo_lang_filesystem::ids::FileId;
use cairo_lang_lowering::db::LoweringGroup;
use cairo_lang_lowering::ids::{self as low, SemanticFunctionWithBodyIdEx};
use cairo_lang_lowering::{Statement, StatementCall};
use cairo_lang_semantic as semantic;
use cairo_lang_syntax::node::ast::{self, Expr};
use cairo_lang_syntax::node::{SyntaxNode, TypedSyntaxNode};
use semantic::db::SemanticGroup;
use semantic::diagnostic::SemanticDiagnostics;
use semantic::expr::compute::{ComputationContext, Environment};
use semantic::expr::inference::InferenceId;
use semantic::items::function_with_body::SemanticExprLookup;
use semantic::resolve::Resolver;
use semantic::FunctionId;

use crate::inline_macros::utils::WriterLookupDetails;

pub fn find_module_writes(
    db: &RootDatabase,
    module_id: &ModuleId,
    module_writers: Option<&HashMap<String, Vec<WriterLookupDetails>>>,
) -> Vec<String> {
    let mut components: HashMap<String, bool> = HashMap::new();
    // Does the module have writers?
    if let Some(module_writers) = module_writers {
        if let Ok(file_ids) = db.module_files(*module_id) {
            let file_id = *file_ids.iter().last().unwrap();
            // let file_id = *file_ids.iter().next().unwrap();

            // Get module fn ids. And generate lookup hashmap
            if let Ok(module_fns) = db.module_free_functions_ids(*module_id) {
                for fn_id in module_fns.iter() {
                    find_function_writes(
                        db,
                        module_id,
                        file_id,
                        module_writers,
                        FunctionWithBodyId::Free(*fn_id),
                        &mut components,
                    );
                }
            }
            if let Ok(module_impls) = db.module_impls_ids(*module_id) {
                for module_impl_id in module_impls.iter() {
                    if let Ok(module_fns) = db.impl_functions(*module_impl_id) {
                        for (_, fn_id) in module_fns.iter() {
                            find_function_writes(
                                db,
                                module_id,
                                file_id,
                                module_writers,
                                FunctionWithBodyId::Impl(*fn_id),
                                &mut components,
                            );
                        }
                    }
                }
            }
        }
    }

    components.into_iter().map(|(component, _)| component).collect()
}

pub fn find_function_writes(
    db: &RootDatabase,
    _module_id: &ModuleId,
    file_id: FileId,
    module_writers: &HashMap<String, Vec<WriterLookupDetails>>,
    fn_id: FunctionWithBodyId,
    components: &mut HashMap<String, bool>,
) {
    let fn_name: String = fn_id.name(db).into();
    // let fn_expr = db.expr_semantic(fn_id, fn_body.body_expr);
    if let Some(module_fn_writers) = module_writers.get(&fn_name) {
        // This functions has writers
        // Do stuff with the writers

        let resolver = function_resolver(db, fn_id);

        let mut diagnostics = SemanticDiagnostics::new(file_id);

        let mut _ctx = semantic_computation_ctx(db, fn_id, resolver, &mut diagnostics);

        for writer_lookup in module_fn_writers.iter() {
            match writer_lookup {
                WriterLookupDetails::StructCtor(expr) => {
                    let component = expr.path(db).as_syntax_node().get_text_without_trivia(db);
                    components.insert(component, true);
                }
                WriterLookupDetails::Path(expr_path, _expr_block) => {
                    let expr = Expr::Path(expr_path.clone());
                    // let expr_sem =
                    //     nearest_semantic_expr(db, expr_path.as_syntax_node(), fn_id).unwrap();

                    let fn_id_low = fn_id.lowered(db);

                    let flat_lowered = db.function_with_body_lowering(fn_id_low).unwrap();
                    for (_, flat_block) in flat_lowered.blocks.iter() {
                        let mut last_layout_fn_semantic: Option<FunctionId> = None;

                        for statement in flat_block.statements.iter() {
                            if let Statement::Call(statement_call) = statement {
                                if let low::FunctionLongId::Semantic(fn_id) =
                                    db.lookup_intern_lowering_function(statement_call.function)
                                {
                                    if let Ok(Some(conc_body_fn)) = fn_id.get_concrete(db).body(db)
                                    {
                                        let fn_body_id = conc_body_fn.function_with_body_id(db);
                                        let fn_name = fn_body_id.name(db);
                                        if fn_name == "set_entity" {
                                            if let Some(layout_fn) = last_layout_fn_semantic {
                                                match db.concrete_function_signature(layout_fn) {
                                                    Ok(signature) => {
                                                        if let Some(params) =
                                                            signature.params.get(0)
                                                        {
                                                            // looks like
                                                            // "@dojo_examples::models::Position"
                                                            let component = params.ty.format(db);
                                                            let component_segments =
                                                                component.split("::");
                                                            let component =
                                                                component_segments.last().expect(
                                                                    "layout signature params not \
                                                                     found",
                                                                );
                                                            components
                                                                .insert(component.into(), true);
                                                        }
                                                    }
                                                    Err(_) => {
                                                        eprintln!(
                                                            "error: could't get entity model(s)"
                                                        )
                                                    }
                                                }
                                            } else {
                                                eprintln!(
                                                    "type reference not found for set_entity"
                                                );
                                            }
                                        } else if fn_name == "layout" {
                                            last_layout_fn_semantic = Some(fn_id);
                                        }
                                    }
                                }
                            } else {
                                // println!("{:#?} {:?}", statement, statement.outputs());
                            }
                        }
                    }
                }
            }
        }
    }
}

fn function_resolver(db: &RootDatabase, fn_id: FunctionWithBodyId) -> Resolver {
    let resolver_data = match fn_id {
        FunctionWithBodyId::Free(fn_id) => {
            let interference = InferenceId::LookupItemDefinition(LookupItemId::ModuleItem(
                ModuleItemId::FreeFunction(fn_id),
            ));
            db.free_function_body_resolver_data(fn_id)
                .unwrap()
                .clone_with_inference_id(db, interference)
        }
        FunctionWithBodyId::Impl(fn_id) => {
            // let fn_ast = db.lookup_intern_impl_function(fn_id).1.lookup(db);
            let interference = InferenceId::LookupItemDefinition(LookupItemId::ImplFunction(fn_id));
            db.impl_function_body_resolver_data(fn_id)
                .unwrap()
                .clone_with_inference_id(db, interference)
        }
    };
    Resolver::with_data(db, resolver_data)
}
/// Returns the semantic expression for the current node.
fn nearest_semantic_expr(
    db: &dyn SemanticGroup,
    mut node: SyntaxNode,
    function_id: FunctionWithBodyId,
) -> Option<cairo_lang_semantic::Expr> {
    loop {
        let syntax_db = db.upcast();
        if ast::Expr::is_variant(node.kind(syntax_db)) {
            let expr_node = ast::Expr::from_syntax_node(syntax_db, node.clone());
            if let Ok(expr_id) = db.lookup_expr_by_ptr(function_id, expr_node.stable_ptr()) {
                let semantic_expr = db.expr_semantic(function_id, expr_id);
                return Some(semantic_expr);
            }
        }
        node = node.parent()?;
    }
}

pub fn semantic_computation_ctx<'a>(
    db: &'a RootDatabase,
    fn_id: FunctionWithBodyId,
    resolver: Resolver<'a>,
    diagnostics: &'a mut SemanticDiagnostics,
) -> ComputationContext<'a> {
    // db.function_resolver;

    ComputationContext::new(db, diagnostics, Some(fn_id), resolver, None, Environment::default())
    // maybe_compute_expr_semantic(ctx, syntax);
}
