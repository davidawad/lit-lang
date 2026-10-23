//! Document and workspace symbols over in-memory workspaces: containers,
//! files the analysis lacks, and roots a URI cannot be made from.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod support;

use litigate_base::Profile;
use litigate_cli::ws::Session;
use litigate_lsp::analysis::Analysis;
use litigate_lsp::symbols::{document, workspace};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use tower_lsp::lsp_types::SymbolKind;

#[test]
fn variants_are_contained_by_their_enum() {
    let root = support::worked("symbols-variants", &[]);
    let mut host = support::host(&root);
    let a = host.analyze();
    let found: Vec<(String, SymbolKind, Option<String>)> = workspace(a, "COMPLAINT")
        .into_iter()
        .map(|s| (s.name, s.kind, s.container_name))
        .collect();
    assert_eq!(
        found,
        [
            (
                "complaint".into(),
                SymbolKind::ENUM_MEMBER,
                Some("Document".into())
            ),
            (
                "amended_complaint".into(),
                SymbolKind::ENUM_MEMBER,
                Some("Document".into())
            ),
        ]
    );
    assert!(document(a, "lit/ex/gone.lit").is_empty());
}

#[test]
fn without_an_absolute_root_no_symbol_has_a_location() {
    let checked = litigate_cli::check_session(
        Session::from_loaded(&[], Vec::new()),
        Profile::Dev,
        &litigate_lower::Options::default(),
    );
    let open: BTreeMap<PathBuf, String> = std::iter::once((
        PathBuf::from("loose.lit"),
        "module loose\nstart s\nstate s: \"S\"\n".to_owned(),
    ))
    .collect();
    let a = Analysis::new(Path::new("").to_path_buf(), checked, open);
    assert_eq!(a.index.defs.len(), 2, "the module and its state");
    assert!(workspace(&a, "").is_empty());
    assert_eq!(
        document(&a, "loose.lit").len(),
        2,
        "a document needs no URI"
    );
}
