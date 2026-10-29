
### `wy-criminal-trial-court/wy-rules-of-criminal-procedure.txt` — Wyoming Rules of Criminal Procedure with the bond schedule (190 pp; Rule 48 dismissal/speedy trial)

- Source: https://www.wyocourts.gov/app/uploads/2026/06/WRCrP-with-bond-schedule.pdf (official site; 190 pages, sha256 e8c4e71032a1fc14c8e8f12f4a52c8798433651f0efbf18918d52e421b393489).
- Edition: Wyoming Rules of Criminal Procedure, the 'Download Rule' PDF (WRCrP-with-bond-schedule, uploaded 2026/06; PDF metadata modified 2026-07-08) linked from https://www.wyocourts.gov/court-rules/wyoming-rules-of-criminal-procedure/; no edition date stated on the document, CURRENCY UNCONFIRMED. The page also lists Appendix I (the uniform bail and forfeiture schedule) and an Appendix of Forms, not vendored separately. www.wyocourts.gov robots.txt read; no Disallow covering this path; Privacy Page link found but the page returned 404; fetched on David's explicit ruling 2026-10-28 despite robots/terms; tool: curl with the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 36046bd974f85b1d2c905bf3d9a5fff0ced19ac55befe99d977db1f594f7cfd2. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `wy-criminal-trial-court/original/wy-rules-of-criminal-procedure.pdf` as served (sha256 e8c4e71032a1fc14c8e8f12f4a52c8798433651f0efbf18918d52e421b393489); extraction sha256 36046bd974f85b1d2c905bf3d9a5fff0ced19ac55befe99d977db1f594f7cfd2.

- Note (lg-gddt): Criminal appeals: Wyoming Rules of Appellate Procedure, covered by sources/wy-state-appellate/.

### `wy-criminal-trial-court/wy-rules-of-criminal-procedure-sectioned.txt` — W.R.Cr.P., sectioned

- Derived in the pod (lg-gddt.24) from `wy-criminal-trial-court/wy-rules-of-criminal-procedure.txt` (extract sha256 36046bd974f85b1d2c905bf3d9a5fff0ced19ac55befe99d977db1f594f7cfd2); nothing fetched. One `## W.R.Cr.P. <n>` line is inserted before each rule's own heading line `Rule <n>. <Title>` (W.R.Cr.P. 1 through 61, 82 rules including the decimal rules); three body lines that merely begin with a wrapped `Rule <n>.` reference (extract lines 2221, 2445 and 3672) carry no marker. The bond schedule and forms after Rule 61 fall in Rule 61's section. Rule text was not edited: deleting the marker lines (each `## W.R.Cr.P. ` line with its newline) restores the extract byte for byte.
- Source: as the extract `wy-criminal-trial-court/wy-rules-of-criminal-procedure.txt`, https://www.wyocourts.gov/app/uploads/2026/06/WRCrP-with-bond-schedule.pdf; Retrieved 2026-10-28 with the extract (this file was derived from it in the pod, not fetched itself).
- Edition: as the extract; CURRENCY UNCONFIRMED.
- Sectioned sha256 903752441cc027c945523d8789e1f8152d92196264aaa0d24961f084ea3976af (the `path_sha256` pinned by `lit/wy-criminal-trial-court/Lit.toml`).
