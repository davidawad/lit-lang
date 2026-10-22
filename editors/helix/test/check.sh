#!/usr/bin/env bash
# Check the Helix configuration for litigate.
#
# 1. queries/litigate/*.scm are in sync with the canonical queries.
# 2. Every Helix query compiles against the grammar and runs over the
#    corpus (`tree-sitter query`; the pinned CLI from PATH, else npx).
# 3. languages.toml's grammar rev exists and carries the grammar.
# 4. With `hx` on PATH: `hx --grammar build` builds the grammar from this
#    checkout in a throwaway config dir, and `hx --health litigate` reports
#    the parser and highlight/textobject/indent queries.
#
# Usage: editors/helix/test/check.sh
# Steps 2 and 4 skip (reason on stderr, appended to $TEST_SKIP_LOG when set)
# when their tool is unavailable; every other failure exits non-zero.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
helix="$here/.."
repo="$(cd "$helix/../.." && pwd)"
grammar="$repo/editors/tree-sitter-litigate"
ts_cli_version="0.27.1"
hx="${HX:-hx}"

skip() {
  echo "SKIP editors/helix: $1" >&2
  if [ -n "${TEST_SKIP_LOG:-}" ]; then echo "editors/helix: $1" >>"$TEST_SKIP_LOG"; fi
}

"$helix/sync-queries.sh" --check
echo "ok   helix queries in sync with editors/tree-sitter-litigate/queries"

mapfile -t corpus < <(cd "$repo" && find . \( -name .git -o -name target -o -name node_modules -o -name .worktrees \) -prune \
  -o -type f \( -name '*.lit' -o -name '*.case' \) -print | sort)
[ "${#corpus[@]}" -ge 2 ] || { echo "corpus is empty" >&2; exit 1; }

if tree-sitter --version 2>/dev/null | grep -q " $ts_cli_version"; then ts=(tree-sitter);
elif command -v npx >/dev/null 2>&1; then ts=(npx --yes "tree-sitter-cli@$ts_cli_version");
else ts=(); fi
if [ "${#ts[@]}" -gt 0 ]; then
  for query in "$helix"/queries/litigate/*.scm; do
    (cd "$grammar" && "${ts[@]}" query --quiet "$query" "${corpus[@]/#./$repo}" >/dev/null 2>"$here/.query.err") || {
      cat "$here/.query.err" >&2; rm -f "$here/.query.err"; echo "FAIL $query" >&2; exit 1; }
    rm -f "$here/.query.err"
    echo "ok   $(basename "$query") compiles and runs over ${#corpus[@]} corpus files"
  done
else
  skip "query compilation: no tree-sitter $ts_cli_version on PATH and no npx"
fi

rev="$(sed -n 's/^source = { git = .*rev = "\([0-9a-f]*\)".*/\1/p' "$helix/languages.toml")"
[ -n "$rev" ] || { echo "languages.toml: no grammar rev" >&2; exit 1; }
if git -C "$repo" cat-file -e "$rev^{commit}" 2>/dev/null; then
  git -C "$repo" cat-file -e "$rev:editors/tree-sitter-litigate/src/parser.c"
  echo "ok   languages.toml grammar rev $rev carries editors/tree-sitter-litigate"
else
  skip "grammar rev $rev not in this clone's history (shallow clone?)"
fi

if ! command -v "$hx" >/dev/null 2>&1; then
  skip "hx not on PATH (hx --grammar build / hx --health litigate not run)"
  exit 0
fi
cfg="$(mktemp -d)"
trap 'rm -rf "$cfg"' EXIT
mkdir -p "$cfg/helix/runtime/grammars" "$cfg/helix/runtime/queries"
{
  echo 'use-grammars = { only = ["litigate"] }'
  sed "s#^source = { git = .*#source = { path = \"$grammar\" }#" "$helix/languages.toml"
} >"$cfg/helix/languages.toml"
cp -R "$helix/queries/litigate" "$cfg/helix/runtime/queries/"
run_hx() { XDG_CONFIG_HOME="$cfg" NO_COLOR=1 "$hx" "$@"; }
run_hx --grammar build >&2
[ -f "$cfg/helix/runtime/grammars/litigate.so" ] || [ -f "$cfg/helix/runtime/grammars/litigate.dylib" ] \
  || { echo "hx --grammar build produced no litigate grammar" >&2; exit 1; }
health="$(run_hx --health litigate 2>&1 | sed 's/\x1b\[[0-9;]*m//g')"
echo "$health"
for line in "Tree-sitter parser: ✓" "Highlight queries: ✓" "Textobject queries: ✓" "Indent queries: ✓"; do
  grep -qF "$line" <<<"$health" || { echo "hx --health litigate: missing '$line'" >&2; exit 1; }
done
echo "ok   hx --grammar build and hx --health litigate ($("$hx" --version))"
