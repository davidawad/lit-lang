#!/usr/bin/env bash
# Build the tree-sitter-litigate grammar as a shared library Emacs can load.
#
# Usage: editors/emacs/build-grammar.sh [--out DIR] [--abi N]
#
#   --out DIR  where to put libtree-sitter-litigate.{so,dylib}
#              (default: ~/.emacs.d/tree-sitter, Emacs's default search dir)
#   --abi N    tree-sitter ABI to build for. Default: ask `emacs` for
#              `treesit-library-abi-version`, else use the committed parser.
#
# The committed parser (editors/tree-sitter-litigate/src) is ABI 15. Emacs
# linked against libtree-sitter older than 0.25 only loads ABI <= 14, so for
# those the parser is regenerated into a temporary copy with
# `tree-sitter generate --abi N` (the pinned CLI from PATH, else npx). The
# committed src/ is never touched.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
grammar="$here/../tree-sitter-litigate"
ts_cli_version="0.27.1"
out="${HOME}/.emacs.d/tree-sitter"
abi=""

while [ $# -gt 0 ]; do
  case "$1" in
    --out) out="$2"; shift 2 ;;
    --abi) abi="$2"; shift 2 ;;
    -h|--help) sed -n '2,15p' "$0"; exit 0 ;;
    *) echo "build-grammar: unknown argument: $1" >&2; exit 1 ;;
  esac
done

committed_abi="$(sed -n 's/^#define LANGUAGE_VERSION \([0-9]*\)$/\1/p' "$grammar/src/parser.c")"
if [ -z "$abi" ]; then
  abi="$(emacs --batch --eval '(princ (if (treesit-available-p) (treesit-library-abi-version) ""))' 2>/dev/null || true)"
  abi="${abi:-$committed_abi}"
fi

case "$(uname -s)" in
  Darwin) ext=dylib ;;
  MINGW*|MSYS*|CYGWIN*) ext=dll ;;
  *) ext=so ;;
esac

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

if [ "$abi" -ge "$committed_abi" ]; then
  src="$grammar/src"
else
  cp -R "$grammar/grammar.js" "$grammar/grammar" "$grammar/tree-sitter.json" "$grammar/package.json" "$work/"
  mkdir -p "$work/src"
  cp "$grammar/src/scanner.c" "$work/src/"
  if tree-sitter --version 2>/dev/null | grep -q " $ts_cli_version"; then ts=(tree-sitter);
  else ts=(npx --yes "tree-sitter-cli@$ts_cli_version"); fi
  (cd "$work" && "${ts[@]}" generate --abi "$abi" >&2)
  src="$work/src"
fi

mkdir -p "$out"
"${CC:-cc}" -shared -fPIC -O2 -I"$src" "$src/parser.c" "$src/scanner.c" \
  -o "$out/libtree-sitter-litigate.$ext"
echo "$out/libtree-sitter-litigate.$ext (ABI $abi)"
