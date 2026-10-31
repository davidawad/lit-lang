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

for package in "${!packages[@]}"; do
  [[ -d "lit/$package" ]] || unset "packages[$package]"
done

if [[ ${#packages[@]} -eq 0 ]]; then
  echo "strict-ratchet: no changed lit packages"
  exit 0
fi

counts=$(mktemp)
trap 'rm -f "$counts"' EXIT

# Use the compiler already prepared by the quality job when available. A
# standalone invocation still builds once if neither supported source exists.
bin="${LITIGATE_BIN:-$(command -v litigate || true)}"
if [[ -z $bin || ! -x $bin ]]; then
  cargo build -q -p litigate-cli
  bin="$(cargo metadata --format-version 1 --no-deps | jq -r .target_directory)/debug/litigate"
fi
if [[ ! -x $bin ]]; then
  echo "strict-ratchet: no executable litigate at $bin" >&2
  exit 1
fi

printf 'package\tin_scope_errors\n' > "$counts"
status=0
for package in "${!packages[@]}"; do
  dir="lit/$package"
  if [[ ! -d $dir ]]; then
    continue
  fi
  output=$(mktemp)
  if timeout 120 "$bin" check --profile strict --format json "$dir" >"$output" 2>"$output.err"; then
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
  printf '%s\n' "$row" >> "$counts"
done

if [[ $status -ne 0 ]]; then
  exit "$status"
fi
scripts/qa/strict-ratchet-compare.sh --baseline "$baseline" --counts "$counts" --update
