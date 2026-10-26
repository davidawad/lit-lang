//! Post-dismissal eval (lg-60eq): a synthetic matter in the posture of a
//! with-prejudice Rule 41(a)(1)(A)(i) notice met by a same-day Rule 60(b)
//! motion, seen from an unserved defendant appearing specially, with a
//! 28 U.S.C. 636(c) magistrate-judge consent decision and the FRCP
//! 54(d)(2)(B) / L.R. 54-5(a) fee clock
//! (`tests/fixtures/post-dismissal`). `litigate solve` must prefer
//! declining consent over silence (Roell v. Withrow, 538 U.S. 580 (2003);
//! Williams v. King, 875 F.3d 500 (9th Cir. 2017)), which turns on the
//! option-scoped consent flags (lg-teqz), and a protective fee step over
//! doing nothing (lg-i0eq), valued with the fee award offsetting our own
//! costs (lg-vwwr). Public law only: parties, dates and numbers are
//! invented and illustrative.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use serde_json::Value;
use std::path::PathBuf;
use support::{copy_dir, litigate_in, repo, scratch};

const CASE: &str = "cases/post-dismissal.case";

/// The fixture over a copy of `lit/core`, locked.
fn matter(name: &str) -> PathBuf {
    let dir = scratch(name);
    copy_dir(
        &repo().join("crates/litigate-cli/tests/fixtures/post-dismissal"),
        &dir,
    );
    copy_dir(&repo().join("lit/core"), &dir.join("lit/core"));
    // The copied locks pin the vendored L0 texts (SPEC §4.2).
    #[cfg(unix)]
    std::os::unix::fs::symlink(repo().join("sources"), dir.join("sources")).unwrap();
    #[cfg(not(unix))]
    copy_dir(&repo().join("sources"), &dir.join("sources"));
    for target in ["lit/post-dismissal", "cases"] {
        let lock = litigate_in(&dir, &["lock", target]);
        assert_eq!(lock.code, 0, "{}", lock.stdout);
    }
    dir
}

/// The solve result with every node's value and the full policy.
fn solve(name: &str) -> Value {
    let dir = matter(name);
    let run = litigate_in(
        &dir,
        &[
            "run",
            CASE,
            "--op",
            "solve",
            "--arg",
            "full_policy=true",
            "--arg",
            "all_values=true",
        ],
    );
    assert_eq!(run.code, 0, "{}", run.stdout);
    run.json["result"]["result"].clone()
}

/// The policy's choice at `node`: its edge id and Q.
fn choice<'a>(result: &'a Value, node: &str) -> (&'a str, f64) {
    let row = result["policy"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["node"] == node)
        .unwrap_or_else(|| panic!("no policy row for {node}"));
    (row["edge"].as_str().unwrap(), row["q"].as_f64().unwrap())
}

#[test]
fn the_package_and_matter_check_with_no_diagnostic_of_their_own() {
    let dir = matter("post-dismissal-check");
    // `lit/core` carries strict-profile holes of its own; none is ours.
    let strict = litigate_in(
        &dir,
        &["check", "lit/post-dismissal", "--profile", "strict"],
    );
    let ours: Vec<&Value> = strict.json["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| !d["span"]["file"].as_str().unwrap().contains("/core/"))
        .collect();
    assert!(ours.is_empty(), "{ours:?}");
    let check = litigate_in(&dir, &["check", "lit/post-dismissal", CASE]);
    assert_eq!(check.code, 0, "{}", check.stdout);
}

#[test]
fn the_fee_clock_runs_fourteen_days_from_the_notice() {
    let dir = matter("post-dismissal-calendar");
    let ics = dir.join("fee.ics");
    let run = litigate_in(&dir, &["calendar", CASE, "--out", ics.to_str().unwrap()]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    let events = run.json["result"]["events"].as_array().unwrap();
    assert_eq!(events.len(), 1, "{events:?}");
    assert_eq!(events[0]["deadline"], "post-dismissal::fee-motion-period");
    assert_eq!(events[0]["trigger_date"], "2026-09-01");
    assert_eq!(events[0]["due"], "2026-09-15");
    assert_eq!(events[0]["status"], "open");
}

#[test]
fn solve_declines_consent_and_files_the_fee_motion_protectively() {
    let result = solve("post-dismissal-solve");
    // Decline (100) and file (1,500): 0.7 x (30,000 + 1,600 recovered)
    // - 0.3 x 50,000 - 1,600.
    assert_eq!(result["value"].as_f64(), Some(5520.0));
    let line: Vec<&str> = result["best_line"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["id"].as_str().unwrap())
        .collect();
    assert_eq!(
        line[..2],
        [
            "post-dismissal::decline",
            "post-dismissal::protective{declined}"
        ],
        "{line:?}"
    );
    assert!(
        line.contains(&"post-dismissal::reassigned{declined,fees-preserved}"),
        "{line:?}"
    );
}

#[test]
fn declining_consent_beats_silence() {
    let result = solve("post-dismissal-consent");
    let (edge, decline) = choice(&result, "post-dismissal::consent");
    assert_eq!(edge, "post-dismissal::decline");
    // Silence charges nothing and leads to the unflagged fee step, so its
    // Q is that node's value: the ruling stays open to vacatur unless
    // later conduct implies consent, at a price.
    let silence = result["values"]["post-dismissal::fee-step"]
        .as_f64()
        .unwrap();
    assert!(decline > silence, "decline {decline} <= silence {silence}");
    assert_eq!(silence, -11750.0);
    let (conduct, _) = choice(&result, "post-dismissal::conduct{fees-preserved}");
    assert_eq!(conduct, "post-dismissal::appear{fees-preserved}");
}

#[test]
fn a_protective_fee_step_beats_doing_nothing_whatever_the_consent() {
    let result = solve("post-dismissal-fees");
    for flags in ["", "{consented}", "{declined}"] {
        let (edge, _) = choice(&result, &format!("post-dismissal::fee-step{flags}"));
        assert_eq!(edge, format!("post-dismissal::protective{flags}"));
    }
}
