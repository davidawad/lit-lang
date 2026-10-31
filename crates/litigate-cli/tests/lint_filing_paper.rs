//! `litigate lint-filing`'s paper check (lg-962.10.2): a filing's counts
//! (`--words`/`--pages`/`--lines` or a document) and certificates against
//! the `limit` of its paper kind in force on the filing date, each
//! violation naming the limit, its cite and the layer that set it; the
//! event's declared kind, `--paper` only for an event that declares none,
//! a conflict an error and neither a hole.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support_cmds;

use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use support_cmds::{copy_dir, fixture, litigate_in, repo, scratch};

/// Widget has been served; its answer is timely on 2025-03-07.
const MATTER: &str = r#"case lf-paper
uses cov-example ^0.1.0, cov-local ^0.1.0
as_of 2025-03-10
summary "Paper fixture: Widget is served."

perspective widget

parties {
  acme: Plaintiff Corporation
  widget: Defendant Corporation
}

events {
  2025-03-05 Served(complaint, Personal) on widget
}

facts { time-barred: not-barred }
"#;

/// The base package's paper kinds: `brief` (100 words, a certificate,
/// delegable to a local rule) filed by `BriefFiled()`, `motion` and
/// `transcript` (50 lines).
const BASE: &str = "
limit brief delegable { to [LocalRule], op set } {
  words <= 100
  requires certificate compliance
}
limit motion {
  pages <= 10
}
limit transcript {
  lines <= 50
}
event BriefFiled() paper brief
";

/// A local rule raising `brief` to 200 words.
const LAYER_TOML: &str = r#"[package]
name = "cov-local"
version = "0.1.0"
language = 1
pack_id = "cov-local"
forum = "frcp"
authority = "LocalRule"
title = "Local rule layer (paper fixture)"
in_force = "2009-12-01"
known_at = "2025-01-01"
kind = "local-rule"

[package.roles]
court = "nature"

[package.instrument]
source = "lr"
issuer = "A district court"
date = "2009-12-01"

[dependencies]
cov-example = { version = "^0.1.0", path = "../cov" }

[sources.lr]
title = "Local rule (paper fixture)"
url = "https://www.nysd.uscourts.gov/"
path = "sources/change-sdny-edny-local-civil-rule-6.txt"
path_sha256 = "d7ea4a0e259c8609c98efe869d04776cb7e9ca119c701c3760679a01e3110aae"
as_of = "2026-01-02"
cite_prefixes = ["S.D.N.Y. Local Civ. R."]
section_heading = "S.D.N.Y. Local Civ. R. {n}"
"#;

const LAYER: &str = "module local.briefs

import frcp.cov.{ brief }

limit local-brief override brief {
  words <= 200
  requires certificate compliance
}
";

/// The coverage fixture with [`BASE`], the [`LAYER`] package and
/// [`MATTER`], locked.
fn matter(name: &str) -> PathBuf {
    let dir = scratch(name);
    copy_dir(&fixture(), &dir);
    let lit = dir.join("lit/cov/cov.lit");
    let text = std::fs::read_to_string(&lit).unwrap();
    std::fs::write(&lit, format!("{text}{BASE}")).unwrap();
    std::fs::create_dir_all(dir.join("lit/local")).unwrap();
    std::fs::write(dir.join("lit/local/Lit.toml"), LAYER_TOML).unwrap();
    std::fs::write(dir.join("lit/local/local.lit"), LAYER).unwrap();
    std::fs::create_dir_all(dir.join("sources")).unwrap();
    let source = "change-sdny-edny-local-civil-rule-6.txt";
    std::fs::copy(
        repo().join("sources").join(source),
        dir.join("sources").join(source),
    )
    .unwrap();
    std::fs::write(dir.join("cases/lf.case"), MATTER).unwrap();
    assert_eq!(litigate_in(&dir, &["lock", "lit/local"]).code, 0);
    assert_eq!(litigate_in(&dir, &["lock", "cases"]).code, 0);
    dir
}

/// `lint-filing` of `event` by Widget on 2025-03-07 in the fixture.
fn lint(dir: &Path, event: &str, args: &[&str]) -> support_cmds::Run {
    let mut all = vec![
        "lint-filing",
        "cases/lf.case",
        event,
        "--date",
        "2025-03-07",
        "--by",
        "widget",
    ];
    all.extend(args);
    litigate_in(dir, &all)
}

/// `lint-filing` of an IPR petition in the PTAB matter on 2026-09-10.
fn ptab(event: &str, args: &[&str]) -> support_cmds::Run {
    let mut all = vec![
        "lint-filing",
        "cases/ptab-ipr-defense.case",
        event,
        "--date",
        "2026-09-10",
        "--by",
        "petitioner",
    ];
    all.extend(args);
    litigate_in(&repo(), &all)
}

