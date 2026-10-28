
### `id-district/ircp.txt` — Idaho Rules of Civil Procedure (I.R.C.P.)

- Source: https://isc.idaho.gov/rules-procedure/ircp (official site; fetched file sha256 ffb067680f553dda924e692ad3c94e4e9763867aea6c6d39a7866a6f86d4bb20).
- Edition: Idaho Rules of Civil Procedure as published on the Idaho Supreme Court rules page (page dated 2026-07-01; the Recent Amendments page https://isc.idaho.gov/rules-procedure/recent-amendments states all amendments are incorporated into the rule text on their effective date; fetched 2026-10-27).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); plain curl with User-Agent litgraph-rules-vendor/1.0; rule text extracted from the server-rendered page HTML by a script (idaho_extract.py) into one ## section per rule, unedited text.
- Extract sha256 3e2fe9d88270fd60b6c5cdfd1d1a04b5993afffb9a2da21cddfcedbc16341110. Raw extract, not yet sectioned into `## <ref>` form beyond any per-rule headings.
- Original: `id-district/original/ircp.html` as served (sha256 ffb067680f553dda924e692ad3c94e4e9763867aea6c6d39a7866a6f86d4bb20); extraction sha256 3e2fe9d88270fd60b6c5cdfd1d1a04b5993afffb9a2da21cddfcedbc16341110.

### `id-district/ircp-sectioned.txt` — Idaho Rules of Civil Procedure (page dated 2026-07-01), sectioned

- Derived in the pod (lg-f3q8.43) from `id-district/ircp.txt` (extract sha256 3e2fe9d88270fd60b6c5cdfd1d1a04b5993afffb9a2da21cddfcedbc16341110); nothing fetched. One bare `## I.R.C.P. <n>` line is inserted right after each rule's titled heading line `## I.R.C.P. <n>. <title>`: 113 markers, I.R.C.P. 1 through the last rule of the extract. Rule text was not edited: deleting the marker lines (`sed -E '/^## I\.R\.C\.P\. [0-9.]+$/d'`) restores the extract byte for byte.
- Edition: as the extract, page dated 2026-07-01.
- Sectioned sha256 7a82de244c1ee2726e46e9276dc0c82dd080f5c934f79b25f016ee8d170eb762 (the `path_sha256` pinned by `lit/id-district/Lit.toml`).
