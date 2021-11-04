use deno_ast::swc::parser::Syntax;
use deno_ast::view as ast_view;
use deno_ast::Diagnostic;
use deno_ast::MediaType;
use deno_ast::ParsedSource;
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

fn lint_program(parsed_source: &ParsedSource) -> &ParsedSource {
    let diagnostics = parsed_source.with_view(|pg| pg);
    parsed_source
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

    ast.with_view(|pg| {});
    Ok(())
}
