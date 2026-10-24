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
        report = report.with_help(fix_help(fix));
    }
    let mut buffer = Vec::new();
    // Writing to a `Vec` cannot fail.
    let _ = report
        .finish()
        .write((file, Source::from(text.clone())), &mut buffer);
    String::from_utf8_lossy(&buffer).into_owned()
}

/// One fix as help text: its message, then every edit it applies.
fn fix_help(fix: &litigate_base::Fix) -> String {
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
    format!("{lead}{} ({:?})", edits.join("; "), fix.applicability)
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
#[cfg(test)]
pub fn human(envelope: &Envelope<Value>, sources: &[(String, String)], color: bool) -> String {
    human_with_check(envelope, sources, color, None)
}

/// Human output with optional check-specific filtering. The serialized envelope
/// is never changed by this view.
#[must_use]
pub fn human_with_check(
    envelope: &Envelope<Value>,
    sources: &[(String, String)],
    color: bool,
    check: Option<&crate::out::CheckRender>,
) -> String {
    use std::collections::BTreeMap;
    let mut visible = Vec::new();
    let mut grouped: BTreeMap<String, [usize; 5]> = BTreeMap::new();
    for (index, d) in envelope.diagnostics.iter().enumerate() {
        let Some(policy) = check.filter(|p| !p.verbose) else {
            visible.push(index);
            continue;
        };
        let file = &d.span.file.0;
        if policy.own_files.contains(file) {
            if d.code.as_str() == "L9002" {
                let name = policy
                    .package_by_file
                    .get(file)
                    .cloned()
                    .unwrap_or_else(|| "matter".to_owned());
                grouped.entry(name).or_default()[4] += 1;
            } else if matches!(d.severity, Severity::Error | Severity::Warning) {
                visible.push(index);
            }
        } else if let Some(name) = policy.package_by_file.get(file) {
            let counts = grouped.entry(name.clone()).or_default();
            if d.code.as_str() == "L9002" {
                counts[4] += 1;
            } else {
                match d.severity {
                    Severity::Error => counts[0] += 1,
                    Severity::Warning => counts[1] += 1,
                    Severity::Info => counts[2] += 1,
                    Severity::Hint => counts[3] += 1,
                }
            }
        }
    }
    let mut out = String::new();
    for index in visible {
        out.push_str(&diagnostic(&envelope.diagnostics[index], sources, color));
    }
    for (name, counts) in grouped {
        let mut parts = Vec::new();
        for (i, label) in [(0, "error"), (1, "warning"), (2, "info"), (3, "hint")] {
            if counts[i] > 0 {
                parts.push(format!("{} {label}", counts[i]));
            }
        }
        if counts[4] > 0 {
            parts.push(format!("{} holes", counts[4]));
        }
        let _ = writeln!(out, "{name}: {}; --verbose to show", parts.join(", "));
    }
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
    fn an_envelope_without_a_result_prints_only_its_status() {
        let envelope = crate::out::envelope("fmt", &crate::out::Report::default(), 0.0);
        assert_eq!(
            human(&envelope, &[], false),
            "litigate fmt: ok (0 error(s), 0 warning(s))\n"
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn check_human_output_filters_workspace_noise_and_groups_holes() {
        let span = |file: &str| Span {
            file: file.into(),
            line: 1,
            col: 1,
            end_line: 1,
            end_col: 2,
        };
        let diagnostic = |file: &str, code, severity, message: &str| Diagnostic {
            severity,
            ..Diagnostic::error(code, span(file), message)
        };
        let own_warning = Diagnostic {
            severity: Severity::Warning,
            ..Diagnostic::error(DiagnosticCode::HolePresent, span("case.case"), "unknown")
                .note("details")
        };
        let own_error =
            Diagnostic::error(DiagnosticCode::UnresolvedName, span("mine.lit"), "bad name");
        let report = crate::out::Report {
            diagnostics: vec![
                own_warning,
                own_error,
                diagnostic(
                    "lit/ttab/rules.lit",
                    DiagnosticCode::AnonymousEdge,
                    Severity::Info,
                    "anonymous",
                ),
                diagnostic(
                    "lit/ttab/rules.lit",
                    DiagnosticCode::HolePresent,
                    Severity::Warning,
                    "unknown",
                ),
                diagnostic(
                    "lit/ttab/rules.lit",
                    DiagnosticCode::UnresolvedName,
                    Severity::Error,
                    "bad dependency name",
                ),
                diagnostic(
                    "lit/ttab/rules.lit",
                    DiagnosticCode::UnresolvedName,
                    Severity::Warning,
                    "dependency warning",
                ),
                diagnostic(
                    "lit/ttab/rules.lit",
                    DiagnosticCode::UnresolvedName,
                    Severity::Hint,
                    "dependency hint",
                ),
                diagnostic(
                    "unknown/file.lit",
                    DiagnosticCode::AnonymousEdge,
                    Severity::Info,
                    "unmapped dependency info",
                ),
                diagnostic(
                    "lit/itc/rules.lit",
                    DiagnosticCode::AnonymousEdge,
                    Severity::Info,
                    "ITC dependency info",
                ),
            ],
            check_render: Some(crate::out::CheckRender {
                own_files: ["mine.lit".to_owned(), "case.case".to_owned()].into(),
                package_by_file: [
                    ("mine.lit".to_owned(), "mine".to_owned()),
                    ("lit/ttab/rules.lit".to_owned(), "ttab".to_owned()),
                    ("lit/itc/rules.lit".to_owned(), "itc".to_owned()),
                ]
                .into(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let envelope = crate::out::envelope("check", &report, 0.0);
        let text = human_with_check(&envelope, &[], false, report.check_render.as_ref());
        assert!(text.contains("bad name"), "{text}");
        assert!(!text.contains("anonymous"), "{text}");
        assert!(
            text.contains("ttab: 1 error, 1 warning, 1 info, 1 hint, 1 holes; --verbose to show"),
            "{text}"
        );
        assert!(text.contains("itc: 1 info; --verbose to show"), "{text}");
        assert!(!text.contains("bad dependency name"), "{text}");
        assert!(!text.contains("dependency warning"), "{text}");
        assert!(!text.contains("dependency hint"), "{text}");
        assert!(!text.contains("unmapped dependency info"), "{text}");
        assert!(
            text.contains("matter: 1 holes; --verbose to show"),
            "{text}"
        );
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
