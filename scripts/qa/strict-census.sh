#!/usr/bin/env bash
set -uo pipefail

cd "$(dirname "$0")/../.."
mkdir -p docs/qa
out=docs/qa/strict-census.tsv
if [[ ! -f "$out" ]]; then
  printf 'package\trc\tseconds\tdiagnostics_by_code\tdeferred_states_L5002\tin_scope_errors\n' > "$out"
fi
workers=${1:-4}
export out

find lit -mindepth 1 -maxdepth 1 -type d -printf '%p\n' | sort | while IFS= read -r dir; do
  pkg=${dir#lit/}
  case "$pkg" in
    core|links|examples|frcp|frap|frcrimp|bap8|ca1|ca8|cadc|cafc|cofc|itc|mpep|ptab|ttab) continue ;;
  esac
  printf '%s\n' "$dir"
done | xargs -r -P "$workers" -I '{}' bash -c '
  dir="$1"; pkg=${dir#lit/}
  if awk -F "\t" -v p="$pkg" "NR>1 && \$1==p {found=1} END {exit !found}" "$out"; then exit 0; fi
  tmp=$(mktemp)
  start=$(date +%s)
  timeout 300 target/debug/litigate check --profile strict --format json "$dir" >"$tmp" 2>/dev/null
  rc=$?
  seconds=$(( $(date +%s) - start ))
  row=$(python3 - "$tmp" "$pkg" "$rc" "$seconds" <<"PY"
import json,sys,collections
path,pkg,rc,secs=sys.argv[1:]
try:
 d=json.load(open(path)); diags=d.get("result",{}).get("diagnostics",[])
except Exception: diags=[]
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
  rm -f "$tmp"
  flock "$out" -c "printf \"%s\\n\" \"$row\" >> \"$out\""
  printf "%s rc=%s seconds=%s in_scope=%s\n" "$pkg" "$rc" "$seconds" "${row##*$"\t"}"
' _ '{}'
