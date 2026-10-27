//! `anytime` edges (SPEC §5.6; lg-xqu8): a serial package declares the
//! opposition legal from any posture while its own period is open, so an
//! opposition filed while the magistrate-consent decision is pending passes
//! `lint-filing` and replay, without concurrent tracks, and the package
//! still lowers (no L6001). The package and its dates are synthetic.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support_cmds;

use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use support_cmds::{litigate_in, scratch, Run};

const MANIFEST: &str = r#"[package]
name = "nd-serial"
version = "0.1.0"
language = 1
pack_id = "nd-serial"
forum = "frcp"
authority = "FederalRule"
title = "Post-dismissal obligations in series (synthetic)"
description = "Magistrate consent, then an opposition, then the fee motion."
jurisdiction = "United States district courts"
in_force = "2009-12-01"
known_at = "2025-01-01"

[package.roles]
Plaintiff = "self"
Defendant = "opponent"
Court = "nature"

[sources.frcp]
title = "Federal Rules of Civil Procedure"
url = "https://www.law.cornell.edu/rules/frcp"
as_of = "2024-12-01"
cite_prefixes = ["FRCP"]
"#;

/// One track: the consent decision, then the opposition, then the fee
/// motion, each period 14 days from the dismissal. The opposition is
/// `anytime` (written `ANYTIME`, so a test can drop it).
const LIT: &str = r#"module nd.serial

package nd-serial { forum frcp, authority FederalRule }

start dismissed

event DismissalEntered()
event ConsentFiled()
event DeclinationFiled()
event OppositionFiled()
event FeeMotionFiled()

counting days cite "FRCP 6(a)"
{
  exclude_trigger_day
  include_last_day
  holidays federal-holidays
  anchor trigger_day
}

calendar federal-holidays cite "FRCP 6(a)(6)"
{
  new-year: fixed(1, 1) effective [1870-06-28, ..) observed nearest_weekday
}

deadline consent-period: 14.days after DismissalEntered() counted_by days cite "FRCP 73(b)(1)"
deadline opposition-period: 14.days after DismissalEntered() counted_by days cite "FRCP 7(b)"
deadline fee-period: 14.days after DismissalEntered() counted_by days cite "FRCP 54(d)(2)(B)"

state dismissed: "Dismissal entered" cite "FRCP 41(b)"
{
  enter "Enter dismissal" -> consent-decision by Court { event DismissalEntered() }
}
decision consent-decision: "Consent to a magistrate judge or decline" by Plaintiff cite "FRCP 73(b)(1)"
{
  consent "Consent" -> opposition-due { event ConsentFiled(), within consent-period }
  decline "Decline" -> opposition-due { event DeclinationFiled(), within consent-period }
}
decision opposition-due: "Oppose the motion" by Plaintiff cite "FRCP 7(b)"
{
  oppose "File the opposition" -> fee-clock { event OppositionFiled(), within opposition-periodANYTIME }
}
decision fee-clock: "Move for fees" by Plaintiff cite "FRCP 54(d)(2)(B)"
{
  fee "File the fee motion" -> fee-filed { event FeeMotionFiled(), within fee-period }
}
terminal fee-filed: "Fee motion filed" cite "FRCP 54(d)(2)(B)" { payoff 0.usd }
"#;

/// The matter: the dismissal entered 2025-06-02 (every period due
/// 2025-06-16), `events` after it; `anytime` on the opposition or not.
fn matter(name: &str, anytime: bool, events: &str) -> PathBuf {
    let dir = scratch(name);
    std::fs::create_dir_all(dir.join("lit/nd")).unwrap();
    std::fs::create_dir_all(dir.join("cases")).unwrap();
    std::fs::write(dir.join("lit/nd/Lit.toml"), MANIFEST).unwrap();
    let lit = LIT.replace("ANYTIME", if anytime { ", anytime" } else { "" });
    std::fs::write(dir.join("lit/nd/nd.lit"), lit).unwrap();
    std::fs::write(
        dir.join("cases/nd.case"),
        format!(
            "case nd\nuses nd-serial ^0.1.0\nas_of 2025-06-12\nsummary \"s\"\n\n\
             parties {{\n  movant: Plaintiff Corporation\n  other: Defendant Corporation\n  judge: Court Tribunal\n}}\n\n\
             events {{\n  d-1: 2025-06-02 DismissalEntered() by judge\n{events}}}\n"
        ),
    )
    .unwrap();
    let lock = litigate_in(&dir, &["lock", "cases"]);
    assert_eq!(lock.code, 0, "{}", lock.stdout);
    dir
}

fn lint(dir: &Path, event: &str, date: &str, party: &str) -> Run {
    litigate_in(
        dir,
        &[
            "lint-filing",
            "cases/nd.case",
            event,
            "--date",
            date,
            "--by",
            party,
        ],
    )
}

