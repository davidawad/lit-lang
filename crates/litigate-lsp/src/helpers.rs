//! LSP compiler adapters and UTF-16 coordinate conversion.
use super::Document;
use litigate_base::{Diagnostic as CompilerDiagnostic, LineIndex, Severity};
use litigate_syntax::{parse, FileKind, SyntaxKind};
use std::path::Path;
use tower_lsp::lsp_types::{
    Diagnostic as LspDiag, DiagnosticSeverity, NumberOrString, Position, Range, SymbolKind, Url,
};

pub(super) fn kind(uri: &Url) -> Option<FileKind> {
    FileKind::from_path(Path::new(uri.path()))
}
pub(super) fn diagnostics(uri: &Url, doc: &Document) -> Vec<LspDiag> {
    parse(&doc.text, doc.kind)
        .diagnostics(&litigate_base::FileId(uri.path().to_owned()))
        .iter()
        .map(|d| lsp_diagnostic(d, &doc.text))
        .collect()
}
pub(super) fn lsp_diagnostic(d: &CompilerDiagnostic, text: &str) -> LspDiag {
    LspDiag {
        range: span_range(&d.span, text),
        severity: Some(match d.severity {
            Severity::Error => DiagnosticSeverity::ERROR,
            Severity::Warning => DiagnosticSeverity::WARNING,
            Severity::Info => DiagnosticSeverity::INFORMATION,
            Severity::Hint => DiagnosticSeverity::HINT,
        }),
        code: Some(NumberOrString::String(d.code.as_str().to_owned())),
        source: Some("litigate".into()),
        message: d.message.clone(),
        ..LspDiag::default()
    }
}
pub(super) fn is_declaration(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::StateDecl
            | SyntaxKind::DecisionDecl
            | SyntaxKind::ChanceDecl
            | SyntaxKind::TerminalDecl
            | SyntaxKind::DeadlineDecl
            | SyntaxKind::GroupDecl
            | SyntaxKind::TypeDecl
            | SyntaxKind::EnumDecl
            | SyntaxKind::EventDecl
            | SyntaxKind::CountingDecl
            | SyntaxKind::CalendarDecl
            | SyntaxKind::TransitionDecl
            | SyntaxKind::RuleDecl
            | SyntaxKind::InstanceDecl
    )
}
pub(super) fn symbol_kind(kind: SyntaxKind) -> SymbolKind {
    match kind {
        SyntaxKind::DeadlineDecl => SymbolKind::CONSTANT,
        SyntaxKind::EventDecl => SymbolKind::FUNCTION,
        SyntaxKind::TypeDecl | SyntaxKind::EnumDecl => SymbolKind::STRUCT,
        SyntaxKind::StateDecl
        | SyntaxKind::DecisionDecl
        | SyntaxKind::ChanceDecl
        | SyntaxKind::TerminalDecl => SymbolKind::OBJECT,
        _ => SymbolKind::NAMESPACE,
    }
}
pub(super) fn token_at(
    root: &litigate_syntax::SyntaxNode,
    pos: Position,
) -> Option<litigate_syntax::SyntaxToken> {
    let offset = offset_at(&root.to_string(), pos);
    root.token_at_offset(litigate_syntax::TextSize::try_from(offset).ok()?)
        .right_biased()
}
pub(super) fn offset_at(text: &str, p: Position) -> usize {
    let mut line = 0_u32;
    let mut start = 0;
    for (i, b) in text.bytes().enumerate() {
        if line == p.line {
            break;
        }
        if b == b'\n' {
            line += 1;
            start = i + 1;
        }
    }
    let tail = text.get(start..).unwrap_or_default();
    start
        + tail
            .char_indices()
            .take_while(|(i, _)| {
                u32::try_from(tail[..*i].encode_utf16().count()).unwrap_or(u32::MAX) < p.character
            })
            .last()
            .map_or(0, |(i, c)| i + c.len_utf8())
}
pub(super) fn to_range(range: litigate_syntax::TextRange, text: &str) -> Range {
    let start = offset_position(text, usize::from(range.start()));
    let end = offset_position(text, usize::from(range.end()));
    Range::new(start, end)
}
fn offset_position(text: &str, offset: usize) -> Position {
    let index = LineIndex::new(text);
    let (line, _) = index.line_col(offset);
    let line_start = index.line_start(line).unwrap_or(0);
    let before = text
        .get(line_start..offset.min(text.len()))
        .unwrap_or_default();
    Position::new(
        line.saturating_sub(1),
        u32::try_from(before.encode_utf16().count()).unwrap_or(u32::MAX),
    )
}
pub(super) fn span_range(span: &litigate_base::Span, text: &str) -> Range {
    let idx = LineIndex::new(text);
    let at = |line: u32, col: u32| {
        let start = idx.line_start(line).unwrap_or(text.len());
        let s = text.get(start..).unwrap_or_default();
        let off = s
            .char_indices()
            .nth(usize::try_from(col.saturating_sub(1)).unwrap_or(usize::MAX))
            .map_or(text.len(), |(i, _)| start + i);
        offset_position(text, off)
    };
    Range::new(at(span.line, span.col), at(span.end_line, span.end_col))
}
pub(super) fn full_range(text: &str) -> Range {
    Range::new(Position::new(0, 0), offset_position(text, text.len()))
}
pub(super) fn replace_range(text: &str, range: Range, replacement: &str) -> String {
    let start = offset_at(text, range.start);
    let end = offset_at(text, range.end);
    if start <= end && text.is_char_boundary(start) && text.is_char_boundary(end) {
        format!("{}{}{}", &text[..start], replacement, &text[end..])
    } else {
        text.to_owned()
    }
}
pub(super) fn cite_source(cite: &str, root: Option<&Path>) -> Option<String> {
    let root = root?;
    let source_root = root.join("sources");
    for entry in std::fs::read_dir(source_root).ok()?.flatten() {
        let content = std::fs::read_to_string(entry.path()).ok()?;
        if let Some(line) = content
            .lines()
            .find(|line| line.to_lowercase().contains(&cite.to_lowercase()))
        {
            return Some(line.trim().to_owned());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use litigate_syntax::format;

    #[test]
    fn publishes_parser_diagnostics_with_utf16_ranges() {
        let uri = Url::parse("file:///tmp/rule.lit").unwrap();
        let doc = Document {
            text: "module λ\nstate x: \"open\" {\n".into(),
            version: 4,
            kind: FileKind::Lit,
        };
        let found = diagnostics(&uri, &doc);
        assert!(!found.is_empty());
        assert_eq!(found[0].source.as_deref(), Some("litigate"));
        assert!(found[0].range.start.line >= 1);
    }

    #[test]
    fn incremental_edit_uses_utf16_positions() {
        let original = "αβ\nnext";
        let edited = replace_range(
            original,
            Range::new(Position::new(0, 1), Position::new(0, 2)),
            "x",
        );
        assert_eq!(edited, "αx\nnext");
    }

    #[test]
    fn formatter_returns_a_full_document_edit() {
        let source = "module m\nstart s\n";
        assert_eq!(
            format::format(source, FileKind::Lit).as_deref(),
            Ok("module m\n\nstart s\n")
        );
        let range = full_range(source);
        assert_eq!(range.start, Position::new(0, 0));
        assert_eq!(range.end.line, 2);
    }
}
