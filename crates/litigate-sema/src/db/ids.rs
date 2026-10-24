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

