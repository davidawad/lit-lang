#!/usr/bin/env bash
# Strict-profile per-package error ratchet for changed lit packages.
set -euo pipefail

cd "$(dirname "$0")/../.."
baseline=docs/qa/strict-baseline.tsv

usage() {
  echo "usage: scripts/qa/strict-ratchet.sh <merge-base> <changed-files>" >&2
  exit 2
}
[[ $# -eq 2 ]] || usage
base=$1
files=$2

# Keep a core edit bounded: these are the shared packages excluded from the
# census and explicitly held at zero in the baseline.
core_packages=(core links examples frcp frap frcrimp bap8 ca1 ca8 cadc cafc cofc itc mpep ptab ttab)
declare -A packages=()
while IFS= read -r path; do
  [[ $path =~ ^lit/([^/]+)/ ]] || continue
  packages["${BASH_REMATCH[1]}"]=1
done <<< "$files"

if [[ ${packages[core]+yes} ]]; then
  packages=()
  for package in "${core_packages[@]}"; do packages["$package"]=1; done
fi

if [[ ${#packages[@]} -eq 0 ]]; then
  echo "strict-ratchet: no changed lit packages"
  exit 0
fi

tmpbase=$(mktemp)
trap 'rm -f "$tmpbase"' EXIT

# The CLI executable is needed by every strict check. Build it once for this
# invocation, then reuse it for all changed packages.
cargo build -q -p litigate-cli

status=0
for package in "${!packages[@]}"; do
  dir="lit/$package"
  if [[ ! -d $dir ]]; then
    continue
  fi
  output=$(mktemp)
  if timeout 120 target/debug/litigate check --profile strict --format json "$dir" >"$output" 2>"$output.err"; then
    rc=0
  else
    rc=$?
  fi
  if [[ $rc -eq 124 || ! -s $output ]]; then
    echo "strict-ratchet: $package strict check failed (exit $rc or no JSON output)" >&2
    cat "$output.err" >&2 || true
    status=1
    rm -f "$output" "$output.err"
    continue
  fi
  if ! row=$(python3 - "$output" "$package" "$rc" <<'PY'
import json, sys
path, package, rc = sys.argv[1:]
try:
    result = json.load(open(path))
    diagnostics = result.get("diagnostics", [])
except Exception as exc:
    print(f"strict-ratchet: invalid JSON for {package}: {exc}", file=sys.stderr)
    raise SystemExit(1)
if not isinstance(diagnostics, list):
    print(f"strict-ratchet: missing diagnostics array for {package}", file=sys.stderr)
    raise SystemExit(1)
count = 0
for diagnostic in diagnostics:
    if diagnostic.get("severity") != "error":
        continue
    span = diagnostic.get("span", {})
    file = span.get("file", "").replace("\\", "/")
    if diagnostic.get("code") == "L5002" and ("lit/core/states/" in file or file.startswith("core/states/")):
        continue
    count += 1
print(f"{package}\t{count}")
PY
  ); then
    status=1
    rm -f "$output" "$output.err"
    continue
  fi
  rm -f "$output" "$output.err"
  current=${row#*$'\t'}
  baseline_count=$(awk -F '\t' -v p="$package" 'NR>1 && $1==p {print $2; found=1} END {if (!found) print ""}' "$baseline")
  if [[ -z $baseline_count ]]; then
    if (( current > 0 )); then
      echo "strict-ratchet: $package is missing from baseline with $current in-scope error(s)" >&2
      status=1
    else
      printf '%s\t0\n' "$package" >> "$tmpbase"
      echo "strict-ratchet: $package is new and strict-clean; add baseline row"
    fi
  elif (( current > baseline_count )); then
    echo "strict-ratchet: $package increased from $baseline_count to $current in-scope error(s)" >&2
    status=1
  elif (( current < baseline_count )); then
    printf '%s\t%s\n' "$package" "$current" >> "$tmpbase"
    echo "strict-ratchet: $package improved from $baseline_count to $current; commit the updated baseline row"
  else
    echo "strict-ratchet: $package unchanged at $current in-scope error(s)"
  fi
done

if [[ -s $tmpbase ]]; then
  python3 - "$baseline" "$tmpbase" <<'PY'
import pathlib, sys
baseline, updates = map(pathlib.Path, sys.argv[1:])
rows = {}
for line in baseline.read_text().splitlines()[1:]:
    if line:
        package, count = line.split("\t")
        rows[package] = count
for line in updates.read_text().splitlines():
    package, count = line.split("\t")
    rows[package] = count
baseline.write_text("package\tin_scope_errors\n" + "".join(f"{p}\t{rows[p]}\n" for p in sorted(rows)))
PY
fi

exit "$status"
