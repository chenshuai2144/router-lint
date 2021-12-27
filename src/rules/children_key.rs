use crate::handler::{Handler, Traverse};

use super::{Context, LintRule, Program, ProgramRef};
use deno_ast::{
    swc::common::Spanned,
    view::{self as ast_view, NodeTrait},
};
use std::sync::Arc;

const MESSAGE: &str =
    "🚨 不应该使用 children 来配置子路由, children 已经废弃，请使用 routes 来代替！";

#[derive(Debug)]
pub struct ChildrenKey;

const CODE: &str = "no-use-children";

impl LintRule for ChildrenKey {
    fn code(&self) -> &'static str {
        CODE
    }

    fn lint_program<'view>(&self, _context: &mut Context<'view>, _program: ProgramRef<'view>) {
        unreachable!();
    }

    fn lint_program_with_ast_view(&self, context: &mut Context, program: Program<'_>) {
        ChildrenKeyHandler.traverse(program, context);
    }

    fn new() -> Arc<Self> {
        Arc::new(ChildrenKey)
    }
}

struct ChildrenKeyHandler;

impl Handler for ChildrenKeyHandler {
    fn object_lit(&mut self, object_lit: &ast_view::ObjectLit, ctx: &mut Context) {
        let obj_keys: Vec<String> = object_lit
            .children()
            .iter()
            .map(|obj_name| obj_name.children()[0].text().to_string())
            .collect();
        if obj_keys.contains(&String::from("children")) {
            ctx.add_diagnostic(object_lit.span(), CODE, MESSAGE);
        }
    }
}
