//! How a matter names an event type (lg-d2pi): bare when unambiguous (the
//! primary package's, else the one package of the vocabulary declaring
//! it), else by package (pack id or package name) or by the declaring
//! module's dotted path; and `names::spell` gives the shortest name that
//! reads back. A matter may write one `uses` per package. `court` imports
//! `Noticed` from `core`; `appeals`, used beside it, declares a `Noticed`
//! and a `Filed` of its own.
mod case_support;

use case_support::{check_with, clock, lock, render};
use litigate_base::{DiagnosticCode as C, Profile};
use litigate_sema::case::names::{self, Resolution};
use litigate_sema::hir::{ExprKind, Package};
use litigate_sema::pkg::load::{LoadedFile, LoadedPackage};
use std::sync::OnceLock;

fn loaded(name: &str, pack_id: &str, deps: &str, files: &[(&str, &str)]) -> LoadedPackage {
    LoadedPackage {
        texts: std::collections::BTreeMap::new(),
        root: format!("pkgs/{name}"),
        manifest: format!(
            "[package]\nname = \"{name}\"\nversion = \"1.0.0\"\nlanguage = 1\npack_id = \"{pack_id}\"\n\
             forum = \"frcp\"\nauthority = \"FederalRule\"\ntitle = \"T\"\nin_force = \"2007-12-01\"\n\
             known_at = \"2025-01-01\"\n\n[package.roles]\nPlaintiff = \"self\"\nDefendant = \"opponent\"\n\
             Court = \"nature\"\n{deps}"
        ),
        lock: None,
        files: files
            .iter()
            .map(|(p, t)| LoadedFile {
                relative: (*p).to_owned(),
                text: (*t).to_owned(),
            })
            .collect(),
    }
}

const COURT: &str = "module court.answer\n\nimport core.time.{ Noticed }\n\nstart filed\n\n\
event Filed()\n\n\
state filed: \"Complaint filed\" by Plaintiff\n{\n  \
notice \"Notice\" -> noticed { event Noticed() }\n}\n\n\
decision noticed: \"Answer or default\" by Defendant\n{\n  \
answer \"File an answer\" -> answered { event Filed() }\n  default \"Let it pass\" -> defaulted\n}\n\n\
terminal answered: \"Answered\" outcome [judgment]\n\
terminal defaulted: \"Default\" outcome [judgment, loss]\n";

const APPEALS: &str = "module appeals\n\nstart docketed\n\n\
event Noticed()\nevent Filed()\nevent Briefed()\n\n\
state docketed: \"Docketed\" by Court\n{\n  \
brief \"Brief\" -> decided { event Briefed() }\n}\n\n\
terminal decided: \"Decided\" outcome [judgment]\n";

/// `core` (pack `litigate-core`), `court`, `appeals`.
fn packages() -> &'static [Package] {
    static P: OnceLock<Vec<Package>> = OnceLock::new();
    P.get_or_init(|| {
        let core = loaded(
            "core",
            "litigate-core",
            "",
            &[
                ("time.lit", "module core.time\n\nevent Noticed()\n"),
                ("start.lit", "module core.start\nstart z\nstate z: \"Z\"\n"),
            ],
        );
        let court = loaded(
            "court",
            "court",
            "\n[dependencies]\ncore = { version = \"^1.0.0\", path = \"../core\" }\n",
            &[("answer.lit", COURT)],
        );
        let appeals = loaded("appeals", "appeals", "", &[("appeals.lit", APPEALS)]);
        let mut db = litigate_sema::LitDatabase::default();
        let ws = litigate_sema::db::inputs::create_workspace(&mut db, &[core, court, appeals]);
        litigate_sema::check_workspace(&db, ws)
            .iter()
            .map(|c| {
                c.package.clone().unwrap_or_else(|| {
                    panic!(
                        "{}",
                        serde_json::to_string_pretty(&c.diagnostics).unwrap_or_default()
                    )
                })
            })
            .collect()
    })
}

fn package(name: &str) -> &'static Package {
    packages()
        .iter()
        .find(|p| p.manifest.name == name)
        .unwrap_or_else(|| panic!("no package {name}"))
}

/// The vocabulary of a matter using `used`.
fn vocabulary(used: &[&str]) -> Vec<&'static Package> {
    let all: Vec<&Package> = packages().iter().collect();
    let used: Vec<&Package> = used.iter().map(|n| package(n)).collect();
    names::vocabulary(&used, &all)
}

/// A matter writing `uses`, noticed on 2025-03-03 by `noticed`.
fn check(uses: &str, noticed: &str) -> litigate_sema::case::CheckedCase {
    let lock = lock(packages());
    let text = format!(
        "case t\n{uses}as_of 2025-04-01\nsummary \"S\"\n\n\
         parties {{\n  acme: Plaintiff Corporation\n  roadrunner: Defendant Individual\n}}\n\n\
         events {{\n  2025-03-03 {noticed} by acme\n}}\n"
    );
    check_with(
        &text,
        packages(),
        Some(&lock),
        &clock(packages()),
        Profile::Dev,
    )
}

