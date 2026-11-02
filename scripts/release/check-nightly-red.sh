#!/usr/bin/env bash
# Refuse a release while the lit-lang bead store has an open nightly failure.
set -euo pipefail

if ! command -v br >/dev/null 2>&1; then
  echo "release blocked: br is required to check open nightly-red beads" >&2
  exit 1
fi

open_beads="$(br list --status open --label nightly-red 2>&1)" || {
  echo "release blocked: could not check open nightly-red beads: $open_beads" >&2
  exit 1
}

if [ -n "$open_beads" ]; then
  echo "release blocked by open nightly-red bead(s):" >&2
  printf '%s\n' "$open_beads" >&2
  exit 1
fi
