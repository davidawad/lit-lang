
### `fed-ca9-bap/bap-rules.txt` — Ninth Circuit BAP Rules

- Source: https://cdn.ca9.uscourts.gov/datastore/bap/2015/06/18/baprules.pdf (official site; 14 pages, sha256 5981518c291767df123476c621db8286c0443438c773e77b65277c17ba17a9e7).
- Edition: Rules of the United States Bankruptcy Appellate Panel of the Ninth Circuit; https://www.bap9.uscourts.gov/rules states "The latest rules were adopted February 24, 2000 and revised June 15, 2015" (page lists only non-substantive proposed revisions, not in force, not vendored). Fetched with curl (robots.txt allows all).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 8c3d8a8cf96dc7f66681ec7fa8df1f23ae5f1003b204c2890caced661e9e381e. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `fed-ca9-bap/original/bap-rules.pdf` as served (sha256 5981518c291767df123476c621db8286c0443438c773e77b65277c17ba17a9e7); extraction sha256 8c3d8a8cf96dc7f66681ec7fa8df1f23ae5f1003b204c2890caced661e9e381e.

### `fed-ca9-bap/bap-rules-sectioned.txt` — Ninth Circuit BAP Rules, sectioned

- Derived in the pod (lg-f3q8.38) from `fed-ca9-bap/bap-rules.txt` (extract sha256 8c3d8a8cf96dc7f66681ec7fa8df1f23ae5f1003b204c2890caced661e9e381e); nothing fetched. One `## 9th Cir. BAP R. <n>` line (the citation form of Rule 8026-2) is inserted before each rule's own heading line in the body (16 markers; the table of contents is unmarked). A rule numbered with a parenthetical (`8011(a)-1`, `8013(d)-1`, `8015(a)-1`, `8018(a)-1`) is marked by its base number (`## 9th Cir. BAP R. 8011`, `8013`, `8015`, `8018`), the heading the cite parser keys `9th Cir. BAP R. 8018(a)-2` to, and that section holds every rule of the base number. Rule text was not edited: deleting the marker lines (`sed '/^## 9th Cir. BAP R. /d'`) restores the extract byte for byte.
- Edition: as the extract, adopted February 24, 2000, as revised June 15, 2015.
- Sectioned sha256 b5aec944ffeb327f606162ed9bc35cb452019b03e27ee87d7d1e19d27fa5ac6c (the `path_sha256` pinned by `lit/bap9/Lit.toml`).
