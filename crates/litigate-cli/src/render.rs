//! Human rendering: diagnostics with source excerpts (ariadne), then a
//! status line and the result as indented JSON.
use crate::span::byte_range;
use ariadne::{Color, Config, Label, Report, ReportKind, Source};
use litigate_base::{Diagnostic, Envelope, Severity};
use serde_json::Value;
use std::fmt::Write as _;

/// One diagnostic as text, with its excerpt when the source is known.
#[must_use]
pub fn diagnostic(d: &Diagnostic, sources: &[(String, String)], color: bool) -> String {
    let file = d.span.file.0.clone();
    let Some((_, text)) = sources.iter().find(|(path, _)| *path == file) else {
        return format!(
            "{}[{}]: {}\n  --> {}:{}:{}\n",
            severity_word(d.severity),
            d.code.as_str(),
            d.message,
            file,
            d.span.line,
            d.span.col
        );
    };
    let range = byte_range(text, &d.span);
    let kind = match d.severity {
        Severity::Error => ReportKind::Error,
        Severity::Warning => ReportKind::Warning,
        Severity::Info | Severity::Hint => ReportKind::Advice,
    };
    let tint = match d.severity {
        Severity::Error => Color::Red,
        Severity::Warning => Color::Yellow,
        Severity::Info | Severity::Hint => Color::Blue,
    };
    let mut report = Report::build(kind, (file.clone(), range.clone()))
        .with_config(Config::default().with_color(color))
        .with_code(d.code.as_str())
        .with_message(&d.message)
        .with_label(
            Label::new((file.clone(), range))
                .with_message(d.code.explain())
                .with_color(tint),
        );
    for label in &d.labels {
        if label.span.file.0 == file {
            report = report.with_label(
                Label::new((file.clone(), byte_range(text, &label.span)))
                    .with_message(&label.message),
            );
        } else {
            report = report.with_note(format!(
                "{}:{}:{}: {}",
                label.span.file.0, label.span.line, label.span.col, label.message
            ));
        }
    }
    for note in &d.notes {
        report = report.with_note(&note.message);
    }
    if let Some(derivation) = &d.derivation {
        let mut tree = String::from("derivation:");
        derivation_lines(derivation, 1, &mut tree);
        report = report.with_note(tree);
    }
    for fix in &d.fixes {
        let edits: Vec<String> = fix
            .edits
            .iter()
            .map(|e| {
                format!(
                    "replace {}:{}:{}-{}:{} with `{}`",
                    e.span.file.0,
                    e.span.line,
                    e.span.col,
                    e.span.end_line,
                    e.span.end_col,
                    e.replacement
                )
            })
            .collect();
        let lead = fix
            .message
            .as_ref()
            .map_or(String::new(), |m| format!("{m}: "));
        report = report.with_help(format!(
            "{lead}{} ({:?})",
            edits.join("; "),
            fix.applicability
        ));
    }
    let mut buffer = Vec::new();
    // Writing to a `Vec` cannot fail.
    let _ = report
        .finish()
        .write((file, Source::from(text.clone())), &mut buffer);
    String::from_utf8_lossy(&buffer).into_owned()
}

/// `derivation` as an indented tree, one step per line.
fn derivation_lines(derivation: &litigate_base::Derivation, depth: usize, out: &mut String) {
    let _ = write!(
        out,
        "\n{}{}: {}",
        "  ".repeat(depth),
        derivation.rule,
        derivation.fact
    );
    for premise in &derivation.premises {
        derivation_lines(premise, depth + 1, out);
    }
}

/// The lowercase severity name (`error`, `warning`, `info`, `hint`).
pub(crate) fn severity_word(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
        Severity::Hint => "hint",
    }
}

