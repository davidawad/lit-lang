//! `litigate fmt PATHS [--check]` and `litigate fmt --stdin` (SPEC §10).
use crate::cli::Format;
use crate::out::{Outcome, Printed, Report};
use crate::ws;
use litigate_base::{FileId, Severity};
use litigate_syntax::{FileKind, SyntaxKind};
use serde_json::json;

/// `litigate fmt`.
pub fn fmt(paths: &[String], check: bool) -> Outcome {
    let targets = ws::discover(paths)?;
    let mut report = Report::of(serde_json::Value::Null);
    let mut files = Vec::new();
    for path in &targets.files {
        let shown = ws::shown(path);
        let text = std::fs::read_to_string(path)
            .map_err(|e| crate::out::Failure::new(format!("{shown}: {e}")))?;
        // Discovery finds `.case` matters and `.lit` modules only.
        let kind = if targets.cases.binary_search(path).is_ok() {
            FileKind::Case
        } else {
            FileKind::Lit
        };
        let file = FileId(shown.clone());
        report.sources.push((shown.clone(), text.clone()));
        match litigate_syntax::format(&text, kind) {
            Err(_) => report
                .diagnostics
                .extend(litigate_syntax::parse(&text, kind).diagnostics(&file)),
            Ok(formatted) => {
                let changed = formatted != text;
                if changed && check {
                    report.diagnostics.extend(
                        litigate_syntax::check_format(&text, kind, &file).map(|d| {
                            litigate_base::Diagnostic {
                                severity: Severity::Error,
                                ..d
                            }
                        }),
                    );
                } else if changed {
                    crate::cmd::build::write_atomic(path, &formatted)?;
                }
                files.push(json!({"path": shown, "changed": changed}));
            }
        }
    }
    report.result = json!({ "files": files });
    Ok(report)
}

/// `litigate fmt --stdin`: `text` formatted, alone on standard output (an
/// editor's formatter replaces the buffer with it). Text that does not
/// parse leaves standard output empty and exits 2, its diagnostics on
/// standard error.
#[must_use]
pub fn stdin(text: &str) -> Printed {
    let kind = sniff(text);
    if let Ok(formatted) = litigate_syntax::format(text, kind) {
        return Printed {
            stdout: formatted,
            stderr: String::new(),
            code: 0,
        };
    }
    let shown = "<stdin>".to_owned();
    let mut report = Report::of(serde_json::Value::Null);
    report.diagnostics = litigate_syntax::parse(text, kind).diagnostics(&FileId(shown.clone()));
    report.sources.push((shown, text.to_owned()));
    let printed = crate::out::print("fmt", Ok(report), Format::Human, false, 0.0);
    Printed {
        stdout: String::new(),
        stderr: printed.stdout + &printed.stderr,
        code: 2,
    }
}

/// A matter when the first token past comments is `case`, else a module.
fn sniff(text: &str) -> FileKind {
    let (tokens, _) = litigate_syntax::lexer::lex(text);
    let first = tokens.iter().find(|token| {
        !matches!(
            token.kind,
            SyntaxKind::Whitespace | SyntaxKind::Comment | SyntaxKind::DocComment
        )
    });
    match first {
        Some(token) if &text[token.range] == "case" => FileKind::Case,
        _ => FileKind::Lit,
    }
}
