use deno_ast::view::NodeTrait;
use std::{collections::HashMap, string::String};
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

#[derive(Clone)]
pub struct LineAndColumnDisplay {
    // 行号
    line: usize,
    // 列数
    column: usize,
    //相关的代码内容
    line_text: Vec<String>,
}
#[derive(Clone, PartialEq)]
pub enum RouterSyntaxError {
    // 重复的路由
    Repeat,
    // 冗余的路由
    Redundancy,
    // 不推荐继续使用 children
    DeprecatedChildren,
    // Layout 节点不应该设置 component
    LayoutComponent,
}

pub struct RouteDiagnostic {
    pub specifier: String,
    pub display_position: Vec<LineAndColumnDisplay>,
    pub kind: RouterSyntaxError,
    pub source_file_name: String,
}

#[derive(Clone)]
pub struct RoutePathObj {
    pub path: String,
    pub parent_path: String,
    pub node_source: String,
    pub display_position: LineAndColumnDisplay,
}

/**
 * 遍历routers 的结构，可能一直互相嵌套
 */
fn loops_router_array(
    array_node: deno_ast::view::Node,
    parent_path: &str,
    mut context: Vec<RoutePathObj>,
) -> Vec<RoutePathObj> {
    for item in array_node.children() {
        if item.kind() == deno_ast::view::NodeKind::ExprOrSpread {
            let obj = item.children()[0];
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
                                display_position: LineAndColumnDisplay {
                                    column: value.start_column(),
                                    line: value.start_line(),
                                    line_text: vec![obj_name.text().to_string()],
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

fn gen_diagnostic_repeat(
    node: &RoutePathObj,
    repeat_node: &RoutePathObj,
    source_file_name: String,
) -> RouteDiagnostic {
    let mut line_text = Vec::new();
    line_text.push(node.node_source.to_string());
    line_text.push(repeat_node.node_source.to_string());

    let mut display_position = Vec::new();
    display_position.push(node.display_position.clone());
    display_position.push(repeat_node.display_position.clone());
    let route_diagnostic = RouteDiagnostic {
        specifier: node.path.clone(),
        display_position: display_position,
        kind: RouterSyntaxError::Repeat,
        source_file_name: source_file_name,
    };
    route_diagnostic
}

fn print_diagnostic(diagnostic: &RouteDiagnostic) {
    if diagnostic.kind == RouterSyntaxError::Repeat {
        print_diagnostic_repeat(diagnostic);
    }
    if diagnostic.kind == RouterSyntaxError::Redundancy {
        print_diagnostic_repeat(diagnostic);
    }
}

fn print_diagnostic_repeat(diagnostic: &RouteDiagnostic) {
    println!("🚨 {} 重复声明，发现于以下行：", diagnostic.specifier);
    for line_and_column in &diagnostic.display_position {
        println!(
            "   ---> {}:{}:{} 的 {}",
            diagnostic.source_file_name,
            line_and_column.line,
            line_and_column.column,
            line_and_column.line_text[0]
        );
    }
    println!("");
    println!("如果是父子路由，请使用 ./ 来代替",);
    let message = "\
    💡  更改方案：
    {
        path: '/user',
        layout: false,
        routes: [
            {
                path: '/user',
                component: './user/Login',
            },
        ],
    },

    可以转化为 ======>

    {
        path: '/user',
        layout: false,
        routes: [
            {
                path: './',
                component: './user/Login',
            },
        ],
    },
    
";
    println!("{}", message);
}

fn gen_route_diagnostic(
    path_array: Vec<RoutePathObj>,
    source_file_name: String,
) -> Vec<RouteDiagnostic> {
    let mut path_map = HashMap::new();
    let mut route_diagnostic_array: Vec<RouteDiagnostic> = Vec::new();
    path_array.iter().for_each(|item| {
        if path_map.contains_key(&item.path) {
            route_diagnostic_array.push(gen_diagnostic_repeat(
                &item,
                path_map.get(&item.path).unwrap(),
                source_file_name.clone(),
            ));
        }
        path_map.insert(item.path.to_string(), item.clone());
    });
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
    // 定义一个 map 来存我们需要的分析数据
    let mut path_array: Vec<RoutePathObj> = Vec::new();
    ast.with_view(|program| {
        let array_node = program.children()[0].children()[0];
        if array_node.kind() == deno_ast::view::NodeKind::ArrayLit {
            path_array = loops_router_array(array_node, "/", path_array.clone());
        }
    });

    // 生成错误并且打印出来
    let diagnostic_list = gen_route_diagnostic(path_array, path_str);
    if diagnostic_list.len() > 0 {
        diagnostic_list.iter().for_each(|diagnostic| {
            print_diagnostic(diagnostic);
        });
    } else {
        println!("👍 没有发现任何问题，非常好!")
    }

    Ok(())
}
