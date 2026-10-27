//! `litigate actions` (DESIGN §10, agent action sets): a matter's replayed
//! marking projected onto a party. A party sees only its legal moves for
//! the posture; an illegal move (another party's, a later stage's, a
//! late one) is absent from the set, not offered and refused when filed.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support_cmds;

use litigate_cli::mcp::Server;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use support_cmds::{copy_dir, errors, fixture, litigate_in, scratch};

/// Widget has been served; it must answer or move to dismiss by
/// 2025-03-19 (the fixture's `answer-period`).
const MATTER: &str = r#"case act-matter
uses cov-example ^0.1.0
as_of 2025-03-10
summary "Action-set fixture: Widget is served."

perspective widget

parties {
  acme: Plaintiff Corporation
  widget: Defendant Corporation
}

events {
  served: 2025-03-05 Served(complaint, Personal) on widget
}

facts { time-barred: not-barred }
"#;

/// The fixture copied to a scratch directory with [`MATTER`] as
/// `cases/act.case`, locked.
fn matter(name: &str) -> PathBuf {
    let dir = scratch(name);
    copy_dir(&fixture(), &dir);
    std::fs::write(dir.join("cases/act.case"), MATTER).unwrap();
    assert_eq!(litigate_in(&dir, &["lock", "cases"]).code, 0);
    dir
}

fn actions(dir: &Path, args: &[&str]) -> support_cmds::Run {
    let mut all = vec!["actions", "cases/act.case"];
    all.extend(args);
    litigate_in(dir, &all)
}

/// The events of an action set.
fn events(json: &Value) -> Vec<String> {
    json["result"]["actions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["event"].as_str().unwrap().to_owned())
        .collect()
}

/// The error codes `lint-filing` reports for `event` by `party` on `date`.
fn lint(dir: &Path, event: &str, date: &str, party: &str) -> Vec<String> {
    let run = litigate_in(
        dir,
        &[
            "lint-filing",
            "cases/act.case",
            event,
            "--date",
            date,
            "--by",
            party,
        ],
    );
    errors(&run.json)
        .into_iter()
        .map(|(code, _)| code)
        .collect()
}

#[test]
fn a_party_sees_only_its_legal_moves() {
    let dir = matter("actions-legal");
    let run = actions(&dir, &["--party", "widget"]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    let r = &run.json["result"];
    assert_eq!(r["case"], json!("act-matter"));
    assert_eq!(r["actor"], json!("Defendant"));
    assert_eq!(r["as_of"], json!("2025-03-10"));
    assert_eq!(r["marking"], json!(["cov-pack::respond"]));
    // `default` ("Let the deadline pass") takes no event: no filing makes it.
    assert_eq!(
        events(&run.json),
        ["Answered(complaint)", "MovedToDismiss(complaint)"]
    );
    let answer = &r["actions"][0];
    assert_eq!(
        answer["line"],
        json!("2025-03-10 Answered(complaint) by widget")
    );
    assert_eq!(answer["edge"]["id"], json!("cov-pack::answer"));
    assert_eq!(answer["edge"]["to"], json!("cov-pack::answered"));
    assert_eq!(
        answer["discharges"][0]["deadline"],
        json!("cov-pack::answer-period")
    );
    // Each listed move is one replay accepts.
    for event in events(&run.json) {
        assert!(
            lint(&dir, &event, "2025-03-10", "widget").is_empty(),
            "{event}"
        );
    }
}

#[test]
fn another_partys_move_is_absent_not_refused_late() {
    let dir = matter("actions-other-party");
    let run = actions(&dir, &["--party", "acme"]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert_eq!(run.json["result"]["actor"], json!("Plaintiff"));
    assert!(events(&run.json).is_empty(), "{}", run.stdout);
    // Filed anyway, replay refuses it: the set left it out.
    assert_eq!(
        lint(&dir, "Answered(complaint)", "2025-03-10", "acme"),
        ["L4001"]
    );
}

#[test]
fn a_late_move_is_absent() {
    let dir = matter("actions-late");
    let run = actions(&dir, &["--party", "widget", "--date", "2025-04-15"]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert_eq!(run.json["result"]["as_of"], json!("2025-04-15"));
    assert!(events(&run.json).is_empty(), "{}", run.stdout);
    assert_eq!(
        lint(&dir, "Answered(complaint)", "2025-04-15", "widget"),
        ["L4002"]
    );
    // On the last day both moves are still legal.
    let run = actions(&dir, &["--party", "widget", "--date", "2025-03-19"]);
    assert_eq!(events(&run.json).len(), 2, "{}", run.stdout);
}

#[test]
fn a_move_of_a_passed_stage_is_absent() {
    // The coverage matter has moved to dismiss: the court rules next.
    let run = litigate_in(
        &fixture(),
        &["actions", "cases/cov.case", "--party", "widget"],
    );
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert_eq!(
        run.json["result"]["marking"],
        json!(["cov-pack::mtd-ruling"])
    );
    assert!(events(&run.json).is_empty(), "{}", run.stdout);
}

#[test]
fn an_undeclared_party_a_bad_date_or_a_broken_matter_fails() {
    let dir = matter("actions-failures");
    let run = actions(&dir, &["--party", "nobody"]);
    assert_eq!(run.code, 1);
    assert!(
        run.stderr
            .contains("`nobody` is not a party of `act-matter`; parties: acme, widget"),
        "{}",
        run.stderr
    );
    let run = actions(&dir, &["--party", "widget", "--date", "2025-13-01"]);
    assert_eq!(run.code, 1);
    assert!(run.stderr.contains("--date 2025-13-01"), "{}", run.stderr);
    let run = litigate_in(&dir, &["actions", "lit/cov/cov.lit", "--party", "widget"]);
    assert_eq!(run.code, 1);
    assert!(run.stderr.contains("not a .case file"), "{}", run.stderr);
    std::fs::write(
        dir.join("cases/act.case"),
        MATTER.replace("Served(complaint, Personal)", "Served(complaint, Pigeon)"),
    )
    .unwrap();
    let run = actions(&dir, &["--party", "widget"]);
    assert_ne!(run.code, 0, "{}", run.stdout);
    assert!(!errors(&run.json).is_empty(), "{}", run.stdout);
    assert!(run.json["result"]["actions"].is_null(), "{}", run.stdout);
}

#[test]
fn the_action_set_is_an_mcp_tool() {
    let dir = matter("actions-mcp");
    let case = dir.join("cases/act.case").display().to_string();
    let call = |party: &str| {
        let message = json!({
            "jsonrpc": "2.0", "id": 1, "method": "tools/call",
            "params": {"name": "litigate_actions", "arguments": {"case": case, "party": party}},
        });
        Server::default()
            .handle(&message.to_string())
            .unwrap_or_default()
    };
    let widget = call("widget");
    assert_eq!(widget["result"]["isError"], json!(false), "{widget}");
    let envelope = &widget["result"]["structuredContent"];
    assert_eq!(envelope["op"], json!("actions"));
    assert_eq!(
        events(envelope),
        ["Answered(complaint)", "MovedToDismiss(complaint)"]
    );
    let acme = call("acme");
    assert!(events(&acme["result"]["structuredContent"]).is_empty());
    let nobody = call("nobody");
    assert_eq!(nobody["result"]["isError"], json!(true), "{nobody}");
}
