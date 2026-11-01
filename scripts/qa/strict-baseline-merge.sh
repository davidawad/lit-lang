#!/usr/bin/env bash
# Merge strict-baseline TSVs by keeping the lowest count for each package.
set -euo pipefail

usage() {
  echo "usage: strict-baseline-merge.sh BASE OURS THEIRS" >&2
  exit 2
}
[[ $# -eq 3 ]] || usage

python3 - "$@" <<'PY'
import sys

header = "package\tin_scope_errors"
rows = {}
for path in sys.argv[1:]:
    with open(path, encoding="utf-8") as source:
        lines = source.read().splitlines()
    if not lines or lines[0] != header:
        raise SystemExit(f"strict-baseline-merge: invalid header in {path}")
    for line in lines[1:]:
        fields = line.split("\t")
        if len(fields) != 2 or not fields[0] or not fields[1].isdigit():
            raise SystemExit(f"strict-baseline-merge: invalid row in {path}: {line!r}")
        package, count = fields[0], int(fields[1])
        rows[package] = min(rows.get(package, count), count)

with open(sys.argv[2], "w", encoding="utf-8", newline="") as output:
    output.write(header + "\n")
    for package in sorted(rows):
        output.write(f"{package}\t{rows[package]}\n")
PY
