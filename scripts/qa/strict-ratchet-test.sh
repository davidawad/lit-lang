#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
compare=scripts/qa/strict-ratchet-compare.sh
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

check_status() {
  local expected=$1
  shift
  local rc=0
  "$@" || rc=$?
  if [[ $rc -ne $expected ]]; then
    echo "strict-ratchet-test: expected exit $expected, got $rc" >&2
    exit 1
  fi
}

printf 'package\tin_scope_errors\npkg\t2\n' > "$tmp/baseline.tsv"
printf 'package\tin_scope_errors\npkg\t3\n' > "$tmp/up.tsv"
check_status 1 "$compare" --baseline "$tmp/baseline.tsv" --counts "$tmp/up.tsv"

printf 'package\tin_scope_errors\npkg\t1\n' > "$tmp/down.tsv"
check_status 0 "$compare" --baseline "$tmp/baseline.tsv" --counts "$tmp/down.tsv" --update
grep -qx $'pkg\t1' <(tail -n +2 "$tmp/baseline.tsv")

printf 'package\tin_scope_errors\nnewpkg\t0\n' > "$tmp/new-clean.tsv"
check_status 0 "$compare" --baseline "$tmp/baseline.tsv" --counts "$tmp/new-clean.tsv"
printf 'package\tin_scope_errors\nnewpkg\t3\n' > "$tmp/new-dirty.tsv"
check_status 1 "$compare" --baseline "$tmp/baseline.tsv" --counts "$tmp/new-dirty.tsv"

check_status 0 scripts/qa/strict-ratchet.sh "$(git rev-parse HEAD)" ''
echo 'strict-ratchet-test: pass'
