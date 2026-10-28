
### `fl-state-appellate/fl-rules-appellate-procedure-2026-10-01.txt` — Florida Rules of Appellate Procedure

- Source: https://www-media.floridabar.org/uploads/2026/10/Appellate-Court-Rules-10-01-26.pdf (Chapter 9, linked from https://www.floridabar.org/rules/ctproc/) (official site; 245 pages, sha256 7b462cb689ec328b5138c6693c164c0a81adbe5914f02cd88f7bc7d87f1a7042).
- Edition: Florida Rules of Appellate Procedure, the Florida Bar's Oct 1, 2026 edition (file name 10-01-26).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 6bb31e827e889b22082a0fda4e4cd23694d7d583e4e9565458134baae501dcaf. Raw extract, not yet sectioned into `## <ref>` form.
- Fetch method: BrowserOS neo; site served a bot challenge to plain automated requests (CoS ruling 2026-10-27). floridabar.org robots.txt disallows only /private/ for general agents (the www-media host serves none); the Disclaimer & Terms of Use page bars no automated access. flcourts.gov hosts were not touched.
- Original: `fl-state-appellate/original/fl-rules-appellate-procedure-2026-10-01.pdf` as served (sha256 7b462cb689ec328b5138c6693c164c0a81adbe5914f02cd88f7bc7d87f1a7042); extraction sha256 6bb31e827e889b22082a0fda4e4cd23694d7d583e4e9565458134baae501dcaf.

### `fl-state-appellate/fl-rules-appellate-procedure-2026-10-01-sectioned.txt` — Florida Rules of Appellate Procedure, sectioned

- Derived in the pod (lg-f3q8.14) from `fl-state-appellate/fl-rules-appellate-procedure-2026-10-01.txt` (extract sha256 6bb31e827e889b22082a0fda4e4cd23694d7d583e4e9565458134baae501dcaf); nothing fetched. One `## Rule <n>` line is inserted before each rule's own heading line in the body (`RULE 9.010. EFFECTIVE DATE; SCOPE; ...` through `RULE 9.900. FORMS`, 55 rules; the table of contents is unmarked). Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Source: as the extract `fl-state-appellate/fl-rules-appellate-procedure-2026-10-01.txt`, https://www-media.floridabar.org/uploads/2026/10/Appellate-Court-Rules-10-01-26.pdf, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, the Florida Bar's October 1, 2026 edition.
- Sectioned sha256 529ece835412697830636b5054b02b830048cf561f1f99edc4298967df7369d4 (the `path_sha256` pinned by `lit/fl-state-civil/Lit.toml` and `lit/fl-district-court-of-appeal/Lit.toml`).