/// The callee `noticed` types to, or the name errors it reports.
fn callee(uses: &str, noticed: &str) -> Result<String, Vec<String>> {
    let checked = check(uses, noticed);
    let errors: Vec<_> = checked
        .diagnostics
        .iter()
        .filter(|d| matches!(d.code, C::UnresolvedName | C::UnexpectedToken))
        .cloned()
        .collect();
    if !errors.is_empty() {
        return Err(render(&errors));
    }
    match checked
        .case
        .and_then(|c| c.events.first().cloned())
        .map(|e| e.action.kind)
    {
        Some(ExprKind::Call { callee, .. }) => Ok(callee),
        other => Err(vec![format!(
            "{other:?}: {:?}",
            render(&checked.diagnostics)
        )]),
    }
}

const COURT_ONLY: &str = "uses court ^1.0.0\n";
const BOTH: &str = "uses court ^1.0.0\nuses appeals ^1.0.0\n";

#[test]
fn a_bare_dependency_event_resolves_when_no_other_package_declares_it() {
    assert_eq!(
        callee(COURT_ONLY, "Noticed()"),
        Ok("litigate-core::Noticed".to_owned())
    );
}

#[test]
fn one_uses_per_package_uses_both() {
    assert_eq!(callee(BOTH, "Briefed()"), Ok("appeals::Briefed".to_owned()));
    assert_eq!(
        callee("uses court ^1.0.0, appeals ^1.0.0\n", "Briefed()"),
        Ok("appeals::Briefed".to_owned())
    );
}

#[test]
fn the_primary_package_keeps_its_bare_names() {
    assert_eq!(callee(BOTH, "Filed()"), Ok("Filed".to_owned()));
    assert_eq!(
        callee(BOTH, "appeals::Filed()"),
        Ok("appeals::Filed".to_owned())
    );
}

#[test]
fn an_ambiguous_bare_event_lists_every_qualified_name() {
    let errors = callee(BOTH, "Noticed()").unwrap_err();
    insta::assert_snapshot!(errors.join("\n"), @r#"
    13:14 L1002 `Noticed` is an event of 2 packages: write one of `appeals::Noticed`, `litigate-core::Noticed`
        note: a bare event name is the primary package's, else the one package that declares it
        fix @13:14-13:21 MaybeIncorrect: "appeals::Noticed"
        fix @13:14-13:21 MaybeIncorrect: "litigate-core::Noticed"
    "#);
}

#[test]
fn an_event_is_named_by_pack_id_package_name_or_module() {
    for written in [
        "litigate-core::Noticed()",
        "core::Noticed()",
        "core.time.Noticed()",
    ] {
        assert_eq!(
            callee(BOTH, written),
            Ok("litigate-core::Noticed".to_owned()),
            "{written}"
        );
    }
}

#[test]
fn a_name_no_package_declares_is_unresolved() {
    for (written, error) in [
        ("core::Filed()", "`litigate-core` declares no event `Filed`"),
        (
            "nowhere::Noticed()",
            "`court` declares no event `nowhere::Noticed`",
        ),
        (
            "core.nowhere.Noticed()",
            "`court` declares no event `core.nowhere.Noticed`",
        ),
        ("Filde()", "`court` declares no event `Filde`"),
    ] {
        let errors = callee(BOTH, written).unwrap_err();
        assert!(errors[0].contains(error), "{written}: {errors:?}");
    }
}

#[test]
fn spell_gives_the_shortest_name_that_reads_back() {
    let alone = vocabulary(&["court"]);
    let both = vocabulary(&["court", "appeals"]);
    for (vocabulary, pack, name, spelled) in [
        (&alone, "litigate-core", "Noticed", "Noticed"),
        (&both, "litigate-core", "Noticed", "litigate-core::Noticed"),
        (&both, "appeals", "Noticed", "appeals::Noticed"),
        (&both, "court", "Filed", "Filed"),
        (&both, "appeals", "Filed", "appeals::Filed"),
        (&both, "appeals", "Briefed", "Briefed"),
    ] {
        assert_eq!(
            names::spell(vocabulary, pack, name),
            spelled,
            "{pack}::{name}"
        );
        assert!(
            matches!(names::resolve(vocabulary, spelled), Resolution::Event(p, t) if p == pack && t.name == name),
            "{spelled}"
        );
    }
    assert_eq!(
        names::resolve(&[], "Noticed"),
        Resolution::Missing(String::new(), "Noticed".to_owned())
    );
}
