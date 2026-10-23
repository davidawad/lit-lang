//! Compact, assertable renderings of parse diagnostics.
use litigate_base::{Applicability, FileId, Span};
use litigate_syntax::{parse, FileKind};

fn span(span: &Span) -> String {
    format!(
        "{}:{}-{}:{}",
        span.line, span.col, span.end_line, span.end_col
    )
}

/// Each diagnostic of `text` as `CODE line:col-line:col message`, followed
/// by ` | note: ...` and ` | fix line:col-line:col "text"` parts (a fix that
/// needs review is marked `(review)`).
pub fn render(text: &str, kind: FileKind) -> Vec<String> {
    let parsed = parse(text, kind);
    assert_eq!(parsed.syntax().to_string(), text, "parse is not lossless");
    parsed
        .diagnostics(&FileId::from("t"))
        .iter()
        .map(|diagnostic| {
            let head = format!(
                "{} {} {}",
                diagnostic.code.as_str(),
                span(&diagnostic.span),
                diagnostic.message
            );
            let notes = diagnostic
                .notes
                .iter()
                .map(|note| format!("note: {}", note.message));
            let fixes = diagnostic.fixes.iter().flat_map(|fix| {
                let review = if fix.applicability == Applicability::MachineApplicable {
                    ""
                } else {
                    " (review)"
                };
                fix.edits
                    .iter()
                    .map(move |e| format!("fix {} {:?}{review}", span(&e.span), e.replacement))
            });
            std::iter::once(head)
                .chain(notes)
                .chain(fixes)
                .collect::<Vec<_>>()
                .join(" | ")
        })
        .collect()
}

/// Asserts the rendered diagnostics of every `(source, expected)` case.
pub fn check_all(kind: FileKind, cases: &[(&str, &[&str])]) {
    let mut failures = Vec::new();
    for (text, expected) in cases {
        let actual = render(text, kind);
        if actual != *expected {
            failures.push(format!("{text:?}\n  actual: {actual:#?}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}
