//! Computed compatibility classes (SPEC §4.4): each kind of change between
//! two releases of a package, and the bump it needs.
#![allow(clippy::unwrap_used)]
mod cov_case_support;
use cov_case_support::{compile, package};
use litigate_sema::hir::Package;
use litigate_sema::pkg::compat::{classify, Bump, Class};
use litigate_sema::pkg::load::LoadedPackage;

const BASE: &str = "module alpha
start intake
enum Document { complaint, motion }
event Served(document: Document)
counting ac { exclude_trigger_day, include_last_day, holidays acal }
calendar acal { new-year: fixed(1, 1) effective [1870-06-28, ..) observed nearest_weekday }
deadline answer-period: 21.days from Served(complaint) counted_by ac
state intake: \"Intake\" by Plaintiff {
  serve \"Serve\" -> responded { event Served(complaint) within answer-period }
  wait \"Wait\" -> responded
}
terminal responded: \"Responded\"
test \"answer\" { given Served(complaint) on 2025-03-03 expect deadline(answer-period) == 2025-03-24 }
";

fn loaded(version: &str, roles: &str, text: &str) -> LoadedPackage {
    let mut p = package("alpha", &[], roles, text);
    p.manifest = p
        .manifest
        .replace("version = \"1.0.0\"", &format!("version = \"{version}\""));
    p
}

fn hir(version: &str, text: &str) -> Package {
    compile(&[loaded(version, "", text)]).remove(0)
}

/// `(class, required, declared, changes)` of `new` (at `version`) against
/// `BASE` at 1.0.0.
fn against_base(version: &str, new: &str) -> (Class, Bump, Option<Bump>, Vec<String>) {
    let c = classify(&hir("1.0.0", BASE), &hir(version, new));
    let changes = c
        .changes
        .iter()
        .map(|c| {
            let now = c
                .now
                .as_ref()
                .map(|n| format!(" -> {n}"))
                .unwrap_or_default();
            format!("{:?} {} {}{now}", c.class, c.kind, c.what)
        })
        .collect();
    (c.class, c.required, c.declared, changes)
}

#[test]
fn reformatting_labels_notes_and_tests_change_nothing() {
    let edited = BASE
        .replace("\"Intake\"", "\"Intake (renamed label)\"")
        .replace(
            "terminal responded: \"Responded\"",
            "// a comment\nterminal responded: \"Responded\" { note \"n\" }",
        )
        .replace(
            "2025-03-24 }",
            "2025-03-24 expect deadline(answer-period) > 2025-03-03 }",
        );
    let (class, required, declared, changes) = against_base("1.0.1", &edited);
    assert_eq!(
        (class, required, declared),
        (Class::None, Bump::Pre, Some(Bump::Patch))
    );
    assert!(changes.is_empty(), "{changes:?}");
}

#[test]
fn a_changed_period_or_an_added_declaration_is_a_value_change() {
    let shorter = BASE
        .replace("21.days", "14.days")
        .replace("2025-03-24", "2025-03-17");
    let added = format!("{BASE}deadline reply-period: 7.days from Served(motion) counted_by ac\n");
    let table: Vec<_> = [("1.1.0", shorter.as_str()), ("1.0.1", added.as_str())]
        .into_iter()
        .map(|(version, text)| against_base(version, text))
        .collect();
    insta::assert_debug_snapshot!(table);
}

#[test]
fn removed_renamed_and_retyped_ids_are_interface_changes() {
    let renamed = BASE
        .replace("responded", "answered")
        .replace(
            "terminal answered: \"Responded\"",
            "terminal answered: \"Responded\" { formerly [responded] }",
        )
        .replace(
            "serve \"Serve\" -> answered { event",
            "served \"Serve\" -> answered { formerly [serve] event",
        );
    let removed = BASE.replace("  wait \"Wait\" -> responded\n", "");
    let retyped = BASE.replace("{ complaint, motion }", "{ complaint, motion, brief }");
    let gone = BASE
        .replace(
            "deadline answer-period: 21.days from Served(complaint) counted_by ac\n",
            "",
        )
        .replace(" within answer-period", "")
        .replace(
            "expect deadline(answer-period) == 2025-03-24",
            "expect 1 == 1",
        );
    let table: Vec<_> = [
        ("2.0.0", renamed.as_str()),
        ("1.1.0", removed.as_str()),
        ("2.0.0", retyped.as_str()),
        ("2.0.0", gone.as_str()),
    ]
    .into_iter()
    .map(|(version, text)| against_base(version, text))
    .collect();
    insta::assert_debug_snapshot!(table);
}

#[test]
fn identity_and_actors_are_interface() {
    let old = hir("1.0.0", BASE);
    let mut moved = loaded("2.0.0", "Clerk = \"nature\"\n", BASE);
    moved.manifest = moved
        .manifest
        .replace("pack_id = \"alpha\"", "pack_id = \"alpha-two\"")
        .replace("Defendant = \"opponent\"", "Defendant = \"nature\"");
    let new = compile(&[moved]).remove(0);
    let c = classify(&old, &new);
    let changes: Vec<String> = c
        .changes
        .iter()
        .map(|c| format!("{:?} {}", c.class, c.what))
        .collect();
    assert_eq!(
        changes,
        [
            "Interface manifest pack_id",
            "Interface actor Defendant",
            "Value package",
        ]
    );
    assert!(c.ok(false));
    let back = classify(&new, &old);
    assert_eq!(
        (back.class, back.declared, back.ok(false)),
        (Class::Interface, None, false)
    );
}

#[test]
fn links_are_keyed_by_source_and_edge() {
    let base = "module beta\nstart b\nstate b: \"B\" { go \"Go\" -> b2 }\nterminal b2: \"B2\"\n";
    let links = |edge: &str| {
        format!(
            "module links\nimport beta\nlink {edge} \"Go on\": beta::b -> beta::b2\nlink _ \"Anon\": beta::b2 -> beta::b\n"
        )
    };
    let links_package = |version: &str, edge: &str| {
        let mut p = package("delta", &["beta"], "", &links(edge));
        p.manifest = format!(
            "[package]\nname = \"delta\"\nversion = \"{version}\"\nlanguage = 1\nin_force = \"2009-12-01\"\nknown_at = \"2025-01-01\"\n\n[dependencies]\nbeta = {{ version = \"^1.0.0\", path = \"../beta\" }}\n"
        );
        p
    };
    let compiled = |version: &str, edge: &str| {
        compile(&[package("beta", &[], "", base), links_package(version, edge)]).remove(1)
    };
    let c = classify(&compiled("1.0.0", "on"), &compiled("2.0.0", "onward"));
    let what: Vec<String> = c
        .changes
        .iter()
        .map(|c| format!("{} {}", c.kind, c.what))
        .collect();
    assert_eq!(
        what,
        [
            "removed link \"beta::b\"#\"on\"",
            "added link \"beta::b\"#\"onward\""
        ]
    );
    assert_eq!(c.class, Class::Interface);
}
