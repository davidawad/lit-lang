
### `ak-supreme/alaska-appellate-procedure.txt` — Alaska Rules of Appellate Procedure

- Source: https://courts.alaska.gov/rules/docs/app.pdf (official site; 56 pages, sha256 b5f480666b750db7d9409a3db7d3dc91e796738ee105ea63802d40a737ad5f0c).
- Edition: Alaska Rules of Court, Rules of Appellate Procedure, PDF (created 2025-07-24, amended through SCO 2035 eff. April 15, 2025) as linked from https://courts.alaska.gov/rules/index.htm, which on 2026-10-27 lists no recent changes to Appellate Procedure. Fetched with curl (robots.txt allows /rules/).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 a24cd1ccf4e2cbe159440e9717587f81778711b33997cf8e8c2c5d8283d3a6e4. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `ak-supreme/original/alaska-appellate-procedure.pdf` as served (sha256 b5f480666b750db7d9409a3db7d3dc91e796738ee105ea63802d40a737ad5f0c); extraction sha256 a24cd1ccf4e2cbe159440e9717587f81778711b33997cf8e8c2c5d8283d3a6e4.

### `ak-supreme/alaska-appellate-procedure-sectioned.txt` — Alaska Rules of Appellate Procedure, single-column and sectioned

- Derived in the pod (lg-f3q8.35) from `ak-supreme/alaska-appellate-procedure.txt` (extract sha256 a24cd1ccf4e2cbe159440e9717587f81778711b33997cf8e8c2c5d8283d3a6e4); nothing fetched. The extract is two-column `pdftotext -layout` text, so each page (form feed) is first split into its left column followed by its right column: the gutter is the page's most common start index (40-90) of text that follows a run of two or more spaces; a line is cut there when the preceding character is a space, otherwise at the nearest two-space run within 12 characters, and a page with fewer than five such starts is kept as is. Column text is otherwise unchanged (trailing spaces of left-column lines and leading spaces of right-column lines are dropped). Then one `## Rule <n>` line is inserted before each rule's own heading line in the body (from line 470 of the single-column text, Rule 101. through Rule 612., 87 rules): a line beginning `Rule <n>` followed by two or more spaces, or by a period and one space, and a title (not the running head `ALASKA COURT RULES`, and not a sentence opening with The, That, When, This, If, A, An or In). The table of contents is unmarked. Deleting the marker lines restores the single-column text; the column split is not byte-reversible to the extract.
- Source: as the extract `ak-supreme/alaska-appellate-procedure.txt`, https://courts.alaska.gov/rules/docs/app.pdf, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract; the latest amendment it states is SCO 2035, effective April 15, 2025.
- Sectioned sha256 98b14637ade4bd8b2ce42679338cf16575ff1be63cd074e2cfcfe7019bd31775 (the `path_sha256` pinned by `lit/ak-superior/Lit.toml` and `lit/ak-supreme/Lit.toml`).
