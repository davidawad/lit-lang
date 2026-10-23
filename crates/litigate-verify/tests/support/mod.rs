//! Minimal programs checked, lowered, and verified in memory.
#![allow(dead_code)]
use litgraph::cite::SourceCorpus;
use litigate_base::Profile;
use litigate_lower::{Lowerer, Options};
use litigate_sema::db::inputs::create_workspace;
use litigate_sema::pkg::load::{LoadedFile, LoadedPackage};
use litigate_sema::{hir, LitDatabase};
use litigate_verify::{
    check_package, verify_package, Checked, Finding, Obligation, PackageInput, Verification,
};
use serde::Serialize;
use std::fmt::Write as _;

/// The `[sources.frcp]` binding to the vendored FRCP text.
pub const FRCP: &str = "[sources.frcp]\ntitle = \"Federal Rules of Civil Procedure\"\n\
     url = \"https://www.law.cornell.edu/rules/frcp\"\npath = \"sources/frcp.txt\"\n\
     path_sha256 = \"cb9a19a0ad99acdbd04f46acd3893bb96fbd34c9a8f66de57f5d171dbd268b44\"\n\
     cite_prefixes = [\"FRCP\"]\nsection_heading = \"Rule {n}\"\n";

/// A package `name` at `pkgs/<name>` with one module file `main.lit`,
/// whose declarations default to `authority`, bound to the vendored FRCP
/// text, depending on `deps`.
pub fn pkg(name: &str, authority: &str, deps: &[&str], lit: &str) -> LoadedPackage {
    pkg_with_sources(name, authority, deps, FRCP, lit)
}

/// [`pkg`] with the manifest's source bindings replaced by `sources`.
pub fn pkg_with_sources(
    name: &str,
    authority: &str,
    deps: &[&str],
    sources: &str,
    lit: &str,
) -> LoadedPackage {
    let identity = format!(
        "pack_id = \"{name}\"\nforum = \"demo\"\nauthority = \"{authority}\"\ntitle = \"Demo\"\n"
    );
    let roles = "\n[package.roles]\nPlaintiff = \"self\"\nDefendant = \"opponent\"\n\
                 Court = \"nature\"\n\n";
    loaded(name, &format!("{identity}{roles}{sources}"), deps, lit)
}

/// A links package (no pack identity, roles, or sources) composing `deps`.
pub fn links_pkg(name: &str, deps: &[&str], lit: &str) -> LoadedPackage {
    loaded(name, "", deps, lit)
}

fn loaded(name: &str, body: &str, deps: &[&str], lit: &str) -> LoadedPackage {
    let mut manifest = format!(
        "[package]\nname = \"{name}\"\nversion = \"1.0.0\"\nlanguage = 1\nin_force = \"1990-01-01\"\n\
         known_at = \"2025-01-01\"\n{body}"
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

/// Checks and lowers one program, applies `edit` to its HIR (an input sema
/// would not produce from source, as a linked or future front end might),
/// and verifies the edited HIR in `strict` against the program's lowering
/// (an edited package need not lower, and `tests-pass` needs a lowering).
pub fn verify_edited(lit: &str, edit: impl FnOnce(&mut hir::Package)) -> Verification {
    verify_edited_in(lit, edit, true)
}

/// [`verify_edited`], listing the package among the loaded ones only when
/// `listed` (a caller may pass just its dependencies).
pub fn verify_edited_in(
    lit: &str,
    edit: impl FnOnce(&mut hir::Package),
    listed: bool,
) -> Verification {
    let mut db = LitDatabase::default();
    let ws = create_workspace(&mut db, &[pkg("demo", "FederalRule", &[], lit)]);
    let checks = litigate_sema::check_workspace(&db, ws);
    let Some(check) = checks.first() else {
        panic!("no packages");
    };
    let Some(mut package) = check.package.clone() else {
        panic!("the program does not check: {:#?}", check.diagnostics);
    };
    let options = Options::default();
    let lowered = Lowerer::new(&[&package])
        .with_corpus(SourceCorpus::embedded_ref())
        .package(&package, &options);
    edit(&mut package);
    let packages = [&package];
    let input = PackageInput {
        package: &package,
        check,
        lowered: lowered.as_ref().map_err(Vec::as_slice),
        packages: if listed { &packages } else { &[] },
        options: &options,
    };
    verify_package(&input, SourceCorpus::embedded_ref(), Profile::Strict)
}

/// Every declaration of `package`, mutably.
pub fn declarations(package: &mut hir::Package) -> impl Iterator<Item = &mut hir::Declaration> {
    package
        .modules
        .iter_mut()
        .flat_map(|m| m.declarations.iter_mut())
}

/// The tests of `package`, mutably.
pub fn tests(package: &mut hir::Package) -> impl Iterator<Item = &mut hir::Test> {
    declarations(package).filter_map(|d| match d {
        hir::Declaration::Test(t) => Some(t),
        _ => None,
    })
}

/// The deadlines of `package`, mutably.
pub fn deadlines(package: &mut hir::Package) -> impl Iterator<Item = &mut hir::Deadline> {
    declarations(package).filter_map(|d| match d {
        hir::Declaration::Deadline(x) => Some(x),
        _ => None,
    })
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
    outcome_of(&checked.verification, name)
}

/// The `name` finding of `v` and its diagnostics.
pub fn outcome_of(v: &Verification, name: Obligation) -> Outcome<'_> {
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

counting frcp6 cite "FRCP 6(a)"
{
  exclude_trigger_day
  include_last_day
  roll_forward_on [weekend, legal_holiday]
  service_extension { Mail: 3.days } cite "FRCP 6(d)"
  holidays federal
  anchor trigger_day
}

calendar federal cite "FRCP 6(a)(6)"
{
  new-year: fixed(1, 1) effective [1870-06-28, ..) observed nearest_weekday
  juneteenth: fixed(6, 19) effective [2021-06-17, ..) observed nearest_weekday
}

rule duty answer-duty cite "FRCP 12(a)"
{
  deadline answer-period: 21.days from Served(complaint) counted_by frcp6
}

state served: "Complaint served" by Defendant cite "FRCP 12(a)"
{
  answer "File an answer" -> answered { event Answered(complaint), within answer-period }
}

terminal answered: "Answer filed" outcome [answered] { payoff 0.usd }

test "answer due 21 days after personal service" cite "FRCP 12(a)"
{
  given Served(complaint, Personal) on 2025-03-03
  expect deadline(answer-period) == 2025-03-24
}

test "mail service adds three days" cite "FRCP 6(d)"
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
