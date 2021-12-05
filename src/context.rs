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
    program: ast_view::Program<'view>,

    /// Scope analysis result
    scope: Scope,

    /// Control-flow analysis result
    control_flow: ControlFlow,

    /// The `SyntaxContext` of the top level
    top_level_ctxt: SyntaxContext,
}

impl<'view> Context<'view> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        file_name: String,
        media_type: MediaType,
        source_file: &'view impl SourceFile,
        program: ast_view::Program<'view>,
        scope: Scope,
        control_flow: ControlFlow,
        top_level_ctxt: SyntaxContext,
    ) -> Self {
        Self {
            file_name,
            media_type,
            source_file,
            program,
            scope,
            control_flow,
            top_level_ctxt,
            diagnostics: Vec::new(),
        }
    }
    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    pub fn media_type(&self) -> MediaType {
        self.media_type
    }

    pub fn diagnostics(&self) -> &[LintDiagnostic] {
        &self.diagnostics
    }

    pub fn source_file(&self) -> &dyn SourceFile {
        self.source_file
    }

    pub fn file_text_substring(&self, span: &Span) -> &str {
        &self.source_file.text()[span.lo.0 as usize..span.hi.0 as usize]
    }

    pub fn program(&self) -> &ast_view::Program<'view> {
        &self.program
    }

    pub fn file_ignore_directive(&self) -> Option<&FileIgnoreDirective> {
        self.file_ignore_directive.as_ref()
    }

    pub fn line_ignore_directives(&self) -> &HashMap<usize, LineIgnoreDirective> {
        &self.line_ignore_directives
    }

    pub fn scope(&self) -> &Scope {
        &self.scope
    }

    pub fn control_flow(&self) -> &ControlFlow {
        &self.control_flow
    }

    pub(crate) fn top_level_ctxt(&self) -> SyntaxContext {
        self.top_level_ctxt
    }

    pub fn all_comments(&self) -> impl Iterator<Item = &'view Comment> {
        self.program.comment_container().unwrap().all_comments()
    }

    pub fn leading_comments_at(&self, lo: BytePos) -> impl Iterator<Item = &'view Comment> {
        self.program
            .comment_container()
            .unwrap()
            .leading_comments(lo)
    }

    pub fn trailing_comments_at(&self, hi: BytePos) -> impl Iterator<Item = &'view Comment> {
        self.program
            .comment_container()
            .unwrap()
            .trailing_comments(hi)
    }

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

    pub(crate) fn create_diagnostic(
        &self,
        span: Span,
        code: impl ToString,
        message: impl ToString,
        maybe_hint: Option<String>,
    ) -> LintDiagnostic {
        let time_start = Instant::now();
        let start = Position::new(span.lo(), self.source_file.line_and_column_index(span.lo()));
        let end = Position::new(span.hi(), self.source_file.line_and_column_index(span.hi()));

        let diagnostic = LintDiagnostic {
            range: Range { start, end },
            filename: self.file_name.clone(),
            message: message.to_string(),
            code: code.to_string(),
            hint: maybe_hint,
        };

        let time_end = Instant::now();
        debug!(
            "Context::create_diagnostic took {:?}",
            time_end - time_start
        );
        diagnostic
    }
}
