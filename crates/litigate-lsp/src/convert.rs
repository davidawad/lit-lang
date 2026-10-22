//! Coordinates: compiler spans (one-based lines, Unicode-scalar columns,
//! SPEC §9.2) and rowan byte ranges to LSP positions (zero-based lines,
//! UTF-16 columns), and back.
use litigate_base::{LineIndex, Span};
use litigate_syntax::TextRange;
use tower_lsp::lsp_types::{Position, Range};

/// Byte offset of the start of zero-based `line` (the end of `text` past it).
fn line_start(text: &str, line: u32) -> usize {
    if line == 0 {
        return 0;
    }
    text.match_indices('\n')
        .nth(usize::try_from(line - 1).unwrap_or(usize::MAX))
        .map_or(text.len(), |(i, _)| i + 1)
}

/// Byte offset of LSP `position` in `text`, clamped to its line's end
/// (and to a character boundary inside a surrogate pair).
#[must_use]
pub fn offset(text: &str, position: Position) -> usize {
    let start = line_start(text, position.line);
    let line = text[start..].split('\n').next().unwrap_or_default();
    let mut units = 0_u32;
    for (i, c) in line.char_indices() {
        let width = u32::try_from(c.len_utf16()).unwrap_or(2);
        if units + width > position.character {
            return start + i;
        }
        units += width;
    }
    start + line.len()
}

/// LSP position of byte `offset` in `text` (clamped, snapped back to a
/// character boundary).
#[must_use]
pub fn position(text: &str, offset: usize) -> Position {
    let mut offset = offset.min(text.len());
    while !text.is_char_boundary(offset) {
        offset -= 1;
    }
    let before = &text[..offset];
    let line = before.matches('\n').count();
    let start = before.rfind('\n').map_or(0, |i| i + 1);
    let character = before[start..].encode_utf16().count();
    Position::new(
        u32::try_from(line).unwrap_or(u32::MAX),
        u32::try_from(character).unwrap_or(u32::MAX),
    )
}

/// The LSP range of a rowan text range.
#[must_use]
pub fn range(text: &str, range: TextRange) -> Range {
    Range::new(
        position(text, usize::from(range.start())),
        position(text, usize::from(range.end())),
    )
}

/// The LSP range of a compiler span over `text`.
#[must_use]
pub fn span_range(text: &str, span: &Span) -> Range {
    let bytes = litigate_cli::span::byte_range(text, span);
    Range::new(position(text, bytes.start), position(text, bytes.end))
}

/// The whole of `text`.
#[must_use]
pub fn full_range(text: &str) -> Range {
    Range::new(Position::new(0, 0), position(text, text.len()))
}

/// `text` with `range` replaced by `with` (an incremental `didChange`).
#[must_use]
pub fn apply_change(text: &str, range: Range, with: &str) -> String {
    let start = offset(text, range.start);
    let end = offset(text, range.end).max(start);
    format!("{}{with}{}", &text[..start], &text[end..])
}

/// Whether `range` holds `position` (end inclusive, as a cursor touching
/// the end of a word still points at it).
#[must_use]
pub fn touches(range: &Range, position: Position) -> bool {
    range.start <= position && position <= range.end
}

/// Whether two ranges share any position.
#[must_use]
pub fn overlaps(a: &Range, b: &Range) -> bool {
    a.start <= b.end && b.start <= a.end
}

/// One-based `(line, col)` of byte `offset`, as compiler spans count.
#[must_use]
pub fn line_col(text: &str, offset: usize) -> (u32, u32) {
    LineIndex::new(text).line_col(offset)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn columns_count_utf16_units() {
        let text = "a𝄞b\nλx";
        assert_eq!(offset(text, Position::new(0, 3)), 5);
        assert_eq!(position(text, 5), Position::new(0, 3));
        assert_eq!(offset(text, Position::new(1, 1)), 9);
        assert_eq!(offset(text, Position::new(9, 0)), text.len());
        assert_eq!(offset(text, Position::new(0, 99)), 6);
    }

    #[test]
    fn spans_convert_scalars_to_utf16() {
        let text = "𝄞 state\n";
        let span = Span {
            file: "f".into(),
            line: 1,
            col: 3,
            end_line: 1,
            end_col: 8,
        };
        assert_eq!(
            span_range(text, &span),
            Range::new(Position::new(0, 3), Position::new(0, 8))
        );
    }

    #[test]
    fn incremental_edits_splice_utf16_ranges() {
        let edited = apply_change(
            "αβ\nnext",
            Range::new(Position::new(0, 1), Position::new(0, 2)),
            "x",
        );
        assert_eq!(edited, "αx\nnext");
        assert_eq!(full_range("a\nbc").end, Position::new(1, 2));
    }

    proptest! {
        #[test]
        fn prop_positions_round_trip(text in "[a-zλ𝄞\n ]{0,40}") {
            for (i, _) in text.char_indices().chain(std::iter::once((text.len(), ' '))) {
                prop_assert_eq!(offset(&text, position(&text, i)), i);
            }
        }

        #[test]
        fn prop_edits_match_string_splices(
            text in "[a-zλ\n]{0,30}",
            a in 0_usize..40,
            b in 0_usize..40,
            with in "[a-z\n]{0,5}",
        ) {
            let bounds: Vec<usize> = text.char_indices().map(|(i, _)| i).chain([text.len()]).collect();
            let (a, b) = (bounds[a % bounds.len()], bounds[b % bounds.len()]);
            let (start, end) = (a.min(b), a.max(b));
            let range = Range::new(position(&text, start), position(&text, end));
            let expected = format!("{}{with}{}", &text[..start], &text[end..]);
            prop_assert_eq!(apply_change(&text, range, &with), expected);
        }
    }
}
