#!/usr/bin/env bash
# Regenerate the engine fixtures in test/fixtures/ from the real binary:
# each is the JSON envelope `litigate run TARGET --op OP --format json`
# prints, run from the repository root, with its `diagnostics` dropped
# (the compiler's hole warnings about the vendored law, ~100KB a run,
# which no unit test reads).  `compile-error' keeps them: it is a run
# that fails to compile.
#
#   LITIGATE=target/debug/litigate scripts/fixtures.sh   (or `make fixtures')
#
# The timeline fixtures come from `litigate timeline'; see the README.
set -euo pipefail

here="$(cd "$(dirname "$0")/.." && pwd)"
repo="$(cd "$here/../.." && pwd)"
out="$here/test/fixtures"
litigate="${LITIGATE:-litigate}"
case "$litigate" in */*) litigate="$(cd "$(dirname "$litigate")" && pwd)/$(basename "$litigate")" ;; esac
matter=cases/cofc-1498-patent-case.case

# NAME OP [EXIT]: the envelope of OP run on the matter, without
# diagnostics; the run must exit EXIT (default 0).
fixture() {
  local name="$1" op="$2" want="${3:-0}" status=0
  (cd "$repo" && "$litigate" run "$matter" --op "$op" --format json) \
    > "$out/$name.json.raw" || status=$?
  if [ "$status" -ne "$want" ]; then
    echo "fixtures: $name: litigate exited $status, expected $want" >&2
    exit 1
  fi
  jq -c 'del(.diagnostics)' "$out/$name.json.raw" > "$out/$name.json"
  rm "$out/$name.json.raw"
  echo "fixtures: $name"
}

fixture describe '{"op":"describe"}'
fixture packs '{"op":"packs"}'
fixture lint '{"op":"lint"}'
fixture graph-cofc '{"op":"graph"}'
fixture solve '{"op":"solve"}'
fixture chain '{"op":"chain"}'
fixture explain '{"op":"explain","node":"cofc::claim-accrues"}'
fixture simulate '{"op":"simulate","runs":10000,"seed":7}'
fixture path '{"op":"path","to":"terminals","k":3}'
fixture pareto '{"op":"pareto"}'
fixture sweep '{"op":"sweep","param":"rate"}'
fixture tornado '{"op":"tornado"}'
fixture structure '{"op":"structure","what":"summary"}'
fixture calibration '{"op":"calibration"}'
fixture metric '{"op":"metric","spec":"hours*rate","top":5}'
fixture deadlines-node '{"op":"deadlines","trigger":"2026-10-01","node":"cofc::government-response-fork"}'
fixture compare '{"op":"compare","variant":{"params":{"rate":900}},"inner":{"op":"chain"}}'
fixture batch '{"op":"batch","ops":[{"op":"solve"},{"op":"chain"},{"op":"tornado"}]}'
# An engine error: the envelope is ok, the engine response it wraps is not.
fixture error '{"op":"explain","node":"nope"}' 2

# A matter that does not compile: no engine response, only diagnostics.
scratch="$(mktemp -d)"
trap 'rm -rf "$scratch"' EXIT
printf 'case broken\nuses nope ^0.1.0\n' > "$scratch/broken.case"
status=0
(cd "$scratch" && "$litigate" run broken.case --op solve --format json) \
  > "$out/compile-error.json" || status=$?
if [ "$status" -ne 2 ]; then
  echo "fixtures: compile-error: expected exit 2, got $status" >&2
  exit 1
fi
echo "fixtures: compile-error"
