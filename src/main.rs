use deno_ast::swc::parser::Syntax;
use deno_ast::view::Program;
use deno_ast::view::RootNode;
use deno_ast::Diagnostic;
use deno_ast::MediaType;
use deno_ast::ParsedSource;
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
    syntax: Syntax,
    source_code: String,
) -> Result<ParsedSource, Diagnostic> {
    deno_ast::parse_program(deno_ast::ParseParams {
        specifier: file_name.to_string(),
        media_type: MediaType::Unknown,
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

    let content = std::fs::read_to_string(&args.path)
        .map_err(|err| ReadFileError(format!("读取文件异常： `{}`: {}", path_str, err)))?;
    let syntax = deno_ast::get_syntax(MediaType::TypeScript);
    let ast = parse_program(&path_str, syntax, content).unwrap();
    let mut path_map = HashMap::new();
    ast.with_view(|program| {
        let mut path_span = false;
        let token_container = program.token_container().unwrap();

        for token in token_container.tokens {
            let span = token.span;
            let whitespace_text = program.source_file().unwrap().text().to_string();
            let line_text = whitespace_text[span.lo().0 as usize..span.hi().0 as usize].to_string();

            if path_span && !line_text.contains(":") {
                path_map.insert(String::from(&line_text), 20);
            }

            if line_text.contains("path") || line_text.contains(":") {
                path_span = true
            } else {
                path_span = false
            }
        }
    });

    println!("{:?}", path_map);
    Ok(())
}
