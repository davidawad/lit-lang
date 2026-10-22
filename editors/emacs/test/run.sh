#!/usr/bin/env bash
# Batch-test litigate-ts-mode: build the grammar for this Emacs, then run the
# ERT suite (every .lit/.case corpus file loads, parses, fontifies, indents
# canonically, plus face/imenu/registration checks).
#
# Usage: editors/emacs/test/run.sh
#
# Skips (exit 0, reason on stderr and appended to $TEST_SKIP_LOG when set)
# when Emacs is missing, lacks tree-sitter, or is older than 29.1.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
emacs="${EMACS:-emacs}"

skip() {
  echo "SKIP editors/emacs: $1" >&2
  if [ -n "${TEST_SKIP_LOG:-}" ]; then echo "editors/emacs ert: $1" >>"$TEST_SKIP_LOG"; fi
  exit 0
}

command -v "$emacs" >/dev/null 2>&1 || skip "emacs not on PATH"
ok="$("$emacs" --batch --eval '(princ (if (and (>= emacs-major-version 29) (fboundp (quote treesit-available-p)) (treesit-available-p)) "yes" "no"))' 2>/dev/null || true)"
[ "$ok" = "yes" ] || skip "emacs lacks tree-sitter support (needs 29.1+ built --with-tree-sitter)"
abi="$("$emacs" --batch --eval '(princ (treesit-library-abi-version))' 2>/dev/null)"

grammar_dir="$(mktemp -d)"
trap 'rm -rf "$grammar_dir"' EXIT
EMACS="$emacs" "$here/../build-grammar.sh" --out "$grammar_dir" --abi "$abi" >&2

LITIGATE_REPO="$repo" LITIGATE_TS_GRAMMAR_DIR="$grammar_dir" \
  "$emacs" --batch -Q \
  -L "$here/.." -L "$here" \
  --eval '(setq byte-compile-error-on-warn t)' \
  -f batch-byte-compile "$here/../litigate-ts-mode.el"
rm -f "$here/../litigate-ts-mode.elc"

LITIGATE_REPO="$repo" LITIGATE_TS_GRAMMAR_DIR="$grammar_dir" \
  "$emacs" --batch -Q \
  -L "$here/.." -L "$here" \
  -l litigate-ts-mode-test \
  -f ert-run-tests-batch-and-exit
