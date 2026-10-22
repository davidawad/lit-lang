//! Incrementality: editing one module re-checks only that module and its
//! dependents, observed through salsa's `WillExecute` event log.
mod common;
use litigate_sema::db::inputs::create_workspace;
use litigate_sema::db::queries::check_package;
use litigate_sema::{LitDatabase, SourceFile};
use salsa::plumbing::AsId;
use salsa::Setter;

const A: &str = "module a\nstart s\nstate s: \"S\" { _ \"go\" -> end }\nterminal end: \"End\"\n";
const B: &str = "module b\nimport a.{end}\nstate b1: \"B1\" { _ \"go\" -> end }\n";
const C: &str = "module c\nstate c1: \"C1\" { _ \"go\" -> c1 }\n";

fn ran(log: &[String], query: &str, file: SourceFile) -> bool {
    let key = format!("{query}({:?})", file.as_id());
    log.iter().any(|e| *e == key)
}

#[test]
fn edits_recheck_only_the_module_and_its_dependents() {
    let (mut db, _) = LitDatabase::with_event_log();
    let ws = create_workspace(
        &mut db,
        &[common::package(
            "demo",
            &[],
            &[("a.lit", A), ("b.lit", B), ("c.lit", C)],
        )],
    );
    let pkg = ws.packages(&db)[0];
    let files: Vec<SourceFile> = pkg.files(&db).clone();
    let (a, b, c) = (files[0], files[1], files[2]);
    assert!(
        check_package(&db, pkg).diagnostics.is_empty(),
        "{:?}",
        check_package(&db, pkg).diagnostics
    );
    let first = db.take_log();
    assert!([a, b, c].iter().all(|f| ran(&first, "check_module", *f)));

    // A body-only edit (a label) keeps a's signature: b and c are not re-checked.
    a.set_text(&mut db).to(A.replace("\"End\"", "\"The end\""));
    assert!(check_package(&db, pkg).diagnostics.is_empty());
    let log = db.take_log();
    assert!(
        ran(&log, "parse_file", a) && ran(&log, "signature", a) && ran(&log, "check_module", a)
    );
    assert!(
        !ran(&log, "check_module", b) && !ran(&log, "check_module", c),
        "{log:?}"
    );
    assert!(
        !ran(&log, "module_graph", a) && !log.iter().any(|e| e.starts_with("module_graph")),
        "{log:?}"
    );

    // An interface edit (a new node) re-checks the dependent b, never c.
    a.set_text(&mut db).to(format!("{A}state extra: \"X\"\n"));
    let _ = check_package(&db, pkg);
    let log = db.take_log();
    assert!(
        ran(&log, "check_module", a) && ran(&log, "check_module", b),
        "{log:?}"
    );
    assert!(
        !ran(&log, "check_module", c) && !ran(&log, "parse_file", c),
        "{log:?}"
    );

    // Editing c touches only c.
    c.set_text(&mut db).to(C.replace("C1", "C one"));
    let _ = check_package(&db, pkg);
    let log = db.take_log();
    assert!(ran(&log, "check_module", c));
    assert!(
        !ran(&log, "check_module", a) && !ran(&log, "check_module", b),
        "{log:?}"
    );
}

#[test]
fn breaking_an_import_is_reported_and_repairing_it_clears_it() {
    let mut db = LitDatabase::default();
    let ws = create_workspace(
        &mut db,
        &[common::package("demo", &[], &[("a.lit", A), ("b.lit", B)])],
    );
    let pkg = ws.packages(&db)[0];
    let a = pkg.files(&db)[0];
    a.set_text(&mut db).to(A.replace("end", "finish"));
    let codes: Vec<String> = check_package(&db, pkg)
        .diagnostics
        .iter()
        .map(|d| format!("{} {}", d.code.as_str(), d.message))
        .collect();
    assert_eq!(codes, ["L1002 module `a` declares no `end`"]);
    a.set_text(&mut db).to(A.to_owned());
    assert!(check_package(&db, pkg).diagnostics.is_empty());
}
