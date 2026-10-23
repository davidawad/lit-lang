//! `fix` beyond machine-applicable fixes: `maybe_incorrect` ones only with
//! `--include-maybe`, an unknown `--code`, and an unwritable file.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use support::{example, litigate_in};

/// A misspelled edge target: L1002 with a `maybe_incorrect` suggestion.
const TYPO: &str = "state extra: \"Extra\" { _ \"Onward\" -> answerd }";

#[test]
fn maybe_incorrect_fixes_need_include_maybe() {
    let dir = example("fix-maybe", TYPO, None);
    let module = dir.join("lit/examples/rule12.lit");
    let before = std::fs::read_to_string(&module).unwrap();
    let plain = litigate_in(&dir, &["fix", "lit/examples"]);
    assert_eq!(plain.code, 0, "{}", plain.stderr);
    assert_eq!(plain.json["result"]["fixes"], serde_json::json!([]));
    assert_eq!(std::fs::read_to_string(&module).unwrap(), before);
    let maybe = litigate_in(&dir, &["fix", "--include-maybe", "lit/examples"]);
    assert_eq!(maybe.code, 0, "{}", maybe.stderr);
    insta::assert_json_snapshot!(maybe.json["result"]);
    assert_eq!(
        std::fs::read_to_string(&module).unwrap(),
        before.replace("-> answerd", "-> answered")
    );
}

#[test]
fn an_unknown_code_is_a_usage_error() {
    let dir = example("fix-code", "", None);
    let run = litigate_in(&dir, &["fix", "--code", "L0000", "lit/examples"]);
    assert_eq!(run.code, 1);
    insta::assert_snapshot!(run.stderr, @"litigate fix: `L0000` is not a diagnostic code");
}

#[cfg(unix)]
#[test]
fn an_unwritable_file_fails_fix() {
    let dir = example("fix-unwritable", TYPO, None);
    support::chmod(&dir.join("lit/examples"), 0o555);
    let run = litigate_in(&dir, &["fix", "--include-maybe", "lit/examples"]);
    support::chmod(&dir.join("lit/examples"), 0o755);
    assert_eq!(run.code, 1, "{}", run.stdout);
    insta::assert_snapshot!(run.stderr, @"litigate fix: lit/examples/rule12.litigate-tmp: Permission denied (os error 13)");
}
