# Editor support for litigate

Editor modes for litigate rule modules (`.lit`) and matters (`.case`). They
all build on one grammar, [`tree-sitter-litigate`](tree-sitter-litigate/README.md),
and its queries (`tree-sitter-litigate/queries/*.scm`). Each one also
registers the language server, `litigate lsp --stdio` (SPEC §9).

Tooling (grammar builds, Helix query generation, the editor tests) lives in
the [`litigate-editors`](litigate-editors) crate.

## Standalone packages (install these)

Each editor's support is also published as its own repository, laid out the
way the Bend 2 packages are, so a user can install one without cloning this
repository. The grammar repo is the source of truth for editors: the others
pin its commit (`GRAMMAR_REV` in litigate.vim, `rev` in helix-litigate and in
`languages.toml`, `litigate-mode-grammar-rev` in litigate-mode.el).

| Editor | Repository | Install |
|---|---|---|
| grammar (any tree-sitter editor) | [davidawad/tree-sitter-litigate](https://github.com/davidawad/tree-sitter-litigate) | compile `src/parser.c` |
| Vim, Neovim | [davidawad/litigate.vim](https://github.com/davidawad/litigate.vim) | `Plug 'davidawad/litigate.vim', { 'branch': 'main', 'do': 'bash scripts/build-parser.sh' }` |
| Emacs 29+ | [davidawad/litigate-mode.el](https://github.com/davidawad/litigate-mode.el) | add `src/` to `load-path`, then `M-x litigate-mode-install-grammar` |
| Helix | [davidawad/helix-litigate](https://github.com/davidawad/helix-litigate) | `./install.sh`, then merge its `languages.toml` |

The directories below stay in this repository as the in-tree sources the
`litigate-editors` tests exercise; the standalone repos were seeded from them
at `6dbf033`. A change to the grammar or queries lands here and in
tree-sitter-litigate; the consumers then bump their pins. All of them
register `litigate lsp --stdio`.

| Editor | Directory | Highlight | Indent | Folds | Symbols | LSP |
|---|---|---|---|---|---|---|
| Emacs 29+ (`treesit`) | [`emacs/`](emacs) | `litigate-ts-mode` font-lock (mirrors `highlights.scm`) | `treesit-simple-indent-rules` (mirrors `indents.scm`) | outline over top-level declarations (Emacs 30+) | imenu, `beginning-of-defun` | eglot |
| Helix | [`helix/`](helix) | generated `highlights.scm` | generated `indents.scm` | (none in Helix) | `textobjects.scm`, `locals.scm` | `languages.toml` |
| Neovim 0.10+ | [`nvim/`](nvim) | `queries/` (symlinks to the canonical queries) | `indents.scm` through nvim-treesitter | `folds.scm` | `locals.scm` | `lsp/litigate.lua` (0.11+), lspconfig snippet |

> **Status.** `litigate lsp` is the SPEC §9 command. Until the
> `litigate-lsp` bead lands it, starting the server fails with "unrecognized
> subcommand". Highlighting, indentation and navigation don't need it.

**Grammar ABI.** The committed parser (`tree-sitter-litigate/src/parser.c`)
is tree-sitter ABI 15. Editors linked against libtree-sitter 0.25 or later
load it as is: Neovim 0.12+, Helix 25.07+, and Emacs built with
tree-sitter 0.25+. Older ones need an ABI 14 parser regenerated from
`grammar.js`. That takes `tree-sitter generate --abi 14`, which needs node
and the pinned `tree-sitter-cli`, run through `npx` when it isn't on
`PATH`. Each section below shows how.

## Emacs (29.1+, built with tree-sitter)

1. Build the grammar. From a checkout:

   ```sh
   cargo run -p litigate-editors --bin litigate-grammar   # -> ~/.emacs.d/tree-sitter/libtree-sitter-litigate.so
   ```

   The tool asks your `emacs` for `treesit-library-abi-version` and
   regenerates an ABI 14 parser when it needs one. `--out DIR` and `--abi N`
   override both. If your Emacs loads ABI 15 (libtree-sitter 0.25+), you can
   use `M-x treesit-install-language-grammar RET litigate` instead. The
   mode adds the source entry to `treesit-language-source-alist`.

2. Load the mode:

   ```elisp
   (add-to-list 'load-path "/path/to/litgraph/editors/emacs")
   (require 'litigate-ts-mode)
   ;; or: (use-package litigate-ts-mode :load-path "/path/to/litgraph/editors/emacs")
   ```

   This maps `.lit` and `.case` to `litigate-ts-mode` in `auto-mode-alist`.
   Once `eglot` loads, it also adds `litigate lsp --stdio` to
   `eglot-server-programs`. Run `M-x eglot` in a litigate buffer to start
   the server. `litigate-ts-mode-lsp-command` changes the command and
   `litigate-ts-mode-indent-offset` changes the indent width (default 2).

3. Check: open `lit/examples/rule12.lit`. `M-x imenu` lists nodes, rules,
   deadlines, types, events and tests, and `C-M-a` and `C-M-e` move between
   declarations. `treesit-font-lock-level` 4 enables all highlighting
   (operators, brackets, plain identifiers).

**Test.** `editors/litigate-editors/tests/emacs.rs` builds the grammar into a
scratch directory for the `emacs` on `PATH` and byte-compiles the mode with
warnings as errors. It then runs the ERT suite in `emacs --batch`, which:

- loads every `.lit`/`.case` file in the repository through `auto-mode-alist`,
- asserts a clean parse, runs every font-lock query, and checks that
  re-indenting changes nothing,
- checks faces, imenu and defun navigation,
- checks the eglot, `auto-mode-alist` and `treesit-language-source-alist`
  registrations,
- checks that the keyword, operator and indent lists equal
  `highlights.scm`/`indents.scm`.

It skips (logged to `$TEST_SKIP_LOG`) when Emacs is missing or lacks
tree-sitter.

## Helix

1. Merge [`helix/languages.toml`](helix/languages.toml) into
   `~/.config/helix/languages.toml`. It defines the `litigate` language for
   `.lit` and `.case`, with `//` comments, two-space indentation and
   `Lit.toml`/`Lit.lock` as roots. It also defines the `litigate-lsp`
   language server and the grammar source (this repository at a pinned
   `rev`, `subpath = "editors/tree-sitter-litigate"`). To use a local
   checkout instead, write
   `source = { path = "/path/to/litgraph/editors/tree-sitter-litigate" }`.
2. Install the queries and build the grammar:

   ```sh
   mkdir -p ~/.config/helix/runtime/queries
   cp -r editors/helix/queries/litigate ~/.config/helix/runtime/queries/
   hx --grammar fetch && hx --grammar build
   ```

3. Check:

   ```sh
   hx --health litigate
   ```

   It should report `Tree-sitter parser: ✓`, `Highlight queries: ✓`,
   `Textobject queries: ✓` and `Indent queries: ✓`. The language server
   shows `✘` until `litigate` is on `PATH`.

Helix's capture names differ from the common convention (for example
`@constant.numeric` instead of `@number`, `@namespace` instead of `@module`).
So `helix/queries/litigate/{highlights,indents,locals,injections}.scm` are
generated from the canonical queries by
`cargo run -p litigate-editors --bin helix-queries`
([`litigate-editors/src/helix.rs`](litigate-editors/src/helix.rs) holds the
capture renames). After you change a canonical query, rerun it. `textobjects.scm` is hand-written and Helix-only:
`mf`/`]f` select declarations, `mt` selects tests and `me` selects edges.

**Test.** `editors/litigate-editors/tests/helix.rs` runs these checks:

- the generated queries are current (`helix-queries --check` in test form),
- every Helix query compiles against the grammar and runs over the corpus
  (the `tree-sitter` crate, so no CLI is needed), and textobjects use
  Helix's object names,
- the pinned `rev` carries the grammar,
- when `hx` is on `PATH`, `hx --grammar build` succeeds in a throwaway
  config dir and `hx --health litigate` reports all four ✓ lines.

Only the `hx` step skips (logged) when `hx` is missing.

## Neovim (0.10+)

1. Put `editors/nvim` on the runtimepath. That loads `ftdetect/` (`.lit`,
   `.case` → `litigate`), `ftplugin/`, `queries/litigate/`, `plugin/` and
   `lsp/`. With lazy.nvim:

   ```lua
   { dir = '/path/to/litgraph/editors/nvim', ft = 'litigate', lazy = false }
   ```

   Or with no plugin manager: `vim.opt.rtp:prepend('/path/to/litgraph/editors/nvim')`.

2. Install the parser. With [nvim-treesitter](https://github.com/nvim-treesitter/nvim-treesitter),
   `plugin/litigate.lua` registers `litigate`. It supports the `master`
   branch (`get_parser_configs()`) and the `main` branch (the parsers table,
   re-registered on `User TSUpdate`). Then:

   ```vim
   :TSInstall litigate
   ```

   On Neovim older than 0.12 the registration asks nvim-treesitter to
   regenerate the parser (ABI 14), which needs the `tree-sitter` CLI.
   Without nvim-treesitter, build the library yourself and put it in a
   `parser/` directory on the runtimepath:

   ```sh
   cargo run -p litigate-editors --bin litigate-grammar -- --abi 15 --out ~/.local/share/nvim/site/parser
   mv ~/.local/share/nvim/site/parser/libtree-sitter-litigate.so ~/.local/share/nvim/site/parser/litigate.so
   ```

   (Use `--abi 14` for Neovim 0.10/0.11.) `ftplugin/litigate.lua` starts
   tree-sitter highlighting and expression folding when the parser loads.
   It also uses nvim-treesitter's `indentexpr` when that plugin is present.

3. Language server. On Neovim 0.11+, `lsp/litigate.lua` is discovered from
   the runtimepath:

   ```lua
   vim.lsp.enable('litigate')
   ```

   With nvim-lspconfig on Neovim 0.10:

   ```lua
   local configs = require('lspconfig.configs')
   if not configs.litigate then
     configs.litigate = {
       default_config = {
         cmd = { 'litigate', 'lsp', '--stdio' },
         filetypes = { 'litigate' },
         root_dir = require('lspconfig.util').root_pattern('Lit.toml', 'Lit.lock', '.git'),
       },
     }
   end
   require('lspconfig').litigate.setup({})
   ```

4. Check: `:checkhealth vim.treesitter` lists `litigate`, and
   `:Inspect` on a keyword in a `.lit` buffer shows `@keyword`.

**Test.** `editors/litigate-editors/tests/nvim.rs` builds the parser for the
`nvim` on `PATH` and runs [`nvim/test/litigate_test.lua`](nvim/test/litigate_test.lua)
headless. It checks filetype detection and the ftplugin, and that every
corpus file parses clean with the highlighter attached. Every query
compiles and runs over the corpus. The tree-sitter highlight assertions in
`tree-sitter-litigate/test/highlight/` must hold under Neovim's
last-match-wins rule. It also checks folds, the `lsp/` config, and
nvim-treesitter registration for both branches. It skips (logged) when
`nvim` is missing or older than 0.10.

## Running all editor tests

```sh
TEST_SKIP_LOG=/tmp/editors-skips.log cargo nextest run -p litigate-editors   # or: just editors-test
```

These tests are part of the workspace gate (`cargo nextest run --workspace`).
When an editor isn't installed, its test passes but writes a `SKIP` line
to stderr and to `$TEST_SKIP_LOG`. A skip is not a pass, so check the log.
The Emacs and Neovim tests regenerate the parser through `npx` when the
editor needs ABI 14, so node must be installed.

## Keeping the editors in sync with the grammar

- Change a highlight or indent query: edit
  `tree-sitter-litigate/queries/*.scm`, run
  `cargo run -p litigate-editors --bin helix-queries`,
  and mirror any new keyword, operator or indent node type in
  `emacs/litigate-ts-mode.el`. The ERT suite fails until the lists match.
  Neovim follows automatically through the symlinks.
- Change the grammar: once it lands, bump the `rev` in
  `helix/languages.toml`.
