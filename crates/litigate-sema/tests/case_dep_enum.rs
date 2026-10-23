//! A `.case` event argument may name a variant of an enum the used package
//! imports from a dependency (`Served(Personal)` against
//! `core::ServiceMethod`): the dependency is not in `uses`, but its enums
//! are part of the used package's vocabulary.
mod case_support;

use case_support::{check_with, clock, lock, render};
use litigate_base::{DiagnosticCode as C, Profile};
use litigate_sema::hir::{ExprKind, Package, Value};
use litigate_sema::pkg::load::{LoadedFile, LoadedPackage};
use std::sync::OnceLock;

fn loaded(name: &str, deps: &str, files: &[(&str, &str)]) -> LoadedPackage {
    LoadedPackage {
 texts: std::collections::BTreeMap::new(),
        root: format!("pkgs/{name}"),
        manifest: format!(
            "[package]\nname = \"{name}\"\nversion = \"1.0.0\"\nlanguage = 1\npack_id = \"{name}\"\n\
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

const COURT: &str = "module court.answer\n\nimport core.time.{ ServiceMethod }\n\nstart filed\n\n\
event Served(method: ServiceMethod)\n\n\
state filed: \"Complaint filed\" by Plaintiff\n{\n  \
personal \"Serve personally\" -> served { event Served(Personal) }\n  \
mail \"Serve by mail\" -> served { event Served(ServiceMethod::Mail) }\n}\n\n\
decision served: \"Answer or default\" by Defendant\n{\n  \
answer \"File an answer\" -> answered\n  default \"Let it pass\" -> defaulted\n}\n\n\
terminal answered: \"Answered\" outcome [judgment]\n\
terminal defaulted: \"Default\" outcome [judgment, loss]\n";

/// `core` declares `ServiceMethod`; `court` imports it and is the only
/// package a matter uses.
fn packages() -> &'static [Package] {
    static P: OnceLock<Vec<Package>> = OnceLock::new();
    P.get_or_init(|| {
        let core = loaded(
            "core",
            "",
            &[
                (
                    "time.lit",
                    "module core.time\n\nenum ServiceMethod { Personal, Mail }\n",
                ),
                ("start.lit", "module core.start\nstart z\nstate z: \"Z\"\n"),
            ],
        );
        let court = loaded(
            "court",
            "\n[dependencies]\ncore = { version = \"^1.0.0\", path = \"../core\" }\n",
            &[("answer.lit", COURT)],
        );
        let mut db = litigate_sema::LitDatabase::default();
        let ws = litigate_sema::db::inputs::create_workspace(&mut db, &[core, court]);
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

fn matter(served: &str) -> String {
    format!(
        "case t\nuses court ^1.0.0\nas_of 2025-04-01\nsummary \"S\"\n\n\
         parties {{\n  acme: Plaintiff Corporation\n  roadrunner: Defendant Individual\n}}\n\n\
         events {{\n  2025-03-03 {served} on roadrunner\n}}\n\n\
         stakes {{ answered: 0.usd, defaulted: 100_000.usd }}\n"
    )
}

fn check(served: &str) -> litigate_sema::case::CheckedCase {
    let lock = lock(packages());
    check_with(
        &matter(served),
        packages(),
        Some(&lock),
        &clock(packages()),
        Profile::Dev,
    )
}

fn variant_of(served: &str) -> Option<String> {
    let checked = check(served);
    assert!(
        checked.diagnostics.is_empty(),
        "{:#?}",
        render(&checked.diagnostics)
    );
    let case = checked.case?;
    let ExprKind::Call { args, .. } = &case.events.first()?.action.kind else {
        return None;
    };
    match &args.first()?.kind {
        ExprKind::Literal {
            value: Value::Variant(v),
        } => Some(v.clone()),
        _ => None,
    }
}

#[test]
fn a_bare_dependency_variant_resolves() {
    assert_eq!(
        variant_of("Served(Personal)").as_deref(),
        Some("core::ServiceMethod::Personal")
    );
}

#[test]
fn qualified_dependency_variants_resolve() {
    for written in [
        "Served(ServiceMethod::Mail)",
        "Served(core::ServiceMethod::Mail)",
    ] {
        assert_eq!(
            variant_of(written).as_deref(),
            Some("core::ServiceMethod::Mail"),
            "{written}"
        );
    }
}

#[test]
fn an_unknown_dependency_variant_lists_the_real_variants() {
    let checked = check("Served(Persnal)");
    let lines = render(&checked.diagnostics);
    assert!(
        checked
            .diagnostics
            .iter()
            .any(|d| d.code == C::TypeMismatch),
        "{lines:#?}"
    );
    insta::assert_snapshot!(lines.join("\n"));
}

/// A dependency the matter also uses is replayed as a used package, not
/// listed again as vocabulary: its variants still resolve once.
#[test]
fn a_used_dependency_is_not_vocabulary_twice() {
    let lock = lock(packages());
    let text =
        matter("Served(Personal)").replace("uses court ^1.0.0", "uses court ^1.0.0, core ^1.0.0");
    let checked = check_with(
        &text,
        packages(),
        Some(&lock),
        &clock(packages()),
        Profile::Dev,
    );
    assert!(
        checked.diagnostics.is_empty(),
        "{:#?}",
        render(&checked.diagnostics)
    );
    assert!(checked.case.is_some());
}
