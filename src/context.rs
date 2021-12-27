use deno_ast::{
    swc::common::{comments::Comment, BytePos, Span},
    view::{RootNode, SourceFile},
    MediaType,
};

use crate::diagnostic::{LintDiagnostic, Position, Range};

pub struct Context<'view> {
    /// File name on which the lint rule is run
    file_name: String,

    /// The media type which linter was configured with. Can be used
    /// to skip checking some rules.
    media_type: MediaType,

    /// Stores diagnostics that are generated while linting
    diagnostics: Vec<LintDiagnostic>,

    /// Information about the file text.
    source_file: &'view dyn SourceFile,

    /// The AST view of the program, which for example can be used for getting
    /// comments
    program: deno_ast::view::Program<'view>,
}

impl<'view> Context<'view> {
    pub fn add_diagnostic(&mut self, span: Span, code: impl ToString, message: impl ToString) {
        let diagnostic = self.create_diagnostic(span, code.to_string(), message.to_string(), None);
        self.diagnostics.push(diagnostic);
    }
    pub fn add_diagnostic_with_hint(
        &mut self,
        span: Span,
        code: impl ToString,
        message: impl ToString,
        hint: impl ToString,
    ) {
        let diagnostic = self.create_diagnostic(span, code, message, Some(hint.to_string()));
        self.diagnostics.push(diagnostic);
    }

    pub fn all_comments(&self) -> impl Iterator<Item = &'view Comment> {
        self.program.comment_container().unwrap().all_comments()
    }

    pub(crate) fn create_diagnostic(
        &self,
        span: Span,
        code: impl ToString,
        message: impl ToString,
        maybe_hint: Option<String>,
    ) -> LintDiagnostic {
        let start = Position::new(span.lo(), self.source_file.line_and_column_index(span.lo()));
        let end = Position::new(span.hi(), self.source_file.line_and_column_index(span.hi()));

        let diagnostic = LintDiagnostic {
            range: Range { start, end },
            filename: self.file_name.clone(),
            message: message.to_string(),
            code: code.to_string(),
            hint: maybe_hint,
        };

        diagnostic
    }

    pub fn diagnostics(&self) -> &[LintDiagnostic] {
        &self.diagnostics
    }

    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    pub fn file_text_substring(&self, span: &Span) -> &str {
        &self.source_file.text()[span.lo.0 as usize..span.hi.0 as usize]
    }

    pub fn leading_comments_at(&self, lo: BytePos) -> impl Iterator<Item = &'view Comment> {
        self.program
            .comment_container()
            .unwrap()
            .leading_comments(lo)
    }

    pub fn media_type(&self) -> MediaType {
        self.media_type
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        file_name: String,
        media_type: MediaType,
        source_file: &'view impl SourceFile,
        program: deno_ast::view::Program<'view>,
    ) -> Self {
        Self {
            file_name,
            media_type,
            source_file,
            program,
            diagnostics: Vec::new(),
        }
    }

    pub fn program(&self) -> &deno_ast::view::Program<'view> {
        &self.program
    }

    pub fn source_file(&self) -> &dyn SourceFile {
        self.source_file
    }

    pub fn trailing_comments_at(&self, hi: BytePos) -> impl Iterator<Item = &'view Comment> {
        self.program
            .comment_container()
            .unwrap()
            .trailing_comments(hi)
    }
}
