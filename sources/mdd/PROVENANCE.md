
### `mdd/mdd-local-rules.txt` — U.S. District Court for the District of Maryland Local Rules

- Source: https://www.mdd.uscourts.gov/sites/mdd/files/LocalRules.pdf (official site; 157 pages, sha256 0548bdddab14f278b80ad91c2625837e65ad06ddfe9613107a72a0121174cae9).
- Edition: Local Rules, December 1, 2025 (cover date; page https://www.mdd.uscourts.gov/local-rules links it and lists adopted amendments effective December 1, 2025 as the latest; also governs the District of Maryland only).
- Also the criminal local rules (lg-gddt): 'II. CRIMINAL' part, from extract line 2241.
- Criminal forum (lg-gddt): mdd-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 4d55d4b343dd7975aebea4b9351b831f2a7ff7e050e72e1e36c490de7dcbacdd. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `mdd/original/mdd-local-rules.pdf` as served (sha256 0548bdddab14f278b80ad91c2625837e65ad06ddfe9613107a72a0121174cae9); extraction sha256 4d55d4b343dd7975aebea4b9351b831f2a7ff7e050e72e1e36c490de7dcbacdd.

### `mdd/mdd-local-rules-sectioned.txt` — D. Md. Local Rules, sectioned

- Derived in the pod (lg-f3q8.28) from `mdd/mdd-local-rules.txt` (extract sha256 4d55d4b343dd7975aebea4b9351b831f2a7ff7e050e72e1e36c490de7dcbacdd); nothing fetched. One `## Local Rule <n>` line is inserted before each rule's own heading line in the body (`R ULE 101. C OUNSEL` through `R ULE 806.`) and one `## Local Rule <n>.<m>` line before each numbered part of a rule (`2.       Filing Schedule` under Rule 105 is `## Local Rule 105.2`), 246 markers; the table of contents, the Local Admiralty Rules and the appendices are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## Local Rule /d'`) restores the extract byte for byte.
- Edition: as the extract, Local Rules dated December 1, 2025.
- Sectioned sha256 287ac2420ed78aaff0f6d8b3d6c081393df5111e19f4180b88a054b951e286b4 (the `path_sha256` pinned by `lit/mdd/Lit.toml`).
