#!/usr/bin/env bash
# Compare changed-package strict counts with the checked-in ratchet baseline.
set -euo pipefail

usage() {
  echo "usage: scripts/qa/strict-ratchet-compare.sh --baseline TSV --counts TSV [--update]" >&2
  exit 2
}
baseline= counts= update=0
while (($#)); do
  case "$1" in
    --baseline) (($# >= 2)) || usage; baseline=$2; shift 2 ;;
    --counts) (($# >= 2)) || usage; counts=$2; shift 2 ;;
    --update) update=1; shift ;;
    *) usage ;;
  esac
done
[[ -n $baseline && -n $counts && -f $baseline && -f $counts ]] || usage

python3 - "$baseline" "$counts" "$update" <<'PY'
import pathlib, sys

baseline_path, counts_path = map(pathlib.Path, sys.argv[1:3])
update = sys.argv[3] == "1"

def read(path):
    lines = path.read_text().splitlines()
    if not lines or lines[0] != "package\tin_scope_errors":
        raise ValueError(f"{path}: expected package/in_scope_errors header")
    rows = {}
    for number, line in enumerate(lines[1:], 2):
        if not line:
            continue
        fields = line.split("\t")
        if len(fields) != 2 or not fields[0] or not fields[1].isdigit():
            raise ValueError(f"{path}:{number}: expected package<TAB>nonnegative_integer")
        if fields[0] in rows:
            raise ValueError(f"{path}:{number}: duplicate package {fields[0]}")
        rows[fields[0]] = int(fields[1])
    return rows

try:
    baseline = read(baseline_path)
    counts = read(counts_path)
except ValueError as exc:
    print(f"strict-ratchet: {exc}", file=sys.stderr)
    raise SystemExit(2)

failed = False
updates = {}
for package, current in sorted(counts.items()):
    previous = baseline.get(package)
    if previous is None:
        if current:
            print(f"strict-ratchet: {package} is missing from baseline with {current} in-scope error(s)", file=sys.stderr)
            failed = True
        else:
            print(f"strict-ratchet: {package} is new and strict-clean; add baseline row")
            if update:
                updates[package] = 0
    elif current > previous:
        print(f"strict-ratchet: {package} increased from {previous} to {current} in-scope error(s)", file=sys.stderr)
        failed = True
    elif current < previous:
        print(f"strict-ratchet: {package} improved from {previous} to {current}; commit the updated baseline row")
        if update:
            updates[package] = current
    else:
        print(f"strict-ratchet: {package} unchanged at {current} in-scope error(s)")

if update and updates:
    baseline.update(updates)
    baseline_path.write_text("package\tin_scope_errors\n" + "".join(f"{p}\t{baseline[p]}\n" for p in sorted(baseline)))
raise SystemExit(1 if failed else 0)
PY
