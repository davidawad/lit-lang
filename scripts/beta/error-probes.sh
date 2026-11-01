#!/usr/bin/env bash
set -uo pipefail

# Parallel CLI error-boundary matrix. Each worker is one independently timed row.
ROOT=$(git rev-parse --show-toplevel)
BIN=${LITIGATE_BIN:-"$ROOT/target/debug/litigate"}
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

if [[ ${1:-} == --probe ]]; then
  op=$2 label=$3 target=$4 extra=${5:-} index=$6
  out="$TMP/out-$index" err="$TMP/err-$index"
  # Per-worker temp paths are isolated by the parent-provided index.
  payload="{\"op\":\"$op\"}"
  case "$op" in
    graph|explain|structure|deadlines) payload="{\"op\":\"$op\",\"node\":\"__unknown_beta_node__\"}" ;;
  esac
  args=(run "$target" --op "$payload")
  [[ -n "$extra" ]] && args+=(--set "$extra")
  timeout 10s "$BIN" "${args[@]}" >"$out" 2>"$err"
  rc=$?
  if [[ $rc == 124 ]]; then class=TIMEOUT
  elif rg -qi 'panicked at|stack backtrace|backtrace:' "$out" "$err"; then class=PANIC
  elif jq -e '(.ok == false) and (.error.code|type == "string") and (.error.message|type == "string") and (.error.hint|type == "string")' "$out" >/dev/null 2>&1; then class=ENVELOPE
  elif [[ $rc == 0 ]] && jq -e 'type == "object" and has("ok")' "$out" >/dev/null 2>&1; then class=OK
  elif [[ $rc == 1 ]] && [[ -s $err ]]; then class=CLI_DIAGNOSTIC
  else class=BARE_ERROR
  fi
  printf '%s\t%s\t%s\t%s\t%s\n' "$index" "$op" "$label" "$rc" "$class"
  exit 0
fi

OPS=(describe packs lint validate graph metric explain solve chain simulate path pareto sweep tornado calibration posterior voi structure deadlines settle compare batch)
CASE="$ROOT/cases/e2e-ttab-unknown.case"
printf 'op\tinput\texit\tclassification\n'
index=0
for op in "${OPS[@]}"; do
  for row in "case:$CASE" 'missing-file:/__litigate_beta_missing__.case' 'syntax-error:/__litigate_beta_syntax__.case' 'unknown-set:case'; do
    label=${row%%:*}; target=${row#*:}; extra=
    if [[ $label == syntax-error ]]; then printf 'matter {\n' >"$TMP/syntax.case"; target="$TMP/syntax.case"; fi
    if [[ $label == unknown-set ]]; then target=$CASE; extra=definitely_unknown_beta_key=1; fi
    printf '%s\0%s\0%s\0%s\0%s\0' "$op" "$label" "$target" "$extra" "$index" >>"$TMP/tasks"
    index=$((index + 1))
  done
  for label in unknown-node; do
    printf '%s\0%s\0%s\0%s\0%s\0' "$op" "$label" "$CASE" '' "$index" >>"$TMP/tasks"
    index=$((index + 1))
  done
done

# Workers capture into their own temporary directories; the parent sorts their
# indexed rows back into matrix order.
xargs -0 -n5 -P8 bash "$ROOT/scripts/beta/error-probes.sh" --probe <"$TMP/tasks" >"$TMP/results"
sort -t $'\t' -k1,1n "$TMP/results" | cut -f2- | while IFS=$'\t' read -r op label rc class; do
  printf '%s\t%s\t%s\t%s\n' "$op" "$label" "$rc" "$class"
done

# Date selection is a separate CLI boundary and gets its own timed row.
timeout 10s "$BIN" check "$CASE" --as-of 0001-01-01 >"$TMP/check-out" 2>"$TMP/check-err"; rc=$?
if [[ $rc == 124 ]]; then class=TIMEOUT
elif rg -qi 'panicked at|stack backtrace|backtrace:' "$TMP/check-out" "$TMP/check-err"; then class=PANIC
elif [[ $rc == 2 ]] && rg -qi 'date|interval|effective' "$TMP/check-out" "$TMP/check-err"; then class=CLI_DIAGNOSTIC
else class=UNEXPECTED; fi
printf 'check\tout-of-interval-date\t%s\t%s\n' "$rc" "$class"
