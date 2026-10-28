
### `me-superior-civil/rules-of-civil-procedure.txt` — Maine Rules of Civil Procedure (rules only)

- Source: https://www.courts.maine.gov/rules/text/mr_civ_p_only_2026-06-01.pdf (official site; 286 pages, sha256 263f9b45c296db9ad9f3dbafabefcd6106373baedddce38e326ecde5387584d0).
- Edition: Maine Rules of Civil Procedure, rules only, 'Last reviewed and edited May 14, 2026; includes amendments effective June 1, 2026' (cover); listed as the current rules-only PDF on https://www.courts.maine.gov/rules/index.html (matches the 2026 Me. Rules 02 civil amendment effective 2026-06-01); fetched with curl (plain User-Agent), no bot refusal; courts.maine.gov robots.txt absent (404) and the site publishes no terms page (its footer links only the maine.gov accessibility page, which returned an unexplained 403 to curl and was not retried).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 9f815c01f1bc7bfacfdab53a5946093842d4f7af0ddf57785584f4e8d18290c7. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `me-superior-civil/original/rules-of-civil-procedure.pdf` as served (sha256 263f9b45c296db9ad9f3dbafabefcd6106373baedddce38e326ecde5387584d0); extraction sha256 9f815c01f1bc7bfacfdab53a5946093842d4f7af0ddf57785584f4e8d18290c7.

### `me-superior-civil/rules-of-civil-procedure-sectioned.txt` — Maine Rules of Civil Procedure, sectioned

- Derived in the pod (lg-f3q8.22) from `me-superior-civil/rules-of-civil-procedure.txt` (extract sha256 9f815c01f1bc7bfacfdab53a5946093842d4f7af0ddf57785584f4e8d18290c7); nothing fetched. One `## Rule <n>` line is inserted at each rule's own heading in the body (`RULE 1. SCOPE OF RULES` through `RULE 140. ELECTRONIC SERVICE`, 163 rules; the table of contents is unmarked): before the heading line, or right after it when the heading holds a colon (Rules 7, 37 and 59), since the corpus reader opens a section at a `RULE ...:` line. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Source: as the extract `me-superior-civil/rules-of-civil-procedure.txt`, https://www.courts.maine.gov/rules/text/mr_civ_p_only_2026-06-01.pdf, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, the rules-only edition including amendments effective June 1, 2026.
- Sectioned sha256 47ab6b9de401d759363fa2a911db3bb6d1f733089296e56b181e852e7e1e2950 (the `path_sha256` pinned by `lit/me-superior-civil/Lit.toml`).
