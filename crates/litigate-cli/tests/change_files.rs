//! The deadline domain's change files (lg-fq2.6;
//! docs/litigate/CHANGE-REPORT.md): the FRCP and FRAP base layers, the
//! amendments of 2005 and 2009 as separately versioned layers on them, the
//! actual world at each circuit, and every collected ruling that counts one
//! of their periods as a matter, replayed with `litigate worlds` under the
//! layers in force on its own dates.
//!
//! The validation package (docs/litigate/validation, `change_validation.rs`)
//! declares each period's versions in one package; here each version is the
//! instrument that made it, and a ruling's period is selected from the
//! layers of its world. The two must give the court's answer alike.
// Test code: a panic is the failed assertion.
#![allow(clippy::unwrap_used, clippy::expect_used)]
mod support;

use serde_json::Value;
use std::collections::BTreeSet;
use support::{litigate, repo};

const DIR: &str = "docs/litigate/change";

/// The layers, one package each: the base of each rule set and the orders
/// that amended it.
const LAYERS: [(&str, &str); 5] = [
    ("frcp-base", "base"),
    ("frcp-2009", "amendment"),
    ("frap-base", "base"),
    ("frap-2005", "amendment"),
    ("frap-2009", "amendment"),
];

/// A ruling replayed through the change files: its matter (named by its
/// MANIFEST.json id), the world it runs in (the family and the circuit of
/// the court that decided it), the period the filing answers, the due date
/// the layers give, and whether the court held the filing timely.
struct Ruling {
    id: &'static str,
    world: &'static str,
    period: &'static str,
    due: &'static str,
    court_timely: bool,
}

const fn ruling(
    id: &'static str,
    world: &'static str,
    period: &'static str,
    due: &'static str,
    court_timely: bool,
) -> Ruling {
    Ruling {
        id,
        world,
        period,
        due,
        court_timely,
    }
}

/// Each due date is the court's own, or the corpus's DERIVED one
/// (VALIDATION.md §3), except Circuitronix's: the court's July 8, 2019
/// rests on the courthouse closed July 5, which no matter can state (D2).
const RULINGS: [Ruling; 19] = [
    ruling(
        "cavaliere-11cir-1993",
        "district-civil-ca11",
        "new-trial-motion",
        "1992-04-07",
        false,
    ),
    ruling(
        "parker-10cir-1996",
        "district-civil-ca10",
        "alter-amend-motion",
        "1994-08-15",
        false,
    ),
    ruling(
        "derrington-bey-dccir-1994",
        "district-civil-cadc",
        "alter-amend-motion",
        "1993-07-16",
        false,
    ),
    ruling(
        "halicki-5cir-1998",
        "district-civil-ca5",
        "alter-amend-motion",
        "1997-03-19",
        false,
    ),
    ruling(
        "midwest-employers-5cir-1998",
        "district-civil-ca5",
        "new-trial-motion",
        "1997-10-29",
        false,
    ),
    ruling(
        "albright-3cir-2001",
        "district-civil-ca3",
        "alter-amend-motion",
        "2000-11-28",
        false,
    ),
    ruling(
        "jackson-v-crosby-11cir-2004",
        "district-civil-ca11",
        "alter-amend-motion",
        "2003-12-30",
        false,
    ),
    ruling(
        "justice-7cir-2012",
        "district-civil-ca7",
        "alter-amend-motion",
        "2011-11-22",
        false,
    ),
    ruling(
        "malek-2cir-2024",
        "district-civil-ca2",
        "alter-amend-motion",
        "2023-04-28",
        false,
    ),
    ruling(
        "eastman-3cir-2013",
        "district-civil-ca3",
        "class-appeal-petition",
        "2013-08-14",
        false,
    ),
    ruling(
        "lorenzo-4cir-2015",
        "district-civil-ca4",
        "class-appeal-petition",
        "2014-04-07",
        false,
    ),
    ruling(
        "circuitronix-11cir-2021",
        "district-civil-ca11",
        "renewed-jmol-motion",
        "2019-07-05",
        true,
    ),
    ruling(
        "lashley-5cir-1975",
        "circuit-appellate-ca5",
        "notice-of-appeal",
        "1975-05-27",
        false,
    ),
    ruling(
        "ultimate-appliance-6cir-2010",
        "circuit-appellate-ca6",
        "appeal-extension-motion",
        "2008-12-01",
        false,
    ),
    ruling(
        "pierce-6cir-2021",
        "circuit-appellate-ca6",
        "notice-of-appeal",
        "2020-09-11",
        true,
    ),
    ruling(
        "benavides-dccir-1996",
        "circuit-appellate-cadc",
        "reopen-motion-window",
        "1993-03-19",
        true,
    ),
    ruling(
        "ryan-2cir-1999",
        "circuit-appellate-ca2",
        "reopen-motion-window",
        "1998-07-07",
        false,
    ),
    ruling(
        "bass-5cir-2000",
        "circuit-appellate-ca5",
        "reopen-motion-window",
        "1999-06-04",
        true,
    ),
    ruling(
        "wilkens-5cir-2001",
        "circuit-appellate-ca5",
        "reopen-motion-window",
        "1999-09-14",
        false,
    ),
];

