//! Resolution per world (design review §4.1, §6 item 25): the world is an
//! interned query key, resolution is memoized per (package, world) and read
//! per (declaration, world), and nothing world-independent re-runs when the
//! world changes.
mod legal;
use legal::{pkg, PRELUDE};
use litigate_sema::db::ids::{DeclId, WorldId};
use litigate_sema::db::inputs::create_workspace;
use litigate_sema::db::queries::{check_module, item_tree};
use litigate_sema::precedence::worlds::{resolve_in, world_selection, Resolution};
use litigate_sema::{LitDatabase, PackageInput, SourceFile};
use salsa::plumbing::AsId;

const VERSIONS: &str =
    "deadline p: 20.days from Served(complaint) counted_by c6 effective [2009-12-01, 2016-12-01)
deadline p: 21.days from Served(complaint) counted_by c6 effective [2016-12-01, ..)
";

fn demo(extra: &str) -> (LitDatabase, PackageInput, SourceFile) {
    let (mut db, _) = LitDatabase::with_event_log();
    let text = format!("{PRELUDE}{VERSIONS}{extra}");
    let ws = create_workspace(
        &mut db,
        &[pkg(
            "demo",
            "FederalRule",
            "2009-12-01",
            &[],
            &[("main.lit", &text)],
        )],
    );
    let pkg = ws.packages(&db)[0];
    let file = pkg.files(&db)[0];
    (db, pkg, file)
}

/// The index of the first top-level item named `name`.
fn index_of(db: &LitDatabase, file: SourceFile, name: &str) -> u32 {
    let tree = &item_tree(db, file).tree;
    let i = tree
        .items
        .iter()
        .position(|i| litigate_sema::types::build::item_name(i).is_some_and(|n| n.text == name))
        .unwrap_or(usize::MAX);
    u32::try_from(i).unwrap_or(u32::MAX)
}

fn governed_from(r: Option<&Resolution>) -> Option<&str> {
    match r {
        Some(Resolution::Governed { from, .. }) => Some(from),
        _ => None,
    }
}

#[test]
fn each_world_selects_the_version_in_force() {
    let (db, pkg, file) = demo("");
    assert!(
        check_module(&db, file).module.is_some(),
        "{:?}",
        check_module(&db, file).diagnostics
    );
    let old = WorldId::new(&db, "2012-01-01".to_owned(), None);
    let new = WorldId::new(&db, "2020-01-01".to_owned(), None);
    let before = WorldId::new(&db, "2001-01-01".to_owned(), None);
    assert_eq!(
        governed_from(world_selection(&db, pkg, old).get("p")),
        Some("2009-12-01")
    );
    assert_eq!(
        governed_from(world_selection(&db, pkg, new).get("p")),
        Some("2016-12-01")
    );
    match world_selection(&db, pkg, before).get("p") {
        Some(Resolution::Unselected(why)) => {
            assert_eq!(why, "no version of `p` is in force on 2001-01-01");
        }
        other => panic!("{other:?}"),
    }
    let Some(Resolution::Governed { pack, id, span, .. }) = world_selection(&db, pkg, new).get("p")
    else {
        panic!("p governs");
    };
    assert_eq!((pack.as_str(), id.as_str()), ("demo", "p"));
    assert_eq!(span.as_ref().map(|s| s.line), Some(14));
    // Read per declaration: the second `p` resolves like the first; an
    // enum is not rule-bearing, and an index past the items has no item.
    let second = DeclId::new(&db, file, index_of(&db, file, "p") + 1);
    assert_eq!(
        governed_from(resolve_in(&db, second, old).as_ref()),
        Some("2009-12-01")
    );
    let document = DeclId::new(&db, file, index_of(&db, file, "Document"));
    assert_eq!(resolve_in(&db, document, old), &None);
    assert_eq!(resolve_in(&db, DeclId::new(&db, file, 999), old), &None);
}

#[test]
fn a_new_world_reruns_only_resolution() {
    let (db, pkg, file) = demo("");
    let p = DeclId::new(&db, file, index_of(&db, file, "p"));
    let s = DeclId::new(&db, file, index_of(&db, file, "s"));
    let old = WorldId::new(&db, "2012-01-01".to_owned(), None);
    let _ = (resolve_in(&db, p, old), resolve_in(&db, s, old));
    let _ = db.take_log();
    // A corpus from before either version was recorded (2025-01-01).
    let corpus = WorldId::new(&db, "2020-01-01".to_owned(), Some("2010-01-01".to_owned()));
    assert!(matches!(
        resolve_in(&db, p, corpus),
        Some(Resolution::Unselected(_))
    ));
    let _ = resolve_in(&db, s, corpus);
    let log = db.take_log();
    let ran = |q: &str, id: salsa::Id| log.contains(&format!("{q}({id:?})"));
    assert!(
        log.iter().any(|e| e.starts_with("world_selection(")),
        "{log:?}"
    );
    for q in ["check_module", "item_tree", "signature"] {
        assert!(!ran(q, file.as_id()), "{q} re-ran: {log:?}");
    }
    assert!(!log.iter().any(|e| e.starts_with("check_decl(")), "{log:?}");
    assert!(
        !log.iter().any(|e| e.starts_with("check_package(")),
        "{log:?}"
    );
    assert_eq!(
        world_selection(&db, pkg, corpus).len(),
        5,
        "c6, cal, s, t, p"
    );
}

#[test]
fn a_package_without_a_valid_manifest_resolves_nothing() {
    let mut db = LitDatabase::default();
    let mut broken = pkg(
        "demo",
        "FederalRule",
        "2009-12-01",
        &[],
        &[("main.lit", PRELUDE)],
    );
    broken.manifest = "not toml".into();
    let ws = create_workspace(&mut db, &[broken]);
    let world = WorldId::new(&db, "2020-01-01".to_owned(), None);
    assert!(world_selection(&db, ws.packages(&db)[0], world).is_empty());
}

#[test]
fn a_claim_on_a_declaration_governs_it() {
    let (db, pkg, _) = demo(
        "deadline q: 14.days from Served(complaint) counted_by c6 effective [2012-01-01, ..) notwithstanding p\n",
    );
    let world = WorldId::new(&db, "2020-01-01".to_owned(), None);
    let Some(Resolution::Governed { id, from, .. }) = world_selection(&db, pkg, world).get("p")
    else {
        panic!("p is governed");
    };
    assert_eq!((id.as_str(), from.as_str()), ("q", "2012-01-01"));
}
