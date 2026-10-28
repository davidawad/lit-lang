
### `scd/scd-local-civil-rules.txt` — U.S. District Court for the District of South Carolina Local Civil Rules

- Source: https://www.scd.uscourts.gov/Rules/Civil%20Rules%20-%20Current.pdf (official site; 75 pages, sha256 b605c3567e665b034281969d9bbcf630820a300c064c981fee6f3af19457a588).
- Edition: Local Civil Rules, February 22, 2023 (cover date; https://www.scd.uscourts.gov/navtop/rules.asp lists "Local Rules - With Revisions through February 2023" as current; the site disclaimer page linked from its home page returns a 404, robots.txt allows /Rules/).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 367f37856cc9b41c5ceeee657cc19353e3f892dec340bdc1e2c7a9de1bdeb2ff. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `scd/original/scd-local-civil-rules.pdf` as served (sha256 b605c3567e665b034281969d9bbcf630820a300c064c981fee6f3af19457a588); extraction sha256 367f37856cc9b41c5ceeee657cc19353e3f892dec340bdc1e2c7a9de1bdeb2ff.

### `scd/scd-local-civil-rules-sectioned.txt` — D.S.C. Local Civil Rules, sectioned

- Derived in the pod (lg-f3q8.30) from `scd/scd-local-civil-rules.txt` (extract sha256 367f37856cc9b41c5ceeee657cc19353e3f892dec340bdc1e2c7a9de1bdeb2ff); nothing fetched. One `## Rule <n>` line is inserted before each rule's own heading line in the body (`1.01:` through `83.IX.04:`, 131 rules, a page's leading form feed kept on the heading line); the table of contents is unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Edition: as the extract, the Local Civil Rules of February 22, 2023.
- Sectioned sha256 45fb4f5e82c9e6ef606e4b3beca7cabd537692a4317885f396014a2ae0852957 (the `path_sha256` pinned by `lit/scd/Lit.toml`).
