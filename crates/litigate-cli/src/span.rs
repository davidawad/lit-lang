//! Span arithmetic: one-based line/column (Unicode scalars) to byte offsets.
use litigate_base::{LineIndex, Span};
use std::ops::Range;

/// Byte offset of one-based `(line, col)` in `text`, clamped to its end.
#[must_use]
pub fn offset(text: &str, index: &LineIndex, line: u32, col: u32) -> usize {
    let Some(start) = index.line_start(line) else {
        return text.len();
    };
    let skip = usize::try_from(col.saturating_sub(1)).unwrap_or(usize::MAX);
    text[start..]
        .char_indices()
        .map(|(i, _)| start + i)
        .chain(std::iter::once(text.len()))
        .nth(skip)
        .unwrap_or(text.len())
}

/// The byte range `span` covers in `text`.
#[must_use]
pub fn byte_range(text: &str, span: &Span) -> Range<usize> {
    let index = LineIndex::new(text);
    let start = offset(text, &index, span.line, span.col);
    let end = offset(text, &index, span.end_line, span.end_col).max(start);
    start..end
}

/// The text `span` covers.
#[must_use]
pub fn slice<'t>(text: &'t str, span: &Span) -> &'t str {
    let range = byte_range(text, span);
    text.get(range).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn span(line: u32, col: u32, end_line: u32, end_col: u32) -> Span {
        Span {
            file: "f".into(),
            line,
            col,
            end_line,
            end_col,
        }
    }

    #[test]
    fn counts_scalars_not_bytes() {
        let text = "é = 1\nab";
        assert_eq!(slice(text, &span(1, 1, 1, 2)), "é");
        assert_eq!(slice(text, &span(2, 1, 2, 3)), "ab");
        assert_eq!(byte_range(text, &span(9, 1, 9, 1)), text.len()..text.len());
    }

    proptest! {
        #[test]
        fn line_col_round_trips(text in "[a-zé\n]{0,40}") {
            let index = LineIndex::new(&text);
            for (i, _) in text.char_indices() {
                let (line, col) = index.line_col(i);
                prop_assert_eq!(offset(&text, &index, line, col), i);
            }
        }
    }
}
