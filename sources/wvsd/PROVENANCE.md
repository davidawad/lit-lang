
### `wvsd/wvsd-local-rules.txt` — U.S. District Court for the Southern District of West Virginia Local Rules of Procedure

- Source: https://www.wvsd.uscourts.gov/sites/wvsd/files/LocalRulesofProcedure-June2017.pdf (official site; 95 pages, sha256 3309a2002517b83741b178802b7885eaa4c9d660fa267ba6d66dd2cbd5c91210).
- Edition: Local Rules of Procedure, effective December 1, 2009 with amendments through the June 2017 amendment (cover list; file named LocalRulesofProcedure-June2017.pdf, PDF modified 2022-02-01); linked as "local rules in PDF format" at https://www.wvsd.uscourts.gov/court-info/local-rules-and-orders/local-rules, which lists no later amendment.
- Also the criminal local rules (lg-gddt): 'II. LOCAL RULES OF CRIMINAL PROCEDURE' part, from extract line 2828.
- Criminal forum (lg-gddt): wvsd-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 4028c62dee18a52a4ed478e96ccb0a3c5cef7602b3ca1b777ce19bfd20498a22. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `wvsd/original/wvsd-local-rules.pdf` as served (sha256 3309a2002517b83741b178802b7885eaa4c9d660fa267ba6d66dd2cbd5c91210); extraction sha256 4028c62dee18a52a4ed478e96ccb0a3c5cef7602b3ca1b777ce19bfd20498a22.

### `wvsd/wvsd-local-rules-sectioned.txt` — S.D. W. Va. Local Rules of Procedure (amended through June 8, 2017), civil rules sectioned

- Derived in the pod (lg-f3q8.32) from `wvsd/wvsd-local-rules.txt` (extract sha256 4028c62dee18a52a4ed478e96ccb0a3c5cef7602b3ca1b777ce19bfd20498a22); nothing fetched. One `## LR Civ P <n>` line is inserted right before each Local Rule of Civil Procedure's own body heading line `LR Civ P <n>. <title>` (some after a form feed; LR Civ P 26.5 has no period after its number): 86 markers, LR Civ P 3.1 through LR Civ P 83.15. The table of contents and the Local Rules of Criminal Procedure are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## LR Civ P /d'`) restores the extract byte for byte.
- Upstream: https://www.wvsd.uscourts.gov/sites/wvsd/files/LocalRulesofProcedure-June2017.pdf, retrieved 2026-10-27, as recorded on the `wvsd/wvsd-local-rules.txt` entry this file is derived from.
- Edition: as the extract, effective December 1, 2009, amended through June 8, 2017 (cover list).
- Sectioned sha256 7ec71571c36002fb45da41a13630daaeac73b03da54c33da50bc386bf8f9c535 (the `path_sha256` pinned by `lit/wvsd/Lit.toml`).

### `wvsd/wvsd-local-criminal-rules-sectioned.txt` — S.D. W. Va. Local Rules of Procedure (amended through June 8, 2017), criminal rules sectioned

- Derived in the pod (lg-gddt.12) from `wvsd/wvsd-local-rules.txt` (extract sha256 4028c62dee18a52a4ed478e96ccb0a3c5cef7602b3ca1b777ce19bfd20498a22); nothing fetched. One `## LR Cr P <n>` line is inserted right before each Local Rule of Criminal Procedure's own body heading line `LR Cr P <n>.  <title>` (from extract line 2822 on): 44 markers, LR Cr P 7.1 through LR Cr P 59.6 (with 32.1.1 and 49.1.1 as numbered). The table of contents and the civil rules are unmarked (the civil rules are sectioned separately in `wvsd/wvsd-local-rules-sectioned.txt`). Rule text was not edited: deleting the marker lines (`sed '/^## LR Cr P /d'`) restores the extract byte for byte.
- Source: as the extract `wvsd/wvsd-local-rules.txt`, https://www.wvsd.uscourts.gov/sites/wvsd/files/LocalRulesofProcedure-June2017.pdf; Retrieved 2026-10-27 with the extract (this file was derived from it in the pod, not fetched itself).
- Edition: as the extract, effective December 1, 2009, amended through June 8, 2017.
- Sectioned sha256 bfb1d1fd78084aa5fe60495874d71a993895993586dead8752c0b80114753089 (the `path_sha256` pinned by `lit/wvsd-criminal/Lit.toml`).
