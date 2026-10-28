# `ndd` sources: provenance

### `ndd/local-rules.txt` — Local Rules, D.N.D.

- Source: https://www.ndd.uscourts.gov/lci/Local_Rules.pdf (official site; 92 pages, sha256 a5ac31739c05d89026e64d3656f9fc1fc8b1c6ceeecced44b33d9b3f84a5a119).
- Edition: Local Rules of the U.S. District Court for the District of North Dakota, "Effective October 6, 2025"; linked as "Local Rules" (with the "2025 Summary of Revisions") from https://www.ndd.uscourts.gov/court-info/local-rules-and-orders.
- Also the criminal local rules (lg-gddt): 'CRIMINAL RULES' part, from extract line 2741.
- Criminal forum (lg-gddt): ndd-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 f98554911b9c98b8e88105416993afb9799887105a7c64eb1b53d94cdd9c39c5. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `ndd/original/local-rules.pdf` as served (sha256 a5ac31739c05d89026e64d3656f9fc1fc8b1c6ceeecced44b33d9b3f84a5a119); extraction sha256 f98554911b9c98b8e88105416993afb9799887105a7c64eb1b53d94cdd9c39c5.

### `ndd/local-rules-civil-sectioned.txt` — D.N.D. Local Rules, civil rules sectioned

- Derived in the pod (lg-f3q8.10) from `ndd/local-rules.txt` (extract sha256 f98554911b9c98b8e88105416993afb9799887105a7c64eb1b53d94cdd9c39c5); nothing fetched. One `## Civ. L.R. <n>` line is inserted before each civil rule's own `CIVIL RULE <n>` heading line (`Civ. L.R. 3.1` through `Civ. L.R. 72.1`, 17 rules, the repealed Civ. L.R. 6.1 included). The general rules and the criminal rules are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## Civ\. L\.R\. /d'`) restores the extract byte for byte.
- Source: as the extract `ndd/local-rules.txt`, https://www.ndd.uscourts.gov/lci/Local_Rules.pdf, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, "Effective October 6, 2025".
- Sectioned sha256 2611310f064b74eb36bc64db419c121daeae343be9d36e8e0b910ba607fa693d (the `path_sha256` pinned by `lit/ndd/Lit.toml`).
