
### `id-supreme/iar.txt` — Idaho Appellate Rules (I.A.R.)

- Source: https://isc.idaho.gov/rules-procedure/iar (official site; fetched file sha256 2f02140390fc0a619208872c1128fc132c8f8908f17235d601bc0c224dcfb3c0).
- Edition: Idaho Appellate Rules as published on the Idaho Supreme Court rules page (page dated 2026-07-01; the Recent Amendments page https://isc.idaho.gov/rules-procedure/recent-amendments states all amendments are incorporated into the rule text on their effective date; fetched 2026-10-27).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); plain curl with User-Agent litgraph-rules-vendor/1.0; rule text extracted from the server-rendered page HTML by a script (idaho_extract.py) into one ## section per rule, unedited text.
- Extract sha256 5b0a7876798906733a5294b201ac34a52ef53e2a50d50f4e56951e80c596dad4. Raw extract, not yet sectioned into `## <ref>` form beyond any per-rule headings.
- Original: `id-supreme/original/iar.html` as served (sha256 2f02140390fc0a619208872c1128fc132c8f8908f17235d601bc0c224dcfb3c0); extraction sha256 5b0a7876798906733a5294b201ac34a52ef53e2a50d50f4e56951e80c596dad4.

### `id-supreme/iar-sectioned.txt` — Idaho Appellate Rules (page dated 2026-07-01), sectioned

- Derived in the pod (lg-f3q8.43) from `id-supreme/iar.txt` (extract sha256 5b0a7876798906733a5294b201ac34a52ef53e2a50d50f4e56951e80c596dad4); nothing fetched. One bare `## I.A.R. <n>` line is inserted right after each rule's titled heading line `## I.A.R. <n>. <title>` (`I.A.R. 34-1` keeps its hyphen): 86 markers. Rule text was not edited: deleting the marker lines (`sed -E '/^## I\.A\.R\. [0-9.-]+$/d'`) restores the extract byte for byte.
- Edition: as the extract, page dated 2026-07-01.
- Sectioned sha256 b98cd1e96172263e1b994535e8b5a3250c13af10b95a0e2f5e6c52f55969f18e (the `path_sha256` pinned by `lit/id-district/Lit.toml` and `lit/id-supreme/Lit.toml`).
