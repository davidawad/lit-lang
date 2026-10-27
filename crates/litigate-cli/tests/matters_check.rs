//! Every matter checks clean (lg-flu.48): each `.case` under `cases/` (the
//! end-to-end TTAB matters among them) and the timeline fixture matters,
//! with every rule package they use, is `ok: true` with no error in
//! `litigate check` and `litigate timeline`, so a package whose cite pins
//! do not bind to the vendored text cannot pass the gate. A matter wrong
//! by design (`REJECTED`) must fail with its one error code and no other.
//! A binary's corpus is the `sources/` it embedded when built, so a
//! `litigate` older than a pin (the PM's 5 L1010 on `Sup. Ct. R.` after
//! the 2026 Rules were vendored) reports the mismatch with how to fix it,
//! and `LITGRAPH_SOURCES` points it at the vendored text instead.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use serde_json::Value;
use std::path::{Path, PathBuf};
use support::{copy_dir, litigate_in, repo, scratch, Run};

/// Every `.case` file under `dir`, relative to `root`, sorted.
fn cases_under(root: &Path, dir: &Path) -> Vec<String> {
    let mut found = Vec::new();
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            found.extend(cases_under(root, &path));
        } else if path.extension().is_some_and(|e| e == "case") {
            let shown = path.strip_prefix(root).unwrap();
            found.push(shown.to_string_lossy().replace('\\', "/"));
        }
    }
    found.sort();
    found
}

/// The matters that are wrong by design, each with the one error code it
/// must report: a notice designating less than the thirty days 37 C.F.R.
/// § 2.105(a) allows (`e2e_ttab.rs`). Every other matter is clean.
const REJECTED: &[(&str, &str)] = &[("cases/e2e-ttab-short.case", "L4001")];

/// The code `case` is rejected with by design, if any.
fn rejected(case: &str) -> Option<&'static str> {
    REJECTED
        .iter()
        .find(|(file, _)| *file == case)
        .map(|(_, code)| *code)
}

