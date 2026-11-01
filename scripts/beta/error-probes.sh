#!/usr/bin/env bash
set -uo pipefail

# Full CLI error-boundary probe. Output is one TSV row per op/input pair.
ROOT=$(git rev-parse --show-toplevel)
BIN=${LITIGATE_BIN:-"$ROOT/target/debug/litigate"}
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT
OPS=(describe packs lint validate graph metric explain solve chain simulate path pareto sweep tornado calibration posterior voi structure deadlines settle compare batch)
printf 'op\tinput\texit\tclassification\n'

probe() {
  local op=$1 label=$2 target=$3 extra=${4:-} rc class payload
  payload="{\"op\":\"$op\"}"
  case "$op" in
    graph|explain|structure|deadlines) payload="{\"op\":\"$op\",\"node\":\"__unknown_beta_node__\"}" ;;
  esac
  local args=(run "$target" --op "$payload")
  if [[ -n $extra ]]; then args+=(--set "$extra"); fi
  "$BIN" "${args[@]}" >"$TMP/out" 2>"$TMP/err"; rc=$?
  if rg -qi 'panicked at|stack backtrace|backtrace:' "$TMP/out" "$TMP/err"; then
    class=PANIC
  elif jq -e '(.ok == false) and (.error.code|type == "string") and (.error.message|type == "string") and (.error.hint|type == "string")' "$TMP/out" >/dev/null 2>&1; then
    class=ENVELOPE
  elif [[ $rc == 0 ]] && jq -e 'type == "object" and has("ok")' "$TMP/out" >/dev/null 2>&1; then
    class=OK
  elif [[ $rc == 1 ]] && [[ -s $TMP/err ]]; then
    class=CLI_DIAGNOSTIC
  else
    class=BARE_ERROR
  fi
  printf '%s\t%s\t%s\t%s\n' "$op" "$label" "$rc" "$class"
}

mapfile -t CASES < <(find "$ROOT/cases" -type f -name '*.case' | sort)
if [[ ${PROBE_CASE_LIMIT:-0} -gt 0 ]]; then CASES=("${CASES[@]:0:$PROBE_CASE_LIMIT}"); fi
for op in "${OPS[@]}"; do
  if [[ ${PROBE_SKIP_CASES:-0} != 1 ]]; then
    for target in "${CASES[@]}"; do probe "$op" "case:${target#$ROOT/}" "$target"; done
  fi
  probe "$op" missing-file "$TMP/missing.case"
  printf 'matter {\n' >"$TMP/syntax.case"
  probe "$op" syntax-error "$TMP/syntax.case"
  probe "$op" unknown-set "${CASES[0]}" 'definitely_unknown_beta_key=1'
  probe "$op" unknown-node "${CASES[0]}" # op-specific unknown id is also supplied below where supported
  # `run` has no --as-of option; the corresponding date-selection boundary
  # belongs to `check`, so test it once through that command after the op loop.
done
"$BIN" check "${CASES[0]}" --as-of 0001-01-01 >"$TMP/out" 2>"$TMP/err"; rc=$?
if rg -qi 'panicked at|stack backtrace|backtrace:' "$TMP/out" "$TMP/err"; then class=PANIC
elif [[ $rc == 2 ]] && rg -qi 'date|interval|effective' "$TMP/out" "$TMP/err"; then class=CLI_DIAGNOSTIC
else class=UNEXPECTED; fi
printf 'check\tout-of-interval-date\t%s\t%s\n' "$rc" "$class"
