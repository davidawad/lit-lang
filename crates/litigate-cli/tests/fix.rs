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
    copy_dir(&repo().join("sources"), &dir.join("sources"));
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
    // The two generated ids (L9006) rewrite dates inside the lines the
    // sort moves, so the sort waits for the next run.
    let dry = litigate_in(&dir, &["fix", "cases/m.case", "--dry-run"]);
    assert_eq!(dry.json["result"]["fixes"].as_array().unwrap().len(), 2);
    assert_eq!(dry.json["result"]["skipped_overlapping"], 1);
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
fn actor_designations_are_typed_and_case_fixes_are_applied() {
    let dir = workspace("actor-designation", OUT_OF_ORDER);
    let module = dir.join("lit/examples/rule12.lit");
    let source = std::fs::read_to_string(&module).unwrap();
    std::fs::write(&module, source.replacen("by court", "by Court", 1)).unwrap();
    let lock = litigate_in(&dir, &["lock", "lit/examples"]);
    assert_eq!(lock.code, 0, "{}", lock.stdout);
    let before = litigate_in(&dir, &["check", "lit/examples"]);
    assert_eq!(before.code, 2, "{}", before.stdout);
    assert!(codes(&before.json)
        .iter()
        .any(|(code, severity)| { code == "L1016" && severity == "error" }));
    let fixed = litigate_in(&dir, &["fix", "lit/examples", "--code", "L1016"]);
    assert_eq!(fixed.code, 0, "{}", fixed.stdout);
    assert!(std::fs::read_to_string(&module)
        .unwrap()
        .contains("by court"));
    let lock = litigate_in(&dir, &["lock", "lit/examples"]);
    assert_eq!(lock.code, 0, "{}", lock.stdout);
    let after = litigate_in(&dir, &["check", "lit/examples"]);
    assert_eq!(after.code, 0, "{}", after.stdout);

    let dir = workspace("undeclared-actor", OUT_OF_ORDER);
    let module = dir.join("lit/examples/rule12.lit");
    let source = std::fs::read_to_string(&module).unwrap();
    std::fs::write(&module, source.replacen("by court", "by accused", 1)).unwrap();
    let lock = litigate_in(&dir, &["lock", "lit/examples"]);
    assert_eq!(lock.code, 0, "{}", lock.stdout);
    let invalid = litigate_in(&dir, &["check", "lit/examples"]);
    assert_eq!(invalid.code, 2, "{}", invalid.stdout);
    assert!(codes(&invalid.json)
        .iter()
        .any(|(code, severity)| { code == "L1016" && severity == "error" }));
}

#[test]
fn l9011_fix_runs_through_cli_and_is_idempotent() {
    let dir = workspace("l9011", OUT_OF_ORDER);
    let path = dir.join("lit/akd/local-rules.lit");
    let formatted = litigate_in(&dir, &["fmt", "lit/akd/local-rules.lit"]);
    assert_eq!(formatted.code, 0, "{}", formatted.stdout);
    let test = "test \"Counting boundary: signature-objection-period (weekend)\" cite \"D. Alaska L.Civ.R. 5.1(e)\"\n{\n  given DocumentServed() on 2025-10-18\n  expect deadline(signature-objection-period) == 2025-11-03\n}\n";
    let source = std::fs::read_to_string(&path).unwrap();
    assert!(source.contains(test));
    std::fs::write(&path, source.replacen(test, "", 1)).unwrap();
    let formatted = litigate_in(&dir, &["fmt", "lit/akd/local-rules.lit"]);
    assert_eq!(formatted.code, 0, "{}", formatted.stdout);
    for package in ["lit/core", "lit/frcp", "lit/akd"] {
        let lock = litigate_in(&dir, &["lock", package]);
        assert_eq!(lock.code, 0, "{}", lock.stdout);
    }
    let canonical = litigate_in(&dir, &["fmt", "--check", "lit/akd/local-rules.lit"]);
    assert_eq!(canonical.code, 0, "{}", canonical.stdout);

    let fixed = litigate_in(&dir, &["fix", "lit/akd/local-rules.lit", "--code", "L9011"]);
    assert_eq!(fixed.code, 0, "{}", fixed.stdout);
    assert!(fixed.json["result"]["changed_files"]
        .as_array()
        .unwrap()
        .iter()
        .any(|file| file == "lit/akd/local-rules.lit"));
    let after = std::fs::read_to_string(&path).unwrap();
    assert!(after.contains("Counting boundary: signature-objection-period"));
    assert!(
        after.contains("cite \"D. Alaska L.Civ.R. 5.1(e)\"\n{"),
        "{after}"
    );
    let canonical = litigate_in(&dir, &["fmt", "--check", "lit/akd/local-rules.lit"]);
    assert_eq!(canonical.code, 0, "{}", canonical.stdout);
    for package in ["lit/core", "lit/frcp", "lit/akd"] {
        let lock = litigate_in(&dir, &["lock", package]);
        assert_eq!(lock.code, 0, "{}", lock.stdout);
    }
    let checked = litigate_in(&dir, &["check", "lit/akd/local-rules.lit"]);
    assert_eq!(checked.code, 0, "{}", checked.stdout);
    assert!(!checked.json["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .any(|d| {
            d["code"] == "L9011"
                && d["message"]
                    .as_str()
                    .is_some_and(|message| message.contains("signature-objection-period"))
        }));
    let tests = litigate_in(&dir, &["test", "lit/akd/local-rules.lit"]);
    assert_eq!(tests.code, 0, "{}", tests.stdout);

    let second = litigate_in(&dir, &["fix", "lit/akd/local-rules.lit", "--code", "L9011"]);
    assert_eq!(second.code, 0, "{}", second.stdout);
    assert_eq!(std::fs::read_to_string(path).unwrap(), after);
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
    let error = run.json["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["severity"] == "error")
        .unwrap();
    assert!(
        error["message"]
            .as_str()
            .unwrap()
            .contains("is taken by defendant, but `acme` plays Plaintiff"),
        "{}",
        run.stdout
    );
}

#[test]
fn generated_event_ids_stay_with_their_lines_through_a_sort() {
    let dir = workspace("ids", OUT_OF_ORDER);
    let ids = litigate_in(&dir, &["fix", "cases/m.case"]);
    assert_eq!(
        ids.json["result"]["skipped_overlapping"], 1,
        "{}",
        ids.stdout
    );
    let sorted = litigate_in(&dir, &["fix", "cases/m.case"]);
    assert_eq!(sorted.json["result"]["fixes"][0]["code"], "L4001");
    let text = std::fs::read_to_string(dir.join("cases/m.case")).unwrap();
    assert!(
        text.contains(
            "  served-1: 2025-03-05 Served(complaint, Personal) on widget\n  \
             moved-to-dismiss-1: 2025-03-20 MovedToDismiss(complaint) by widget\n"
        ),
        "{text}"
    );
    let after = litigate_in(&dir, &["check", "cases/m.case"]);
    assert_eq!(after.code, 0, "{}", after.stdout);
    assert!(
        !codes(&after.json).iter().any(|c| c.0 == "L9006"),
        "{}",
        after.stdout
    );
    let fmt = litigate_in(&dir, &["fmt", "--check", "cases/m.case"]);
    assert_eq!(fmt.code, 0, "{}", fmt.stdout);
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
