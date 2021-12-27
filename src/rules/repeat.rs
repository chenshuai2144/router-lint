use crate::handler::{Handler, Traverse};

use super::{Context, LintRule, Program, ProgramRef};
use deno_ast::swc::common::Spanned;
use deno_ast::view::{self as ast_view, NodeTrait};
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

fn find_all_router_path<'a>(node_list: deno_ast::view::Node, mut parent_path: String) -> String {
    use deno_ast::view::Node::*;
    if node_list.kind() == deno_ast::view::NodeKind::ExportDefaultExpr
        || node_list.kind() == deno_ast::view::NodeKind::Module
    {
        return parent_path;
    }
    for node in node_list.parent() {
        match node {
            ObjectLit(n) => {
                parent_path = find_all_router_path(n.as_node(), parent_path);
                for child in n.children() {
                    if child.children()[0].text() == "path" {
                        let children_path = child.children()[1].text().to_string().replace("'", "");
                        if !children_path.starts_with("/") && !children_path.starts_with(".") {
                            parent_path.push_str(&children_path.as_str());
                        } else if children_path.starts_with(".") {
                            parent_path.push_str(children_path.as_str().replace("./", "").as_str());
                        } else {
                            parent_path = children_path;
                        }
                    }
                }
            }
            n => {
                parent_path = find_all_router_path(n, parent_path);
            }
        }
    }

    parent_path
}

/**
 * 遍历routes 的结构
 */
fn loops_router_array(array_node: &ast_view::ArrayLit, mut context: Vec<String>) -> Vec<String> {
    for item in array_node.children() {
        if item.kind() == deno_ast::view::NodeKind::ExprOrSpread {
            let obj = item.children()[0];
            for obj_name in obj.children() {
                // * 寻找 children 找到子节点
                if obj_name.kind() == deno_ast::view::NodeKind::KeyValueProp {
                    let key = obj_name.children()[0];
                    let value = obj_name.children()[1];

                    if key.kind() == deno_ast::view::NodeKind::Ident {
                        if key.text() == "path" {
                            let children_path = value.text().replace("'", "");
                            let mut parent_path = "/".to_string();
                            // 如果是用 / 开头的不用拼接 ，别的都需要
                            if !children_path.starts_with("/") {
                                parent_path =
                                    find_all_router_path(array_node.parent(), "/".to_string());
                            }

                            if children_path != "/" {
                                if !children_path.starts_with("/")
                                    && !children_path.starts_with(".")
                                {
                                    parent_path.push_str(children_path.as_str());
                                } else if children_path.starts_with(".") {
                                    parent_path.push_str(
                                        children_path.as_str().replace("./", "/").as_str(),
                                    );
                                } else {
                                    parent_path = children_path;
                                }

                                if !parent_path.eq("/") && parent_path.len() > 1 {
                                    context.push(parent_path);
                                }
                            }
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
            } else if !path.eq("'/'") {
                path_map.insert(path, true);
            }
        }
    }
}
