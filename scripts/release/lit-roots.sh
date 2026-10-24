#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# Print the Merkle root of every committed Lit.lock (SPEC §4.2), one
# `<lockfile> <root>` line each: the LITLOCK-ROOTS manifest a release ships
# in each archive and records in its notes. `lock --root` verifies each
# lock first; a stale one fails the release.
#
# Usage: scripts/release/lit-roots.sh <litigate binary>   (at the repo root)
set -euo pipefail

litigate="$1"
out="$(mktemp)"
trap 'rm -f "$out"' EXIT
roots() {
  "$litigate" --format json lock --root "$1" \
    | jq -r '.result | if has("lockfiles") then .lockfiles[] else . end
        | "\(.lockfile) \(.root // "STALE")"'
}
{
  for lockfile in lit/*/Lit.lock; do
    roots "${lockfile%/Lit.lock}"
  done
  roots cases
} | sort > "$out"
if grep ' STALE$' "$out" >&2; then
  echo "lit-roots: a Lit.lock is stale; run \`just lit-sync\`" >&2
  exit 1
fi
cat "$out"
