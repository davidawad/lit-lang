//! Tracked queries, from parse to package HIR. Each is keyed by one input so
//! the salsa event log names exactly which file or package re-ran.
use super::inputs::{PackageInput, SourceFile, Workspace};
use super::parse::{self, tree::dotted, Parsed};
use super::span::Report;
use crate::pkg::manifest::{parse_manifest, ParsedManifest};
use crate::resolve::graph::{self, ModuleGraph};
use crate::resolve::sig::{self, Signature};
use crate::types::build::{self, ModuleCheck, PackageCheck};

/// Parses a file into its item tree.
#[salsa::tracked(returns(ref))]
pub fn parse_file(db: &dyn salsa::Database, file: SourceFile) -> Parsed {
    parse::parse(file.text(db))
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
    let tree = &parse_file(db, file).tree;
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
