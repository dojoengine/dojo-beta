use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_semantic::patcher::RewriteNode;
use cairo_lang_syntax::node::ast::PathSegmentSimple;
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::{ast, Terminal};
use dojo_world::manifest::Dependency;
use smol_str::SmolStr;

pub mod entities;
pub mod entity;
pub mod execute;
pub mod set;
pub mod uuid;

const CAIRO_ERR_MSG_LEN: usize = 31;

pub trait CommandTrait {
    fn from_ast(
        db: &dyn SyntaxGroup,
        let_pattern: Option<ast::Pattern>,
        command_ast: ast::ExprFunctionCall,
    ) -> Self;

    fn rewrite_nodes(&self) -> Vec<RewriteNode>;
    fn diagnostics(&self) -> Vec<PluginDiagnostic>;
}

pub trait CommandMacroTrait: Into<Command> {
    fn from_ast(
        db: &dyn SyntaxGroup,
        let_pattern: Option<ast::Pattern>,
        command_ast: ast::ExprInlineMacro,
    ) -> Self;
}

#[derive(Clone)]
pub struct CommandData {
    rewrite_nodes: Vec<RewriteNode>,
    diagnostics: Vec<PluginDiagnostic>,
}

impl CommandData {
    pub fn new() -> Self {
        CommandData { rewrite_nodes: vec![], diagnostics: vec![] }
    }
}
pub struct Command {
    pub rewrite_nodes: Vec<RewriteNode>,
    pub diagnostics: Vec<PluginDiagnostic>,
    pub component_deps: Vec<Dependency>,
}

impl Command {
    pub fn with_data(data: CommandData) -> Self {
        Command {
            rewrite_nodes: data.rewrite_nodes,
            diagnostics: data.diagnostics,
            component_deps: vec![],
        }
    }

    /// With component dependencies
    pub fn with_cmp_deps(data: CommandData, component_deps: Vec<Dependency>) -> Self {
        Command { rewrite_nodes: data.rewrite_nodes, diagnostics: data.diagnostics, component_deps }
    }

    pub fn from_ast(
        db: &dyn SyntaxGroup,
        let_pattern: Option<ast::Pattern>,
        command_ast: ast::ExprFunctionCall,
    ) -> Self {
        let mut command =
            Command { rewrite_nodes: vec![], diagnostics: vec![], component_deps: vec![] };

        match command_name(db, command_ast.clone()).as_str() {
            "entities" => {
                let sc = entities::EntitiesCommand::from_ast(db, let_pattern, command_ast);
                command.rewrite_nodes.extend(sc.rewrite_nodes());
                command.diagnostics.extend(sc.diagnostics());
                command.component_deps.extend(sc.components);
            }
            "execute" => {
                let sc = execute::ExecuteCommand::from_ast(db, let_pattern, command_ast);
                command.rewrite_nodes.extend(sc.rewrite_nodes());
                command.diagnostics.extend(sc.diagnostics());
            }
            _ => {}
        }

        command
    }
}

pub fn command_name(db: &dyn SyntaxGroup, command_ast: ast::ExprFunctionCall) -> SmolStr {
    let elements = command_ast.path(db).elements(db);
    let segment = elements.last().unwrap();
    if let ast::PathSegment::Simple(method) = segment {
        method.ident(db).text(db)
    } else if let ast::PathSegment::WithGenericArgs(generic) = segment {
        generic.ident(db).text(db)
    } else {
        SmolStr::new("")
    }
}

pub fn macro_name(db: &dyn SyntaxGroup, macro_ast: ast::ExprInlineMacro) -> SmolStr {
    let elements = macro_ast.path(db).elements(db);
    let segment = elements.last().unwrap();
    match segment {
        ast::PathSegment::Simple(method) => method.ident(db).text(db),
        _ => panic!("Macro's name must be a simple identifier!"),
    }
}

pub fn ast_arg_to_expr(db: &dyn SyntaxGroup, arg: &ast::Arg) -> Option<ast::Expr> {
    match arg.arg_clause(db) {
        ast::ArgClause::Unnamed(clause) => Some(clause.value(db)),
        _ => None,
    }
}

fn ast_arg_to_path_segment_simple(
    db: &dyn SyntaxGroup,
    arg: &ast::Arg,
) -> Option<PathSegmentSimple> {
    if let Some(ast::Expr::Path(path)) = ast_arg_to_expr(db, arg) {
        if path.elements(db).len() != 1 {
            return None;
        }
        if let Some(ast::PathSegment::Simple(segment)) = path.elements(db).last() {
            return Some(segment.clone());
        }
    }
    None
}

pub fn context_arg_as_path_segment_simple_or_panic(
    db: &dyn SyntaxGroup,
    context: &ast::Arg,
) -> PathSegmentSimple {
    ast_arg_to_path_segment_simple(db, context).expect("Context must be a simple literal!")
}
