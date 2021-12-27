use crate::handler::{Handler, Traverse};

use super::{Context, LintRule, Program, ProgramRef};
use crate::rules::repeat::ast_view::Node::ArrayLit;
use deno_ast::swc::common::Spanned;
use deno_ast::view::NodeTrait;
use deno_ast::view::{self as ast_view};
use std::collections::HashMap;
use std::sync::Arc;

const MESSAGE: &str = "🚨 path发现重复，可能会导致路径渲染错误，请检查后删除！";

#[derive(Debug)]
pub struct RepeatPath;

const CODE: &str = "redirect-only-has-redirect-and-path";

impl LintRule for RepeatPath {
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
        Arc::new(RepeatPath)
    }
}

struct RedirectKeysHandler;

/**
 * 遍历routes 的结构，可能一直互相嵌套
 */
fn loops_router_array(array_node: &ast_view::ArrayLit, mut context: Vec<String>) -> Vec<String> {
    let mut parent_path = "/".to_string();

    let parent_node = array_node.parent();

    let obj = parent_node;
    for obj_name in obj.children() {
        for object_kit in obj_name.children() {
            for key_object in object_kit.children() {
                for key_value in key_object.children() {
                    let key = key_value.children()[0];
                    let value = key_value.children()[1];
                    if key.text() == "path" {
                        let path_value = value.text();
                        if !path_value.eq("'/'") && !path_value.eq("'./'") {
                            parent_path = path_value.to_string();
                        }
                    }
                }
            }
        }
    }
    for item in array_node.children() {
        if item.kind() == deno_ast::view::NodeKind::ExprOrSpread {
            let obj = item.children()[0];
            for obj_name in obj.children() {
                let mut path: String = parent_path.clone();

                if obj_name.kind() == deno_ast::view::NodeKind::KeyValueProp {
                    let key = obj_name.children()[0];
                    let value = obj_name.children()[1];
                    if key.kind() == deno_ast::view::NodeKind::Ident {
                        if key.text() == "path" {
                            let children_path = value.text();
                            if !children_path.starts_with("/") {
                                path.push_str("/");
                                path.push_str(value.text());
                            } else {
                                path = String::from("/");
                            }
                        }
                        for child in value.children() {
                            if let ArrayLit(n) = child {
                                context = loops_router_array(n, context.clone())
                            }
                        }
                        if !path.eq("/") && path.len() > 1 {
                            context.push(path);
                        }
                    }
                }
            }
        }
    }
    return context;
}

impl Handler for RedirectKeysHandler {
    fn array_lit(&mut self, array_lit: &ast_view::ArrayLit, ctx: &mut Context) {
        // 遍历dom，获取所有的 path
        let path_array: Vec<String> = loops_router_array(array_lit, vec![]);

        let mut path_map: HashMap<String, bool> = HashMap::new();
        // 判断是否有重复的path
        for path in path_array {
            if path_map.contains_key(&path) {
                ctx.add_diagnostic(array_lit.span(), CODE, MESSAGE);
            } else if path.eq("'/'") {
                path_map.insert(path, true);
            }
        }
    }
}
