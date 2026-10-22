#!/usr/bin/env bash
# Generate editors/helix/queries/litigate/*.scm from the canonical queries in
# editors/tree-sitter-litigate/queries, renaming captures to Helix's scopes
# (https://docs.helix-editor.com/themes.html#syntax-highlighting).
#
# Usage: editors/helix/sync-queries.sh [--check]
#   --check  exit 1 (and print a diff) when the committed copies are stale
#
# textobjects.scm is Helix-only and hand-written; it is not generated.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
canon="$here/../tree-sitter-litigate/queries"
dest="$here/queries/litigate"
check=0
[ "${1:-}" = "--check" ] && check=1

header() {
  printf '; Generated from editors/tree-sitter-litigate/queries/%s by\n' "$1"
  printf '; editors/helix/sync-queries.sh. Do not edit; edit the source and rerun.\n\n'
}

# Capture renames, most specific first (sed applies them in order).
highlights() {
  sed -E \
    -e 's/@keyword\.conditional\b/@keyword.control.conditional/g' \
    -e 's/@string\.escape\b/@constant.character.escape/g' \
    -e 's/@comment\.documentation\b/@comment.line.documentation/g' \
    -e 's/@number\b/@constant.numeric/g' \
    -e 's/@boolean\b/@constant.builtin.boolean/g' \
    -e 's/@module\b/@namespace/g' \
    -e 's/@type\.definition\b/@type/g' \
    -e 's/@property\b/@variable.other.member/g' \
    -e 's/@function\.call\b/@function/g'
}

indents() {
  # Helix: @indent indents a node's inner lines, @outdent the closing line.
  # It has no @indent.auto; strings and block comments keep their text.
  sed -E \
    -e 's/@indent\.begin\b/@indent/g' \
    -e 's/@indent\.branch @indent\.end\b/@outdent/g' \
    -e '/@indent\.auto/d'
}

locals() {
  # Helix highlights a reference with the scope named after its definition.
  sed -E \
    -e 's/@local\.definition\.parameter\b/@local.definition.variable.parameter/g' \
    -e 's/@local\.definition\.var\b/@local.definition.label/g' \
    -e 's/@local\.definition\.field\b/@local.definition.variable.other.member/g' \
    -e 's/@local\.definition\.import\b/@local.definition.variable/g'
}

render() {
  local file="$1" filter="$2"
  { header "$file"; "$filter" <"$canon/$file"; }
}

status=0
for pair in highlights.scm:highlights indents.scm:indents locals.scm:locals injections.scm:cat; do
  file="${pair%%:*}"
  filter="${pair#*:}"
  if [ "$check" = 1 ]; then
    if ! diff -u "$dest/$file" <(render "$file" "$filter"); then
      echo "sync-queries: $dest/$file is stale; run editors/helix/sync-queries.sh" >&2
      status=1
    fi
  else
    render "$file" "$filter" >"$dest/$file"
  fi
done
exit "$status"
