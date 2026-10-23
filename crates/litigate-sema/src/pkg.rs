//! Packages: `Lit.toml`, versions, digests, `Lit.lock`, and loading package
//! trees from disk or memory (SPEC §4.1-§4.2).
pub mod compat;
pub mod load;
pub mod lock;
pub mod manifest;
pub mod resolve;
pub mod semver;
pub mod source;
