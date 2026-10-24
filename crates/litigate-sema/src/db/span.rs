//! Byte ranges inside one file and file-local diagnostic reports that become
//! [`litigate_base::Diagnostic`]s once a [`LineIndex`] is at hand.
use litigate_base::{Applicability, Diagnostic, DiagnosticCode, FileId, LineIndex, Profile, Span};

/// Half-open byte range `[start, end)` in one source file: absolute when
/// `anchor` is 0, else relative to the start of the file's top-level
/// syntax node number `anchor` (one-based, in source order).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct TextRange {
    /// The anchoring top-level node (0: none, the range is absolute).
    /// First, so ranges of different declarations order by source order.
    pub anchor: u32,
    /// First byte.
    pub start: u32,
    /// One past the last byte.
    pub end: u32,
}

impl TextRange {
    /// Absolute range from `start` to `end`.
    #[must_use]
    pub const fn new(start: u32, end: u32) -> Self {
        Self {
            anchor: 0,
            start,
            end,
        }
    }

    /// Smallest range covering both; both lie in one anchor (a
    /// declaration's parts), whose anchor the cover keeps.
    #[must_use]
    pub fn cover(self, other: Self) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
            ..self
        }
    }

    /// Empty range at this range's end.
    #[must_use]
    pub const fn at_end(self) -> Self {
        Self {
            start: self.end,
            ..self
        }
    }

    /// Empty range at this range's last byte (its end when it is empty at
    /// the start of its anchor).
    #[must_use]
    pub const fn at_last(self) -> Self {
        let at = self.end.saturating_sub(1);
        Self {
            start: at,
            end: at,
            ..self
        }
    }
}

/// The first significant byte of every top-level syntax node of a file,
/// in source order: anchor `k` starts at `self.0[k - 1]`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Anchors(pub Vec<u32>);

impl Anchors {
    /// `range` (absolute) anchored at the last anchor at or before its
    /// start; a range before every anchor stays absolute.
    #[must_use]
    pub fn anchor(&self, range: TextRange) -> TextRange {
        let k = self.0.partition_point(|&s| s <= range.start);
        match k.checked_sub(1) {
            Some(i) => TextRange {
                anchor: u32::try_from(k).unwrap_or(u32::MAX),
                start: range.start - self.0[i],
                end: range.end - self.0[i],
            },
            None => range,
        }
    }

    /// `range` made absolute again (an anchor this table does not have
    /// counts from the start of the file).
    #[must_use]
    pub fn absolute(&self, range: TextRange) -> TextRange {
        let base = (range.anchor as usize)
            .checked_sub(1)
            .and_then(|i| self.0.get(i))
            .copied()
            .unwrap_or(0);
        TextRange::new(base + range.start, base + range.end)
    }
}

/// Places ranges of one file: turns them into one-based line/column
/// [`Span`]s.
pub trait Positions {
    /// The span of `range` in `file`.
    fn span(&self, file: &FileId, range: TextRange) -> Span;
}

/// A file without anchors (a `.case`, `Lit.toml` or `Lit.lock`): every
/// range is absolute.
impl Positions for LineIndex {
    fn span(&self, file: &FileId, range: TextRange) -> Span {
        line_col_span(file, self, range)
    }
}

/// A `.lit` file's line index with its anchors.
#[derive(Debug, Clone)]
pub struct SpanMap {
    index: LineIndex,
    anchors: Anchors,
}

impl SpanMap {
    /// The positions of `text`, whose top-level nodes start at `anchors`.
    #[must_use]
    pub fn new(text: &str, anchors: Anchors) -> Self {
        Self {
            index: LineIndex::new(text),
            anchors,
        }
    }

    /// `range` made absolute.
    #[must_use]
    pub fn absolute(&self, range: TextRange) -> TextRange {
        self.anchors.absolute(range)
    }

    /// A span written by [`Encoded`] (in this map's file), located.
    pub fn decode(&self, span: &mut Span) {
        let range = TextRange {
            anchor: span.line,
            start: span.col,
            end: span.end_col,
        };
        *span = self.span(&span.file, range);
    }
}

