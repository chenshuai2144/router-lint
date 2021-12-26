pub mod context;
pub mod diagnostic;
pub mod handler;
pub mod rules;

use context::Context;
use deno_ast::view::NodeTrait;
use std::string::String;
use structopt::StructOpt;
/// Search for a pattern in a file and display the lines that contain it.
#[derive(StructOpt)]
struct Cli {
    /// The path to the file to read
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

#[derive(Debug)]
struct ReadFileError(String);

/**
 * 把文件内容转化为语法树
 */
fn parse_program(
    file_name: &str,
    syntax: deno_ast::swc::parser::Syntax,
    source_code: String,
) -> Result<deno_ast::ParsedSource, deno_ast::Diagnostic> {
    deno_ast::parse_program(deno_ast::ParseParams {
        specifier: file_name.to_string(),
        media_type: deno_ast::MediaType::Unknown,
        source: deno_ast::SourceTextInfo::from_string(source_code),
        capture_tokens: true,
        maybe_syntax: Some(syntax),
        scope_analysis: true,
    })
}

/**
 * 用于打印错误信息的类，里面存了原始代码
 */
#[derive(Clone)]
pub struct LineAndColumnDisplay {
    // 行号
    line: usize,
    // 列数
    column: usize,
    //相关的代码内容
    line_text: Vec<String>,
    // 当前路由的配置
    router_source_code: String,
}

/**
 * 错误类型的枚举
 */
#[derive(Clone, PartialEq)]
pub enum RouteSyntaxError {
    // 重复的路由
    Repeat,
    // 包含 Redirect 冗余的路由
    RedirectRedundancy,
    // 不推荐继续使用 children
    DeprecatedChildren,
    // Layout 节点不应该设置 component
    LayoutComponent,
}

/**
 * 错误信息的基础模板
 */
pub struct RouteDiagnostic {
    pub specifier: String,
    pub display_position: Vec<LineAndColumnDisplay>,
    pub kind: RouteSyntaxError,
    pub source_file_name: String,
}

/**
 * 用于传递的基础列
 */
#[derive(Clone)]
pub struct RoutePathObj {
    pub path: String,
    pub parent_path: String,
    pub node_source: String,
    pub obj_keys: Vec<String>,
    pub display_position: LineAndColumnDisplay,
}

/**
 * 遍历routes 的结构，可能一直互相嵌套
 */
fn loops_router_array(
    array_node: deno_ast::view::Node,
    parent_path: &str,
    mut context: Vec<RoutePathObj>,
) -> Vec<RoutePathObj> {
    for item in array_node.children() {
        if item.kind() == deno_ast::view::NodeKind::ExprOrSpread {
            let obj = item.children()[0];
            let obj_keys: Vec<String> = obj
                .children()
                .iter()
                .map(|obj_name| obj_name.children()[0].text().to_string())
                .collect();

            for obj_name in obj.children() {
                let mut path: String = String::from(parent_path);

                if obj_name.kind() == deno_ast::view::NodeKind::KeyValueProp {
                    let key = obj_name.children()[0];
                    let value = obj_name.children()[1];
                    if key.kind() == deno_ast::view::NodeKind::Ident {
                        if key.text() == "path" {
                            let route_path_obj = RoutePathObj {
                                path: value.text().to_string(),
                                parent_path: path.clone(),
                                node_source: obj_name.text().to_string(),
                                obj_keys: obj_keys.clone(),
                                display_position: LineAndColumnDisplay {
                                    column: value.start_column(),
                                    line: value.start_line(),
                                    line_text: vec![obj_name.text().to_string()],
                                    router_source_code: obj.text().to_string(),
                                },
                            };
                            context.push(route_path_obj);
                            let children_path = value.text();
                            if !children_path.starts_with("/") {
                                path.push_str("/");
                                path.push_str(value.text());
                            } else {
                                path = String::from("/");
                            }
                        }
                        if value.kind() == deno_ast::view::NodeKind::ArrayLit {
                            context = loops_router_array(value, &path, context.clone());
                        }
                    }
                }
            }
        }
    }
    return context;
}

// fn print_diagnostic(diagnostic: &RouteDiagnostic) {
//     if diagnostic.kind == RouteSyntaxError::Repeat {
//         print_diagnostic_repeat(diagnostic);
//     }
//     if diagnostic.kind == RouteSyntaxError::RedirectRedundancy {
//         print_diagnostic_redirect(diagnostic);
//     }

//     if diagnostic.kind == RouteSyntaxError::DeprecatedChildren {
//         print_diagnostic_children_key_router(diagnostic);
//     }
// }

fn gen_route_diagnostic(
    path_array: Vec<RoutePathObj>,
    source_file_name: String,
) -> Vec<RouteDiagnostic> {
    let mut route_diagnostic_array: Vec<RouteDiagnostic> = Vec::new();
    // path_array.iter().for_each(|item| {
    //     if path_map.contains_key(&item.path) {
    //         route_diagnostic_array.push(gen_diagnostic_repeat(
    //             &item,
    //             path_map.get(&item.path).unwrap(),
    //             source_file_name.clone(),
    //         ));
    //     }
    //     if is_warning_redirect_router(item.clone()) {
    //         route_diagnostic_array.push(gen_diagnostic_redirect(&item, source_file_name.clone()));
    //     }

    //     if is_warning_children_key_router(item.clone()) {
    //         route_diagnostic_array
    //             .push(gen_diagnostic_children_key(&item, source_file_name.clone()));
    //     }
    //     path_map.insert(item.path.to_string(), item.clone());
    // });
    route_diagnostic_array
}

fn main() -> Result<(), ReadFileError> {
    let args = Cli::from_args();
    let path = &args.path;

    // display 可以转化成需要显示的文案
    let path_str: String = path.as_path().display().to_string();

    // 读取文件内容
    let content = std::fs::read_to_string(&args.path)
        .map_err(|err| ReadFileError(format!("读取文件异常： `{}`: {}", path_str, err)))?;

    // 定义一下是一个 ts ast 的格式
    let syntax = deno_ast::get_syntax(deno_ast::MediaType::TypeScript);
    // 转化为语法树
    let ast = parse_program(&path_str, syntax, content).unwrap();

    ast.with_view(|program| {
        // 生成一个context，用于存储错误信息并且被各个规则消费
        let mut context = Context::new(
            path_str.clone(),
            deno_ast::MediaType::TypeScript,
            ast.source(),
            program,
        );
    });

    println!("👍 没有发现任何问题，非常好!");

    Ok(())
}
