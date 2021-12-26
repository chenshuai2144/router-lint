use crate::handler::Handler;

use super::{Context, LintRule, Program, ProgramRef};
use deno_ast::view as ast_view;
use std::sync::Arc;

const MESSAGE: &str = "🚨 不应该使用 children 来配置子路由";

fn gen_diagnostic_children_key(node: &RoutePathObj, source_file_name: String) -> RouteDiagnostic {
    let mut line_text = Vec::new();
    line_text.push(node.node_source.to_string());

    let mut display_position = Vec::new();
    display_position.push(node.display_position.clone());
    let route_diagnostic = RouteDiagnostic {
        specifier: node.path.clone(),
        display_position: display_position,
        kind: RouteSyntaxError::DeprecatedChildren,
        source_file_name: source_file_name,
    };
    route_diagnostic
}

fn is_warning_children_key_router(router: RoutePathObj) -> bool {
    if router.obj_keys.contains(&String::from("children")) {
        return true;
    }
    false
}

fn print_diagnostic_children_key_router(diagnostic: &RouteDiagnostic) {
    println!("🚨 不应该使用 children 来配置子路由： ",);
    for line_and_column in &diagnostic.display_position {
        println!(
            "   ---> {}:{}:{} 的 {}",
            diagnostic.source_file_name,
            line_and_column.line,
            line_and_column.column,
            line_and_column.router_source_code
        );
    }
    println!("");
    println!("children 已经废弃，请属于 routes 来代替！",);
}

pub struct ChildrenKey;

const CODE: &str = "no-use-children";

impl LintRule for ChildrenKey {
    fn code(&self) -> &'static str {
        CODE
    }

    fn lint_program<'view>(&self, _context: &mut Context<'view>, _program: ProgramRef<'view>) {
        unreachable!();
    }

    fn lint_program_with_ast_view(&self, context: &mut Context, program: Program) {}

    fn new() -> Arc<Self> {
        Arc::new(ChildrenKey)
    }
}

struct ChildrenKeyHandler;

impl Handler for ChildrenKeyHandler {
    fn with_stmt(&mut self, with_stmt: &ast_view::WithStmt, ctx: &mut Context) {
        ctx.add_diagnostic(with_stmt.span(), CODE, MESSAGE);
    }
}
