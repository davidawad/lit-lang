//! Printed names round-trip through `litigate lint-filing` (lg-d2pi): a
//! package's moves may name events it imports from a dependency, and a
//! matter names them bare when that is unambiguous, else qualified. Every
//! move `lint-filing` prints is accepted when proposed back. A matter may
//! write one `uses` per package. The packages are synthetic: the SPEC §12
//! worked example (`lit/examples`), a local answer rule over it, and
//! another package declaring an `Answered` and an `ExtensionMoved` of its
//! own.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use serde_json::Value;
use std::path::{Path, PathBuf};
use support::{copy_dir, litigate_in, repo, scratch, Run};

const LOCAL_TOML: &str = r#"[package]
name = "local-rules"
version = "0.1.0"
language = 1
pack_id = "local-rules"
forum = "frcp"
authority = "FederalRule"
title = "Local answer rule (synthetic)"
in_force = "2009-12-01"
known_at = "2025-01-01"

[package.roles]
Plaintiff = "self"
Defendant = "opponent"
Court = "nature"

[dependencies]
frcp-rule12-example = { version = "^0.1.0", path = "../rule12" }
"#;

/// Answering and moving to extend are the worked example's events,
/// imported; the order extending the period is the package's own.
const LOCAL: &str = r#"module local.answer

import frcp.rule12.{ Answered, ExtensionMoved, Served, frcp6 }

start filed

event AnswerPeriodExtended(until: Date) extends local-answer-period to until

rule duty local-answer-duty
{
  deadline local-answer-period: 14.days from Served(complaint) counted_by frcp6
  {
    extend { by Court, discretion GoodCause, on ExtensionMoved() }
  }
}

state filed: "Complaint filed" by Plaintiff
{
  serve "Serve the complaint" -> served { event Served(complaint, Personal) }
}

decision served: "Answer or let it pass" by Defendant
{
  answer "File an answer" -> answered {
    event Answered(complaint)
    within local-answer-period
  }
  default "Let it pass" -> defaulted
}

terminal answered: "Answered" outcome [judgment] { payoff 0.usd }
terminal defaulted: "Default" outcome [judgment, loss] { payoff 0.usd }
"#;

/// Named `other`, lowered to pack `other-court`.
const OTHER_TOML: &str = r#"[package]
name = "other"
version = "0.1.0"
language = 1
pack_id = "other-court"
forum = "frcp"
authority = "FederalRule"
title = "Another answer (synthetic)"
in_force = "2009-12-01"
known_at = "2025-01-01"

[package.roles]
Plaintiff = "self"
Defendant = "opponent"
Court = "nature"
"#;

const OTHER: &str = r#"module other.answer

start open

enum Paper { brief }
event Answered(paper: Paper)
event ExtensionMoved()

state open: "Open" by Court
{
  close "Close" -> closed { event Answered(brief) }
}

terminal closed: "Closed" outcome [judgment] { payoff 0.usd }
"#;

/// A matter served 2025-03-05 (the answer is due 2025-03-19) writing
/// `uses` (one line per package), its service event as `served`.
fn matter(uses: &str, served: &str) -> String {
    format!(
        "case printed\n{uses}as_of 2025-03-10\nsummary \"A served complaint under a local answer rule.\"\n\n\
         parties {{\n  acme: Plaintiff Corporation\n  widget: Defendant Corporation\n}}\n\n\
         events {{\n  served-1: 2025-03-05 {served} on widget\n}}\n"
    )
}

/// The fixture: `cases/alone.case` uses the local rule alone, so its
/// imports are unambiguous; `cases/both.case` also uses `other`, on a
/// second `uses` line, so `Answered` and `ExtensionMoved` are ambiguous.
fn fixture(name: &str) -> PathBuf {
    let dir = scratch(name);
    let lit = dir.join("lit");
    copy_dir(&repo().join("lit/examples"), &lit.join("rule12"));
    let rule12 = lit.join("rule12/rule12.lit");
    let text = std::fs::read_to_string(&rule12).unwrap();
    std::fs::write(&rule12, format!("{text}\nevent ExtensionMoved()\n")).unwrap();
    for (pkg, toml, file, text) in [
        ("local", LOCAL_TOML, "answer.lit", LOCAL),
        ("other", OTHER_TOML, "other.lit", OTHER),
    ] {
        std::fs::create_dir_all(lit.join(pkg)).unwrap();
        std::fs::write(lit.join(pkg).join("Lit.toml"), toml).unwrap();
        std::fs::write(lit.join(pkg).join(file), text).unwrap();
    }
    std::fs::create_dir_all(dir.join("cases")).unwrap();
    let served = "Served(complaint, Personal)";
    for (case, uses) in [
        ("alone", "uses local-rules ^0.1.0\n"),
        ("both", "uses local-rules ^0.1.0\nuses other ^0.1.0\n"),
    ] {
        std::fs::write(dir.join(format!("cases/{case}.case")), matter(uses, served)).unwrap();
    }
    for target in ["lit/rule12", "lit/local", "lit/other", "cases"] {
        let lock = litigate_in(&dir, &["lock", target]);
        assert_eq!(lock.code, 0, "{target}: {}", lock.stdout);
    }
    dir
}

