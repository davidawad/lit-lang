//! The position-independent item tree (design review §4.1, §6 item 25):
//! ranges are anchored at their top-level declaration, typing runs per
//! declaration, and HIR is located only when its module assembles it. An
//! edit that only moves declarations re-types none of them, and every
//! located result equals a check from scratch.
mod common;
use litigate_base::FileId;
use litigate_sema::db::ids::DeclId;
use litigate_sema::db::inputs::create_workspace;
use litigate_sema::db::queries::{check_decl, check_module, item_tree, span_map};
use litigate_sema::db::span::{Anchors, Encoded, Positions, SpanMap, TextRange};
use litigate_sema::hir::spans::Spans;
use litigate_sema::types::build::{check_decl_at, DeclCheck};
use litigate_sema::{LitDatabase, SourceFile};
use salsa::plumbing::AsId;
use salsa::Setter;
use std::path::{Path, PathBuf};

const A: &str = "module a
start s
/// The first state.
state s: \"S\" { _ \"go\" -> end }
terminal end: \"End\"
deadline d: 21.days from Served(complaint) counted_by c6
";

fn one(text: &str) -> (LitDatabase, SourceFile) {
    let (mut db, _) = LitDatabase::with_event_log();
    let ws = create_workspace(&mut db, &[common::package("demo", &[], &[("a.lit", text)])]);
    let file = ws.packages(&db)[0].files(&db)[0];
    (db, file)
}

fn ran(log: &[String], query: &str, id: impl std::fmt::Debug) -> bool {
    log.contains(&format!("{query}({id:?})"))
}

fn decls(db: &LitDatabase, file: SourceFile) -> Vec<DeclId<'_>> {
    let n = item_tree(db, file).tree.items.len();
    (0..n)
        .map(|i| DeclId::new(db, file, u32::try_from(i).unwrap_or(u32::MAX)))
        .collect()
}

#[test]
fn moving_declarations_retypes_none_of_them() {
    let (mut db, file) = one(A);
    let first = check_module(&db, file).clone();
    assert!(!first.diagnostics.is_empty(), "the unknown counting rule");
    let _ = db.take_log();
    // Blank lines, a comment, and reindentation above every declaration.
    let moved = A
        .replace("start s\n", "\n\n// moved\nstart s\n")
        .replace("terminal end", "\n   terminal end");
    file.set_text(&mut db).to(moved.clone());
    let again = check_module(&db, file).clone();
    let log = db.take_log();
    let file_id = file.as_id();
    assert!(ran(&log, "item_tree", file_id) && ran(&log, "anchors", file_id));
    assert!(ran(&log, "check_module", file_id), "{log:?}");
    for decl in decls(&db, file) {
        assert!(!ran(&log, "check_decl", decl.as_id()), "{log:?}");
        assert!(!ran(&log, "decl_item", decl.as_id()), "{log:?}");
    }
    for query in ["signature", "module_header", "module_context_of"] {
        assert!(!ran(&log, query, file_id), "{query} re-ran: {log:?}");
    }
    // Located from the same position-free results, the module equals a
    // check from scratch of the moved text, and its spans moved.
    let (fresh, fresh_file) = one(&moved);
    assert_eq!(&again, check_module(&fresh, fresh_file));
    let lines = |m: &litigate_sema::ModuleCheck| -> Vec<u32> {
        m.diagnostics.iter().map(|d| d.span.line).collect()
    };
    assert_eq!(
        lines(&again),
        lines(&first).iter().map(|l| l + 4).collect::<Vec<_>>()
    );
    assert_eq!(first.starts[0].1.line + 3, again.starts[0].1.line);
}

#[test]
fn editing_one_declaration_retypes_only_it() {
    let (mut db, file) = one(A);
    let _ = check_module(&db, file);
    let _ = db.take_log();
    file.set_text(&mut db)
        .to(A.replace("\"End\"", "\"The end\""));
    let _ = check_module(&db, file);
    let log = db.take_log();
    let ids = decls(&db, file);
    let retyped: Vec<bool> = ids
        .iter()
        .map(|d| ran(&log, "check_decl", d.as_id()))
        .collect();
    assert_eq!(retyped, [false, false, true, false], "{log:?}");
    // An index past the items (a declaration since deleted) checks to
    // nothing.
    let gone = DeclId::new(&db, file, 99);
    assert_eq!(check_decl(&db, gone), &DeclCheck::default());
}

#[test]
fn anchors_place_ranges_and_spans() {
    let anchors = Anchors(vec![10, 20]);
    let early = TextRange::new(2, 4);
    assert_eq!(anchors.anchor(early), early);
    let anchored = anchors.anchor(TextRange::new(23, 25));
    assert_eq!((anchored.anchor, anchored.start, anchored.end), (2, 3, 5));
    assert_eq!(anchors.absolute(anchored), TextRange::new(23, 25));
    // An anchor the table lacks counts from the start of the file.
    let stray = TextRange {
        anchor: 9,
        ..anchored
    };
    assert_eq!(anchors.absolute(stray), TextRange::new(3, 5));
    assert_eq!(
        anchored.at_last(),
        TextRange {
            start: 4,
            end: 4,
            ..anchored
        }
    );
    assert_eq!(
        TextRange::new(0, 0).at_last(),
        TextRange::new(0, 0),
        "an empty range at the start stays put"
    );

    let text = "module m\nstart s\n";
    let map = SpanMap::new(text, Anchors(vec![0, 9]));
    let file = FileId("m.lit".into());
    let start = TextRange {
        anchor: 2,
        start: 6,
        end: 7,
    };
    let mut encoded = Encoded.span(&file, start);
    assert_eq!((encoded.line, encoded.col, encoded.end_col), (2, 6, 7));
    map.decode(&mut encoded);
    let located = map.span(&file, start);
    assert_eq!((located.line, located.col), (2, 7));
    assert_eq!(encoded, located);
}

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Every declaration of the whole `lit/` corpus, built position-free and
/// then located, equals the same declaration built where it is: the span
/// visitor reaches every span HIR has.
#[test]
fn located_hir_equals_hir_built_in_place() -> std::io::Result<()> {
    let repo = repo();
    let mut roots: Vec<PathBuf> = std::fs::read_dir(repo.join("lit"))?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.join("Lit.toml").is_file())
        .collect();
    roots.sort();
    let (db, ws) = litigate_sema::load(&repo, &roots, None)?;
    let mut spans = 0;
    for pkg in ws.packages(&db) {
        for &file in pkg.files(&db) {
            let map = span_map(&db, file);
            let mut assembled = Vec::new();
            for decl in decls(&db, file) {
                let reference = check_decl_at(&db, decl, &map);
                let mut located = check_decl(&db, decl).declarations.clone();
                located.spans_mut(&mut |s| map.decode(s));
                located.spans_mut(&mut |_| spans += 1);
                assert_eq!(located, reference.declarations, "{}", file.path(&db));
                assembled.extend(located);
            }
            if let Some(module) = &check_module(&db, file).module {
                assert_eq!(module.declarations, assembled);
            }
        }
    }
    assert!(spans > 1000, "{spans} spans");
    Ok(())
}
