#!/usr/bin/env bash
set -uo pipefail

cd "$(dirname "$0")/../.."
mkdir -p docs/qa
out=docs/qa/strict-census.tsv
if [[ ! -f "$out" ]]; then
  printf 'package\trc\tseconds\tdiagnostics_by_code\tdeferred_states_L5002\tin_scope_errors\n' > "$out"
fi

limit=8
workers=4
while (($#)); do
  case "$1" in
    --limit) limit=${2:?--limit needs a number}; shift 2 ;;
    --workers) workers=${2:?--workers needs a number}; shift 2 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done
[[ "$limit" =~ ^[1-9][0-9]*$ && "$workers" =~ ^[1-9][0-9]*$ ]] || { echo 'limit and workers must be positive integers' >&2; exit 2; }

packages=$(mktemp)
rows=$(mktemp -d)
trap 'rm -f "$packages"; rm -rf "$rows"' EXIT
find lit -mindepth 1 -maxdepth 1 -type d -printf '%p\n' | sort | while IFS= read -r dir; do
  pkg=${dir#lit/}
  case "$pkg" in
    core|links|examples|frcp|frap|frcrimp|bap8|ca1|ca8|cadc|cafc|cofc|itc|mpep|ptab|ttab) continue ;;
  esac
  if ! awk -F '\t' -v p="$pkg" 'NR>1 && $1==p {found=1} END {exit !found}' "$out"; then
    printf '%s\n' "$dir"
  fi
done | head -n "$limit" > "$packages"

run_one() {
  local dir=$1 pkg tmp start rc seconds row
  pkg=${dir#lit/}
  tmp=$(mktemp)
  start=$(date +%s)
  timeout 120 target/debug/litigate check --profile strict --format json "$dir" >"$tmp" 2>"$tmp.err"
  rc=$?
  seconds=$(( $(date +%s) - start ))
  row=$(python3 - "$tmp" "$pkg" "$rc" "$seconds" <<'PY'
import json,sys,collections
path,pkg,rc,secs=sys.argv[1:]
try:
    d=json.load(open(path)); diags=d.get("diagnostics",[])
except Exception:
    diags=[]
counts=collections.Counter(x.get("code","UNKNOWN") for x in diags if x.get("severity")=="error")
deferred=0; scope=0
for x in diags:
    if x.get("severity")!="error": continue
    span=x.get("span",{}); f=span.get("file","").replace("\\","/")
    if x.get("code")=="L5002" and ("lit/core/states/" in f or f.startswith("core/states/")): deferred+=1
    else: scope+=1
codes=",".join(f"{k}={counts[k]}" for k in sorted(counts)) or "-"
print("\t".join((pkg,rc,secs,codes,str(deferred),str(scope))))
PY
  )
  if [[ $rc -eq 124 ]]; then row=$(printf '%s\t124\t%s\ttimeout\t0\t0' "$pkg" "$seconds"); fi
  printf '%s\n' "$row" > "$rows/$pkg"
  rm -f "$tmp" "$tmp.err"
}
export -f run_one
export out rows
if [[ -s "$packages" ]]; then
  xargs -r -P "$workers" -I '{}' bash -c 'run_one "$1"' _ '{}' < "$packages"
  while IFS= read -r dir; do
    pkg=${dir#lit/}
    flock "$out" -c "cat '$rows/$pkg' >> '$out'"
    IFS=$'\t' read -r _ rc seconds _ _ scope < "$rows/$pkg"
    printf '%s rc=%s seconds=%s in_scope=%s\n' "$pkg" "$rc" "$seconds" "$scope"
  done < "$packages"
fi
