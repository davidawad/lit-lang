
### `wiwd/wiwd-local-rules.txt` — W.D. Wisconsin Local Rules

- Source: https://www.wiwd.uscourts.gov/sites/default/files/Local_Rules.pdf (official site; 2 pages, sha256 a538634232b7809314ff7baa9c1947dad3931c8dac4782083eddc2de74880ff9).
- Edition: Local Rules, 2-page PDF dated 5/2018 (Rules 1 to 5; same five rules as the HTML page https://www.wiwd.uscourts.gov/local-rules.html, the court current local-rules page, which lists no later edition); fetched by curl.
- No criminal local rules (lg-gddt): the current Local_Rules.pdf (5/2018) was re-read from https://www.wiwd.uscourts.gov/sites/default/files/Local_Rules.pdf on 2026-10-28 and is byte-for-byte the same text as this extract; it holds only Rules 1-5 (LR 83.5, 72.1, 16.6, 47.2, 5.1) and no criminal rule. The court's local-rules page lists nothing else. Criminal forum (lg-gddt): wiwd-criminal. Fetched with curl and the identifying UA; no robots.txt served (404).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 65c3e70149adf22108d699b5c0cb0634b6a6f786cbe2521740be40f2716af2dc. Raw extract, not yet sectioned into `## <ref>` form.

- Fetch method: first fetched with curl and a desktop browser User-Agent before the no-browser-identity rule (PM 2026-10-27); re-fetched on 2026-10-27 with plain curl (its default User-Agent, no browser headers): byte-identical (same sha256).
- Original: `wiwd/original/wiwd-local-rules.pdf` as served (sha256 a538634232b7809314ff7baa9c1947dad3931c8dac4782083eddc2de74880ff9); extraction sha256 65c3e70149adf22108d699b5c0cb0634b6a6f786cbe2521740be40f2716af2dc.

### `wiwd/wiwd-local-rules-sectioned.txt` — W.D. Wisconsin Local Rules, sectioned

- Derived in the pod (lg-f3q8.56) from `wiwd/wiwd-local-rules.txt` (extract sha256 65c3e70149adf22108d699b5c0cb0634b6a6f786cbe2521740be40f2716af2dc); nothing fetched. One `## Rule <n>` line is inserted right after each rule's own `RULE <n> (LR ...): ...` heading line (`Rule 1` through `Rule 5`); after, not before, because the cite corpus also opens a section at a `RULE <n>:` line, which would leave a marker placed before it empty. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Source: as the extract `wiwd/wiwd-local-rules.txt`, https://www.wiwd.uscourts.gov/sites/default/files/Local_Rules.pdf, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, Local Rules dated 5/2018.
- Sectioned sha256 24fe18cfdc475ed00773efcc9543061a6fe318741140fe3839903184212f49f4 (the `path_sha256` pinned by `lit/wiwd/Lit.toml`).