/// The rulings the replay disagrees with, and why (CHANGE-REPORT.md).
const DISAGREES: [(&str, &str); 1] = [("circuitronix-11cir-2021", "D2")];

/// `litigate worlds MATTER --world WORLD` over the change files.
fn worlds(matter: &str, world: &str) -> Value {
    let run = litigate(&[
        "worlds",
        &format!("{DIR}/{matter}.case"),
        "--world",
        &format!("{DIR}/worlds/{world}.world"),
        "--path",
        DIR,
    ]);
    assert_eq!(run.code, 0, "{matter}: {}", run.stdout);
    run.json["result"].clone()
}

/// The outcome of the matter's one filing in each world, by world id.
fn outcomes(result: &Value) -> Vec<(String, String)> {
    let [filing] = result["filings"].as_array().unwrap().as_slice() else {
        panic!("one filing: {result}");
    };
    let mut out: Vec<(String, String)> = filing["worlds"]
        .as_object()
        .unwrap()
        .iter()
        .flat_map(|(outcome, ids)| {
            ids.as_array()
                .unwrap()
                .iter()
                .map(move |id| (id.as_str().unwrap().to_owned(), outcome.clone()))
        })
        .collect();
    out.sort();
    out
}

#[test]
fn every_layer_and_world_checks() {
    let mut args = vec!["check".to_owned()];
    args.extend(LAYERS.iter().map(|(dir, _)| format!("{DIR}/{dir}")));
    args.push(format!("{DIR}/worlds"));
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let run = litigate(&args);
    assert_eq!(run.code, 0, "{}", run.stdout);
    let packages = run.json["result"]["packages"].as_array().unwrap();
    for (dir, class) in LAYERS {
        let manifest =
            std::fs::read_to_string(repo().join(DIR).join(dir).join("Lit.toml")).unwrap();
        let amendment = manifest.contains("kind = \"amendment\"");
        assert_eq!(amendment, class == "amendment", "{dir}");
        let name = manifest
            .lines()
            .find_map(|l| l.strip_prefix("name = \""))
            .unwrap()
            .trim_end_matches('"');
        let package = packages.iter().find(|p| p["name"] == name).unwrap();
        assert_eq!(package["holes"], 0, "{dir}: every cite proved");
        let tests = &package["tests"];
        assert_eq!(tests["failed"], 0, "{dir}");
        assert!(tests["passed"].as_u64().unwrap() > 0, "{dir}");
    }
    // Every world holds its layers in force.
    let worlds = run.json["result"]["worlds"].as_array().unwrap();
    assert_eq!(worlds.len(), 13, "twelve circuits' worlds and frap-4a6");
    for w in worlds {
        for layer in w["layers"].as_array().unwrap() {
            assert_eq!(layer["in_force"], true, "{w}");
        }
    }
}

