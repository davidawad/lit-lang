#!/usr/bin/env bash
# Headless Neovim test for editors/nvim: builds the grammar for this
# Neovim's ABI, then runs test/litigate_test.lua (filetype detection,
# ftplugin, every corpus file parses and highlights, every query compiles,
# the highlight assertions in tree-sitter-litigate/test/highlight hold,
# folds, the LSP config, and nvim-treesitter registration).
#
# Usage: editors/nvim/test/run.sh
# Skips (exit 0, reason on stderr and appended to $TEST_SKIP_LOG when set)
# when nvim is missing or older than 0.10.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
nvim="${NVIM:-nvim}"

skip() {
  echo "SKIP editors/nvim: $1" >&2
  if [ -n "${TEST_SKIP_LOG:-}" ]; then echo "editors/nvim: $1" >>"$TEST_SKIP_LOG"; fi
  exit 0
}

command -v "$nvim" >/dev/null 2>&1 || skip "nvim not on PATH"
abi="$("$nvim" --headless --clean -c 'lua if vim.fn.has("nvim-0.10") == 1 then io.stdout:write(vim.treesitter.language_version) end' -c 'qa!' 2>/dev/null || true)"
[ -n "$abi" ] || skip "nvim older than 0.10"

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
"$repo/editors/emacs/build-grammar.sh" --out "$work" --abi "$abi" >&2

LITIGATE_REPO="$repo" LITIGATE_PARSER="$(ls "$work"/libtree-sitter-litigate.*)" \
  XDG_STATE_HOME="$work/state" XDG_DATA_HOME="$work/data" \
  "$nvim" --headless --clean --cmd "set rtp^=$repo/editors/nvim" \
  -c "luafile $here/litigate_test.lua"
