//! Language-server boundary; handlers will share the compiler database.
/// Marker for the tower-lsp service, implemented by the LSP bead.
#[derive(Debug, Default, Clone, Copy)]
pub struct Server;
