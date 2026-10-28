
### `dc-criminal-trial-court/dc-superior-court-rules-of-criminal-procedure.txt` — Superior Court of the District of Columbia, Rules of Criminal Procedure (171 pp; Rule 48 dismissal / speedy trial)

- Source: https://www.dccourts.gov/sites/default/files/superior-rules/Superior%20Court%20Rules%20of%20Criminal%20Procedure.pdf (official site; 171 pages, sha256 983683a5f5518723d8097cb6da58677c103b26588ba592a9d1dd59607cf0b5b3).
- Edition: D.C. Superior Court Rules of Criminal Procedure, the 'Download All Rules' PDF on https://www.dccourts.gov/superior-court/superior-court-resources/rules-of-the-superior-court; the page shows a 'Last Updated' line and the PDF states no edition date, CURRENCY UNCONFIRMED. www.dccourts.gov serves no robots.txt (no robots file returned); home page not retrievable (403); terms not read; fetched on David's explicit ruling 2026-10-28 despite robots/terms (barrier: a bot challenge, not a robots or terms refusal: plain curl answered with a WAF filter that returned HTTP 403 to curl (dccourts.gov) (HTTP 403); the pages were fetched in BrowserOS neo with an in-page fetch from the loaded origin at about 1 request per 1.1 s); tool: BrowserOS neo with the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 e690ce58a0b8b05348e75db8f323a94498fe957d82938ff74a64b5ea541adbc8. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `dc-criminal-trial-court/original/dc-superior-court-rules-of-criminal-procedure.pdf` as served (sha256 983683a5f5518723d8097cb6da58677c103b26588ba592a9d1dd59607cf0b5b3); extraction sha256 e690ce58a0b8b05348e75db8f323a94498fe957d82938ff74a64b5ea541adbc8.

- Note (lg-gddt): Criminal appeals: D.C. Court of Appeals Rules, covered by sources/dc-state-appellate/. The D.C. Code Title 23 (criminal procedure, incl. 23-102 speedy trial) is not vendored.

### `dc-criminal-trial-court/dc-superior-court-rules-of-criminal-procedure-sectioned.txt` — D.C. Superior Court Rules of Criminal Procedure, sectioned

- Derived in the pod (lg-gddt.4) from `dc-criminal-trial-court/dc-superior-court-rules-of-criminal-procedure.txt` (extract sha256 e690ce58a0b8b05348e75db8f323a94498fe957d82938ff74a64b5ea541adbc8); nothing fetched. One marker line `## Rule <n>` is inserted before each rule's heading line `Rule <n>. <Title>` (107 rules, Rule 1 through Rule 120, including hyphenated and decimal rules such as 4-I and 5.1; the Rule 41 heading is indented; extract lines 157 and 3852, which begin with "Rule 5." and "Rule 47-I." mid-paragraph, are not headings and carry no marker). Every other line is the extract unchanged, so deleting the `## Rule` lines (`sed '/^## Rule /d'`) gives back the extract byte for byte.
- Upstream: https://www.dccourts.gov/sites/default/files/superior-rules/Superior%20Court%20Rules%20of%20Criminal%20Procedure.pdf, retrieved 2026-10-28, as recorded on the source extract entry this file is derived from.
- Edition: as the extract; CURRENCY UNCONFIRMED stands. `lit/dc-criminal-trial-court` cites it for structure only and takes no value from it.
- Sectioned sha256 f108105a55e505ee427ccbe5398aa9ab3b431391115b0c0ca2b94d6cefa10827 (the `path_sha256` pinned by `lit/dc-criminal-trial-court/Lit.toml`).
