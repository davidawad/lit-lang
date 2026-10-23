//! tuq.37: a links package split into link-only modules lowers its links
//! in module order, the order of the module files' paths (SPEC §4.1), not
//! module-name order; and the diff sees the difference.
mod support;

use litigate_base::FileId;
use litigate_lower::{semantic_diff_links, ChangeKind, Document, LinkFile, Lowerer, Options};
use litigate_sema::hir::{self, Declaration};
use support::{corpus, imported_hir, repo_root};

/// `pkg`'s one module split in two link-only modules: the first half of
/// its links in `01-first.lit` under the name that sorts last, the rest
/// (with the instances) in `02-second.lit` under the name that sorts first.
fn split(pkg: &hir::Package) -> hir::Package {
    let module = pkg.modules.first().unwrap_or_else(|| panic!("no module"));
    let links = module
        .declarations
        .iter()
        .filter(|d| matches!(d, Declaration::Link(_)))
        .count();
    let mut seen = 0;
    let (mut first, mut second) = (Vec::new(), Vec::new());
    for d in &module.declarations {
        let mut d = d.clone();
        let into_first = if let Declaration::Link(l) = &mut d {
            seen += 1;
            let file = if seen <= links / 2 {
                "01-first"
            } else {
                "02-second"
            };
            if let Some(span) = &mut l.edge.span {
                span.file = FileId(format!("lit/links/{file}.lit"));
            }
            seen <= links / 2
        } else {
            assert!(
                matches!(d, Declaration::Instance(_)),
                "imported links module holds only links and instances: {d:?}"
            );
            false
        };
        if into_first { &mut first } else { &mut second }.push(d);
    }
    let module = |name: &str, declarations| hir::Module {
        name: name.to_owned(),
        package: module.package.clone(),
        imports: module.imports.clone(),
        declarations,
    };
    hir::Package {
        modules: vec![module("links.zz", first), module("links.aa", second)],
        ..pkg.clone()
    }
}

#[test]
fn link_only_modules_lower_in_file_order() {
    let packages = imported_hir(&corpus());
    let refs: Vec<&hir::Package> = packages.iter().map(|(_, p)| p).collect();
    let (origin, links) = packages
        .iter()
        .find(|(_, p)| p.manifest.links)
        .unwrap_or_else(|| panic!("no links package"));
    let expected: LinkFile = serde_json::from_str(
        &std::fs::read_to_string(repo_root().join(origin)).unwrap_or_default(),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    let split = split(links);
    let lowered = Lowerer::new(&refs)
        .package(&split, &Options::default())
        .unwrap_or_else(|d| panic!("{d:#?}"));
    let Document::Links(file) = &lowered.document else {
        panic!("not a link file");
    };
    let changes = semantic_diff_links(&expected, file);
    assert!(
        changes.is_empty(),
        "{}",
        serde_json::to_string_pretty(&changes).unwrap_or_default()
    );

    // Name order (the old ordering) moves the second file's links first;
    // the diff reports that as one whole-file `order` change.
    let mut swapped = file.clone();
    swapped.links.rotate_left(expected.links.len() / 2);
    let changes = semantic_diff_links(&expected, &swapped);
    assert_eq!(changes.len(), 1, "{changes:?}");
    assert_eq!(changes[0].kind, ChangeKind::Order);
    assert_eq!(changes[0].pointer, "/links/order");
}
