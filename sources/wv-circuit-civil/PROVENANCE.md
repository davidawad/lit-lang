
### `wv-circuit-civil/wv-rules-of-civil-procedure.txt` — West Virginia Rules of Civil Procedure

- Source: https://www.courtswv.gov/legal-community/court-rules/rules-civil-procedure-contents (official site; 148 pages, sha256 39d8884c3cd2ae464b5a304a459f49582550e99260d3a2841b8537046ce7e78b).
- Edition: West Virginia Rules of Civil Procedure as posted on the single page https://www.courtswv.gov/legal-community/court-rules/rules-civil-procedure-contents (no edition line on the page; the court recent-rules-orders list shows no Rules of Civil Procedure amendment in 2026). PDF is a browser print-to-PDF of that page, taken 2026-10-27; the site terms say web copies are not the official versions.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 35a298b32bb41b2e8d2b80e20b0a05186bc02aa01504837d6da5f63149fd8b14. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `wv-circuit-civil/original/wv-rules-of-civil-procedure.pdf` as served (sha256 39d8884c3cd2ae464b5a304a459f49582550e99260d3a2841b8537046ce7e78b); extraction sha256 35a298b32bb41b2e8d2b80e20b0a05186bc02aa01504837d6da5f63149fd8b14.

### `wv-circuit-civil/wv-rules-of-civil-procedure-sectioned.txt` — West Virginia Rules of Civil Procedure, sectioned

- Derived in the pod (lg-f3q8.32) from `wv-circuit-civil/wv-rules-of-civil-procedure.txt` (extract sha256 35a298b32bb41b2e8d2b80e20b0a05186bc02aa01504837d6da5f63149fd8b14); nothing fetched. One `## Rule <n>` line is inserted right before each rule's own body heading line `Rule <n>. <Title>` (some after a form feed; Rules 4.1, 71.1 and 71.2 have no period after the number): 96 markers, Rule 1 through Rule 87. The table of contents is unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Upstream: https://www.courtswv.gov/legal-community/court-rules/rules-civil-procedure-contents, retrieved 2026-10-27, as recorded on the `wv-circuit-civil/wv-rules-of-civil-procedure.txt` entry this file is derived from.
- Edition: as the extract, the rules as posted on 2026-10-27 (no edition line on the page).
- Sectioned sha256 796b579132bd760ffb977ae427fdb1b34bbfa34d85804a186f6750e9fd6d15f1 (the `path_sha256` pinned by `lit/wv-circuit-civil/Lit.toml` and `lit/wv-rules/Lit.toml`).
