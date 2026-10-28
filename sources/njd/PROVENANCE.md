
### `njd/local-rules-2026-10-21.txt` — District of New Jersey Local Civil and Criminal Rules

- Source: https://www.njd.uscourts.gov/sites/njd/files/CompleteLocalRules.pdf (official site; 124 pages, sha256 d77ecfd388077db5fa8ba0ff098751a9e4dd9ad146e7f0fa8dac46ee2597e22e).
- Edition: Local Civil and Criminal Rules, "With Revisions as of October 21, 2026", as linked "Local Rules - Updated 10/21/2026" on https://www.njd.uscourts.gov/local-rules-and-appendices (retrieved 2026-10-27). Page https://www.njd.uscourts.gov/orders-amending-local-rules lists two October 2026 amending orders (L.Civ.R. 5.2, 5.3) with no linked file; the consolidated edition is dated 10/21/2026. Appendices and proposed amendments not vendored.
- Also the criminal local rules (lg-gddt): 'LOCAL CRIMINAL RULES' part, from extract line 5050.
- Criminal forum (lg-gddt): njd-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 d4b254a8b2ae57299bba4bb8b0c850a0b83182536d9f11f51a5c85ca1157e8d5. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `njd/original/local-rules-2026-10-21.pdf` as served (sha256 d77ecfd388077db5fa8ba0ff098751a9e4dd9ad146e7f0fa8dac46ee2597e22e); extraction sha256 d4b254a8b2ae57299bba4bb8b0c850a0b83182536d9f11f51a5c85ca1157e8d5.

### `njd/local-rules-2026-10-21-sectioned.txt` — D.N.J. Local Civil and Criminal Rules, civil rules sectioned

- Derived in the pod (lg-f3q8.74) from `njd/local-rules-2026-10-21.txt` (extract sha256 d4b254a8b2ae57299bba4bb8b0c850a0b83182536d9f11f51a5c85ca1157e8d5); nothing fetched. One `## Rule <n>` line is inserted right after each Local Civil Rule's own body heading line `Civ. RULE <n> <title>` (or `CIV. RULE`, some after a form feed; Rule 9.3's heading reads `L. Civ. R. 9.3`): 85 markers, Rule 1.1 through Rule 601.7. The table of contents is unmarked; the Local Criminal Rules carry their own markers (below). Rule text was not edited: deleting the marker lines (`## Rule <n>` plus its newline) restores the extract byte for byte.
- Source: as the extract `njd/local-rules-2026-10-21.txt`, https://www.njd.uscourts.gov/sites/njd/files/CompleteLocalRules.pdf, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, the Local Civil and Criminal Rules with revisions as of October 21, 2026.
- Criminal rules (lg-gddt.10): one `## L.Cr.R. <n>` line is also inserted right after each Local Criminal Rule's own body heading line `Cr. RULE <n> <title>` (or `Cr. Rule`, some after a form feed): 16 markers, L.Cr.R. 1.1 through L.Cr.R. 101.1; the table of contents is unmarked. Deleting both kinds of marker line (`## Rule <n>` and `## L.Cr.R. <n>`, each plus its newline) restores the extract byte for byte.
- Sectioned sha256 009dd759ad0ef2daf6649cdb3fbe090876e95e62c0bed8a124cc166f21716a6f (the `path_sha256` pinned by `lit/njd/Lit.toml` and `lit/njd-criminal/Lit.toml`).
