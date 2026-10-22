//! Minimal programs checked, lowered, and verified in memory.
#![allow(dead_code)]
use litgraph::cite::SourceCorpus;
use litigate_base::Profile;
use litigate_lower::Options;
use litigate_sema::db::inputs::create_workspace;
use litigate_sema::pkg::load::{LoadedFile, LoadedPackage};
use litigate_sema::{hir, LitDatabase};
use litigate_verify::{check_package, Checked, Finding, Obligation};
use serde::Serialize;
use std::fmt::Write as _;

/// A package `name` at `pkgs/<name>` with one module file `main.lit`,
/// whose declarations default to `authority`, bound to the vendored FRCP
/// text, depending on `deps`.
pub fn pkg(name: &str, authority: &str, deps: &[&str], lit: &str) -> LoadedPackage {
    let mut manifest = format!(
        "[package]\nname = \"{name}\"\nversion = \"1.0.0\"\nlanguage = 1\npack_id = \"{name}\"\n\
         forum = \"demo\"\nauthority = \"{authority}\"\ntitle = \"Demo\"\nin_force = \"1990-01-01\"\n\
         known_at = \"2025-01-01\"\n\n[package.roles]\nPlaintiff = \"self\"\nDefendant = \"opponent\"\n\
         Court = \"nature\"\n\n[sources.frcp]\ntitle = \"Federal Rules of Civil Procedure\"\n\
         url = \"https://www.law.cornell.edu/rules/frcp\"\npath = \"sources/frcp.txt\"\n\
         cite_prefixes = [\"FRCP\"]\nsection_heading = \"Rule {{n}}\"\n"
    );
    if !deps.is_empty() {
        manifest.push_str("\n[dependencies]\n");
        for d in deps {
            let _ = writeln!(
                manifest,
                "{d} = {{ version = \"^1.0.0\", path = \"../{d}\" }}"
            );
        }
    }
    LoadedPackage {
        root: format!("pkgs/{name}"),
        manifest,
        lock: None,
        files: vec![LoadedFile {
            relative: "main.lit".to_owned(),
            text: lit.to_owned(),
        }],
    }
}

/// Checks `packages` and verifies the last one under `profile`.
pub fn verify(packages: &[LoadedPackage], profile: Profile) -> Checked {
    let mut db = LitDatabase::default();
    let ws = create_workspace(&mut db, packages);
    let checks = litigate_sema::check_workspace(&db, ws);
    let hirs: Vec<&hir::Package> = checks.iter().filter_map(|c| c.package.as_ref()).collect();
    let Some(last) = checks.last() else {
        panic!("no packages");
    };
    check_package(
        last,
        &hirs,
        SourceCorpus::embedded_ref(),
        &Options::default(),
        profile,
    )
}

/// One program in `name` (authority `FederalRule`), verified.
pub fn verify_one(lit: &str, profile: Profile) -> Checked {
    verify(&[pkg("demo", "FederalRule", &[], lit)], profile)
}

/// What a snapshot shows for one obligation of one program.
#[derive(Debug, Serialize)]
pub struct Outcome<'a> {
    /// The obligation's finding.
    pub finding: Option<&'a Finding>,
    /// Whether the whole report is accepted.
    pub accepted: bool,
    /// Diagnostics verification emitted for this obligation.
    pub diagnostics: Vec<&'a litigate_base::Diagnostic>,
}

/// The `name` finding and its diagnostics.
pub fn outcome(checked: &Checked, name: Obligation) -> Outcome<'_> {
    let v = &checked.verification;
    Outcome {
        finding: v.report.finding(name),
        accepted: v.is_accepted(),
        diagnostics: v
            .diagnostics
            .iter()
            .filter(|d| litigate_verify::obligation_of(d.code) == Some(name))
            .collect(),
    }
}

/// A clean strict program exercising every rule-package obligation: an
/// FRCP-cited duty deadline with its counting rule, calendar, and test.
pub const BASE: &str = r#"module demo.main

package demo { forum demo, authority FederalRule }

start served

authority Statute > FederalRule > LocalRule

enum Document { complaint }
enum ServiceMethod { Personal, Mail }

event Served(document: Document, method: ServiceMethod)
event Answered(document: Document)

counting frcp6 cite FRCP 6(a)
{
  exclude_trigger_day
  include_last_day
  roll_forward_on [weekend, legal_holiday]
  service_extension { Mail: 3.days } cite FRCP 6(d)
  holidays federal
  anchor trigger_day
}

calendar federal cite FRCP 6(a)(6)
{
  new-year: fixed(1, 1) effective [1870-06-28, ..) observed nearest_weekday
  juneteenth: fixed(6, 19) effective [2021-06-17, ..) observed nearest_weekday
}

rule duty answer-duty cite FRCP 12(a)
{
  deadline answer-period: 21.days from Served(complaint) counted_by frcp6
}

state served: "Complaint served" by Defendant cite FRCP 12(a)
{
  answer "File an answer" -> answered { event Answered(complaint), within answer-period }
}

terminal answered: "Answer filed" outcome [answered] { payoff 0.usd }

test "answer due 21 days after personal service" cite FRCP 12(a)
{
  given Served(complaint, Personal) on 2025-03-03
  expect deadline(answer-period) == 2025-03-24
}

test "mail service adds three days" cite FRCP 6(d)
{
  given Served(complaint, Mail) on 2025-03-03
  expect deadline(answer-period) == 2025-03-27
}
"#;

/// [`BASE`] with `from` replaced by `to` (which must occur).
pub fn edit(from: &str, to: &str) -> String {
    assert!(BASE.contains(from), "BASE has no {from:?}");
    BASE.replacen(from, to, 1)
}

/// [`BASE`] with `extra` appended.
pub fn with(extra: &str) -> String {
    format!("{BASE}\n{extra}\n")
}
