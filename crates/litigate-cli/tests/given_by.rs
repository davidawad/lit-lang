//! A test's `given e by actor on date` (lg-pxkt): an order given by the
//! actor holding a deadline's `extend` grant moves `deadline(d)` as replay
//! moves a matter's (`by judge cite ...`), in `litigate test` and in
//! `litigate check`'s `tests-pass`; an order naming no actor, or another,
//! extends nothing and the assertion fails (L5006). Over copies of the
//! real `lit/core` and `lit/frcp`: FRCP 12(a)(1)(A)(i)'s 21-day answer
//! period and FRCP 6(b)(1)'s court grants.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use support::{copy_dir, litigate_in, repo, scratch};

/// Served 2025-03-06, so the answer is due 2025-03-27 (lg-flu.24).
const TESTS: &str = r#"
test "a court order extends the answer period" cite "FRCP 6(b)(1)(A)" {
  given ProcessServed() on 2025-03-06
  given AnswerPeriodExtended(2025-04-30) by court on 2025-03-20
  expect deadline(answer-period) == 2025-04-30
  expect deadline(answer-period).nominal == 2025-03-27
}

test "after expiry the court extends on a motion" cite "FRCP 6(b)(1)(B)" {
  given ProcessServed() on 2025-03-06
  given ExtensionMoved(AnswerPeriod) by applicant on 2025-04-01
  given AnswerPeriodExtended(2025-05-01) by court on 2025-04-04
  expect deadline(answer-period) == 2025-05-01
}

test "an order given with no actor extends nothing" {
  given ProcessServed() on 2025-03-06
  given AnswerPeriodExtended(2025-04-30) on 2025-03-20
  expect deadline(answer-period) == 2025-04-30
}

test "an order by the applicant extends nothing" {
  given ProcessServed() on 2025-03-06
  given AnswerPeriodExtended(2025-04-30) by applicant on 2025-03-20
  expect deadline(answer-period) == 2025-04-30
}
"#;

/// A scratch copy of `lit/core` and `lit/frcp` with [`TESTS`] appended
/// to the module declaring the answer period.
fn workspace(name: &str) -> std::path::PathBuf {
    let dir = scratch(name);
    for pkg in ["core", "frcp"] {
        copy_dir(&repo().join("lit").join(pkg), &dir.join("lit").join(pkg));
    }
    #[cfg(unix)]
    std::os::unix::fs::symlink(repo().join("sources"), dir.join("sources")).unwrap();
    #[cfg(not(unix))]
    copy_dir(&repo().join("sources"), &dir.join("sources"));
    let periods = dir.join("lit/frcp/periods.lit");
    let text = std::fs::read_to_string(&periods).unwrap() + TESTS;
    std::fs::write(&periods, text).unwrap();
    dir
}

const NO_ACTOR: &str = "only court may extend `answer-period`, but `AnswerPeriodExtended(2025-04-30)` on 2025-03-20 names no actor (`given ... by <actor> on <date>`)";
const APPLICANT: &str = "only court may extend `answer-period`, but `AnswerPeriodExtended(2025-04-30)` on 2025-03-20 is by `applicant`";

#[test]
fn litigate_test_extends_by_the_actor_given() {
    let run = litigate_in(&workspace("given-by-test"), &["test", "lit/frcp"]);
    assert_eq!(run.code, 2, "{}", run.stdout);
    let tests = run.json["result"]["tests"].as_array().unwrap();
    let test = |name: &str| tests.iter().find(|t| t["name"] == name).unwrap();
    let extended = test("a court order extends the answer period");
    assert_eq!(extended["passed"], true, "{extended}");
    assert_eq!(
        extended["expectations"][0]["steps"][2],
        "extended to 2025-04-30 by `AnswerPeriodExtended(2025-04-30)` on 2025-03-20"
    );
    let moved = test("after expiry the court extends on a motion");
    assert_eq!(moved["passed"], true, "{moved}");
    for (name, why) in [
        ("an order given with no actor extends nothing", NO_ACTOR),
        ("an order by the applicant extends nothing", APPLICANT),
    ] {
        let t = test(name);
        assert_eq!(t["passed"], false, "{t}");
        assert_eq!(t["expectations"][0]["error"], why, "{t}");
    }
    let failed: Vec<&str> = run.json["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["code"].as_str().unwrap())
        .collect();
    assert_eq!(failed, ["L5006", "L5006"]);
}

#[test]
fn tests_pass_extends_by_the_actor_given() {
    let run = litigate_in(&workspace("given-by-check"), &["check", "lit/frcp"]);
    let failed: Vec<String> = run.json["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| d["code"] == "L5006")
        .map(|d| d["message"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(
        failed,
        [
            format!("cannot evaluate the assertion: {NO_ACTOR}"),
            format!("cannot evaluate the assertion: {APPLICANT}"),
        ],
        "{}",
        run.stdout
    );
}
