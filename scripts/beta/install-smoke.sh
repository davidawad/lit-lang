#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)

if ! command -v docker >/dev/null 2>&1; then
  echo "install-smoke: docker is required" >&2
  exit 1
fi

# The official Rust image starts with an empty target directory and Cargo home.
# Keeping Cargo's install root inside the container tests PATH as a new user sees it.
docker run --rm \
  -v "$repo_root:/work/repo:ro" \
  -w /work/repo \
  rust:1-slim \
  bash -euc '
    export CARGO_HOME=/tmp/cargo-home
    export PATH="$CARGO_HOME/bin:$PATH"
    cargo install --path crates/litigate-cli
    litigate solve cases/cofc-1498-patent-case.case
    litigate chain cases/cofc-1498-patent-case.case --set rate=900
    litigate explain cases/frcp-civil-case.case --arg node=frcp-civil-procedure::pre-filing
    litigate sweep cases/frcp-civil-case.case --arg param=rate --arg lo=100 --arg hi=1500
    litigate deadlines lit/frcp --arg trigger=2025-01-06 --arg reachable=true
    litigate solve lit/links
    litigate run cases/cofc-1498-patent-case.case --op '\''{"op":"simulate","runs":50000}'\''
  '
