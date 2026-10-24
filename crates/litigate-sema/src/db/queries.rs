//! Tracked queries, from parse to package HIR. Each is keyed by one input
//! (or one declaration, or one world) so the salsa event log names exactly
//! which file, declaration or package re-ran.
//!
//! Granularity follows design review §4.1: [`syntax`] and [`item_tree`]
//! are per file and position-independent ([`anchors`] is their side
//! table), [`signature`] per file, typing per declaration
//! ([`check_decl`]), and resolution per (package, world)
//! ([`crate::precedence::world`]).
use super::ids::DeclId;
use super::inputs::{PackageInput, SourceFile, Workspace};
use super::parse::{self, tree::dotted, Parsed};
use super::span::{Anchors, Report, SpanMap};
use crate::pkg::manifest::{parse_manifest, ParsedManifest};
use crate::resolve::graph::{self, ModuleGraph};
use crate::resolve::sig::{self, Signature};
use crate::types::build::{self, DeclCheck, ModuleCheck, ModuleContext, PackageCheck};
use parse::tree::Item;

/// The file's lossless CST and L0xxx errors (shared with the LSP).
#[salsa::tracked(returns(ref))]
pub fn syntax(db: &dyn salsa::Database, file: SourceFile) -> litigate_syntax::Parse {
    litigate_syntax::parse_lit(file.text(db))
}

/// The file's item tree, lowered from [`syntax`]: position-independent, so
/// an edit that moves declarations without changing them is backdated.
#[salsa::tracked(returns(ref))]
pub fn item_tree(db: &dyn salsa::Database, file: SourceFile) -> Parsed {
    parse::lower(syntax(db, file))
}

/// Where the file's top-level nodes start: the side table that locates
/// [`item_tree`]'s anchored ranges.
#[salsa::tracked(returns(ref))]
pub fn anchors(db: &dyn salsa::Database, file: SourceFile) -> Anchors {
    parse::anchors(syntax(db, file))
}

/// The file's positions: its text's line index and [`anchors`].
#[must_use]
pub fn span_map(db: &dyn salsa::Database, file: SourceFile) -> SpanMap {
    SpanMap::new(file.text(db), anchors(db, file).clone())
}

/// Module name and imported module paths (no ranges, so body edits backdate).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Header {
    /// Dotted module name.
    pub module: String,
    /// Dotted import paths in order.
    pub imports: Vec<String>,
}

/// The file's module header.
#[salsa::tracked(returns(ref))]
pub fn module_header(db: &dyn salsa::Database, file: SourceFile) -> Header {
    let tree = &item_tree(db, file).tree;
    Header {
        module: dotted(&tree.module),
        imports: tree.imports.iter().map(|i| dotted(&i.path)).collect(),
    }
}

/// The validated manifest, or its L1004 reports.
#[salsa::tracked(returns(ref))]
pub fn manifest(
    db: &dyn salsa::Database,
    pkg: PackageInput,
) -> Result<ParsedManifest, Vec<Report>> {
    parse_manifest(pkg.manifest_text(db))
}

/// Every package's dependency resolution, in load order (SPEC §4.2).
#[salsa::tracked(returns(ref))]
pub fn resolutions(db: &dyn salsa::Database, ws: Workspace) -> Vec<super::resolution::Resolution> {
    super::resolution::build(db, ws)
}

/// The workspace module graph.
#[salsa::tracked(returns(ref))]
pub fn module_graph(db: &dyn salsa::Database, ws: Workspace) -> ModuleGraph {
    graph::build(db, ws)
}

/// The module's exported signature (names, kinds, resolved types).
#[salsa::tracked(returns(ref))]
pub fn signature(db: &dyn salsa::Database, file: SourceFile) -> Signature {
    sig::build(db, file)
}

/// The declaration's item, anchored: backdated while the declaration's
/// own text is unchanged, wherever it moves.
#[salsa::tracked(returns(ref))]
pub fn decl_item<'db>(db: &'db dyn salsa::Database, decl: DeclId<'db>) -> Option<Item> {
    item_tree(db, decl.file(db))
        .tree
        .items
        .get(decl.index(db) as usize)
        .cloned()
}

/// What every declaration of the module sees of the rest of it.
#[salsa::tracked(returns(ref))]
pub fn module_context_of(db: &dyn salsa::Database, file: SourceFile) -> ModuleContext {
    build::module_context(&item_tree(db, file).tree)
}

/// Name resolution, type checking, and position-free HIR for one top-level
/// declaration (design review §4.1: typing per declaration).
#[salsa::tracked(returns(ref))]
pub fn check_decl<'db>(db: &'db dyn salsa::Database, decl: DeclId<'db>) -> DeclCheck {
    build::check_decl_in(db, decl)
}

/// Name resolution, type checking, and HIR for one module.
#[salsa::tracked(returns(ref))]
pub fn check_module(db: &dyn salsa::Database, file: SourceFile) -> ModuleCheck {
    build::check_module(db, file)
}

/// Package-level checks and the package HIR.
#[salsa::tracked(returns(ref))]
pub fn check_package(db: &dyn salsa::Database, pkg: PackageInput) -> PackageCheck {
    build::check_package(db, pkg)
}
