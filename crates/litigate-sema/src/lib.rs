//! Semantic analysis for litigate: the salsa database, packages and
//! lockfiles, name resolution, type checking, and HIR (SPEC §4-§6, §11).
pub mod case;
pub mod db;
pub mod guard;
pub mod hir;
pub mod pkg;
pub mod precedence;
pub mod resolve;
pub mod temporal;
pub mod types;

pub use db::inputs::{PackageInput, SourceFile, Workspace};
pub use db::LitDatabase;
pub use types::build::{Hole, ModuleCheck, PackageCheck};

use std::path::{Path, PathBuf};

/// Loads `roots` (package directories), their path dependencies, and the
/// optional stdlib into a fresh database.
///
/// # Errors
/// I/O errors reading package files.
pub fn load(
    repo_root: &Path,
    roots: &[PathBuf],
    stdlib: Option<&Path>,
) -> std::io::Result<(LitDatabase, Workspace)> {
    load_in(&pkg::source::Disk, repo_root, roots, stdlib)
}

/// [`load`] from `src`.
///
/// # Errors
/// I/O errors reading package files.
pub fn load_in(
    src: &dyn pkg::source::Source,
    repo_root: &Path,
    roots: &[PathBuf],
    stdlib: Option<&Path>,
) -> std::io::Result<(LitDatabase, Workspace)> {
    let packages = pkg::load::load_tree_in(src, repo_root, roots, stdlib)?;
    let mut db = LitDatabase::default();
    let ws = db::inputs::create_workspace(&mut db, &packages);
    Ok((db, ws))
}

/// Checks every package of the workspace, in load order.
#[must_use]
pub fn check_workspace(db: &dyn salsa::Database, ws: Workspace) -> Vec<&PackageCheck> {
    ws.packages(db)
        .iter()
        .map(|p| db::queries::check_package(db, *p))
        .collect()
}

/// Verifies `pkg`'s `Lit.lock` against the resolved package graph (L1007).
#[must_use]
pub fn lock_diagnostics(
    db: &dyn salsa::Database,
    ws: Workspace,
    pkg: PackageInput,
) -> Vec<litigate_base::Diagnostic> {
    let expected = types::build::expected_lock(db, ws, pkg);
    let file = litigate_base::FileId(
        format!("{}/Lit.lock", pkg.root(db))
            .trim_start_matches("./")
            .to_owned(),
    );
    let text = pkg.lock_text(db).clone().unwrap_or_default();
    pkg::lock::verify(pkg.lock_text(db).as_deref(), &expected)
        .err()
        .map(|r| vec![r.to_diagnostic(&file, &litigate_base::LineIndex::new(&text))])
        .unwrap_or_default()
}

/// The `Lit.lock` text `litigate lock` writes for `pkg`.
#[must_use]
pub fn render_lock(db: &dyn salsa::Database, ws: Workspace, pkg: PackageInput) -> String {
    types::build::expected_lock(db, ws, pkg).render()
}
