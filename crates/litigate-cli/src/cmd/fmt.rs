//! `litigate fmt PATHS [--check]` (SPEC §10).
use crate::out::{Outcome, Report};
use crate::ws;
use litigate_base::{FileId, Severity};
use litigate_syntax::FileKind;
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
