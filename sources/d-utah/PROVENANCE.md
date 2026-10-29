
### `d-utah/d-utah-local-civil-rules-2025.txt` — D. Utah Local Civil Rules (DUCivR)

- Source: https://www.utd.uscourts.gov/sites/utd/files/Civil%20Rules%20Final%202025.pdf (official site; 100 pages, sha256 c50080f9ff8d260e0ba8096a7bd5142525f24de16015a1a6202e2dae59f22435).
- Edition: Local Civil Rules (DUCivR), listed as "Civil (PDF - Effective December 1, 2025)" on https://www.utd.uscourts.gov/rules-practice; fetched with plain curl (no browser identity); robots.txt allows the path (Crawl-delay 10 honored).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 d7f0564385374e8a15106883fea1bde84e80db45469b1f664e97c034627b5fbd. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `d-utah/original/d-utah-local-civil-rules-2025.pdf` as served (sha256 c50080f9ff8d260e0ba8096a7bd5142525f24de16015a1a6202e2dae59f22435); extraction sha256 d7f0564385374e8a15106883fea1bde84e80db45469b1f664e97c034627b5fbd.

### `d-utah/dutah-local-criminal-rules-2025-12-01.txt` — D. Utah Local Criminal Rules (DUCrimR, effective December 1, 2025)

- Source: https://www.utd.uscourts.gov/sites/utd/files/Criminal%20Rules%20Final%202025.pdf (official site; 28 pages, sha256 688f5d4f237a536ad45bff7aace8dfcee5d93f60f88d84b48309b5fb09d68f52).
- Edition: Local Criminal Rules 'effective December 1, 2025' (DUCrimR 1-1), the file 'Criminal Rules Final 2025' linked at https://www.utd.uscourts.gov/rules-practice. robots.txt of www.utd.uscourts.gov read (Crawl-delay 10 honored where stated; no Disallow covers the fetched path); the judiciary-wide Privacy & Security Policy (https://www.uscourts.gov/privacy-security-policy) read: no prohibition of automated access. Fetched with curl and the identifying UA lit-lang-provenance/1.0, at most 1 request per 10 s.
- Criminal forum (lg-gddt): utd-criminal.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 fbc43db846d8439626a2e327e9cc446bb29bdbaf79b06707ca8121ea7b4e3ff2. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `d-utah/original/dutah-local-criminal-rules-2025-12-01.pdf` as served (sha256 688f5d4f237a536ad45bff7aace8dfcee5d93f60f88d84b48309b5fb09d68f52); extraction sha256 fbc43db846d8439626a2e327e9cc446bb29bdbaf79b06707ca8121ea7b4e3ff2.

### `d-utah/d-utah-local-civil-rules-2025-sectioned.txt` — D. Utah Local Civil Rules, sectioned

- Derived in the pod (lg-f3q8.70) from `d-utah/d-utah-local-civil-rules-2025.txt` (extract sha256 d7f0564385374e8a15106883fea1bde84e80db45469b1f664e97c034627b5fbd); nothing fetched. One `## DUCivR <n>` line is inserted before each rule's own body heading line (`DUCivR 1-1` through `DUCivR 83-6.7`, 61 rules); the table of contents is unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## DUCivR /d'`) restores the extract byte for byte. The extract drops the letters of some top-level subdivisions (e.g. `(a)` of DUCivR 7-1); cites name them as the rules' own cross-references do (`DUCivR 7-1(a)(4)`, `7-1(b)`, `7-1(e)(2)`).
- Upstream: https://www.utd.uscourts.gov/sites/utd/files/Civil%20Rules%20Final%202025.pdf, retrieved 2026-10-27, as recorded on the `d-utah/d-utah-local-civil-rules-2025.txt` entry this file is derived from.
- Edition: as the extract, "These rules are effective December 1, 2025" (DUCivR 1-1).
- Sectioned sha256 aace2552f63987a4f3bdbfd6493f9303e7dbc4a0a292b93db1af985f44bf1ce8 (the `path_sha256` pinned by `lit/utd/Lit.toml`).

### `d-utah/dutah-local-criminal-rules-2025-12-01-sectioned.txt` — D. Utah Local Rules of Criminal Practice, sectioned

- Derived in the pod (lg-gddt.24) from `d-utah/dutah-local-criminal-rules-2025-12-01.txt` (extract sha256 fbc43db846d8439626a2e327e9cc446bb29bdbaf79b06707ca8121ea7b4e3ff2); nothing fetched. In the body (after the table of contents), one marker line `## DUCrimR <n>` is inserted before each rule's own heading line `DUCrimR <n> <TITLE>` (DUCrimR 1-1 through 59-1, 29 rules); indented in-text references to a rule carry no marker. Rule text was not edited: deleting the marker lines (each `## DUCrimR ` line with its newline) restores the extract byte for byte.
- Source: as the extract `d-utah/dutah-local-criminal-rules-2025-12-01.txt`, https://www.utd.uscourts.gov/sites/utd/files/Criminal%20Rules%20Final%202025.pdf; Retrieved 2026-10-28 with the extract (this file was derived from it in the pod, not fetched itself).
- Edition: as the extract, "effective December 1, 2025" (DUCrimR 1-1).
- Sectioned sha256 355e82f2024bf67901d9bc0ce600a47b48df5ade326427b68c13cb6f318ebfba (the `path_sha256` pinned by `lit/utd-criminal/Lit.toml`).
