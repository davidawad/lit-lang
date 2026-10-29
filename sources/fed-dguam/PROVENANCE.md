
### `fed-dguam/guam-civil-local-rules.txt` — D. Guam Civil Local Rules of Practice

- Source: https://www.gud.uscourts.gov/sites/gud/files/civil_rules_effective_20190722_0.pdf (official site; 28 pages, sha256 1fb1637ce3f48ddb24269a09105d40de3489e229a38cd3c63a6515e8da6a887c).
- Edition: D. Guam Civil Local Rules of Practice (CVLR), effective July 22, 2019, linked as "Civil Rules" from https://www.gud.uscourts.gov/Local_Rules_of_Practice (General local rules GNLR not vendored). Fetched with curl (robots.txt crawl-delay 10; one request).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 0ad55d79c5ef263185042c32bd43a1cbe09cb6251a37c5e405507fdb74f625c8. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `fed-dguam/original/guam-civil-local-rules.pdf` as served (sha256 1fb1637ce3f48ddb24269a09105d40de3489e229a38cd3c63a6515e8da6a887c); extraction sha256 0ad55d79c5ef263185042c32bd43a1cbe09cb6251a37c5e405507fdb74f625c8.

### `fed-dguam/guam-civil-local-rules-sectioned.txt` — D. Guam Civil Local Rules of Practice, sectioned

- Derived in the pod (lg-f3q8.41) from `fed-dguam/guam-civil-local-rules.txt` (extract sha256 0ad55d79c5ef263185042c32bd43a1cbe09cb6251a37c5e405507fdb74f625c8); nothing fetched. One `## CVLR <n>` line is inserted before each rule's own heading line (`CVLR 1` through `CVLR 79`, 25 rules; a heading after a page's form feed keeps its form feed); the in-text references at the start of a line (`CVLR 56 attached ...`, `CVLR37(c), ...`) are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## CVLR /d'`) restores the extract byte for byte.
- Source: as the extract `fed-dguam/guam-civil-local-rules.txt`, https://www.gud.uscourts.gov/sites/gud/files/civil_rules_effective_20190722_0.pdf, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, effective July 22, 2019.
- Sectioned sha256 cc6ad77f119104f442bc2a1d17885dd456acf931f0f7bf69113819f58b241d38 (the `path_sha256` pinned by `lit/gud/Lit.toml`).

### `fed-dguam/guam-criminal-local-rules-2020-06-24.txt` — D. Guam Criminal Local Rules (effective June 24, 2020)

- Source: https://www.gud.uscourts.gov/sites/gud/files/crim_rules_final_2020624%20%281%29.pdf (official site; 12 pages, sha256 dc3865f487a195add04c1a85c01be4f4713ce08ce31439ac69591beca54409f3).
- Edition: Criminal Local Rules 'Effective June 24, 2020' (title page), linked as 'Criminal Rules' at https://www.gud.uscourts.gov/Local_Rules_of_Practice. robots.txt of www.gud.uscourts.gov read (Crawl-delay 10 honored where stated; no Disallow covers the fetched path); the judiciary-wide Privacy & Security Policy (https://www.uscourts.gov/privacy-security-policy) read: no prohibition of automated access. Fetched with curl and the identifying UA lit-lang-provenance/1.0, at most 1 request per 10 s.
- Criminal forum (lg-gddt): gud-criminal.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 5b60cc27cd83a049b2b1fe42994899367d64176dc476a9ff46253a3602c532e2. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `fed-dguam/original/guam-criminal-local-rules-2020-06-24.pdf` as served (sha256 dc3865f487a195add04c1a85c01be4f4713ce08ce31439ac69591beca54409f3); extraction sha256 5b60cc27cd83a049b2b1fe42994899367d64176dc476a9ff46253a3602c532e2.

### `fed-dguam/guam-general-local-rules-2022-04-19.txt` — D. Guam General Local Rules (effective April 19, 2022)

- Source: https://www.gud.uscourts.gov/sites/gud/files/General%20Local%20Rules%2020220419%20final_0.pdf (official site; 24 pages, sha256 d6fe7075bb7bba215134fbcd423d861f566c9c2f84b05f0077af6dca20ed38df).
- Edition: General Rules of the District Court of Guam 'Effective April 19, 2022' (GNLR), linked as 'General Rules' at https://www.gud.uscourts.gov/Local_Rules_of_Practice (they govern civil and criminal practice). robots.txt of www.gud.uscourts.gov read (Crawl-delay 10 honored where stated; no Disallow covers the fetched path); the judiciary-wide Privacy & Security Policy (https://www.uscourts.gov/privacy-security-policy) read: no prohibition of automated access. Fetched with curl and the identifying UA lit-lang-provenance/1.0, at most 1 request per 10 s.
- Criminal forum (lg-gddt): gud-criminal.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 dc30487e747a22f350d227e5d23ece811e457c1902903ade12be538dfcf54328. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `fed-dguam/original/guam-general-local-rules-2022-04-19.pdf` as served (sha256 d6fe7075bb7bba215134fbcd423d861f566c9c2f84b05f0077af6dca20ed38df); extraction sha256 dc30487e747a22f350d227e5d23ece811e457c1902903ade12be538dfcf54328.

### `fed-dguam/guam-criminal-local-rules-2020-06-24-sectioned.txt` — D. Guam Criminal Local Rules (effective June 24, 2020), sectioned

- Derived in the pod (lg-gddt.22) from `fed-dguam/guam-criminal-local-rules-2020-06-24.txt` (extract sha256 5b60cc27cd83a049b2b1fe42994899367d64176dc476a9ff46253a3602c532e2); nothing fetched. One marker line `## CRLR <n>` is inserted before each rule's own heading line `CRLR <n> <Title>` (some led by a form feed; CRLR 1 through 59, 23 rules). Every other line is the extract unchanged, so deleting the marker lines gives back the extract byte for byte.
- Source: as the extract `fed-dguam/guam-criminal-local-rules-2020-06-24.txt`, https://www.gud.uscourts.gov/sites/gud/files/crim_rules_final_2020624%20%281%29.pdf; Retrieved 2026-10-28 with the extract (this file was derived from it in the pod, not fetched itself).
- Edition: as the extract, effective June 24, 2020.
- Sectioned sha256 a0d77161409962ca732bb89b0a5278e20b723fd8ca71c3d409a62282c92166e9 (the `path_sha256` pinned by `lit/gud-criminal/Lit.toml`).
