//! `litigate lint-filing` on a matter with concurrent tracks (SPEC §5.6,
//! design review §2.8; lg-16x1): after a dismissal the matter runs a
//! magistrate-consent decision, an opposition and a fee clock at once, so
//! a filing is judged on the track it belongs to, not against the one
//! posture `start` names. The package and its dates are synthetic.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support_cmds;

use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use support_cmds::{litigate_in, scratch, Run};

const MANIFEST: &str = r#"[package]
name = "nd-post-dismissal"
version = "0.1.0"
language = 1
pack_id = "nd-post-dismissal"
forum = "frcp"
authority = "FederalRule"
title = "Post-dismissal concurrent obligations (synthetic)"
description = "Magistrate consent, an opposition and the fee clock on concurrent tracks."
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

/// Entry of the dismissal leaves the matter at the consent decision and
/// spawns the opposition and fee tracks, each with its own 14-day clock.
const LIT: &str = r#"module nd.post

package nd-post-dismissal { forum frcp, authority FederalRule }

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
  enter "Enter dismissal" -> consent-decision by Court { event DismissalEntered(), spawn [opposition-due, fee-clock] }
}
decision consent-decision: "Consent to a magistrate judge or decline" by Plaintiff cite "FRCP 73(b)(1)"
{
  consent "Consent" -> consented { event ConsentFiled(), within consent-period }
  decline "Decline" -> declined { event DeclinationFiled(), within consent-period }
}
decision opposition-due: "Oppose the motion" by Plaintiff cite "FRCP 7(b)"
{
  oppose "File the opposition" -> opposed { event OppositionFiled(), within opposition-period }
}
decision fee-clock: "Move for fees" by Plaintiff cite "FRCP 54(d)(2)(B)"
{
  fee "File the fee motion" -> fee-filed { event FeeMotionFiled(), within fee-period }
}
terminal consented: "Consented" cite "FRCP 73(b)(1)" { payoff 0.usd }
terminal declined: "Declined" cite "FRCP 73(b)(1)" { payoff 0.usd }
terminal opposed: "Opposed" cite "FRCP 7(b)" { payoff 0.usd }
terminal fee-filed: "Fee motion filed" cite "FRCP 54(d)(2)(B)" { payoff 0.usd }
"#;

/// The dismissal is entered 2025-06-02; every clock is due 2025-06-16.
fn matter(name: &str, events: &str) -> PathBuf {
    let dir = scratch(name);
    std::fs::create_dir_all(dir.join("lit/nd")).unwrap();
    std::fs::create_dir_all(dir.join("cases")).unwrap();
    std::fs::write(dir.join("lit/nd/Lit.toml"), MANIFEST).unwrap();
    std::fs::write(dir.join("lit/nd/nd.lit"), LIT).unwrap();
    std::fs::write(
        dir.join("cases/nd.case"),
        format!(
            "case nd\nuses nd-post-dismissal ^0.1.0\nas_of 2025-06-03\nsummary \"s\"\n\n\
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

#[test]
fn each_concurrent_obligation_is_legal_on_its_own_track() {
    let dir = matter("concurrent-legal", "");
    for (event, from, to, deadline) in [
        (
            "OppositionFiled()",
            "opposition-due",
            "opposed",
            "opposition-period",
        ),
        ("FeeMotionFiled()", "fee-clock", "fee-filed", "fee-period"),
        (
            "ConsentFiled()",
            "consent-decision",
            "consented",
            "consent-period",
        ),
    ] {
        let run = lint(&dir, event, "2025-06-10", "movant");
        assert_eq!(run.code, 0, "{event}: {}", run.stdout);
        assert_eq!(errors(&run), Vec::<String>::new(), "{event}");
        let result = &run.json["result"];
        assert_eq!(result["accepted"], true, "{event}");
        let step = &result["replay"][0];
        assert_eq!(
            (&step["from"], &step["to"]),
            (
                &json!(format!("nd-post-dismissal::{from}")),
                &json!(format!("nd-post-dismissal::{to}"))
            ),
            "{event}"
        );
        assert_eq!(
            discharges(&run),
            [(
                format!("nd-post-dismissal::{deadline}"),
                "discharged".to_owned()
            )],
            "{event}"
        );
    }
}

#[test]
fn every_track_lists_the_filings_that_protect_its_clock() {
    let dir = matter("concurrent-moves", "");
    let run = lint(&dir, "OppositionFiled()", "2025-06-10", "movant");
    assert_eq!(run.code, 0, "{}", run.stdout);
    let file: Vec<(String, Value)> = run.json["result"]["open_before"]
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
    let edge =
        |id: &str, event: &str| json!({"edge": format!("nd-post-dismissal::{id}"), "event": event});
    assert_eq!(
        file,
        [
            (
                "nd-post-dismissal::consent-period".to_owned(),
                json!([
                    edge("consent", "ConsentFiled()"),
                    edge("decline", "DeclinationFiled()")
                ])
            ),
            (
                "nd-post-dismissal::fee-period".to_owned(),
                json!([edge("fee", "FeeMotionFiled()")])
            ),
            (
                "nd-post-dismissal::opposition-period".to_owned(),
                json!([edge("oppose", "OppositionFiled()")])
            ),
        ]
    );
}

#[test]
fn a_track_still_judges_its_own_filing() {
    let dir = matter("concurrent-judged", "");
    let late = lint(&dir, "OppositionFiled()", "2025-06-20", "movant");
    assert_eq!(late.code, 2, "{}", late.stdout);
    assert_eq!(errors(&late), ["L4002"]);
    assert_eq!(late.json["result"]["accepted"], false);
    let wrong_party = lint(&dir, "OppositionFiled()", "2025-06-10", "other");
    assert_eq!(wrong_party.code, 2, "{}", wrong_party.stdout);
    assert_eq!(errors(&wrong_party), ["L4001"]);
    let opposed = matter(
        "concurrent-opposed",
        "  o-1: 2025-06-03 OppositionFiled() by movant\n",
    );
    let again = lint(&opposed, "OppositionFiled()", "2025-06-10", "movant");
    assert_eq!(again.code, 2, "{}", again.stdout);
    assert_eq!(errors(&again), ["L4001"]);
}
