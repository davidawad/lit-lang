//! `litigate test PATHS`: run inline `test` blocks (SPEC §4.7) and the
//! expect tests (SPEC §5.8, expect tests): a path at or above a directory holding an
//! `expect.toml` runs the expect tests under it, any other path names
//! packages whose inline tests run.
use super::expect::{self, CaseResult};
use super::impact;
use crate::clock::Clocks;
use crate::inline_tests::{self, TestResult};
use crate::out::{Outcome, Report};
use crate::ws::{self, Session};
use litigate_base::Diagnostic;
use litigate_sema::pkg::source::Disk;
use serde_json::json;
use std::path::Path;

/// `litigate test`; `bless` records each expect test's values found.
pub fn test(paths: &[String], filter: Option<&str>, lib: &[String], bless: bool) -> Outcome {
    let mut dirs = Vec::new();
    let mut packages = Vec::new();
    for path in paths {
        let found = expect::dirs(&Disk, Path::new(path));
        if found.is_empty() {
            packages.push(path.clone());
        } else {
            dirs.extend(found);
        }
    }
    let mut report = Report::of(serde_json::Value::Null);
    let mut results = Vec::new();
    if !packages.is_empty() {
        let targets = ws::discover(&packages)?;
        let session = Session::load(&targets, lib)?;
        let (tests, diagnostics) = run(&session, filter);
        results = tests;
        report.diagnostics = diagnostics;
        report.provenance = crate::prov::session(session.requested(), None, None);
        report.sources = session.sources();
    }
    let mut cases: Vec<CaseResult> = Vec::new();
    for dir in dirs {
        if filter.is_some_and(|f| !expect::name_of(&dir).contains(f)) {
            continue;
        }
        let (result, diagnostics, source) = expect::run(&Disk, &dir, lib, bless)?;
        report.diagnostics.extend(diagnostics);
        report.sources.push(source);
        cases.push(result);
    }
    let passed = results.iter().filter(|t| t.passed).count();
    let expect_passed = cases.iter().filter(|c| c.passed).count();
    if !cases.is_empty() {
        report.text = Some(text(&cases, passed, results.len() - passed));
    }
    report.result = json!({
        "passed": passed + expect_passed,
        "failed": results.len() - passed + cases.len() - expect_passed,
        "tests": results,
        "expect": cases,
    });
    Ok(report)
}

/// The human summary: the counts, then one table of case, world, field,
/// expected and actual per expect test with a value that moved.
fn text(cases: &[CaseResult], passed: usize, failed: usize) -> String {
    let moved: Vec<&CaseResult> = cases.iter().filter(|c| !c.mismatches.is_empty()).collect();
    let blessed = cases.iter().filter(|c| c.blessed).count();
    let mut out = format!(
        "{} expect test(s): {} passed, {} failed{}; {passed} inline test(s) passed, {failed} failed\n",
        cases.len(),
        cases.iter().filter(|c| c.passed && !c.blessed).count(),
        cases.iter().filter(|c| !c.passed).count(),
        if blessed > 0 { format!(", {blessed} re-recorded") } else { String::new() },
    );
    for case in moved {
        let rows: Vec<[&str; 5]> = case
            .mismatches
            .iter()
            .map(|m| [m.case.as_str(), &m.world, &m.field, &m.expected, &m.actual])
            .collect();
        out.push_str(&impact::grid(
            &case.dir,
            ["case", "world", "field", "expected", "actual"],
            &rows,
        ));
    }
    out
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
    super::vendor::retarget(&session.packages, &mut diagnostics);
    (results, diagnostics)
}
