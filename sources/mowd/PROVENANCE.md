# `mowd` sources: provenance

### `mowd/local-rules.txt` — Local Rules, W.D. Mo.

- Source: https://www.mow.uscourts.gov/sites/mow/files/DC-Local_Rules.pdf (official site; 64 pages, sha256 fa1808ed362b08ed336b5d82a3da2e328d71a449b8a5cf66a43ec23d2b8c9e5c).
- Edition: Local Rules of the U.S. District Court for the Western District of Missouri, "Adopted by the Court, Effective July 9, 2026"; linked as "Local Rules Complete Copy" from https://www.mow.uscourts.gov/district/rules.
- Also the criminal local rules (lg-gddt): 'LOCAL CRIMINAL RULES' part, from extract line 2415.
- Criminal forum (lg-gddt): mowd-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 f4f96de42a597d8d27eb170b1fd053a6cb127eb38da60519518658a9a42c234d. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `mowd/original/local-rules.pdf` as served (sha256 fa1808ed362b08ed336b5d82a3da2e328d71a449b8a5cf66a43ec23d2b8c9e5c); extraction sha256 f4f96de42a597d8d27eb170b1fd053a6cb127eb38da60519518658a9a42c234d.

### `mowd/local-rules-sectioned.txt` — Local Rules, W.D. Mo., sectioned

- Derived in the pod (lg-gddt.20) from `mowd/local-rules.txt` (extract sha256 f4f96de42a597d8d27eb170b1fd053a6cb127eb38da60519518658a9a42c234d); nothing fetched. One `## L.R. <n>` line is inserted before each rule's own heading line in the body (`<n>   <TITLE IN CAPITALS>`, some after a page's form feed, which stays on the heading line after the marker): `L.R. 1.1` through `L.R. 99.10`, 54 rules, the Local Civil Rules and the Local Criminal Rules (99.0-99.10). The table of contents is unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## L\.R\. /d'`) restores the extract byte for byte.
- Edition: as the extract, "Adopted by the Court, Effective July 9, 2026".
- Sectioned sha256 15fb1a9d0b6573ed350593a6765e760856defcf7958ed44733ec489b80662a4b (the `path_sha256` pinned by `lit/mowd-criminal/Lit.toml`).
