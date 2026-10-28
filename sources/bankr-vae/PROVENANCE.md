
### `bankr-vae/vae-local-rules-2025-12-01.txt` — E.D. Va. Local Bankruptcy Rules (ver 11/26/2025, effective December 1, 2025)

- Source: https://www.vaeb.uscourts.gov/sites/vaeb/files/LBR%20%20changes%2012-1-2025.pdf (official site; 239 pages, sha256 34ffcae9a88cbb8968e324661c25e8cca6b7e6595c7622702a70935bb75f0388).
- Edition: Local Bankruptcy Rules 'ver 11/26/2025', posted 12/11/2025, effective 12/01/2025 (title page and listing), the current entry at https://www.vaeb.uscourts.gov/vaeb-local-rules; Standing Orders 20-2 and 20-3 listed there (February 2020) precede it and are not vendored. robots.txt (Crawl-delay 10 honored) and the court's privacy/security policy page (https://www.vaeb.uscourts.gov/privacy-policy/) checked: no prohibition of automated access.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 fbaa52c5f9f6f0f8acbfeb05bdec3702df11af20a2c062e614c0ebfdf94e7d2d. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-vae/original/vae-local-rules-2025-12-01.pdf` as served (sha256 34ffcae9a88cbb8968e324661c25e8cca6b7e6595c7622702a70935bb75f0388); extraction sha256 fbaa52c5f9f6f0f8acbfeb05bdec3702df11af20a2c062e614c0ebfdf94e7d2d.

### `bankr-vae/vae-local-rules-2025-12-01-sectioned.txt` — E.D. Va. Local Bankruptcy Rules, sectioned

- Derived in the pod (lg-u561.15) from `bankr-vae/vae-local-rules-2025-12-01.txt` (extract sha256 fbaa52c5f9f6f0f8acbfeb05bdec3702df11af20a2c062e614c0ebfdf94e7d2d); nothing fetched. One `## Rule <n>` line is inserted before the first body heading line of each rule (`RULE 1001-1` through `RULE 9070-1`, 78 rules, including repealed rules, 2015-(a)-1, 3002.1-1 and 4001(a)-1; extract lines 550-6688); running page headers (a form feed, then the rule title), the table of contents and the Exhibits are unmarked, and a repealed rule printed before its replacement under the same number shares that number's section. Two markers carry the rule number as the cite grammar reads it: `## Rule 2015-` heads RULE 2015-(a)-1 and `## Rule 4001` heads RULE 4001(a)-1 (no other rule is numbered 2015- or 4001). Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Edition: as the extract, ver 11/26/2025, effective December 1, 2025.
- Sectioned sha256 606652fba6f5d0bcc42224d061cc9805d995790aa6bd3f38af8dc94845390eca (the `path_sha256` pinned by `lit/bankr-vae/Lit.toml`).
