//! `litigate test PATHS`: run inline `test` blocks (SPEC §4.7).
use crate::clock::Clocks;
use crate::out::{Outcome, Report};
use crate::ws::Session;
use crate::{inline_tests, ws};
use serde_json::json;

/// `litigate test`.
pub fn test(paths: &[String], filter: Option<&str>, lib: &[String]) -> Outcome {
    let targets = ws::discover(paths)?;
    let session = Session::load(&targets, lib)?;
    let clocks = Clocks::new(&session.hir());
    let mut report = Report::of(serde_json::Value::Null);
    let mut results = Vec::new();
    for pkg in session.requested() {
        match &pkg.check.package {
            Some(hir) => {
                let (tests, failures) = inline_tests::run(&clocks, hir, &pkg.files, filter, None);
                results.extend(tests);
                report.diagnostics.extend(failures);
            }
            None => report
                .diagnostics
                .extend(pkg.check.diagnostics.iter().cloned()),
        }
    }
    let passed = results.iter().filter(|t| t.passed).count();
    report.result = json!({
        "passed": passed,
        "failed": results.len() - passed,
        "tests": results,
    });
    report.provenance = crate::prov::session(session.requested(), None, None);
    report.sources = session.sources();
    Ok(report)
}