impl Positions for SpanMap {
    fn span(&self, file: &FileId, range: TextRange) -> Span {
        line_col_span(file, &self.index, self.absolute(range))
    }
}

/// Spans that stay position-independent: a range kept as is, its anchor
/// in `line` and `end_line` and its offsets in `col` and `end_col`. HIR a
/// per-declaration query builds carries these until its module locates
/// them with [`SpanMap::decode`].
#[derive(Debug, Clone, Copy, Default)]
pub struct Encoded;

impl Positions for Encoded {
    fn span(&self, file: &FileId, range: TextRange) -> Span {
        Span {
            file: file.clone(),
            line: range.anchor,
            col: range.start,
            end_line: range.anchor,
            end_col: range.end,
        }
    }
}

fn line_col_span(file: &FileId, index: &LineIndex, range: TextRange) -> Span {
    let (line, col) = index.line_col(range.start as usize);
    let (end_line, end_col) = index.line_col(range.end as usize);
    Span {
        file: file.clone(),
        line,
        col,
        end_line,
        end_col,
    }
}

/// A diagnostic whose locations are byte ranges in the reporting file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    /// Stable code.
    pub code: DiagnosticCode,
    /// Primary range.
    pub range: TextRange,
    /// Message.
    pub message: String,
    /// Notes, optionally located in the same file.
    pub notes: Vec<(Option<TextRange>, String)>,
    /// Suggested edits in the same file.
    pub fixes: Vec<(TextRange, String, Applicability)>,
}

impl Report {
    /// New error report.
    pub fn new(code: DiagnosticCode, range: TextRange, message: impl Into<String>) -> Self {
        Self {
            code,
            range,
            message: message.into(),
            notes: Vec::new(),
            fixes: Vec::new(),
        }
    }

    /// Adds an unlocated note.
    #[must_use]
    pub fn note(mut self, message: impl Into<String>) -> Self {
        self.notes.push((None, message.into()));
        self
    }

    /// Adds a located note.
    #[must_use]
    pub fn note_at(mut self, range: TextRange, message: impl Into<String>) -> Self {
        self.notes.push((Some(range), message.into()));
        self
    }

    /// Adds a suggested edit.
    #[must_use]
    pub fn fix(
        mut self,
        range: TextRange,
        replacement: impl Into<String>,
        applicability: Applicability,
    ) -> Self {
        self.fixes.push((range, replacement.into(), applicability));
        self
    }

    /// Converts to a located diagnostic: located notes become labels.
    /// Sema codes are all errors except the lint range, which is graded
    /// as in `dev` (callers grade it by profile).
    #[must_use]
    pub fn to_diagnostic(&self, file: &FileId, at: &(impl Positions + ?Sized)) -> Diagnostic {
        let span = |range: TextRange| at.span(file, range);
        let mut d = Diagnostic::error(self.code, span(self.range), self.message.clone());
        if self.code.is_lint() {
            d.severity = self.code.severity(Profile::Dev);
        }
        for (range, message) in &self.notes {
            d = d.note_at(range.map(span), message.clone());
        }
        for (range, replacement, applicability) in &self.fixes {
            d = d.fix(span(*range), replacement.clone(), *applicability);
        }
        d
    }
}

/// Converts a range to a one-based line/column span.
#[must_use]
pub fn to_span(file: &FileId, at: &(impl Positions + ?Sized), range: TextRange) -> Span {
    at.span(file, range)
}

/// Sorts diagnostics by `(file, line, col, code)` (SPEC §9.2), drops
/// exact duplicates, and gives every L3xxx error a derivation.
pub fn sort_diagnostics(diagnostics: &mut Vec<Diagnostic>) {
    diagnostics
        .iter_mut()
        .for_each(Diagnostic::ensure_derivation);
    diagnostics.sort_by(|a, b| {
        (&a.span.file, a.span.line, a.span.col, a.code, &a.message).cmp(&(
            &b.span.file,
            b.span.line,
            b.span.col,
            b.code,
            &b.message,
        ))
    });
    diagnostics.dedup();
}
