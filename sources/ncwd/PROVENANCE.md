
### `ncwd/ncwd-local-rules.txt` — U.S. District Court for the Western District of North Carolina Local Rules

- Source: https://www.ncwd.uscourts.gov/sites/default/files/local_rules/Revised%20Local%20Rules%20120122.pdf (official site; 81 pages, sha256 042bd955ca07cbc99ab0d52160e254a7d95221cb5408fc81d27f3daa712dc118).
- Edition: Local Rules, cover reads "2017 Edition"; the page https://www.ncwd.uscourts.gov/court-info/local-rules-and-orders/local-rules links this file as "Local Rules (effective 12/01/2022)", the latest listed (file modified 2022-11-30).
- Also the criminal local rules (lg-gddt): 'LCrR 1.1        APPLICATION OF LOCAL CRIMINAL RULES' part, from extract line 2480.
- Criminal forum (lg-gddt): ncwd-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 ae6d9ef6ea8e518b0d545499df79829427c7295d5c644dcc09704f0cce194bd9. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `ncwd/original/ncwd-local-rules.pdf` as served (sha256 042bd955ca07cbc99ab0d52160e254a7d95221cb5408fc81d27f3daa712dc118); extraction sha256 ae6d9ef6ea8e518b0d545499df79829427c7295d5c644dcc09704f0cce194bd9.

### `ncwd/ncwd-local-rules-sectioned.txt` — W.D.N.C. Local Rules (effective December 1, 2022), civil rules sectioned

- Derived in the pod (lg-f3q8.29) from `ncwd/ncwd-local-rules.txt` (extract sha256 ae6d9ef6ea8e518b0d545499df79829427c7295d5c644dcc09704f0cce194bd9); nothing fetched. One `## LCvR <n>` line is inserted right before each Local Civil Rule's own body heading line `LCvR <n>  <TITLE>` (extract lines 154 to 2479): 28 markers, LCvR 1.1 through LCvR 83.3. The table of contents, the in-body cross-references and the Local Criminal Rules (from extract line 2480) are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## LCvR /d'`) restores the extract byte for byte.
- Edition: as the extract, the Local Rules, 2017 Edition as revised effective December 1, 2022.
- Sectioned sha256 80c87289185d3dc06a74bd352fa7528fe4b2037024a9069b9ebeefd78679a711 (the `path_sha256` pinned by `lit/ncwd/Lit.toml`).

### `ncwd/ncwd-local-criminal-rules-sectioned.txt` — W.D.N.C. Local Rules (effective December 1, 2022), criminal rules sectioned

- Derived in the pod (lg-gddt.12) from `ncwd/ncwd-local-rules.txt` (extract sha256 ae6d9ef6ea8e518b0d545499df79829427c7295d5c644dcc09704f0cce194bd9); nothing fetched. One `## LCrR <n>` line is inserted right before each Local Criminal Rule's own body heading line `LCrR <n>  <TITLE>` (extract lines 2424 on): 19 markers, LCrR 1.1 through LCrR 59.1. The table of contents, the in-body cross-references and the civil rules are unmarked (the civil rules are sectioned separately in `ncwd/ncwd-local-rules-sectioned.txt`). Rule text was not edited: deleting the marker lines (`sed '/^## LCrR /d'`) restores the extract byte for byte.
- Edition: as the extract, the Local Rules, 2017 Edition as revised effective December 1, 2022.
- Sectioned sha256 84d0719b6fddb10043390d2043d684c13c148c108927f80e622a21eab44876ea (the `path_sha256` pinned by `lit/ncwd-criminal/Lit.toml`).
