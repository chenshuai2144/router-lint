use deno_ast::view::NodeTrait;
use std::collections::HashMap;
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

fn merge(
    first_context: HashMap<String, String>,
    second_context: HashMap<String, String>,
) -> HashMap<String, String> {
    let mut new_context = HashMap::new();
    for (key, value) in first_context.iter() {
        new_context.insert(String::from(key), String::from(value));
    }
    for (key, value) in second_context.iter() {
        new_context.insert(String::from(key), String::from(value));
    }
    new_context
}

fn loopsRouterArray(
    array_node: deno_ast::view::Node,
    parent_path: &str,
) -> HashMap<String, String> {
    let mut map = HashMap::new();
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
                            map.insert(String::from(value.text()), String::from(parent_path));
                            let children_path = value.text();
                            if !children_path.starts_with("/") {
                                path.push_str("/");
                                path.push_str(value.text());
                            } else {
                                path = String::from("/");
                            }
                        }
                        if value.kind() == deno_ast::view::NodeKind::ArrayLit {
                            let loop_map = loopsRouterArray(value, &path);
                            map = merge(loop_map, map)
                        }
                    }
                }
            }
        }
    }
    return map;
}

fn main() -> Result<(), ReadFileError> {
    let args = Cli::from_args();
    let path = &args.path;
    // display 可以转化成需要显示的文案
    let path_str: String = path.as_path().display().to_string();

    let content = std::fs::read_to_string(&args.path)
        .map_err(|err| ReadFileError(format!("读取文件异常： `{}`: {}", path_str, err)))?;
    let syntax = deno_ast::get_syntax(deno_ast::MediaType::TypeScript);
    let ast = parse_program(&path_str, syntax, content).unwrap();
    let mut map = HashMap::new();
    ast.with_view(|program| {
        let array_node = program.children()[0].children()[0];
        if array_node.kind() == deno_ast::view::NodeKind::ArrayLit {
            map = loopsRouterArray(array_node, "/")
        }
    });

    map.iter().for_each(|(key, value)| {
        println!("key:{},value:{}", key, value);
    });

    Ok(())
}
