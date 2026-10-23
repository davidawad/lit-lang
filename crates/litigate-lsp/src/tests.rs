use super::*;
use litigate_base::Profile;
use litigate_cli::ws::Session;
use litigate_syntax::TextRange;
use std::path::Path;

/// An analysis of one loose document `loose.lit` at `root`.
fn loose(root: &Path) -> Analysis {
    let checked =
        litigate_cli::check_session(Session::from_loaded(&[], Vec::new()), Profile::Dev, None);
    let open = std::iter::once((root.join("loose.lit"), "module loose\n".to_owned())).collect();
    Analysis::new(root.to_path_buf(), checked, open)
}

#[test]
fn a_location_needs_a_text_and_an_absolute_root() {
    let range = TextRange::new(7.into(), 12.into());
    let root = std::env::temp_dir().join(format!("litigate-lsp-location-{}", std::process::id()));
    let placed = loose(&root);
    let here = Server::location(&placed, "loose.lit", range).map(|l| l.range);
    assert_eq!(
        here,
        Some(tower_lsp::lsp_types::Range::new(
            tower_lsp::lsp_types::Position::new(0, 7),
            tower_lsp::lsp_types::Position::new(0, 12),
        ))
    );
    assert!(Server::location(&placed, "gone.lit", range).is_none());
    assert!(Server::location(&loose(Path::new("")), "loose.lit", range).is_none());
}

#[test]
fn a_definition_needs_a_text() {
    let root = std::env::temp_dir().join(format!("litigate-lsp-definition-{}", std::process::id()));
    let a = loose(&root);
    let start = tower_lsp::lsp_types::Position::new(0, 8);
    assert_eq!(Server::definition_at(&a, "loose.lit", start), Some(0));
    assert_eq!(Server::definition_at(&a, "gone.lit", start), None);
}