/// `lint-filing` of `event` in `case` by the defendant on 2025-03-12.
fn lint(dir: &Path, case: &str, event: &str) -> Run {
    litigate_in(
        dir,
        &[
            "lint-filing",
            &format!("cases/{case}.case"),
            event,
            "--date",
            "2025-03-12",
            "--by",
            "widget",
        ],
    )
}

/// Name and typing errors (L1002, L2001) of a run, as `code message`.
fn unresolved(run: &Run) -> Vec<String> {
    errors(run)
        .into_iter()
        .filter(|e| e.starts_with("L1002") || e.starts_with("L2001"))
        .collect()
}

/// Every error of a run, as `code message`.
fn errors(run: &Run) -> Vec<String> {
    run.json["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| d["severity"] == "error")
        .map(|d| {
            format!(
                "{} {}",
                d["code"].as_str().unwrap(),
                d["message"].as_str().unwrap()
            )
        })
        .collect()
}

/// The moves listed for the open answer period: each act, request and
/// order, as printed.
fn moves(run: &Run) -> (Vec<String>, Vec<String>, Vec<String>) {
    let moves = &run.json["result"]["open_before"][0]["moves"];
    let strings = |v: &Value, key: Option<&str>| -> Vec<String> {
        v.as_array()
            .unwrap()
            .iter()
            .map(|x| key.map_or(x, |k| &x[k]).as_str().unwrap().to_owned())
            .collect()
    };
    (
        strings(&moves["file"], Some("event")),
        strings(&moves["extend"], Some("request")),
        strings(&moves["orders"], None),
    )
}

#[test]
fn an_unambiguous_imported_event_is_named_and_printed_bare() {
    let dir = fixture("names-alone");
    let run = lint(&dir, "alone", "Answered(complaint)");
    assert_eq!(run.json["result"]["accepted"], true, "{}", run.stdout);
    assert_eq!(
        moves(&run),
        (
            vec!["Answered(complaint)".to_owned()],
            vec!["ExtensionMoved()".to_owned()],
            vec!["AnswerPeriodExtended".to_owned()],
        )
    );
}

#[test]
fn an_ambiguous_imported_event_is_printed_qualified() {
    let dir = fixture("names-both");
    let bare = lint(&dir, "both", "Answered(complaint)");
    assert_eq!(
        errors(&bare),
        ["L1002 `Answered` is an event of 2 packages: write one of `other-court::Answered`, `frcp-rule12-example::Answered`"]
    );
    let run = lint(&dir, "both", "frcp-rule12-example::Answered(complaint)");
    assert_eq!(run.json["result"]["accepted"], true, "{}", run.stdout);
    assert_eq!(
        moves(&run),
        (
            vec!["frcp-rule12-example::Answered(complaint)".to_owned()],
            vec!["frcp-rule12-example::ExtensionMoved()".to_owned()],
            vec!["AnswerPeriodExtended".to_owned()],
        )
    );
}

#[test]
fn every_printed_move_is_accepted_back() {
    let dir = fixture("names-round-trip");
    for (case, answer) in [
        ("alone", "Answered(complaint)"),
        ("both", "frcp-rule12-example::Answered(complaint)"),
    ] {
        let (file, requests, orders) = moves(&lint(&dir, case, answer));
        let orders = orders.iter().map(|o| format!("{o}(2025-03-31)"));
        for event in file.into_iter().chain(requests).chain(orders) {
            let run = lint(&dir, case, &event);
            assert_eq!(unresolved(&run), Vec::<String>::new(), "{case}: {event}");
            assert!(
                run.json["result"]["accepted"].is_boolean(),
                "{case}: {event}"
            );
        }
    }
}

#[test]
fn an_event_may_be_named_by_package_or_module() {
    let dir = fixture("names-forms");
    for event in [
        "frcp-rule12-example::Answered(complaint)",
        "frcp.rule12.Answered(complaint)",
        "other::Answered(brief)",
        "other-court::Answered(brief)",
        "other.answer.Answered(brief)",
    ] {
        let run = lint(&dir, "both", event);
        assert_eq!(unresolved(&run), Vec::<String>::new(), "{event}");
    }
    for (event, error) in [
        (
            "nowhere::Answered(brief)",
            "L1002 `local-rules` declares no event `nowhere::Answered`",
        ),
        (
            "other::Served(complaint)",
            "L1002 `other-court` declares no event `Served`",
        ),
        (
            "nowhere.answer.Answered(brief)",
            "L1002 `local-rules` declares no event `nowhere.answer.Answered`",
        ),
    ] {
        let run = lint(&dir, "both", event);
        assert_eq!(errors(&run), [error], "{event}");
    }
}
