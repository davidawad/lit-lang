//! Declaration metadata and field helpers of `types::build` (SPEC §5, §7,
//! §8.1): cites, effective intervals, relations, actors, docs, and fields.
mod common;
use common::{check_body, package, render, workspace};
use litigate_sema::hir::{Declaration, Metadata};

/// Metadata (without spans) of the defaults and deadlines of an error-free
/// package, except the `b*` relation targets.
fn metas(body: &str) -> Vec<Metadata> {
    let text = format!("{}{body}\n", common::PRELUDE);
    let (db, ws) = workspace(&[package("demo", &[], &[("main.lit", &text)])]);
    assert_eq!(render(&db, ws), Vec::<String>::new());
    litigate_sema::check_workspace(&db, ws)
        .into_iter()
        .flat_map(|c| c.package.iter())
        .flat_map(|p| p.modules.iter())
        .flat_map(|m| m.declarations.iter())
        .filter_map(|d| match d {
            Declaration::Default(x) => Some(x.meta.clone()),
            Declaration::Deadline(x) => Some(x.meta.clone()),
            _ => None,
        })
        .filter(|m| !m.id.starts_with('b'))
        .map(|m| Metadata { span: None, ..m })
        .collect()
}

#[test]
fn metadata_lowers_cites_intervals_relations_and_docs() {
    let out = metas(
        "authority FederalRule > Clerk\n\
default b1 when true => 1\ndefault b2 when true => 1\ndefault b3 when true => 1\n\
default b4 when true => 1\ndefault b5 when true => 1\n\
/// imported\n/// Documented default\n\
default rel cite ?(\"FRCP 6(a), (b)\") effective [2000-01-01, ..) known_at 2020-01-01 subject_to b1 notwithstanding b2 supersedes b3 revives b4 override b5 delegable_to [Clerk] when true => 2\n\
/// imported\n\
default marker when true => 3\n\
rule duty r effective [2005-01-01, ..) known_at 2021-01-01 {\n\
  deadline inner: 3.days from Served(complaint) counted_by c6\n\
}",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn metadata_reports_bad_cites_intervals_and_relations() {
    let out = check_body(
        "default c2\n  cite \"FRCP 6\"\n  cite \"FRCP 7\"\n  when true => 1\n\
default e1 effective [2001-01-01, 2000-01-01) when true => 1\n\
default r1 subject_to nope delegable_to [Nope] when true => 1",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn fields_are_given_once_and_id_lists_hold_plain_identifiers() {
    let out = check_body("state f1: \"F\" { note \"a\", note \"b\", tags [a.b] }");
    insta::assert_debug_snapshot!(out);
}

#[test]
fn actors_are_declared_roles_or_either() {
    let out = check_body(
        "state a1: \"A\" by either { _ \"go\" -> t }\n\
state a2: \"A\" by Plaintif { _ \"go\" -> t }\n\
state a3: \"A\" by Plaintiff.x { _ \"go\" -> t }",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn a_pack_without_roles_lists_no_actors() {
    let manifest = common::manifest("demo", &[]).replace(
        "Plaintiff = \"self\"\nDefendant = \"opponent\"\nCourt = \"nature\"\n",
        "",
    );
    let text =
        "module main\nstart s\nstate s: \"S\" by Anyone { _ \"go\" -> t }\nterminal t: \"T\"\n";
    let pkg = litigate_sema::pkg::load::LoadedPackage {
        manifest,
        ..package("demo", &[], &[("main.lit", text)])
    };
    let (db, ws) = workspace(&[pkg]);
    let notes: Vec<_> = litigate_sema::check_workspace(&db, ws)
        .into_iter()
        .flat_map(|c| c.diagnostics.iter())
        .map(|d| {
            (
                d.message.clone(),
                d.notes
                    .iter()
                    .map(|n| n.message.clone())
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    insta::assert_debug_snapshot!(notes);
}

#[test]
fn references_resolve_to_the_expected_kind() {
    let out = check_body("start a.b::c\nstart c6\nstart nope");
    insta::assert_debug_snapshot!(out);
}