/// The whole envelope for a terminal.
#[must_use]
pub fn human(envelope: &Envelope<Value>, sources: &[(String, String)], color: bool) -> String {
    let mut out: String = envelope
        .diagnostics
        .iter()
        .map(|d| diagnostic(d, sources, color))
        .collect();
    let count = |s: Severity| {
        envelope
            .diagnostics
            .iter()
            .filter(|d| d.severity == s)
            .count()
    };
    let _ = writeln!(
        out,
        "litigate {}: {} ({} error(s), {} warning(s))",
        envelope.op,
        if envelope.ok { "ok" } else { "failed" },
        count(Severity::Error),
        count(Severity::Warning)
    );
    if let Some(result) = &envelope.result {
        out.push_str(&serde_json::to_string_pretty(result).unwrap_or_default());
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use litigate_base::{DiagnosticCode, Span};

    #[test]
    fn renders_excerpt_and_fallback() {
        let span = Span {
            file: "a.lit".into(),
            line: 1,
            col: 8,
            end_line: 1,
            end_col: 9,
        };
        let d = Diagnostic::error(DiagnosticCode::UnresolvedName, span, "no `m`");
        let text = diagnostic(&d, &[("a.lit".into(), "module m\n".into())], false);
        assert!(text.contains("L1002"), "{text}");
        assert!(text.contains("module m"), "{text}");
        let bare = diagnostic(&d, &[], false);
        assert!(bare.starts_with("error[L1002]: no `m`"), "{bare}");
    }

    #[test]
    fn every_severity_renders_its_notes_and_fixes() {
        let span = Span {
            file: "a.lit".into(),
            line: 1,
            col: 8,
            end_line: 1,
            end_col: 9,
        };
        let sources = [("a.lit".to_owned(), "module m\n".to_owned())];
        for (severity, word) in [
            (Severity::Error, "error"),
            (Severity::Warning, "warning"),
            (Severity::Info, "info"),
            (Severity::Hint, "hint"),
        ] {
            assert_eq!(severity_word(severity), word);
            let mut d = Diagnostic {
                severity,
                ..Diagnostic::error(DiagnosticCode::UnresolvedName, span.clone(), "no `m`")
            }
            .note("a note")
            .label(
                Span {
                    col: 1,
                    end_col: 7,
                    ..span.clone()
                },
                "in this module",
            )
            .label(
                Span {
                    file: "b.lit".into(),
                    ..span.clone()
                },
                "used here",
            )
            .fix(
                span.clone(),
                "n",
                litigate_base::Applicability::MachineApplicable,
            )
            .derived(
                litigate_base::Derivation::new("conflict", "conflict(m, a, b)")
                    .premise(litigate_base::Derivation::new("winner", "winner(m, a)")),
            );
            d.fixes[0].message = Some("rename it".into());
            insta::assert_snapshot!(
                format!("diagnostic_{word}"),
                diagnostic(&d, &sources, false)
            );
            let bare = diagnostic(&d, &[], false);
            assert!(bare.starts_with(&format!("{word}[L1002]")), "{bare}");
        }
    }

    #[test]
    fn a_clean_envelope_prints_ok_and_its_result() {
        let report = crate::out::Report::of(serde_json::json!({"answer": 42}));
        let envelope = crate::out::envelope("explain", &report, 0.0);
        insta::assert_snapshot!(human(&envelope, &[], true));
    }

    #[test]
    fn a_failed_envelope_prints_its_result_too() {
        let span = Span {
            file: "a.lit".into(),
            line: 1,
            col: 1,
            end_line: 1,
            end_col: 2,
        };
        let report = crate::out::Report {
            diagnostics: vec![Diagnostic::error(
                DiagnosticCode::UnresolvedName,
                span,
                "no `m`",
            )],
            ..crate::out::Report::of(serde_json::json!({"answer": 42}))
        };
        let envelope = crate::out::envelope("check", &report, 0.0);
        insta::assert_snapshot!(human(&envelope, &[], false), @r#"
        error[L1002]: no `m`
          --> a.lit:1:1
        litigate check: failed (1 error(s), 0 warning(s))
        {
          "answer": 42
        }
        "#);
    }
}
