//! Import diagnostics: every value that has no lossless `.lit`/`.case`
//! spelling is an L6002 error at the input file, naming its JSON pointer
//! (SPEC §9.3: import output must round-trip under §8.6).
use litigate_base::{Diagnostic, DiagnosticCode, FileId, Span};

/// Collects the diagnostics of one input file.
#[derive(Debug)]
pub(crate) struct Issues {
    file: FileId,
    list: Vec<Diagnostic>,
}

impl Issues {
    /// A collector for diagnostics about `file` (its repository-relative path).
    pub(crate) fn new(file: &str) -> Self {
        Self {
            file: FileId(file.to_owned()),
            list: Vec::new(),
        }
    }

    /// The span for a JSON input: line and column when known, else the
    /// file start.
    pub(crate) fn span(&self, line: usize, col: usize) -> Span {
        let line = u32::try_from(line.max(1)).unwrap_or(u32::MAX);
        let col = u32::try_from(col.max(1)).unwrap_or(u32::MAX);
        Span {
            file: self.file.clone(),
            line,
            col,
            end_line: line,
            end_col: col,
        }
    }

    /// Reports that the value at JSON `pointer` cannot be imported.
    pub(crate) fn report(&mut self, pointer: &str, message: impl Into<String>) {
        self.list.push(
            Diagnostic::error(DiagnosticCode::SemanticDiff, self.span(1, 1), message)
                .note(format!("at JSON pointer `{pointer}`")),
        );
    }

    /// Reports an input that is not a valid document of its kind.
    pub(crate) fn invalid(&mut self, line: usize, col: usize, message: impl Into<String>) {
        let span = self.span(line, col);
        self.list.push(Diagnostic::error(
            DiagnosticCode::SemanticDiff,
            span,
            message,
        ));
    }

    /// The spelling `value`, or a report under `pointer` and a placeholder
    /// (the output is discarded once any error is reported).
    pub(crate) fn spell(&mut self, value: Option<String>, pointer: &str, what: &str) -> String {
        value.unwrap_or_else(|| {
            self.report(pointer, format!("{what} has no exact .lit spelling"));
            "?".to_owned()
        })
    }

    /// `text` when it is an identifier (or, with `qualified`, a reference),
    /// else a report.
    pub(crate) fn ident(&mut self, text: &str, pointer: &str, qualified: bool) -> String {
        let ok = if qualified {
            crate::text::is_ref(text)
        } else {
            crate::text::is_ident(text)
        };
        if !ok {
            self.report(
                pointer,
                format!("`{text}` is not a litigate identifier (SPEC §2)"),
            );
        }
        text.to_owned()
    }

    /// The collected diagnostics.
    pub(crate) fn into_vec(self) -> Vec<Diagnostic> {
        self.list
    }
}
