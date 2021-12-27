pub mod context;
pub mod diagnostic;
pub mod handler;
pub mod rules;

use context::Context;
use diagnostic::display_diagnostics;
use std::string::String;
use structopt::StructOpt;

use crate::rules::get_all_rules_raw;
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

        let rules = get_all_rules_raw();
        for rule in rules {
            rule.lint_program_with_ast_view(&mut context, program);
        }

        if context.diagnostics().is_empty() {
            println!("👍 没有发现任何问题，非常好!");
        }

        display_diagnostics(&context.diagnostics(), ast.source());
    });

    Ok(())
}
