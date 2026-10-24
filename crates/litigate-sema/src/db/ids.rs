//! Interned query keys: a declaration ([`DeclId`]) and a world
//! ([`WorldId`]), so per-declaration and per-world queries are memoized
//! per key (design review §4.1).
use super::inputs::SourceFile;

/// One top-level declaration of a module: the file and its position among
/// the file's items.
#[salsa::interned(debug)]
pub struct DeclId<'db> {
    /// The declaring file.
    #[returns(copy)]
    pub file: SourceFile,
    /// Index into the item tree's top-level items.
    #[returns(copy)]
    pub index: u32,
}

/// A world as a query coordinate (design review §3.2, §4.1): law in force
/// on `as_of` (valid time) among the declarations recorded by `known_at`
/// (transaction time; `None`: the latest corpus). The package a query
/// names supplies the layers: itself, its dependencies and `core`.
#[salsa::interned(debug)]
pub struct WorldId<'db> {
    /// Valid time, `YYYY-MM-DD`.
    #[returns(ref)]
    pub as_of: String,
    /// Transaction time, `YYYY-MM-DD`.
    #[returns(ref)]
    pub known_at: Option<String>,
}
