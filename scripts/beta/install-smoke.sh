#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)

mode=${1:-host}
if [[ $# -gt 1 || ( "$mode" != host && "$mode" != --docker ) ]]; then
  echo "usage: $0 [--docker]" >&2
  exit 2
fi

if [[ "$mode" == --docker ]]; then
  if ! command -v docker >/dev/null 2>&1; then
    echo "install-smoke: docker is required for --docker" >&2
    exit 1
  fi
  docker run --rm \
    -v "$repo_root:/work/repo:ro" \
    -w /work/repo \
    rust:1-slim \
    bash scripts/beta/install-smoke.sh
  exit
fi

tmp_dir=$(mktemp -d "${TMPDIR:-/tmp}/litigate-install-smoke.XXXXXX")
trap 'rm -rf "$tmp_dir"' EXIT
export CARGO_HOME="$tmp_dir/cargo-home"
export CARGO_TARGET_DIR="$tmp_dir/target"
export PATH="$CARGO_HOME/bin:$PATH"
cd "$repo_root"

# Replays every command in README.md's Quickstart with isolated install and
# build directories, so neither a cached binary nor a cached target can pass.
cargo install --path crates/litigate-cli
litigate solve cases/cofc-1498-patent-case.case
litigate chain cases/cofc-1498-patent-case.case --set rate=900
litigate explain cases/frcp-civil-case.case --arg node=frcp-civil-procedure::pre-filing
litigate sweep cases/frcp-civil-case.case --arg param=rate --arg lo=100 --arg hi=1500
litigate deadlines lit/frcp --arg trigger=2025-01-06 --arg reachable=true
litigate solve lit/links
litigate run cases/cofc-1498-patent-case.case --op '{"op":"simulate","runs":50000}'