/// The codes of a run's errors.
fn errors(run: &Run) -> Vec<String> {
    run.json["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| d["severity"] == "error")
        .map(|d| d["code"].as_str().unwrap().to_owned())
        .collect()
}

/// `(deadline, status)` of each instance a run's filing discharges.
fn discharges(run: &Run) -> Vec<(String, String)> {
    run.json["result"]["discharges"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| {
            (
                d["deadline"].as_str().unwrap().to_owned(),
                d["status"].as_str().unwrap().to_owned(),
            )
        })
        .collect()
}

/// `(effect, from, to)` of each replay step of a `check` run's matter.
fn trace(run: &Run) -> Vec<(String, String, String)> {
    let case = &run.json["result"]["cases"][0];
    case["replay"]
        .as_array()
        .unwrap_or_else(|| panic!("{case}"))
        .iter()
        .map(|s| {
            let at = |k: &str| {
                s[k].as_str()
                    .unwrap()
                    .trim_start_matches("nd-serial::")
                    .to_owned()
            };
            (at("effect"), at("from"), at("to"))
        })
        .collect()
}

fn step(effect: &str, from: &str, to: &str) -> (String, String, String) {
    (effect.to_owned(), from.to_owned(), to.to_owned())
}

#[test]
fn an_opposition_during_the_consent_decision_passes_lint_filing() {
    let dir = matter("anytime-lint", true, "");
    let run = lint(&dir, "OppositionFiled()", "2025-06-10", "movant");
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert_eq!(errors(&run), Vec::<String>::new());
    let result = &run.json["result"];
    assert_eq!(result["accepted"], true);
    assert_eq!(
        result["replay"],
        json!([{
            "date": "2025-06-10", "event": "OppositionFiled()", "effect": "anytime",
            "from": "nd-serial::consent-decision", "to": "nd-serial::consent-decision",
            "edge": "nd-serial::oppose"
        }])
    );
    assert_eq!(
        discharges(&run),
        [(
            "nd-serial::opposition-period".to_owned(),
            "discharged".to_owned()
        )]
    );
    // The opposition is a move protecting its period while the consent
    // decision is pending; the fee motion, not `anytime`, is not.
    let file: Vec<(String, Value)> = result["open_before"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| {
            (
                o["deadline"].as_str().unwrap().to_owned(),
                o["moves"]["file"].clone(),
            )
        })
        .collect();
    let edge = |id: &str, event: &str| json!({"edge": format!("nd-serial::{id}"), "event": event});
    assert_eq!(
        file,
        [
            (
                "nd-serial::consent-period".to_owned(),
                json!([
                    edge("consent", "ConsentFiled()"),
                    edge("decline", "DeclinationFiled()")
                ])
            ),
            ("nd-serial::fee-period".to_owned(), json!([])),
            (
                "nd-serial::opposition-period".to_owned(),
                json!([edge("oppose", "OppositionFiled()")])
            ),
        ]
    );
}

#[test]
fn without_anytime_the_opposition_is_not_a_legal_move() {
    let dir = matter("anytime-without", false, "");
    let run = lint(&dir, "OppositionFiled()", "2025-06-10", "movant");
    assert_eq!(run.code, 2, "{}", run.stdout);
    assert_eq!(errors(&run), ["L4001"]);
    assert_eq!(run.json["result"]["accepted"], false);
}

#[test]
fn replay_takes_the_early_opposition_when_the_track_arrives_and_solve_lowers() {
    let events = "  o-1: 2025-06-05 OppositionFiled() by movant\n  c-1: 2025-06-09 ConsentFiled() by movant\n";
    let dir = matter("anytime-replay", true, events);
    let check = litigate_in(&dir, &["check", "cases/nd.case"]);
    assert_eq!(check.code, 0, "{}", check.stdout);
    assert_eq!(
        trace(&check),
        [
            step("took", "dismissed", "consent-decision"),
            step("anytime", "consent-decision", "consent-decision"),
            step("took", "consent-decision", "opposition-due"),
            step("took", "opposition-due", "fee-clock"),
        ]
    );
    assert_eq!(
        check.json["result"]["cases"][0]["start"],
        json!("nd-serial::fee-clock"),
        "{}",
        check.json["result"]["cases"][0]
    );
    // The serial package lowers: solve runs from the replayed posture.
    let solve = litigate_in(&dir, &["solve", "cases/nd.case"]);
    assert_eq!(solve.code, 0, "{}", solve.stdout);
    let without = matter("anytime-replay-without", false, events);
    let rejected = litigate_in(&without, &["check", "cases/nd.case"]);
    assert_eq!(rejected.code, 2, "{}", rejected.stdout);
    assert!(rejected.stdout.contains("L4001"), "{}", rejected.stdout);
}

#[test]
fn an_anytime_filing_is_still_judged_on_its_own_deadline() {
    let dir = matter("anytime-judged", true, "");
    let late = lint(&dir, "OppositionFiled()", "2025-06-20", "movant");
    assert_eq!(late.code, 2, "{}", late.stdout);
    assert_eq!(errors(&late), ["L4002"]);
    let wrong_party = lint(&dir, "OppositionFiled()", "2025-06-10", "other");
    assert_eq!(wrong_party.code, 2, "{}", wrong_party.stdout);
    assert_eq!(errors(&wrong_party), ["L4001"]);
    assert!(
        wrong_party.stdout.contains("is `anytime`"),
        "{}",
        wrong_party.stdout
    );
    let opposed = matter(
        "anytime-opposed",
        true,
        "  o-1: 2025-06-03 OppositionFiled() by movant\n",
    );
    let again = lint(&opposed, "OppositionFiled()", "2025-06-10", "movant");
    assert_eq!(again.code, 2, "{}", again.stdout);
    assert_eq!(errors(&again), ["L4001"]);
}
