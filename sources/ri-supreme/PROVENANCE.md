
### `ri-supreme/ri-supreme-article-i-appellate-procedure.txt` — R.I. Supreme Court Rules, Article I (Appellate Procedure)

- Source: https://www.courts.ri.gov/Courts/SupremeCourt/Documents/Supreme-Rules-Article1.pdf (official site; 40 pages, sha256 9341c74f7a5dcad574e43e1a1903407da66d538b337958434d21ff6795ca715c).
- Edition: No edition line on the document (PDF created 2026-03-11); Article I Appellate Procedure, linked from https://www.courts.ri.gov/Legal-Resources/Pages/Supreme-Court-Rules.aspx, which states no edition or later amendment.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 4fd8e6536a471bd71754690978149e596583e3d8952e6ec341a03e5049eff749. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `ri-supreme/original/ri-supreme-article-i-appellate-procedure.pdf` as served (sha256 9341c74f7a5dcad574e43e1a1903407da66d538b337958434d21ff6795ca715c); extraction sha256 4fd8e6536a471bd71754690978149e596583e3d8952e6ec341a03e5049eff749.

### `ri-supreme/ri-supreme-article-i-appellate-procedure-sectioned.txt` — R.I. Supreme Court Rules, Article I, sectioned

- Derived in the pod (lg-f3q8.26) from `ri-supreme/ri-supreme-article-i-appellate-procedure.txt` (extract sha256 4fd8e6536a471bd71754690978149e596583e3d8952e6ec341a03e5049eff749); nothing fetched. One `## Art. I Rule <n>` line is inserted before each rule's own heading line (`Rule <n>. <Title>.`, some after a form feed): 41 markers, Rule 1 through Rule 36 with 12A, 18A, 18B, 26A and 26B. A wrapped cross-reference that begins a line (`Rule 28. Nothing in this rule ...`, inside Rule 16(c)) is not a heading and is unmarked. The `Art. I` prefix keeps these headings apart from the Superior Court's `## Rule <n>`. Rule text was not edited: deleting the marker lines (`sed '/^## Art. I Rule /d'`) restores the extract byte for byte.
- Edition: as the extract; every page is footed "Revised February 17, 2026".
- Sectioned sha256 b772b2aec643f4376e27912e04acb1c643d4714990851597fd4d8425f1df6e2f (the `path_sha256` pinned by `lit/ri-superior-civil/Lit.toml` and `lit/ri-supreme/Lit.toml`).