fn requirements(paper: &Value) -> Vec<&str> {
    paper["violations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["requirement"].as_str().unwrap())
        .collect()
}

#[test]
fn over_the_word_bound_is_a_violation_with_its_cite_and_layer() {
    let run = ptab(
        "IprPetitionFiled()",
        &["--words", "14500", "--certificate", "word-count"],
    );
    let paper = &run.json["result"]["paper"];
    assert_eq!(paper["kind"], json!("ipr-petition"), "{}", run.stdout);
    assert_eq!(paper["kind_from"], json!("event"));
    assert_eq!(paper["length"], json!("over"));
    assert_eq!(requirements(paper), ["length"]);
    let v = &paper["violations"][0];
    assert_eq!(
        v["limit"],
        json!("ptab-patent-trial-appeal-board::ipr-petition")
    );
    assert_eq!(v["cite"], json!("37 C.F.R. § 42.24"));
    assert_eq!(
        v["layer"],
        json!({
            "package": "ptab-patent-trial-appeal-board", "pack": "ptab-patent-trial-appeal-board",
            "authority": "Regulation", "version": "ipr-petition@2021-01-08",
            "provenance": ["ptab-patent-trial-appeal-board"],
        })
    );
    assert!(
        v["message"]
            .as_str()
            .unwrap()
            .contains("14500 words over 14000 words (37 C.F.R. § 42.24(a)(1)(i))"),
        "{v}"
    );
}

#[test]
fn a_required_certificate_not_carried_is_a_violation() {
    let run = ptab("IprPetitionFiled()", &["--words", "13500"]);
    let paper = &run.json["result"]["paper"];
    assert_eq!(paper["length"], json!("within"), "{}", run.stdout);
    assert_eq!(requirements(paper), ["certificate"]);
    assert_eq!(paper["violations"][0]["certificate"], json!("word-count"));
    assert_eq!(
        paper["violations"][0]["cite"],
        json!("37 C.F.R. § 42.24(d)")
    );
    let carried = ptab(
        "IprPetitionFiled()",
        &["--words", "13500", "--certificate", "word-count"],
    );
    assert_eq!(
        carried.json["result"]["paper"]["violations"],
        json!([]),
        "{}",
        carried.stdout
    );
}

#[test]
fn a_document_gives_the_word_count() {
    let dir = scratch("paper-document");
    let doc = dir.join("petition.txt");
    std::fs::write(&doc, "word ".repeat(14_001)).unwrap();
    let doc = doc.to_string_lossy().into_owned();
    let run = ptab(
        "IprPetitionFiled()",
        &["--document", &doc, "--certificate", "word-count"],
    );
    let paper = &run.json["result"]["paper"];
    assert_eq!(paper["counts"]["words"], json!(14_001), "{}", run.stdout);
    assert_eq!(paper["counts"]["source"], json!("document"));
    // The document counts the parts § 42.24(a)(1) excludes: over the
    // bound is not decided.
    assert_eq!(paper["length"], json!("unknown"));
    assert_eq!(paper["violations"], json!([]));
    std::fs::write(dir.join("short.txt"), "word ".repeat(10)).unwrap();
    let short = dir.join("short.txt").to_string_lossy().into_owned();
    let run = ptab(
        "IprPetitionFiled()",
        &["--document", &short, "--certificate", "word-count"],
    );
    assert_eq!(run.json["result"]["paper"]["length"], json!("within"));
    let both = ptab(
        "IprPetitionFiled()",
        &["--document", &short, "--words", "3"],
    );
    assert_eq!(both.code, 1);
    assert!(
        both.stderr.contains("both give the word count"),
        "{}",
        both.stderr
    );
}

#[test]
fn a_layer_sets_the_limit_and_is_named_in_each_violation() {
    let dir = matter("paper-layer");
    let within = lint(
        &dir,
        "BriefFiled()",
        &["--words", "150", "--certificate", "compliance"],
    );
    let paper = &within.json["result"]["paper"];
    // 150 words is over the base's 100 but within the local rule's 200.
    assert_eq!(paper["length"], json!("within"), "{}", within.stdout);
    assert_eq!(paper["bounds"][0]["max"], json!(200));
    let over = lint(&dir, "BriefFiled()", &["--words", "250"]);
    let paper = &over.json["result"]["paper"];
    assert_eq!(
        requirements(paper),
        ["length", "certificate"],
        "{}",
        over.stdout
    );
    for v in paper["violations"].as_array().unwrap() {
        assert_eq!(v["limit"], json!("cov-pack::brief"));
        assert_eq!(
            v["layer"],
            json!({
                "package": "cov-local", "pack": "cov-local", "authority": "LocalRule",
                "version": "local-brief@2009-12-01",
                "provenance": ["cov-example", "cov-local"],
            })
        );
    }
}

#[test]
fn a_violation_fails_the_lint_of_an_accepted_filing() {
    let dir = matter("paper-exit");
    let ok = lint(
        &dir,
        "Answered(complaint)",
        &["--paper", "motion", "--pages", "10"],
    );
    assert_eq!(ok.code, 0, "{}", ok.stdout);
    assert_eq!(ok.json["result"]["paper"]["kind_from"], json!("flag"));
    assert_eq!(
        ok.json["result"]["paper"]["limit"],
        json!("cov-pack::motion")
    );
    let over = lint(
        &dir,
        "Answered(complaint)",
        &["--paper", "motion", "--pages", "11"],
    );
    assert_eq!(over.code, 2, "{}", over.stdout);
    assert_eq!(over.json["result"]["accepted"], json!(true));
    assert_eq!(requirements(&over.json["result"]["paper"]), ["length"]);
    // Without anything said of the paper, nothing is checked.
    let plain = lint(&dir, "Answered(complaint)", &[]);
    assert_eq!(plain.code, 0, "{}", plain.stdout);
    assert_eq!(plain.json["result"].get("paper"), None);
}

#[test]
fn a_lines_count_is_judged_against_the_lines_bound() {
    let dir = matter("paper-lines");
    let within = lint(
        &dir,
        "Answered(complaint)",
        &["--paper", "transcript", "--lines", "50"],
    );
    assert_eq!(within.code, 0, "{}", within.stdout);
    let paper = &within.json["result"]["paper"];
    assert_eq!(paper["counts"]["lines"], json!(50));
    assert_eq!(paper["length"], json!("within"));
    let over = lint(
        &dir,
        "Answered(complaint)",
        &["--paper", "transcript", "--lines", "51"],
    );
    assert_eq!(over.code, 2, "{}", over.stdout);
    let paper = &over.json["result"]["paper"];
    assert_eq!(requirements(paper), ["length"]);
    assert!(
        paper["violations"][0]["message"]
            .as_str()
            .unwrap()
            .contains("51 lines over 50 lines"),
        "{paper}"
    );
}

#[test]
fn the_declared_kind_wins_and_a_conflicting_flag_is_an_error() {
    let dir = matter("paper-conflict");
    let same = lint(
        &dir,
        "BriefFiled()",
        &[
            "--paper",
            "brief",
            "--words",
            "1",
            "--certificate",
            "compliance",
        ],
    );
    assert_eq!(
        same.json["result"]["paper"]["kind_from"],
        json!("event"),
        "{}",
        same.stdout
    );
    let conflict = lint(&dir, "BriefFiled()", &["--paper", "motion", "--pages", "1"]);
    assert_eq!(conflict.code, 1, "{}", conflict.stdout);
    assert!(
        conflict.stderr.contains("files paper `cov-pack::brief`"),
        "{}",
        conflict.stderr
    );
    let unknown = lint(&dir, "Answered(complaint)", &["--paper", "nothing"]);
    assert_eq!(unknown.code, 1);
    assert!(
        unknown.stderr.contains("no `limit nothing`"),
        "{}",
        unknown.stderr
    );
}

#[test]
fn neither_a_declared_nor_a_flagged_kind_is_a_hole() {
    let run = ptab("PgrPetitionFiled()", &["--words", "1"]);
    let paper = &run.json["result"]["paper"];
    assert_eq!(paper["kind"], Value::Null, "{}", run.stdout);
    assert_eq!(paper["limit"], Value::Null);
    let reason = paper["holes"][0]["reason"].as_str().unwrap();
    assert!(
        reason.contains("`PgrPetitionFiled()` declares no `paper` and --paper names none"),
        "{reason}"
    );
    // With the flag, the event that declares none files that kind.
    let flagged = ptab(
        "PgrPetitionFiled()",
        &["--paper", "ipr-petition", "--words", "1"],
    );
    assert_eq!(flagged.json["result"]["paper"]["kind_from"], json!("flag"));
}

#[test]
fn a_filing_before_the_encoded_text_has_no_limit() {
    let run = litigate_in(
        &repo(),
        &[
            "lint-filing",
            "cases/ptab-ipr-defense.case",
            "IprPetitionFiled()",
            "--date",
            "2016-01-04",
            "--by",
            "petitioner",
            "--words",
            "1",
        ],
    );
    let paper = &run.json["result"]["paper"];
    assert_eq!(paper["limit"], Value::Null, "{}", run.stdout);
    let reason = paper["holes"][0]["reason"].as_str().unwrap();
    assert!(
        reason.contains("`ptab-patent-trial-appeal-board::ipr-petition`")
            && reason.contains("on 2016-01-04"),
        "{reason}"
    );
}
