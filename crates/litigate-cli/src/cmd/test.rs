//! `litigate test PATHS`: run inline `test` blocks (SPEC §4.7).
use crate::clock::Clocks;
use crate::inline_tests::{self, TestResult};
use crate::out::{Outcome, Report};
use crate::ws::{self, Session};
use litigate_base::Diagnostic;
use serde_json::json;

/// `litigate test`.
pub fn test(paths: &[String], filter: Option<&str>, lib: &[String]) -> Outcome {
    let targets = ws::discover(paths)?;
    let session = Session::load(&targets, lib)?;
    let (results, diagnostics) = run(&session, filter);
    let mut report = Report::of(serde_json::Value::Null);
    report.diagnostics = diagnostics;
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

/// Every inline test of the requested packages of `session`, and the
/// diagnostics of failing ones and of packages that did not check.
pub(crate) fn run(session: &Session, filter: Option<&str>) -> (Vec<TestResult>, Vec<Diagnostic>) {
    let clocks = Clocks::new(&session.hir());
    let mut results = Vec::new();
    let mut diagnostics = Vec::new();
    for pkg in session.requested() {
        match &pkg.check.package {
            Some(hir) => {
                let (tests, failures) = inline_tests::run(&clocks, hir, &pkg.files, filter, None);
                results.extend(tests);
                diagnostics.extend(failures);
            }
            None => diagnostics.extend(pkg.check.diagnostics.iter().cloned()),
        }
    }
    (results, diagnostics)
}
