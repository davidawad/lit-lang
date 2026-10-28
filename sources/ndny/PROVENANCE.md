
### `ndny/local-rules.txt` — Northern District of New York Local Rules of Practice

- Source: https://www.nynd.uscourts.gov/sites/nynd/files/local_rules/Local%20Rules%202026_Final.pdf (official site; 165 pages, sha256 707fdaa7e094b33f33ee9a44753473626949234f550a0e726ac9a9c2eb9dea96).
- Edition: "Effective JANUARY 1, 2026" as stated on the document, linked as Current Local Rules from https://www.nynd.uscourts.gov/content/current-local-rules.
- Also the criminal local rules (lg-gddt): Section XI, Criminal Procedure (contents entry at extract line 414; the section follows the civil sections).
- Criminal forum (lg-gddt): nynd-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 30c9407031855fe3d43b79e137ce81312c4ef9b301c76f0c2f39b5da92712850. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `ndny/original/local-rules.pdf` as served (sha256 707fdaa7e094b33f33ee9a44753473626949234f550a0e726ac9a9c2eb9dea96); extraction sha256 30c9407031855fe3d43b79e137ce81312c4ef9b301c76f0c2f39b5da92712850.

### `ndny/local-rules-criminal-sectioned.txt` — N.D.N.Y. Local Rules of Practice, Section XI (Criminal Procedure) sectioned

- Derived in the pod (lg-gddt.8) from `ndny/local-rules.txt` (extract sha256 30c9407031855fe3d43b79e137ce81312c4ef9b301c76f0c2f39b5da92712850); nothing fetched. In the body of Section XI (from its first rule heading `1.1    Scope of the Rules`, extract line 4394, to the `SECTION XII.` heading), one `## L.R. Cr. P. <n>` line is inserted before each rule's own heading line `<n>   <title>` (30 rules, 1.1 through 59.1, matching the section's contents list; the reserved `THROUGH` ranges are unmarked). The rules cite themselves "L. R. Cr. P." (L.R. Cr. P. 1.1). The civil and other sections and the tables of contents are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## L\.R\. Cr\. P\. /d'`) restores the extract byte for byte.
- Edition: as the extract, effective January 1, 2026.
- Upstream: https://www.nynd.uscourts.gov/sites/nynd/files/local_rules/Local%20Rules%202026_Final.pdf, retrieved 2026-10-27, as recorded on `ndny/local-rules.txt` above; sectioning was done from the vendored extract.
- Sectioned sha256 2091bc964cb5ef05e8342415bf088cb800b402c30c1dfeeaf9460832c56d1a02 (the `path_sha256` pinned by `lit/nynd-criminal/Lit.toml`).

### `ndny/local-rules-civil-sectioned.txt` — N.D.N.Y. Local Rules of Practice, civil rules sectioned

- Derived in the pod (lg-f3q8.51) from `ndny/local-rules.txt` (extract sha256 30c9407031855fe3d43b79e137ce81312c4ef9b301c76f0c2f39b5da92712850), which was retrieved 2026-10-27 from https://www.nynd.uscourts.gov/sites/nynd/files/local_rules/Local%20Rules%202026_Final.pdf as recorded above; nothing fetched in the pod. Method: one `## L.R. <n>` line is inserted before each civil rule's own heading line in Sections I-X (L.R. 1.1 through 86.1, 154 rules, dotted numbers such as `5.1.1` and `65.1.1` kept); the per-section contents lines, including four whose leaders wrapped (5.1.4, 50.1, 56.2, 83.9), are unmarked, as are the criminal, Social Security, habeas corpus, admiralty and patent rules (Sections XI-XV). Rule text was not edited: deleting the marker lines (`sed '/^## L.R. /d'`) restores the extract byte for byte.
- Edition: as the extract, effective January 1, 2026.
- Sectioned sha256 46720686bf917452128290578c6850c859575a83cc016741fefd51b52221e047 (the `path_sha256` pinned by `lit/nynd/Lit.toml`).
