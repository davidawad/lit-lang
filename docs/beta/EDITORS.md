# Editor verification (lg-zmqc.6)

Verified from this checkout on 2026-11-02. The editor modes are source
packages in `editors/`; the language server is provided by the installed
`litigate` and `litigate-lsp` binaries. Install both with
`cargo install --path crates/litigate-cli` and
`cargo install --path crates/litigate-lsp`, as described in
[`editors/README.md`](../../editors/README.md#install-the-compiler-and-language-server).

| Component | Install/setup instructions | Verification |
|---|---|---|
| Emacs | `editors/README.md`, Emacs section: load `editors/emacs` and install/load the tree-sitter grammar. | `cargo test -p litigate-editors` passed; Emacs 30.1 ERT opened/highlighted repository `.lit` and `.case` files. |
| Helix | `editors/README.md`, Helix section: merge the language config, copy queries, then fetch/build the grammar. | Not run in pod: `hx` is not installed. Grammar corpus and Helix queries passed in `cargo test -p litigate-editors`. |
| Neovim | `editors/README.md`, Neovim section: add `editors/nvim` to runtimepath and install/build the parser. | Not run in pod: `nvim` is not installed. Grammar corpus and Neovim query checks passed in `cargo test -p litigate-editors`. |
| tree-sitter-litigate | `editors/tree-sitter-litigate/README.md`: checked-in parser and grammar; use the CLI to regenerate and test corpus when changing it. | Rust grammar/query tests passed in `cargo test -p litigate-editors`; no `tree-sitter` CLI is installed. |
| litigate-editors | Maintainer tooling crate: `cargo run -p litigate-editors --bin litigate-grammar` builds an Emacs parser; `editor-queries` generates Helix queries. Not an end-user editor mode. | `cargo test -p litigate-editors` passed (17 unit/integration tests). |

## LSP protocol smoke test

Verified against release binaries built from this checkout: a scripted
stdio session through `litigate lsp --stdio` completed initialize/shutdown
and received parse diagnostics (`L0001`) for malformed `.lit` and `.case`
buffers, with 1 and 2 diagnostics respectively. The integration session used
`.lit` language id `litigate`, `.case` id `litigate-case`, and loose files in
a scratch workspace.

DECIDED: use `litigate lsp --stdio` in all examples -- the CLI launcher is
the common installed-binary entry point and locates its sibling server or
falls back to `PATH`.
