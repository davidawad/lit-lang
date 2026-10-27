//! `litigate calibrate-replay`: calibration observations derived from a
//! synthetic public-record history, a handful of matters each citing the
//! docket its events were read from. Chance odds and durations come only
//! from docketed edge-taking steps, every estimate keeps its provenance
//! and selection criteria, unsourced and censored histories do not count,
//! and an unreviewed set is never written.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support_cmds;

use litigate_engine::api::CalibrationSet;
use serde_json::Value;
use std::path::{Path, PathBuf};
use support_cmds::{litigate_in, scratch};

const MANIFEST: &str = r#"[package]
name = "recap-example"
version = "0.1.0"
language = 1
pack_id = "recap-pack"
forum = "frcp"
authority = "FederalRule"
title = "FRCP Rule 12 motion to dismiss (replay calibration fixture)"
description = "Answer or move to dismiss, and the court's ruling on the motion."
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

/// FRCP 12: answer or move to dismiss, then the court's ruling.
const PACKAGE: &str = r#"module frcp.recap

package recap-example { forum frcp, authority FederalRule }

start respond

enum Document { complaint }

event Answered(document: Document)
event MovedToDismiss(document: Document)
event DismissalGranted()
event DismissalDenied()

decision respond: "Answer or move under Rule 12" by Defendant cite "FRCP 12(a)(1)(A)(i)"
{
  answer "File an answer" -> answered { event Answered(complaint) }
  move-dismiss "Move to dismiss under Rule 12(b)(6)" -> mtd-ruling { event MovedToDismiss(complaint) }
}

chance mtd-ruling: "Ruling on the motion to dismiss" by Court cite "FRCP 12(b)(6)"
{
  granted "Motion granted" -> dismissed p 1/2 { event DismissalGranted() }
  denied "Motion denied" -> answered p 1/2 { event DismissalDenied() }
}

terminal answered: "Answer filed" outcome [loss] { payoff -100.usd }
terminal dismissed: "Action dismissed" outcome [win] { payoff 0.usd }
"#;

/// A history `id` citing docket `n` (none when `n` is 0), with `events`.
fn history(id: &str, n: u32, events: &str) -> String {
    let source = if n == 0 {
        "source \"FRCP 12(b)(6)\" { url \"https://www.law.cornell.edu/rules/frcp/rule_12\" }"
            .to_owned()
    } else {
        format!(
            "source \"Docket, Acme v. Widget {n}, No. 1:25-cv-0000{n} (synthetic)\" {{\n  \
             url \"https://www.courtlistener.com/docket/{n}/acme-v-widget/\"\n  as_of 2025-06-0{n}\n}}"
        )
    };
    format!(
        "case {id}\nuses recap-example ^0.1.0\nas_of 2025-06-01\nsummary \"Synthetic docket history.\"\n\n\
         {source}\n\nperspective acme\n\nparties {{\n  acme: Plaintiff Corporation\n  \
         widget: Defendant Corporation\n}}\n\nevents {{\n{events}}}\n"
    )
}

/// The synthetic public-record history: three rulings on a motion to
/// dismiss (granted after 60 and 90 days, denied after 30), one motion
/// still pending, one ruling with no docket number, one matter citing no
/// docket, and one answer (a party's choice, not odds).
fn histories(name: &str) -> PathBuf {
    let dir = scratch(name);
    std::fs::create_dir_all(dir.join("lit/recap")).unwrap();
    std::fs::create_dir_all(dir.join("cases")).unwrap();
    std::fs::write(dir.join("lit/recap/Lit.toml"), MANIFEST).unwrap();
    std::fs::write(dir.join("lit/recap/recap.lit"), PACKAGE).unwrap();
    let mtd =
        |date: &str, n: u32| format!("  {date} docket {n} MovedToDismiss(complaint) by widget\n");
    let cases = [
        (
            "h1",
            1,
            mtd("2025-01-10", 5) + "  2025-03-11 docket 9 DismissalGranted() on widget\n",
        ),
        (
            "h2",
            2,
            mtd("2025-02-01", 4) + "  2025-03-03 docket 8 DismissalDenied() on widget\n",
        ),
        (
            "h3",
            3,
            mtd("2025-02-01", 3) + "  2025-05-02 docket 7 DismissalGranted() on widget\n",
        ),
        ("h4", 4, mtd("2025-04-01", 6)),
        (
            "h5",
            5,
            mtd("2025-01-02", 2) + "  2025-02-01 DismissalDenied() on widget\n",
        ),
        (
            "h6",
            0,
            mtd("2025-01-02", 2) + "  2025-02-01 docket 3 DismissalDenied() on widget\n",
        ),
        (
            "h7",
            7,
            "  2025-01-20 docket 4 Answered(complaint) by widget\n".to_owned(),
        ),
    ];
    for (id, n, events) in &cases {
        std::fs::write(
            dir.join(format!("cases/{id}.case")),
            history(id, *n, events),
        )
        .unwrap();
    }
    assert_eq!(litigate_in(&dir, &["lock", "cases"]).code, 0);
    dir
}

