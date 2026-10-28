
### `fed-dguam/guam-civil-local-rules.txt` — D. Guam Civil Local Rules of Practice

- Source: https://www.gud.uscourts.gov/sites/gud/files/civil_rules_effective_20190722_0.pdf (official site; 28 pages, sha256 1fb1637ce3f48ddb24269a09105d40de3489e229a38cd3c63a6515e8da6a887c).
- Edition: D. Guam Civil Local Rules of Practice (CVLR), effective July 22, 2019, linked as "Civil Rules" from https://www.gud.uscourts.gov/Local_Rules_of_Practice (General local rules GNLR not vendored). Fetched with curl (robots.txt crawl-delay 10; one request).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 0ad55d79c5ef263185042c32bd43a1cbe09cb6251a37c5e405507fdb74f625c8. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `fed-dguam/original/guam-civil-local-rules.pdf` as served (sha256 1fb1637ce3f48ddb24269a09105d40de3489e229a38cd3c63a6515e8da6a887c); extraction sha256 0ad55d79c5ef263185042c32bd43a1cbe09cb6251a37c5e405507fdb74f625c8.

### `fed-dguam/guam-civil-local-rules-sectioned.txt` — D. Guam Civil Local Rules of Practice, sectioned

- Derived in the pod (lg-f3q8.41) from `fed-dguam/guam-civil-local-rules.txt` (extract sha256 0ad55d79c5ef263185042c32bd43a1cbe09cb6251a37c5e405507fdb74f625c8); nothing fetched. One `## CVLR <n>` line is inserted before each rule's own heading line (`CVLR 1` through `CVLR 79`, 25 rules; a heading after a page's form feed keeps its form feed); the in-text references at the start of a line (`CVLR 56 attached ...`, `CVLR37(c), ...`) are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## CVLR /d'`) restores the extract byte for byte.
- Edition: as the extract, effective July 22, 2019.
- Sectioned sha256 cc6ad77f119104f442bc2a1d17885dd456acf931f0f7bf69113819f58b241d38 (the `path_sha256` pinned by `lit/gud/Lit.toml`).
