
### `mied/mied-local-rules.txt` — Local Rules, E.D. Michigan

- Source: https://www.mied.uscourts.gov/pdffiles/localRulesPackage.pdf (official site; 192 pages, sha256 a9083ec973436049c1158aa1046c6d842b83db70f7ee25089b5855ba59b13013).
- Edition: E.D. Mich. Local Rules, "Current as of January 1, 2026" as stated on the document and on https://www.mied.uscourts.gov/attorneys/localrules ("These rules are current as of January 1, 2026"). That page also lists "Amendments Effective November 1, 2026" (notice https://www.mied.uscourts.gov/pdffiles/notice_LocalRules_amendments_11-2026.pdf) and proposed amendments with comments due 11-18-26; neither is in force on 2026-10-27 and neither is vendored.
- Also the criminal local rules (lg-gddt): 'CRIMINAL RULES' part, from extract line 397.
- Criminal forum (lg-gddt): mied-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Fetched via curl (single document, first request succeeded, no block encountered).
- Extract sha256 a6a3df153ac0ec9f1d2b550d8fa3e041c0bf8dc1f990ec53246864862a8e03ae. Raw extract, not yet sectioned into `## <ref>` form.

- Fetch method: first fetched with curl and a desktop browser User-Agent before the no-browser-identity rule (PM 2026-10-27); re-fetched on 2026-10-27 with plain curl (default User-Agent, no browser headers): byte-identical (same sha256).
- Original: `mied/original/mied-local-rules.pdf` as served (sha256 a9083ec973436049c1158aa1046c6d842b83db70f7ee25089b5855ba59b13013); extraction sha256 a6a3df153ac0ec9f1d2b550d8fa3e041c0bf8dc1f990ec53246864862a8e03ae.

### `mied/mied-local-rules-civil-sectioned.txt` — E.D. Mich. Local Rules, civil and general rules sectioned

- Derived in the pod (lg-f3q8.60) from `mied/mied-local-rules.txt` (extract sha256 a6a3df153ac0ec9f1d2b550d8fa3e041c0bf8dc1f990ec53246864862a8e03ae); nothing fetched. One `## LR <n>` line is inserted right after each civil or general rule's own body heading line (`LR <n> <Title>`): 72 markers, LR 1.1 through LR 83.50. The cover, the table of contents, the admiralty rules (LR B.1, E.1), the criminal rules (LCrR) and the appendices are unmarked (LR 83.50's section runs to the end of the file). Rule text was not edited: deleting the marker lines (`sed '/^## LR /d'`) restores the extract byte for byte.
- Upstream: https://www.mied.uscourts.gov/pdffiles/localRulesPackage.pdf, retrieved 2026-10-27, as recorded on the `mied/mied-local-rules.txt` entry this file is derived from.
- Edition: as the extract, "Current as of January 1, 2026"; the amendments the court's page lists as effective November 1, 2026 are not vendored.
- Sectioned sha256 be63479120e603ab8db781adcb3548256f799a4d984e4109eeaa3c36dbe0d4d9 (the `path_sha256` pinned by `lit/mied/Lit.toml`).
