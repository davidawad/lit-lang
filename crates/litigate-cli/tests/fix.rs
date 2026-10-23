//! `fix` applies machine-applicable fixes; the matter then checks clean.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use proptest::prelude::*;
use support::{codes, copy_dir, litigate_in, repo, scratch};

const OUT_OF_ORDER: &str = "case acme-v-widget
uses frcp-rule12-example ^0.1.0
as_of 2025-06-01
summary \"x\"

parties {
  acme: Plaintiff Corporation
  widget: Defendant Corporation
}

events {
  2025-03-20 MovedToDismiss(complaint) by widget
  2025-03-05 Served(complaint, Personal) on widget
}

facts { time-barred: not-barred }
stakes { judgment-entered: -2.usd, dismissed: 0.usd }
";

fn workspace(name: &str, case: &str) -> std::path::PathBuf {
    let dir = scratch(name);
    copy_dir(&repo().join("lit"), &dir.join("lit"));
    std::fs::create_dir_all(dir.join("cases")).unwrap();
    std::fs::write(dir.join("cases/m.case"), case).unwrap();
    let lock = litigate_in(&dir, &["lock", "cases"]);
    assert_eq!(lock.code, 0, "{}", lock.stdout);
    dir
}

#[test]
fn out_of_order_events_are_fixed() {
    let dir = workspace("fix", OUT_OF_ORDER);
    let before = litigate_in(&dir, &["check", "cases/m.case"]);
    assert_eq!(before.code, 2);
    assert!(codes(&before.json).contains(&("L4001".to_owned(), "error".to_owned())));
    let dry = litigate_in(&dir, &["fix", "cases/m.case", "--dry-run"]);
    assert_eq!(dry.json["result"]["fixes"].as_array().unwrap().len(), 1);
    assert_eq!(
        std::fs::read_to_string(dir.join("cases/m.case")).unwrap(),
        OUT_OF_ORDER
    );
    let fixed = litigate_in(&dir, &["fix", "cases/m.case", "--code", "L4001"]);
    assert_eq!(fixed.json["result"]["changed_files"][0], "cases/m.case");
    let after = litigate_in(&dir, &["check", "cases/m.case"]);
    assert_eq!(after.code, 0, "{}", after.stdout);
    assert_eq!(
        after.json["result"]["cases"][0]["start"],
        "frcp-rule12-example::mtd-ruling"
    );
}

#[test]
fn replay_errors_are_precise() {
    let late = OUT_OF_ORDER
        .replace(
            "2025-03-20 MovedToDismiss(complaint) by widget\n  2025-03-05",
            "2025-03-05",
        )
        .replace(
            "on widget\n}",
            "on widget\n  2025-04-20 MovedToDismiss(complaint) by widget\n}",
        );
    let dir = workspace("late", &late);
    let run = litigate_in(&dir, &["check", "cases/m.case"]);
    assert_eq!(run.code, 2);
    insta::assert_json_snapshot!(
        "late_motion",
        run.json["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|d| d["severity"] == "error")
            .collect::<Vec<_>>()
    );
    let wrong_party = OUT_OF_ORDER.replace(
        "MovedToDismiss(complaint) by widget",
        "MovedToDismiss(complaint) by acme",
    );
    let wrong_party = wrong_party.replace("  2025-03-20 MovedToDismiss(complaint) by acme\n  2025-03-05 Served(complaint, Personal) on widget", "  2025-03-05 Served(complaint, Personal) on widget\n  2025-03-20 MovedToDismiss(complaint) by acme");
    let dir = workspace("party", &wrong_party);
    let run = litigate_in(&dir, &["check", "cases/m.case"]);
    assert!(
        run.json["diagnostics"][0]["message"]
            .as_str()
            .unwrap()
            .contains("is taken by Defendant, but `acme` plays Plaintiff"),
        "{}",
        run.stdout
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]
    /// Any edit of a matter is either accepted or rejected with an
    /// envelope: never a crash, never an exit code outside the contract.
    #[test]
    fn every_mutation_yields_an_envelope(at in 0usize..400, byte in proptest::sample::select(vec![b'{', b'}', b'x', b' ', b'\n', b'1', b'-'])) {
        let mut text = OUT_OF_ORDER.as_bytes().to_vec();
        let at = at.min(text.len() - 1);
        text[at] = byte;
        let dir = workspace(&format!("mutate-{at}-{byte}"), &String::from_utf8_lossy(&text));
        let run = litigate_in(&dir, &["check", "cases/m.case"]);
        prop_assert!(run.code == 0 || run.code == 2, "exit {} {}", run.code, run.stderr);
        prop_assert_eq!(run.json["ok"].as_bool(), Some(run.code == 0));
        let _ = std::fs::remove_dir_all(dir);
    }
}
