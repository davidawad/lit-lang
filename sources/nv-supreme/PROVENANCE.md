
### `nv-supreme/nrap.txt` — Nevada Rules of Appellate Procedure (NRAP)

- Source: https://nvcourts.gov/nevada_court_rules/nevada_appellate_court_rules2/nevada_rules_of_appellate_procedure (official site).
- Edition: adopted by the Supreme Court of Nevada, effective July 1, 1973, including amendments through May 21, 2025 (page updated 5/27/2025 as the page states; the rule-amendments page lists later orders, so this may lag the operative text).
- Retrieved 2026-10-27. Fetch method: BrowserOS neo; site served a bot challenge to plain automated requests (CoS ruling 2026-10-27). Read in the browser one page per request, at most one request per second; robots.txt and the site policy pages were read first and bar no such access. One page; text taken from the rendered page body. nvcourts.gov robots.txt does not disallow it; its use and privacy policy bars no automated access. NRCP was not vendored: nvcourts.gov offers only committee drafts and per-rule PDFs, no consolidated current text.
- Extract: 0 `## <set> <rule or section>` sections of page text, unedited apart from dropping site navigation. sha256 23edebd69abeaec8311f7baf81e8d309c347dd97ce31e9c976ce83fa9bba67be.
- Original: `nv-supreme/original/nrap.html` as served (sha256 7fae8b53b741809dbea22d8af492bc8571c51ea4c63280f488713086b358c020); extraction sha256 23edebd69abeaec8311f7baf81e8d309c347dd97ce31e9c976ce83fa9bba67be. Served at https://nvcourts.gov/nevada_court_rules/nevada_appellate_court_rules2/nevada_rules_of_appellate_procedure; fetched in BrowserOS (bot challenge on plain curl), retrieved 2026-10-27.

### `nv-supreme/nrap-sectioned.txt` — Nevada Rules of Appellate Procedure (amendments through May 21, 2025), sectioned

- Derived in the pod (lg-f3q8.45) from `nv-supreme/nrap.txt` (extract sha256 23edebd69abeaec8311f7baf81e8d309c347dd97ce31e9c976ce83fa9bba67be); nothing fetched. One `## NRAP <n>` line is inserted right before each rule's own upper-case heading line, `RULE<U+00A0><n>.<U+00A0><U+00A0><TITLE>`: 62 markers, NRAP 1 through NRAP 48 (3A to 3E, 12A, 25A, 26.1, 28.1, 28.2, 40A, 40B, 45A and 46A included). Drafter's notes, which begin `Rule <n>` in mixed case, are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## NRAP /d'`) restores the extract byte for byte.
- Edition: as the extract; "Effective July 1, 1973, and Including Amendments Through May 21, 2025".
- Sectioned sha256 da5766edaa2c73cdcc84090d1ffb4b9c8ed94bf19d19983a8d5fcc647a0b4ee5 (the `path_sha256` pinned by `lit/nv-district/Lit.toml` and `lit/nv-supreme/Lit.toml`).
