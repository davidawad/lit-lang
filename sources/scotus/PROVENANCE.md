
### `scotus/scotus-rules-2026.txt` — Rules of the Supreme Court of the United States (2026)

- Source: https://www.supremecourt.gov/filingandrules/2026RulesoftheCourt_WEB.pdf (official site; 86 pages, sha256 5200e38d590cc40929cb719987c936dc529717b4037259d1b63a2309cb6b0f32).
- Edition: Rules of the Supreme Court of the United States, adopted February 17, 2026, effective March 16, 2026 (as stated on the PDF cover; linked from https://www.supremecourt.gov/filingandrules/rules_guidance.aspx as "Rules of the Supreme Court (Effective March 16, 2026)"). Fetched with curl..
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 523575139ae9b3ead23b555c4926104e007c89974ec36550b4485b96feedbef5. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `scotus/original/scotus-rules-2026.pdf` as served (sha256 5200e38d590cc40929cb719987c936dc529717b4037259d1b63a2309cb6b0f32); extraction sha256 523575139ae9b3ead23b555c4926104e007c89974ec36550b4485b96feedbef5.

### `scotus/scotus-rules-2026-sectioned.txt` — Rules of the Supreme Court of the United States (2026), sectioned

- Derived in the pod (lg-f3q8.63) from `scotus/scotus-rules-2026.txt` (extract sha256 523575139ae9b3ead23b555c4926104e007c89974ec36550b4485b96feedbef5); nothing fetched. The extract is single-column, so no column split is made. After the table of contents (from line 115 of the extract), one `## Sup. Ct. R. <n>` line is inserted before each of the 48 rule heading lines (a line opening `Rule <n>.`, then spaces and a capitalized title word, taking only the next rule number in sequence, so that cross-references such as `Rule 33.1, except ...` opening a line stay unmarked), and one `## Sup. Ct. R. <n>.<m>` line before each numbered paragraph of that rule (a line opening with one to six spaces, `<m>.` and a space, taking only the next paragraph number in sequence within the rule). Text is otherwise unchanged: deleting the marker lines (`sed '/^## Sup. Ct. R. /d'`) restores the extract byte for byte.
- Edition: as the extract, Rules of the Supreme Court of the United States, adopted February 17, 2026, effective March 16, 2026.
- Sectioned sha256 88c34b071eb21c9c25ba6693dfe7a3213d8e8e57ff30db9715f3f63c15119853 (the `path_sha256` pinned by `lit/scotus/Lit.toml`).
