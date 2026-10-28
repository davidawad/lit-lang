
### `scd/scd-local-civil-rules.txt` — U.S. District Court for the District of South Carolina Local Civil Rules

- Source: https://www.scd.uscourts.gov/Rules/Civil%20Rules%20-%20Current.pdf (official site; 75 pages, sha256 b605c3567e665b034281969d9bbcf630820a300c064c981fee6f3af19457a588).
- Edition: Local Civil Rules, February 22, 2023 (cover date; https://www.scd.uscourts.gov/navtop/rules.asp lists "Local Rules - With Revisions through February 2023" as current; the site disclaimer page linked from its home page returns a 404, robots.txt allows /Rules/).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 367f37856cc9b41c5ceeee657cc19353e3f892dec340bdc1e2c7a9de1bdeb2ff. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `scd/original/scd-local-civil-rules.pdf` as served (sha256 b605c3567e665b034281969d9bbcf630820a300c064c981fee6f3af19457a588); extraction sha256 367f37856cc9b41c5ceeee657cc19353e3f892dec340bdc1e2c7a9de1bdeb2ff.

### `scd/scd-local-civil-rules-sectioned.txt` — D.S.C. Local Civil Rules, sectioned

- Derived in the pod (lg-f3q8.30) from `scd/scd-local-civil-rules.txt` (extract sha256 367f37856cc9b41c5ceeee657cc19353e3f892dec340bdc1e2c7a9de1bdeb2ff); nothing fetched. One `## Rule <n>` line is inserted before each rule's own heading line in the body (`1.01:` through `83.IX.04:`, 131 rules, a page's leading form feed kept on the heading line); the table of contents is unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Source: as the extract `scd/scd-local-civil-rules.txt`, https://www.scd.uscourts.gov/Rules/Civil%20Rules%20-%20Current.pdf, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, the Local Civil Rules of February 22, 2023.
- Sectioned sha256 45fb4f5e82c9e6ef606e4b3beca7cabd537692a4317885f396014a2ae0852957 (the `path_sha256` pinned by `lit/scd/Lit.toml`).

### `scd/scd-local-criminal-rules-2019-07-17.txt` — D.S.C. Local Criminal Rules (revisions through July 17, 2019, republished November 23, 2020)

- Source: https://www.scd.uscourts.gov/Rules/Criminal%20Rules%20-%20Current.pdf (official site; 18 pages, sha256 e9a5888b56e35227dfbf5dcd4e32839672029fab10a4177f7b8ab422c7445437).
- Edition: Local Criminal Rules 'with revisions through July 17, 2019', republished November 23, 2020 to correct clerical errors (title page), the 'Criminal Rules - Current' file linked at https://www.scd.uscourts.gov/navtop/rules.asp. OLD EDITION: the court presents it as current. robots.txt of www.scd.uscourts.gov read (Crawl-delay 10 honored where stated; no Disallow covers the fetched path); the judiciary-wide Privacy & Security Policy (https://www.uscourts.gov/privacy-security-policy) read: no prohibition of automated access. Fetched with curl and the identifying UA lit-lang-provenance/1.0, at most 1 request per 10 s.
- Criminal forum (lg-gddt): scd-criminal.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 b7ea78cd12a7859d55c3b23640a9eb959e3efdbdc28d6c457b78226d5f65535e. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `scd/original/scd-local-criminal-rules-2019-07-17.pdf` as served (sha256 e9a5888b56e35227dfbf5dcd4e32839672029fab10a4177f7b8ab422c7445437); extraction sha256 b7ea78cd12a7859d55c3b23640a9eb959e3efdbdc28d6c457b78226d5f65535e.
