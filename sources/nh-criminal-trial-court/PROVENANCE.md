
### `nh-criminal-trial-court/nh-rules-of-criminal-procedure.txt` — New Hampshire Rules of Criminal Procedure (Rules 1-53, all rules on one page)

- Source: https://www.courts.nh.gov/new-hampshire-rules-criminal-procedure (1 official HTML page(s), one GET each).
- Edition: New Hampshire Rules of Criminal Procedure as posted on https://www.courts.nh.gov/new-hampshire-rules-criminal-procedure (the page carries each rule's adoption/effective notes; no consolidated edition date), CURRENCY UNCONFIRMED. www.courts.nh.gov serves no robots.txt (no robots file returned); site terms/privacy page not located or not checked; fetched on David's explicit ruling 2026-10-28 despite robots/terms (barrier: a bot filter, not a robots or terms refusal: plain curl answered HTTP 403 (courts.state.nh.us redirected to courts.nh.gov); the page was fetched in BrowserOS neo with an in-page fetch from the loaded origin); tool: BrowserOS neo with the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); each page extracted with html2txt v2 (stdlib html.parser), unedited, concatenated under one `## <heading>` per page.
- Extract sha256 725f2b6ee07e34049e5c80b7724b8d527275968aed1a1af4a8663083bcda1fa0. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `nh-criminal-trial-court/original/rules-page/0001.html` as served (sha256 ad0ca35af0216220f50bfd9e7322239479c6afb37477c93f31d1abd51839599a). Served at https://www.courts.nh.gov/new-hampshire-rules-criminal-procedure.
- Extraction sha256 725f2b6ee07e34049e5c80b7724b8d527275968aed1a1af4a8663083bcda1fa0.

- Note (lg-gddt): Criminal appeals: Rules of the New Hampshire Supreme Court, covered by sources/nh-supreme/. Speedy trial in New Hampshire is by case law and the Superior Court rules, not a fixed-day rule in these Rules.

### `nh-criminal-trial-court/nh-rules-of-criminal-procedure-sectioned.txt` — New Hampshire Rules of Criminal Procedure, sectioned

- Derived in the pod (lg-gddt.6) from `nh-criminal-trial-court/nh-rules-of-criminal-procedure.txt` (extract sha256 725f2b6ee07e34049e5c80b7724b8d527275968aed1a1af4a8663083bcda1fa0); nothing fetched. One `## Rule <n>` line is inserted before the first body heading line `Rule <n>. <Title>` of each rule after the page's table of contents (53 rules, Rule 1 through Rule 53; each section runs through that rule's comments). The table of contents and comment lines that begin with a cross-reference (`Rule 4(e) provides ...`) are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Edition: as the extract; CURRENCY UNCONFIRMED stands. `lit/nh-criminal-trial-court` cites it for structure only and takes no value from it.
- Sectioned sha256 5ff8ed111d5dfa27b5003ef4d96ba0ea2fc2e616da1d3878a19d5ca2d4554bae (the `path_sha256` pinned by `lit/nh-criminal-trial-court/Lit.toml`).
