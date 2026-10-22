//! Human rendering: diagnostics with source excerpts (ariadne), then a
//! status line and the result as indented JSON.
use crate::span::byte_range;
use ariadne::{Color, Config, Label, Report, ReportKind, Source};
use litigate_base::{Diagnostic, Envelope, Severity};
use serde_json::Value;

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
    for note in &d.notes {
        report = report.with_note(&note.message);
    }
    for fix in &d.fixes {
        report = report.with_help(format!(
            "replace {}:{}-{}:{} with `{}` ({:?})",
            fix.span.line,
            fix.span.col,
            fix.span.end_line,
            fix.span.end_col,
            fix.replacement,
            fix.applicability
        ));
    }
    let mut buffer = Vec::new();
    let written = report
        .finish()
        .write((file, Source::from(text.clone())), &mut buffer);
    match written {
        Ok(()) => String::from_utf8_lossy(&buffer).into_owned(),
        Err(_) => format!(
            "{}[{}]: {}\n",
            severity_word(d.severity),
            d.code.as_str(),
            d.message
        ),
    }
}

fn severity_word(severity: Severity) -> &'static str {
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
    out.push_str(&format!(
        "litigate {}: {} ({} error(s), {} warning(s))\n",
        envelope.op,
        if envelope.ok { "ok" } else { "failed" },
        count(Severity::Error),
        count(Severity::Warning)
    ));
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
}
