#!/usr/bin/env bash
# Fixture test for the release gate's open nightly-red bead check.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

cat >"$tmp/br" <<'EOF'
#!/usr/bin/env bash
if [ "${BR_FIXTURE:-}" = red ]; then
  printf '%s\n' 'lg-nightly-red failing nightly'
fi
EOF
chmod +x "$tmp/br"

if BR_FIXTURE=red PATH="$tmp:$PATH" "$root/scripts/release/check-nightly-red.sh" >"$tmp/out" 2>&1; then
  echo "expected release check to refuse an open nightly-red bead" >&2
  exit 1
fi
grep -Fq 'lg-nightly-red' "$tmp/out"

BR_FIXTURE=clear PATH="$tmp:$PATH" "$root/scripts/release/check-nightly-red.sh"