/// `calibrate-replay cases --id replay-test` with `extra`.
fn run(dir: &Path, extra: &[&str]) -> support_cmds::Run {
    let mut args = vec!["calibrate-replay", "cases", "--id", "replay-test"];
    args.extend(extra);
    litigate_in(dir, &args)
}

/// The set's entry for `(target, edge)`.
fn entry<'v>(set: &'v Value, target: &str, edge: &str) -> &'v Value {
    set["entries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["target"] == target && e["ref"] == edge)
        .unwrap_or_else(|| panic!("no {target} entry for {edge}: {set}"))
}

#[test]
fn chance_odds_and_durations_come_from_docketed_steps_with_their_provenance() {
    let dir = histories("calibrate-replay");
    let r = run(&dir, &[]);
    assert_eq!(r.code, 0, "{}", r.stdout);
    let result = &r.json["result"];
    let set = &result["set"];
    assert_eq!(set["id"], "replay-test");
    assert_eq!(result["written"], Value::Null);
    // Granted twice and denied once among the docketed rulings; h4 is
    // still pending (censored), h5's ruling has no docket number, h6
    // cites no docket.
    let granted = entry(set, "edge-probability", "recap-pack::granted");
    assert_eq!(granted["value"], 0.6667);
    assert_eq!(granted["n"], 2);
    let denied = entry(set, "edge-probability", "recap-pack::denied");
    assert_eq!(denied["value"], 0.3333);
    assert_eq!(denied["n"], 1);
    // Provenance: the docket of every observation, its entries and dates,
    // the arithmetic, the selection criteria, and the review status.
    let method = granted["method"].as_str().unwrap();
    for part in [
        "UNREVIEWED docket matches",
        "2 of 3 docketed exits from `recap-pack::mtd-ruling` took `recap-pack::granted`: 2 / 3 = 0.6667",
        "1 history(ies) censored at the stage",
        "Selection: a share of the histories that left a chance stage by a docketed event",
        "h1 docket 5 (2025-01-10) to docket 9 (2025-03-11) DismissalGranted() <https://www.courtlistener.com/docket/1/acme-v-widget/>",
        "h3 docket 3 (2025-02-01) to docket 7 (2025-05-02) DismissalGranted() <https://www.courtlistener.com/docket/3/acme-v-widget/>",
    ] {
        assert!(method.contains(part), "{part:?} not in {method}");
    }
    assert!(!method.contains("h5") && !method.contains("h6"), "{method}");
    assert_eq!(
        granted["source"]["title"],
        "Replayed docket histories: Docket, Acme v. Widget 1, No. 1:25-cv-00001 (synthetic); \
         Docket, Acme v. Widget 3, No. 1:25-cv-00003 (synthetic)"
    );
    assert_eq!(
        granted["source"]["url"],
        "https://www.courtlistener.com/docket/1/acme-v-widget/"
    );
    assert_eq!(granted["source"]["vintageStart"], "2025-01-10");
    assert_eq!(granted["source"]["vintageEnd"], "2025-05-02");
    assert_eq!(granted["source"]["retrieved"], "2025-06-03");
    assert_eq!(
        denied["source"]["title"],
        "Docket, Acme v. Widget 2, No. 1:25-cv-00002 (synthetic)"
    );
    // The set is in the existing calibration format: a `calibration`
    // declaration in the package it calibrates, lowering to `set`.
    CalibrationSet::from_json(&set.to_string()).unwrap();
    let declaration = &result["declaration"];
    assert_eq!(declaration["path"], "lit/recap/calibration-replay-test.lit");
    let text = declaration["text"].as_str().unwrap();
    for part in [
        "module recap.calibration-replay-test",
        "calibration replay-test: \"Replay calibration: h1, h2, h3, h4, h5, h7\"",
        "p granted 0.6667",
        "duration granted { min 60.days, mode 75.days, max 90.days }",
        "url \"https://www.courtlistener.com/docket/1/acme-v-widget/\"",
        "vintage [2025-01-10, 2025-05-03)",
        "retrieved 2025-06-03",
    ] {
        assert!(text.contains(part), "{part:?} not in {text}");
    }
}

