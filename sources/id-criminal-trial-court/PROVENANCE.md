
### `id-criminal-trial-court/id-criminal-rules.txt` — Idaho Criminal Rules (I.C.R.), as published on the Idaho Supreme Court rules page

- Source: https://isc.idaho.gov/rules-procedure/icr (1 official HTML page(s), one GET each).
- Edition: Idaho Criminal Rules, the rules text published on the page https://isc.idaho.gov/rules-procedure/icr (a single-page app that embeds the rule text); the page lists amending orders (e.g. amended May 1, 2024, effective July 1, 2024; amended September 11, 2024) but states no consolidated edition date, CURRENCY UNCONFIRMED. Speedy trial: Idaho Code 19-3501 and I.C.R. 48 (statute not vendored). isc.idaho.gov robots.txt read; no Disallow covering this path; Read Our Accessibility Statement page (https://isc.idaho.gov/ada-accessibility) read: no automated-access prohibition found; fetched on David's explicit ruling 2026-10-28 despite robots/terms; tool: curl with the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); each page extracted with html2txt v2 (stdlib html.parser), unedited, concatenated under one `## <heading>` per page.
- Extract sha256 8e28633aeb1f03c88d0736a5f9e9e6c16e86d9870aba8c50c014950df96b88bd. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `id-criminal-trial-court/original/icr-page/0001.html` as served (sha256 06bf29467238f8c8ea258de36c7be86d16da07256775292cfc3450bb9cd8c689). Served at https://isc.idaho.gov/rules-procedure/icr.
- Extraction sha256 8e28633aeb1f03c88d0736a5f9e9e6c16e86d9870aba8c50c014950df96b88bd.

- Note (lg-gddt): Criminal appeals: Idaho Appellate Rules, covered by sources/id-supreme/ (rules-procedure/iar).

### `id-criminal-trial-court/id-criminal-rules-sectioned.txt` — Idaho Criminal Rules (rules page), sectioned

- Derived in the pod (lg-gddt.22) from `id-criminal-trial-court/id-criminal-rules.txt` (extract sha256 8e28633aeb1f03c88d0736a5f9e9e6c16e86d9870aba8c50c014950df96b88bd); nothing fetched. One marker line `## I.C.R. <n>` is inserted before each rule's own heading line `Idaho Criminal Rule <n>. <Title>` (83 rules, I.C.R. 1 through 54.1; Rules 44.2 and 44.3, whose headings read `Rule <n>.` without the prefix, carry no marker); the rule index lines and the forms list carry no markers. Every other line is the extract unchanged, so deleting the marker lines gives back the extract byte for byte.
- Edition: as the extract; CURRENCY UNCONFIRMED stands. `lit/id-criminal-trial-court` cites it for structure only and takes no value from it.
- Sectioned sha256 33abf1b9d996d928be260c3af57c94dfe9369250f7f70bd9af0dabadfc116c0a (the `path_sha256` pinned by `lit/id-criminal-trial-court/Lit.toml`).