#[test]
fn every_ruling_replays_under_its_world() {
    for r in &RULINGS {
        let result = worlds(r.id, r.world);
        let [world] = result["worlds"].as_array().unwrap().as_slice() else {
            panic!("{}: one actual world: {result}", r.id);
        };
        let deadline = world["deadlines"]
            .as_array()
            .unwrap()
            .iter()
            .find(|d| {
                d["deadline"]
                    .as_str()
                    .unwrap()
                    .ends_with(&format!("::{}", r.period))
            })
            .unwrap_or_else(|| panic!("{}: {}", r.id, world["deadlines"]));
        assert_eq!(deadline["due"], r.due, "{}", r.id);
        let disagrees = DISAGREES.iter().any(|(id, _)| *id == r.id);
        let model_timely = r.court_timely != disagrees;
        let outcome = if model_timely { "timely" } else { "late" };
        assert_eq!(
            outcomes(&result),
            [(world["id"].as_str().unwrap().to_owned(), outcome.to_owned())],
            "{}",
            r.id
        );
    }
}

/// Every matter in the directory is a ruling here or the synthetic one, and
/// every ruling is a validation ruling of the lg-fq2.3 corpus that
/// VALIDATION.md also counts.
#[test]
fn the_matters_are_the_collected_rulings() {
    let manifest: Value = serde_json::from_str(
        &std::fs::read_to_string(repo().join("sources/change-corpus/rulings/MANIFEST.json"))
            .unwrap(),
    )
    .unwrap();
    let validation: BTreeSet<&str> = manifest["rulings"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["counts_as_validation_ruling"] == true)
        .map(|r| r["id"].as_str().unwrap())
        .collect();
    let matters: BTreeSet<String> = std::fs::read_dir(repo().join(DIR))
        .unwrap()
        .filter_map(|e| {
            let name = e.unwrap().file_name().into_string().unwrap();
            name.strip_suffix(".case").map(str::to_owned)
        })
        .filter(|name| !name.starts_with("synthetic-"))
        .collect();
    let rulings: BTreeSet<String> = RULINGS.iter().map(|r| r.id.to_owned()).collect();
    assert_eq!(matters, rulings);
    for id in &rulings {
        assert!(validation.contains(id.as_str()), "{id}");
    }
}

/// FRAP 4(a)(6) under each of its texts (`worlds/frap-4a6.world`): a motion
/// to reopen twelve days after notice in 2012 is timely only where the 2009
/// order's 14 days are in force; under the 1991 and 2005 texts its 7 days,
/// counted under FRAP 26 as amended in 2009, end March 9.
#[test]
fn the_reopening_window_differs_by_text() {
    let result = worlds("synthetic-reopen-2012", "frap-4a6");
    let mut by_text: Vec<(String, String, String)> = result["worlds"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| {
            (
                w["choices"]["frap-4a6"].as_str().unwrap().to_owned(),
                w["deadlines"][0]["due"].as_str().unwrap().to_owned(),
                w["deadlines"][0]["status"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    by_text.sort();
    let row = |t: &str, due: &str, status: &str| (t.to_owned(), due.to_owned(), status.to_owned());
    assert_eq!(
        by_text,
        [
            row("text-1991", "2012-03-09", "late"),
            row("text-2005", "2012-03-09", "late"),
            row("text-2009", "2012-03-16", "discharged"),
        ]
    );
}

/// CHANGE-REPORT.md states the rate the replay gives.
#[test]
fn the_report_states_the_replay_rate() {
    let agree = RULINGS.len() - DISAGREES.len();
    let report = std::fs::read_to_string(repo().join("docs/litigate/CHANGE-REPORT.md")).unwrap();
    let line = format!(
        "| Rulings replayed through the change files (`litigate worlds`) | {agree} of {} | {} (Circuitronix, D2) |",
        RULINGS.len(),
        DISAGREES.len()
    );
    assert!(report.contains(&line), "CHANGE-REPORT.md lacks: {line}");
}
