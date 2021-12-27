use annotate_snippets::{display_list, snippet};
use deno_ast::SourceTextInfo;
use serde::{Serialize, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    /// The 0-indexed line index.
    #[serde(rename(serialize = "line"))]
    #[serde(serialize_with = "to_one_indexed")]
    pub line_index: usize,
    /// The 0-indexed column index.
    #[serde(rename(serialize = "col"))]
    pub column_index: usize,
    pub byte_pos: usize,
}

impl Position {
    pub fn new(
        byte_pos: deno_ast::swc::common::BytePos,
        loc: deno_ast::LineAndColumnIndex,
    ) -> Self {
        Position {
            line_index: loc.line_index,
            column_index: loc.column_index,
            byte_pos: byte_pos.0 as usize,
        }
    }
}

fn to_one_indexed<S>(x: &usize, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_u32((x + 1) as u32)
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Range {
    pub end: Position,
    pub start: Position,
}

#[derive(Debug, PartialEq, Eq)]
struct CharRange {
    /// 0-indexed number that represents what index this range starts at in the
    /// snippet.
    /// Counted on a character basis, not UTF-8 bytes.
    start_index: usize,

    /// 0-indexed number that represents what index this range ends at in the
    /// snippet.
    /// Counted on a character basis, not UTF-8 bytes.
    end_index: usize,
}

impl CharRange {
    fn as_tuple(&self) -> (usize, usize) {
        (self.start_index, self.end_index)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct LintDiagnostic {
    pub code: String,
    pub filename: String,
    pub hint: Option<String>,
    pub message: String,
    pub range: Range,
}

fn get_slice_source_and_range<'a>(
    source_file: &'a SourceTextInfo,
    range: &Range,
) -> (&'a str, CharRange) {
    let first_line_start = source_file.line_start(range.start.line_index).0 as usize;
    let last_line_end = source_file.line_end(range.end.line_index).0 as usize;
    let text = source_file.text_str();
    let start_index = text[first_line_start..range.start.byte_pos].chars().count();
    let end_index = text[first_line_start..range.end.byte_pos].chars().count();
    let slice_str = &text[first_line_start..last_line_end];
    (
        slice_str,
        CharRange {
            start_index,
            end_index,
        },
    )
}

pub fn display_diagnostics(diagnostics: &[LintDiagnostic], source_file: &SourceTextInfo) {
    for diagnostic in diagnostics {
        let (slice_source, char_range) = get_slice_source_and_range(source_file, &diagnostic.range);
        let footer = if let Some(hint) = &diagnostic.hint {
            vec![snippet::Annotation {
                label: Some(hint),
                id: None,
                annotation_type: snippet::AnnotationType::Help,
            }]
        } else {
            vec![]
        };

        let snippet = snippet::Snippet {
            title: Some(snippet::Annotation {
                label: Some(&diagnostic.message),
                id: Some(&diagnostic.code),
                annotation_type: snippet::AnnotationType::Error,
            }),
            footer,
            slices: vec![snippet::Slice {
                source: slice_source,
                line_start: diagnostic.range.start.line_index + 1, // make 1-indexed
                origin: Some(&diagnostic.filename),
                fold: false,
                annotations: vec![snippet::SourceAnnotation {
                    range: char_range.as_tuple(),
                    label: "",
                    annotation_type: snippet::AnnotationType::Error,
                }],
            }],
            opt: display_list::FormatOptions {
                color: true,
                anonymized_line_numbers: false,
                margin: None,
            },
        };
        let display_list = display_list::DisplayList::from(snippet);
        eprintln!("{}", display_list);
    }
}