/// The error diagnostics of `run`, as `file code message`.
fn errors(run: &Run) -> Vec<String> {
    run.json["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| d["severity"] == "error")
        .map(|d| {
            format!(
                "{} {} {}",
                d["span"]["file"].as_str().unwrap_or_default(),
                d["code"].as_str().unwrap(),
                d["message"]
            )
        })
        .collect()
}

/// `run` is a clean envelope: exit 0, `ok: true`, no error.
fn assert_clean(what: &str, run: &Run) {
    assert_eq!(errors(run), Vec::<String>::new(), "{what}");
    assert_eq!(run.json["ok"], Value::Bool(true), "{what}: {}", run.stdout);
    assert_eq!(run.code, 0, "{what}: {}", run.stderr);
}

/// `run` fails (exit 2, `ok: false`) on errors in `cases` alone, each of
/// them `rejected`'s code for its file, and on each of `cases` at least once.
fn assert_rejected(what: &str, run: &Run, cases: &[&str]) {
    let errors = errors(run);
    for e in &errors {
        let mut words = e.split(' ');
        let (file, code) = (words.next().unwrap(), words.next().unwrap());
        assert!(
            cases.contains(&file) && rejected(file) == Some(code),
            "{what}: {e}"
        );
    }
    for case in cases {
        assert!(
            errors.iter().any(|e| e.starts_with(&format!("{case} "))),
            "{what}: {case} is no longer rejected: {errors:#?}"
        );
    }
    assert_eq!(run.json["ok"], Value::Bool(false), "{what}: {}", run.stdout);
    assert_eq!(run.code, 2, "{what}: {}", run.stderr);
}

/// `run` is clean, or rejected for `rejects` alone when there are any.
fn assert_clean_but(what: &str, run: &Run, rejects: &[&str]) {
    if rejects.is_empty() {
        assert_clean(what, run);
    } else {
        assert_rejected(what, run, rejects);
    }
}

/// `litigate check` of every package under `dir/lit` and every matter
/// under `dir/cases` is clean but for the `REJECTED` ones and covers each
/// matter, and so is the `timeline` of each (which carries the same
/// check's diagnostics, as the PM's run did).
fn every_matter_is_clean(dir: &Path) -> Vec<String> {
    let cases = cases_under(dir, &dir.join("cases"));
    let rejects: Vec<&str> = cases
        .iter()
        .map(String::as_str)
        .filter(|c| rejected(c).is_some())
        .collect();
    let check = litigate_in(dir, &["check", "lit", "cases"]);
    assert_clean_but("check lit cases", &check, &rejects);
    let files = check.json["result"]["files"].as_array().unwrap();
    for case in &cases {
        assert!(files.iter().any(|f| f == case.as_str()), "{case}");
        let own: &[&str] = if rejected(case).is_some() {
            &[case.as_str()]
        } else {
            &[]
        };
        assert_clean_but(
            &format!("timeline {case}"),
            &litigate_in(dir, &["timeline", case]),
            own,
        );
    }
    cases
}

#[test]
fn every_matter_under_cases_checks_ok_with_no_error() {
    let cases = every_matter_is_clean(&repo());
    assert!(cases.len() >= 20, "{cases:?}");
    for (case, _) in REJECTED {
        assert!(cases.contains(&(*case).to_owned()), "{case}");
    }
    for case in [
        "cases/ttab-opposition-applicant-defense.case",
        "cases/e2e-ttab-ok.case",
        "cases/e2e-ttab-unknown.case",
    ] {
        assert!(cases.contains(&case.to_owned()), "{case}");
    }
}

/// A scratch tree with the TTAB, FRCP, FRAP and core packages, the
/// timeline fixture package, and the timeline fixture matters under
/// `cases/`, locked.
fn fixtures(name: &str) -> PathBuf {
    let dir = scratch(name);
    for pkg in ["ttab", "frcp", "frap", "core"] {
        copy_dir(&repo().join("lit").join(pkg), &dir.join("lit").join(pkg));
    }
    copy_dir(&repo().join("sources"), &dir.join("sources"));
    copy_dir(
        &repo().join("crates/litigate-cli/tests/fixtures/timeline-package"),
        &dir.join("lit/fixture"),
    );
    copy_dir(
        &repo().join("crates/litigate-cli/tests/fixtures/timeline"),
        &dir.join("cases"),
    );
    let lock = litigate_in(&dir, &["lock", "cases"]);
    assert_eq!(lock.code, 0, "{}", lock.stdout);
    dir
}

#[test]
fn every_fixture_matter_checks_ok_with_no_error() {
    let cases = every_matter_is_clean(&fixtures("matters-fixtures"));
    assert!(
        cases.contains(&"cases/ttab-published.case".to_owned()),
        "{cases:?}"
    );
}

/// `litigate check` of the TTAB matter with `LITGRAPH_SOURCES` set to
/// `sources`.
fn check_with_sources(sources: &Path) -> Run {
    let out = assert_cmd::Command::cargo_bin("litigate")
        .unwrap()
        .current_dir(repo())
        .env("LITGRAPH_SOURCES", sources)
        .args([
            "--format",
            "json",
            "check",
            "--profile",
            "strict",
            "cases/ttab-opposition-applicant-defense.case",
        ])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    Run {
        json: serde_json::from_str(&stdout).unwrap(),
        stdout,
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        code: out.status.code().unwrap_or(-1),
    }
}

#[test]
fn the_vendored_sources_override_the_embedded_corpus() {
    let run = check_with_sources(&repo().join("sources"));
    let errors = errors(&run);
    // This check's purpose is to ensure the current vendored corpus overrides
    // the one embedded at compile time. The TTAB matter also has authored
    // strict-mode holes (L5002), which are independent of source resolution.
    // Keep the assertion focused on the regression: no missing or mismatched
    // vendored citations when the override points at `sources/`.
    assert!(
        errors.iter().all(|error| error.contains(" L5002 ")),
        "LITGRAPH_SOURCES=sources should resolve vendored citations: {errors:#?}"
    );
    assert_eq!(run.json["ok"], Value::Bool(false), "{}", run.stdout);
    assert_eq!(run.code, 2, "{}", run.stderr);
}

#[test]
fn a_corpus_older_than_the_pin_names_the_stale_binary() {
    // The corpus a binary built before the pin changed embeds: every
    // vendored file but the Supreme Court Rules as the pin does not hold.
    let old = scratch("matters-stale-sources");
    for entry in std::fs::read_dir(repo().join("sources")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "txt") {
            std::fs::copy(&path, old.join(path.file_name().unwrap())).unwrap();
        }
    }
    let rules = old.join("scotus-rules.txt");
    let text = std::fs::read_to_string(&rules).unwrap();
    std::fs::write(&rules, format!("{text}\n")).unwrap();
    let run = check_with_sources(&old);
    assert_eq!(run.code, 2, "{}", run.stdout);
    let errors: Vec<&Value> = run.json["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| d["severity"] == "error")
        .collect();
    let stale: Vec<&Value> = errors
        .iter()
        .copied()
        .filter(|d| {
            d["code"] == "L1010" && d["message"].as_str().unwrap().contains("source `scotus`")
        })
        .collect();
    // `lit/core`'s cites through `scotus`: the current `sup-ct-30` version
    // and three tests (its 2023 version cites the 2023 Rules).
    assert_eq!(stale.len(), 4, "{stale:#?}");
    assert!(
        errors
            .iter()
            .all(|d| d["code"] == "L1010" || d["code"] == "L5002"),
        "{errors:#?}"
    );
    for d in stale {
        assert_eq!(d["code"], "L1010");
        let message = d["message"].as_str().unwrap();
        assert!(
            message.contains("is not the text source `scotus` pins"),
            "{message}"
        );
        let note = d["notes"][0]["message"].as_str().unwrap();
        assert!(
            note.contains("rebuild it or set `LITGRAPH_SOURCES`"),
            "{note}"
        );
    }
}

#[test]
fn an_unreadable_sources_override_fails_every_vendored_cite_closed() {
    let run = check_with_sources(&repo().join("sources/no-such-directory"));
    assert_eq!(run.code, 2, "{}", run.stdout);
    let messages = errors(&run);
    assert!(!messages.is_empty());
    let cite_errors: Vec<&String> = messages.iter().filter(|m| m.contains(" L1010 ")).collect();
    assert!(!cite_errors.is_empty(), "{messages:#?}");
    assert!(
        cite_errors.iter().all(|m| {
            m.contains("is not in the vendored source corpus")
                || m.contains("is not the text source")
        }),
        "{cite_errors:#?}"
    );
    assert!(
        messages
            .iter()
            .all(|m| m.contains(" L1010 ") || m.contains(" L5002 ")),
        "{messages:#?}"
    );
}
