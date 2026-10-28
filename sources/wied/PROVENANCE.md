
### `wied/wied-local-rules.txt` — E.D. Wisconsin Local Rules

- Source: https://www.wied.uscourts.gov/sites/wied/files/documents/Local%20Rules%202010-0201-Amended%202026-0729.pdf (official site; 71 pages, sha256 51726fd2dac1b42fa8943d0ee0755b2a5acddc4b0e030a405f33a41af9c61628).
- Edition: Local Rules effective February 1, 2010 (as amended July 29, 2026) (https://www.wied.uscourts.gov/local-rules-and-guidance); fetched by curl.
- Also the criminal local rules (lg-gddt): 'PART C: CRIMINAL RULES' part, from extract line 2575.
- Criminal forum (lg-gddt): wied-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 4c48fe0b13a732aa32e8ea7f622113fbaf16b30386d475e19ec5d0b273337db8. Raw extract, not yet sectioned into `## <ref>` form.

- Fetch method: first fetched with curl and a desktop browser User-Agent before the no-browser-identity rule (PM 2026-10-27); re-fetched on 2026-10-27 with plain curl (its default User-Agent, no browser headers): byte-identical (same sha256).
- Original: `wied/original/wied-local-rules.pdf` as served (sha256 51726fd2dac1b42fa8943d0ee0755b2a5acddc4b0e030a405f33a41af9c61628); extraction sha256 4c48fe0b13a732aa32e8ea7f622113fbaf16b30386d475e19ec5d0b273337db8.

### `wied/wied-local-rules-sectioned.txt` — E.D. Wisconsin Local Rules, sectioned

- Derived in the pod (lg-f3q8.56) from `wied/wied-local-rules.txt` (extract sha256 4c48fe0b13a732aa32e8ea7f622113fbaf16b30386d475e19ec5d0b273337db8); nothing fetched. One `## General L. R. <n>`, `## Civil L. R. <n>` or `## Criminal L. R. <n>` line is inserted before each rule's own unindented heading line in the body (47 rules, General L. R. 1 through Criminal L. R. 58; `General L.R. 84` and `Criminal L.R. 5` are marked with the spaced form); the table of contents is unmarked. Rule text was not edited: deleting the marker lines (`sed -E '/^## (General|Civil|Criminal) L\. R\. /d'`) restores the extract byte for byte.
- Source: as the extract `wied/wied-local-rules.txt`, https://www.wied.uscourts.gov/sites/wied/files/documents/Local%20Rules%202010-0201-Amended%202026-0729.pdf, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, Local Rules effective February 1, 2010, as amended July 29, 2026.
- Sectioned sha256 c2afa26b772157af4f061f06d67a3828eb3da88bdf9c1e96d927bf8d99a9a472 (the `path_sha256` pinned by `lit/wied/Lit.toml`).

- Note (lg-gddt.18): `lit/wied-criminal` binds the Part C criminal rules through the existing `## Criminal L. R. <n>` markers of `wied/wied-local-rules-sectioned.txt` (sha256 c2afa26b772157af4f061f06d67a3828eb3da88bdf9c1e96d927bf8d99a9a472); the file is unchanged.
