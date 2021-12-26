use crate::handler::{Handler, Traverse};

use super::{Context, LintRule, Program, ProgramRef};
use deno_ast::{
    swc::common::Spanned,
    view::{self as ast_view, NodeTrait},
};
use std::sync::Arc;

const MESSAGE: &str = "🚨 redirect 路由中应该只配置 redirect 和 path 两个属性！";

#[derive(Debug)]
pub struct RedirectKeys;

const CODE: &str = "redirect-only-has-redirect-and-path";

impl LintRule for RedirectKeys {
    fn code(&self) -> &'static str {
        CODE
    }

    fn lint_program<'view>(&self, _context: &mut Context<'view>, _program: ProgramRef<'view>) {
        unreachable!();
    }

    fn lint_program_with_ast_view(&self, context: &mut Context, program: Program<'_>) {
        RedirectKeysHandler.traverse(program, context);
    }

    fn new() -> Arc<Self> {
        Arc::new(RedirectKeys)
    }
}

struct RedirectKeysHandler;

impl Handler for RedirectKeysHandler {
    fn object_lit(&mut self, object_lit: &ast_view::ObjectLit, ctx: &mut Context) {
        let obj_keys: Vec<String> = object_lit
            .children()
            .iter()
            .map(|obj_name| obj_name.children()[0].text().to_string())
            .collect();
        if obj_keys.len() > 2
            && obj_keys.contains(&String::from("path"))
            && obj_keys.contains(&String::from("redirect"))
        {
            ctx.add_diagnostic(object_lit.span(), CODE, MESSAGE);
        }
    }
}
