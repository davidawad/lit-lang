//! Modules, imports, packages, and names (SPEC §4) on real `.lit` sources.
mod common;
use common::{package, render, workspace};

const ENUMS: &str = "module shared.types\nenum Document { complaint }\nevent Served(document: Document)\ngroup motions: \"Motions\"\n";

#[test]
fn imports_resolve_names_and_qualified_paths() {
    let main = "module main\nimport shared.types.{Document, Served, motions}\nimport shared.nodes\nstart s\nstate s: \"S\" { group motions, _ \"go\" -> shared.nodes.end }\ndefault d when has_event(Served(complaint)) => 1\n";
    let nodes = "module shared.nodes\nterminal end: \"End\"\n";
    let (db, ws) = workspace(&[package(
        "demo",
        &[],
        &[
            ("main.lit", main),
            ("types.lit", ENUMS),
            ("nodes.lit", nodes),
        ],
    )]);
    assert_eq!(render(&db, ws), Vec::<String>::new());
}

#[test]
fn module_level_errors() {
    let a = "module a\nimport b\nimport shared.typez\nimport shared.types.{Documnt}\nstart s\nstate s: \"S\" { _ \"go\" -> end }\nterminal end: \"E\"\n";
    let b = "module b\nimport a\n";
    let dup = "module a\n";
    let shadow = "module c\nimport shared.types.{Document}\nenum Document { x }\n";
    let (db, ws) = workspace(&[package(
        "demo",
        &[],
        &[
            ("a.lit", a),
            ("b.lit", b),
            ("dup.lit", dup),
            ("c.lit", shadow),
            ("types.lit", ENUMS),
        ],
    )]);
    insta::assert_debug_snapshot!(render(&db, ws));
}

#[test]
fn package_level_errors() {
    let one = "module one\npackage other { forum demo, authority Statute }\nstart s\nstate s: \"S\" by Judge { _ \"go\" -> s2 }\nstate s2: \"S2\"\n";
    let two = "module two\nstart s3\nstate s3: \"S3\"\nstate s2: \"again\"\n";
    let (db, ws) = workspace(&[package(
        "demo",
        &["missing"],
        &[("one.lit", one), ("two.lit", two)],
    )]);
    insta::assert_debug_snapshot!(render(&db, ws));
}

#[test]
fn cross_package_imports_need_declared_dependencies() {
    let core = "module core.time\nenum Unit { day }\n";
    let other = "module other.mod\nenum X { y }\n";
    let main = "module main\nimport core.time.{Unit}\nimport other.mod\nstart s\nstate s: \"S\"\n";
    let (db, ws) = workspace(&[
        package("demo", &["core"], &[("main.lit", main)]),
        package(
            "core",
            &[],
            &[
                ("time.lit", core),
                ("start.lit", "module core.start\nstart z\nstate z: \"Z\"\n"),
            ],
        ),
        package(
            "other",
            &[],
            &[
                ("mod.lit", other),
                ("start.lit", "module other.start\nstart z\nstate z: \"Z\"\n"),
            ],
        ),
    ]);
    insta::assert_debug_snapshot!(render(&db, ws));
}

#[test]
fn versions_share_a_name_and_select_by_date() {
    let src = "module main\nstart s\nenum D { c }\nevent E(d: D)\ncounting k { holidays cal }\ncalendar cal { x: once(2020-01-01) effective [2020-01-01, ..) }\n\
deadline p: 20.days from E(c) counted_by k effective [2000-01-01, 2009-12-01)\n\
deadline p: 21.days from E(c) counted_by k effective [2009-12-01, ..)\n\
deadline q: 1.days from E(c) counted_by k\ndeadline q: 2.days from E(c) counted_by k\n\
transition t amends p@2009-12-01 { unless has_event(E(c)) -> p@2000-01-01, otherwise -> p@2009-12-01 }\n\
transition u amends p@1999-01-01 { otherwise -> p }\n\
state s: \"S\" { a \"A\" -> s2 { within p } }\nstate s2: \"S2\"\n";
    let (db, ws) = workspace(&[package("demo", &[], &[("main.lit", src)])]);
    insta::assert_debug_snapshot!(render(&db, ws));
}

#[test]
fn unresolved_names_suggest_and_split_hyphens() {
    let src = "module main\nstart s\nstate s: \"S\" { _ \"go\" -> s3 }\nstate s2: \"S2\"\n\
default a when true => 1\ndefault b when true => 2\ndefault c when true => a-b\ndefault e when true => aa\n\
type T = Mony<usd>\ntype U = Money<USD>\ntype V = V\n";
    let (db, ws) = workspace(&[package("demo", &[], &[("main.lit", src)])]);
    insta::assert_debug_snapshot!(render(&db, ws));
}

#[test]
fn links_packages_resolve_qualified_names() {
    let base = "module base\nstart entry\nstate entry: \"E\" { go \"Go\" -> done }\nterminal done: \"D\"\n";
    let links = "module links\nlink appeal \"Appeal\": base::done -> base@copy::entry p 1/2 { replaces [base::go] }\n\
instance base @ copy { pack base, probabilities { go: 1/3 }, roles { Court: nature }, patch go { note \"n\" } }\n\
link bad \"Bad\": base::nowhere -> nope::entry { replaces [base::gone] }\n";
    let (db, ws) = workspace(&[
        package("base", &[], &[("base.lit", base)]),
        package("links", &["base"], &[("links.lit", links)]),
    ]);
    insta::assert_debug_snapshot!(render(&db, ws));
}

#[test]
fn failed_imports_report_once_without_cascades() {
    let main = "module main\nimport shared.types.{Documnt}\nimport shared.nodez\nstart s\nstate s: \"S\" { _ \"go\" -> shared.nodez.end }\ndefault d when has_event(Served(Documnt.complaint)) => 1\ndefault e when true => match Document.complaint { complaint => 1 }\n";
    let (db, ws) = workspace(&[package(
        "demo",
        &[],
        &[("main.lit", main), ("types.lit", ENUMS)],
    )]);
    let out = render(&db, ws);
    // One report per failed import; uses of `Documnt` and `shared.nodez.end`
    // add nothing. Unrelated unresolved names (`Served`, `Document` were not
    // imported) still report.
    assert_eq!(
        out.iter().filter(|l| l.contains("main.lit:2:")).count(),
        1,
        "{out:#?}"
    );
    assert_eq!(
        out.iter().filter(|l| l.contains("main.lit:3:")).count(),
        1,
        "{out:#?}"
    );
    assert!(!out.iter().any(|l| l.contains("main.lit:5:")), "{out:#?}");
    assert!(out.iter().any(|l| l.contains("`Served`")), "{out:#?}");
    insta::assert_debug_snapshot!(out);
}
