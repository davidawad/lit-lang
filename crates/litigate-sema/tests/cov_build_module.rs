//! Module-level checks of `types::build` (SPEC §4.3, §5): duplicate
//! declarations, import diagnostics, package headers, and module identity.
mod common;
use common::{check_body, package, render, workspace};

#[test]
fn a_test_name_is_declared_once() {
    insta::assert_debug_snapshot!(check_body("test \"t\" {}\ntest \"t\" {}\ntest \"u\" {}"));
}

#[test]
fn duplicates_are_per_namespace_and_versions_need_effective_on_each() {
    let out = check_body(
        "enum X { a }\nenum X { b }\n\
default v effective [2000-01-01, ..) when true => 1\n\
deadline v: 1.days from Served(complaint) counted_by c6 effective [2000-01-01, ..)\n\
default w effective [2000-01-01, 2001-01-01) when true => 1\ndefault w when true => 2\n\
default y when true => 1\ndefault y effective [2000-01-01, ..) when true => 2",
    );
    insta::assert_debug_snapshot!(out);
}

const LIB: &str = "module lib\ngroup g: \"G\"\nauthority Court > Clerk\ntype Amount = Money<usd>\ndefault x when true => 1\ndefault y when true => 2\n";
const OTHER: &str = "module other\nstart t\nterminal t: \"T\"\ndefault y when true => 3\n";

fn main_with(imports: &str, body: &str) -> Vec<String> {
    let main = format!("module main\n{imports}start s\nterminal s: \"S\"\n{body}");
    let (db, ws) = workspace(&[
        package("demo", &["dep"], &[("main.lit", &main), ("lib.lit", LIB)]),
        package("dep", &[], &[("other.lit", OTHER)]),
    ]);
    render(&db, ws)
}

#[test]
fn imported_names_exist_in_every_namespace() {
    // Groups, authorities, types, and values all import cleanly.
    let out = main_with("import lib.{g, Court, Amount, x}\n", "");
    assert_eq!(out, Vec::<String>::new());
}

#[test]
fn imported_names_must_not_shadow_or_collide() {
    let out = main_with(
        "import lib.{x, g, Amount}\nimport other.{y}\nimport lib.{y}\nimport lib.{y}\n",
        "default x when true => 4\ngroup g: \"Mine\"\ntype Amount = Money<usd>\n",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn unknown_modules_suggest_only_when_close() {
    let out = main_with("import lbi\nimport zzzzzzzz.qqqqqq\n", "");
    insta::assert_debug_snapshot!(out);
}

#[test]
fn two_headerless_files_are_both_the_unnamed_module() {
    let (db, ws) = workspace(&[package(
        "demo",
        &[],
        &[
            ("a.lit", "start s\nterminal s: \"S\"\n"),
            ("b.lit", "default d when true => 1\n"),
        ],
    )]);
    insta::assert_debug_snapshot!(render(&db, ws));
}

#[test]
fn package_header_fields_are_references() {
    let main = "module main\npackage demo { forum 3, authority FederalRule }\nstart s\nterminal s: \"S\"\n";
    let (db, ws) = workspace(&[package("demo", &[], &[("main.lit", main)])]);
    insta::assert_debug_snapshot!(render(&db, ws));
}

#[test]
fn transitive_imports_follow_the_acyclic_graph() {
    // main -> a -> b -> a: the cycle is broken once; `main` still sees
    // `b`'s declarations transitively through `a`.
    let a = "module a\nimport b\ndefault da when true => 1\n";
    let b = "module b\nimport a\nimport c\ndefault db when true => 2\n";
    let c = "module c\ndefault dc when true => 3\n";
    let main = "module main\nimport a\nstart s\nterminal s: \"S\"\n";
    let (db, ws) = workspace(&[package(
        "demo",
        &[],
        &[("main.lit", main), ("a.lit", a), ("b.lit", b), ("c.lit", c)],
    )]);
    insta::assert_debug_snapshot!(render(&db, ws));
}
