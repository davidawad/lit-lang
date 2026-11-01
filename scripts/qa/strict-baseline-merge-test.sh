#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
repo="$tmp/repo"
mkdir -p "$repo/docs/qa"
git -C "$repo" init -q -b main
git -C "$repo" config user.name test
git -C "$repo" config user.email test@example.invalid
cp .gitattributes "$repo/.gitattributes"
cp scripts/qa/strict-baseline-merge.sh "$repo/merge.sh"
git -C "$repo" config merge.strict-baseline.driver "$repo/merge.sh %O %A %B"
cat > "$repo/docs/qa/strict-baseline.tsv" <<'EOF'
package	in_scope_errors
alpha	10
beta	10
shared	10
EOF
git -C "$repo" add .gitattributes docs/qa/strict-baseline.tsv
git -C "$repo" commit -qm base
git -C "$repo" switch -qc ours
cat > "$repo/docs/qa/strict-baseline.tsv" <<'EOF'
package	in_scope_errors
alpha	8
beta	10
shared	6
EOF
git -C "$repo" commit -qam 'lower alpha and shared'
git -C "$repo" switch -q main
git -C "$repo" switch -qc theirs
cat > "$repo/docs/qa/strict-baseline.tsv" <<'EOF'
package	in_scope_errors
alpha	10
beta	7
shared	4
EOF
git -C "$repo" commit -qam 'lower beta and shared'
git -C "$repo" switch -q ours
git -C "$repo" rebase theirs
cat > "$tmp/expected.tsv" <<'EOF'
package	in_scope_errors
alpha	8
beta	7
shared	4
EOF
diff -u "$tmp/expected.tsv" "$repo/docs/qa/strict-baseline.tsv"
echo 'strict-baseline-merge-test: pass'
