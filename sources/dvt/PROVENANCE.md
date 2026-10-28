
### `dvt/local-rules.txt` — District of Vermont Local Rules of Procedure

- Source: https://www.vtd.uscourts.gov/sites/vtd/files/LocalRules-20260420.pdf (official site; 60 pages, sha256 8048666fab25b6a7f9e35bd7fd8f486cca1041c9d5d047fd81c89338630300fa).
- Edition: "April 20, 2026" as stated on the document; linked as "Local Rules of Procedure (Effective 4/20/2026)" from https://www.vtd.uscourts.gov/court-info/local-rules-and-orders.
- Also the criminal local rules (lg-gddt): 'II. CRIMINAL RULES' part, from extract line 1963.
- Criminal forum (lg-gddt): vtd-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 32fb779eee46f3f0a700856c56689ed4764fc01fe61df9cfa63de7103dda9e7d. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `dvt/original/local-rules.pdf` as served (sha256 8048666fab25b6a7f9e35bd7fd8f486cca1041c9d5d047fd81c89338630300fa); extraction sha256 32fb779eee46f3f0a700856c56689ed4764fc01fe61df9cfa63de7103dda9e7d.

### `dvt/local-rules-civil-sectioned.txt` — D. Vt. Local Rules of Procedure, civil rules sectioned

- Derived in the pod (lg-f3q8.52) from `dvt/local-rules.txt` (extract sha256 32fb779eee46f3f0a700856c56689ed4764fc01fe61df9cfa63de7103dda9e7d); nothing fetched. One `## L.R. <n>` line is inserted before each civil rule's own heading line in Part I (`L.R. 1` through `L.R. 86`, 40 rules, dotted numbers such as `L.R. 7.1` and `L.R. 16.1` kept; a heading after a page's form feed keeps its form feed on the rule line). The criminal rules (Part II) are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## L.R. /d'`) restores the extract byte for byte.
- Source: as the extract `dvt/local-rules.txt`, https://www.vtd.uscourts.gov/sites/vtd/files/LocalRules-20260420.pdf, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, April 20, 2026.
- Sectioned sha256 b1e5dc3351b3b000ac8c3adc91725d483afcb0abd84262c2833cd8daeff90938 (the `path_sha256` pinned by `lit/vtd/Lit.toml`).
