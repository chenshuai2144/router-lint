use deno_ast::view::swc_ast;
use deno_ast::view::Program;

pub mod children_key;
pub mod redirect;
pub mod repeat;

use crate::context::Context;
use std::sync::Arc;

#[derive()]

pub enum ProgramRef<'a> {
    Module(&'a swc_ast::Module),
    Script(&'a swc_ast::Script),
}

pub trait LintRule: std::fmt::Debug + Send + Sync {
    /// Returns the unique code that identifies the rule
    fn code(&self) -> &'static str;

    fn lint_program<'view>(&self, context: &mut Context<'view>, program: ProgramRef<'view>);

    /// Executes lint using `dprint-swc-ecma-ast-view`.
    /// Falls back to the `lint_program` method if not implemented.
    fn lint_program_with_ast_view<'view>(
        &self,
        context: &mut Context<'view>,
        program: Program<'view>,
    ) {
        use Program::*;
        let program_ref = match program {
            Module(m) => ProgramRef::Module(m.inner),
            Script(s) => ProgramRef::Script(s.inner),
        };
        self.lint_program(context, program_ref);
    }

    /// Creates an instance of this rule.
    fn new() -> Arc<Self>
    where
        Self: Sized;
}

pub fn get_all_rules_raw() -> Vec<Arc<dyn LintRule>> {
    vec![
        children_key::ChildrenKey::new(),
        redirect::RedirectKeys::new(),
        repeat::RepeatPath::new(),
    ]
}
