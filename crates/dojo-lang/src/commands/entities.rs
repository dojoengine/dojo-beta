use std::collections::HashMap;

use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_semantic::patcher::RewriteNode;
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::{ast, Terminal, TypedSyntaxNode};
use sanitizer::StringSanitizer;
use smol_str::SmolStr;

use super::{CAIRO_ERR_MSG_LEN, CommandData, CommandTrait};

pub struct EntitiesCommand {
    query_id: String,
    data: CommandData,
}

impl CommandTrait for EntitiesCommand {
    fn from_ast(
        db: &dyn SyntaxGroup,
        let_pattern: Option<ast::Pattern>,
        command_ast: ast::ExprFunctionCall,
    ) -> Self {
        let mut query_id =
            StringSanitizer::from(let_pattern.unwrap().as_syntax_node().get_text(db));
        query_id.to_snake_case();
        let mut command = EntitiesCommand { query_id: query_id.get(), data: CommandData::new() };

        let partition =
            if let Some(partition) = command_ast.arguments(db).args(db).elements(db).first() {
                RewriteNode::new_trimmed(partition.as_syntax_node())
            } else {
                RewriteNode::Text("0".to_string())
            };

        command.data.rewrite_nodes.extend(
            find_components(db, &command_ast)
                .iter()
                .map(|component| {
                    RewriteNode::interpolate_patched(
                        "
                        let (__$query_id$_$query_subtype$_ids, __$query_id$_$query_subtype$_raw) = IWorldDispatcher { \
                            contract_address: world_address
                        }.entities(dojo_core::string::ShortStringTrait::new('$component$'), dojo_core::integer::u250Trait::new($partition$));
                        ",
                        HashMap::from([
                            (
                                "query_subtype".to_string(),
                                RewriteNode::Text(component.to_string().to_ascii_lowercase()),
                            ),
                            ("query_id".to_string(), RewriteNode::Text(command.query_id.clone())),
                            ("partition".to_string(), partition.clone()),
                            ("component".to_string(), RewriteNode::Text(component.to_string())),
                        ]),
                    )
                })
                .collect::<Vec<_>>(),
        );

        let entity_compontents = find_components(db, &command_ast);
        if entity_compontents.len() == 1 {
            // no need to call find_matching if querying only for 1 entity
            command.data.rewrite_nodes.push(
                RewriteNode::interpolate_patched(
                    "let __$query_subtype$_idxs = __$query_id$_$query_subtype$_ids;", 
                HashMap::from([
                    (
                        "query_subtype".to_string(),
                        RewriteNode::Text(entity_compontents[0].to_string().to_ascii_lowercase()),
                    ),
                    ("query_id".to_string(), RewriteNode::Text(command.query_id.clone())),
                ]))
            );
        } else { // handling up to 4 entities
            let mut variables: Vec<String> = Vec::with_capacity(4);
            let mut arguments: Vec<String> = Vec::with_capacity(4);

            for component in entity_compontents {
                variables.push(format!("mut __{}_ids_idxs", component.to_string().to_ascii_lowercase()));
                arguments.push(format!("__{}_{}_ids", String::from(command.query_id.as_str()), component.to_string().to_ascii_lowercase()));
            }
            variables.resize(4, String::from("_"));
            arguments.resize(4,String::from("Option::None(())"));

            command.data.rewrite_nodes.push(
                RewriteNode::Text(
                    format!("let ({}) = dojo_core::storage::utils::find_matching({});", variables.join(", "), arguments.join(", "))
                )
            )
        }

        command.data.rewrite_nodes.extend(
            find_components(db, &command_ast)
            .iter()
            .map(|component| {
                let mut deser_err_msg = format!("{} failed to deserialize", component.to_string());
                deser_err_msg.truncate(CAIRO_ERR_MSG_LEN);

                RewriteNode::interpolate_patched(
                    "
                    let mut __$query_subtype$s: Array<$component$> = ArrayTrait::new();
                    loop {
                        match __$query_subtype$_ids_idxs.pop_front() {
                            Option::Some(idx) => {
                                let entity = serde::Serde::<$component$>::deserialize(ref *(__$query_id$_$query_subtype$_raw[*idx])).expect('$deser_err_msg$');
                                __$query_subtype$s.append(entity);
                            },
                            Option::None(_) => {
                                break ();
                            }
                        };
                    };
                    ",
                    HashMap::from([
                        ("query_subtype".to_string(), RewriteNode::Text(component.to_string().to_ascii_lowercase())),
                        ("component".to_string(), RewriteNode::Text(component.to_string())),
                        ("query_id".to_string(), RewriteNode::Text(command.query_id.clone())),
                        ("deser_err_msg".to_string(), RewriteNode::Text(deser_err_msg)),
                    ])
                )
            })
            .collect::<Vec<_>>(),
        );

        let deser_entities: String = find_components(db, &command_ast)
        .iter()
        .map(|component| {
            format!("__{}s", component.to_string().to_ascii_lowercase())
        })
        .collect::<Vec<String>>()
        .join(", ");

        command.data.rewrite_nodes.push(
            RewriteNode::Text(format!("let {} = ({});\n", command.query_id.clone(), deser_entities))
        );

        command
    }

    fn rewrite_nodes(&self) -> Vec<RewriteNode> {
        self.data.rewrite_nodes.clone()
    }

    fn diagnostics(&self) -> Vec<PluginDiagnostic> {
        self.data.diagnostics.clone()
    }
}

pub fn find_components(db: &dyn SyntaxGroup, command_ast: &ast::ExprFunctionCall) -> Vec<SmolStr> {
    let mut components = vec![];
    if let ast::PathSegment::WithGenericArgs(generic) =
        command_ast.path(db).elements(db).first().unwrap()
    {
        for arg in generic.generic_args(db).generic_args(db).elements(db) {
            if let ast::GenericArg::Expr(expr) = arg {
                components.extend(find_components_inner(db, expr.value(db)));
            }
        }
    }
    components
}

fn find_components_inner(db: &dyn SyntaxGroup, expression: ast::Expr) -> Vec<SmolStr> {
    let mut components = vec![];
    match expression {
        ast::Expr::Tuple(tuple) => {
            for element in tuple.expressions(db).elements(db) {
                components.extend(find_components_inner(db, element));
            }
        }
        ast::Expr::Parenthesized(parenthesized) => {
            components.extend(find_components_inner(db, parenthesized.expr(db)));
        }
        ast::Expr::Path(path) => match path.elements(db).last().unwrap() {
            ast::PathSegment::WithGenericArgs(segment) => {
                let generic = segment.generic_args(db);

                for param in generic.generic_args(db).elements(db) {
                    if let ast::GenericArg::Expr(expr) = param {
                        components.extend(find_components_inner(db, expr.value(db)));
                    }
                }
            }
            ast::PathSegment::Simple(segment) => {
                components.push(segment.ident(db).text(db));
            }
        },
        _ => {
            unimplemented!(
                "Unsupported expression type: {}",
                expression.as_syntax_node().get_text(db)
            );
        }
    }

    components
}