#[test]
fn durations_run_from_the_docketed_entry_and_choices_are_not_odds() {
    let dir = histories("calibrate-replay-durations");
    let r = run(&dir, &[]);
    assert_eq!(r.code, 0, "{}", r.stdout);
    let result = &r.json["result"];
    let set = &result["set"];
    // Durations: from the docketed motion to the docketed ruling.
    let took = entry(set, "edge-duration", "recap-pack::granted");
    assert_eq!(
        took["distribution"],
        serde_json::json!({"min": 60.0, "mode": 75.0, "max": 90.0})
    );
    assert!(took["method"]
        .as_str()
        .unwrap()
        .contains("days 60, 90: min 60, median 75, max 90"));
    assert_eq!(
        entry(set, "edge-duration", "recap-pack::denied")["distribution"]["mode"],
        30.0
    );
    // A party's own choice is not odds, and an edge out of the start
    // (entered by no docketed event) has no duration.
    let withheld = result["withheld"].as_array().unwrap();
    assert!(withheld
        .iter()
        .any(|w| w["ref"] == "recap-pack::move-dismiss"
            && w["reason"].as_str().unwrap().contains("not a chance stage")));
    assert!(set["entries"]
        .as_array()
        .unwrap()
        .iter()
        .all(|e| e["ref"] != "recap-pack::move-dismiss" && e["ref"] != "recap-pack::answer"));
    let h6 = result["histories"]
        .as_array()
        .unwrap()
        .iter()
        .find(|h| h["case"] == "h6")
        .unwrap();
    assert!(h6["excluded"]
        .as_str()
        .unwrap()
        .contains("not a sourced docket history"));
    assert_eq!(h6["observations"], 0);
    // Each observation keeps its docket entry and source.
    let pending = result["observations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|o| o["case"] == "h4")
        .collect::<Vec<_>>();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0]["docket"], 6);
    assert_eq!(
        pending[0]["docket_url"],
        "https://www.courtlistener.com/docket/4/acme-v-widget/"
    );
}

#[test]
fn a_set_is_written_only_once_reviewed_and_withholds_thin_estimates() {
    let dir = histories("calibrate-replay-write");
    let out = "lit/recap/calibration-replay-test.lit";
    let unreviewed = run(&dir, &["--out", out]);
    assert_eq!(unreviewed.code, 1);
    assert!(
        unreviewed.stderr.contains("only with --reviewed"),
        "{}",
        unreviewed.stderr
    );
    assert!(!dir.join(out).exists());
    let r = run(
        &dir,
        &["--reviewed", "--min-n", "2", "--out", out, "--title", "T"],
    );
    assert_eq!(r.code, 0, "{}", r.stdout);
    assert_eq!(r.json["result"]["written"], out);
    // The declaration goes into the package it calibrates, which then
    // checks and builds the set.
    let text = std::fs::read_to_string(dir.join(out)).unwrap();
    assert_eq!(r.json["result"]["declaration"]["text"], text.as_str());
    assert!(text.contains("calibration replay-test: \"T\""), "{text}");
    assert!(text.contains("(docket matches reviewed)"), "{text}");
    assert!(!text.contains("p denied"), "{text}");
    assert_eq!(litigate_in(&dir, &["lock", "lit/recap"]).code, 0);
    let built = litigate_in(&dir, &["build", "lit/recap", "--out", "built"]);
    assert_eq!(built.code, 0, "{}", built.stdout);
    let set = CalibrationSet::from_json(
        &std::fs::read_to_string(dir.join("built/packs/calibration/replay-test.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(set.title, "T");
    let refs: Vec<(String, String)> = set
        .entries
        .iter()
        .map(|e| {
            (
                serde_json::to_value(e.target)
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_owned(),
                e.target_ref.clone(),
            )
        })
        .collect();
    assert_eq!(
        refs,
        [
            (
                "edge-probability".to_owned(),
                "recap-pack::granted".to_owned()
            ),
            ("edge-duration".to_owned(), "recap-pack::granted".to_owned()),
        ]
    );
    assert!(r.json["result"]["withheld"]
        .as_array()
        .unwrap()
        .iter()
        .any(|w| w["ref"] == "recap-pack::denied"
            && w["reason"] == "1 observation(s), fewer than --min-n 2"));
}

#[test]
fn a_history_with_errors_calibrates_nothing() {
    let dir = histories("calibrate-replay-errors");
    // A ruling with no motion pending is not a legal move.
    std::fs::write(
        dir.join("cases/h8.case"),
        history(
            "h8",
            8,
            "  2025-01-02 docket 2 DismissalGranted() on widget\n",
        ),
    )
    .unwrap();
    assert_eq!(litigate_in(&dir, &["lock", "cases"]).code, 0);
    let r = run(&dir, &[]);
    assert_eq!(r.code, 2, "{}", r.stdout);
    assert_eq!(r.json["result"], Value::Null);
    let none = litigate_in(&dir, &["calibrate-replay", "lit", "--id", "x"]);
    assert_eq!(none.code, 1);
    assert!(none.stderr.contains("no .case file"), "{}", none.stderr);
}

#[test]
fn a_set_with_no_declaration_spelling_is_a_diagnostic_not_a_file() {
    let dir = histories("calibrate-replay-spelling");
    let out = "lit/recap/calibration-bad.lit";
    let r = litigate_in(
        &dir,
        &[
            "calibrate-replay",
            "cases",
            "--id",
            "Not An Id",
            "--reviewed",
            "--out",
            out,
        ],
    );
    assert_eq!(r.code, 2, "{}", r.stdout);
    assert_eq!(r.json["result"], Value::Null);
    assert!(
        r.json["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["span"]["file"] == "litigate calibrate-replay"),
        "{}",
        r.stdout
    );
    assert!(!dir.join(out).exists());
}
