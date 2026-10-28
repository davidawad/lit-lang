
### `vi-criminal-trial-court/vi-rules-of-criminal-procedure-2019.txt` — Virgin Islands Rules of Criminal Procedure (81 pp; Rule 48 dismissal / speedy trial)

- Source: https://www.vicourts.org//UserFiles/DBFiles/server_9784218/201911/16185775/vi_rules_of_criminal_procedure.pdf (official site; 81 pages, sha256 3905a1850974d3414ae80978b1bf93042fbc9e9fa53b3818269600c263e485e9).
- Edition: Virgin Islands Rules of Criminal Procedure, the vicourts.org PDF uploaded 2019-11 (PDF metadata modified 2019-11-06); the document states no edition date, and later Supreme Court promulgation orders are NOT checked, so it may be superseded: CURRENCY UNCONFIRMED. The Superior Court also publishes Part VI (procedure in the criminal division) of its own rules at https://superior.vicourts.org/court_rules/part_vi_procedure_in_the_criminal_division/ (not vendored). www.vicourts.org robots.txt read; no Disallow covering this path; home page read: no terms or privacy link found; fetched on David's explicit ruling 2026-10-28 despite robots/terms; tool: curl with the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 f3cac264beeac51f4e6f605cc6a3bcb90a3d0e63b5f857e3149dc0d47eea3d92. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `vi-criminal-trial-court/original/vi-rules-of-criminal-procedure-2019.pdf` as served (sha256 3905a1850974d3414ae80978b1bf93042fbc9e9fa53b3818269600c263e485e9); extraction sha256 f3cac264beeac51f4e6f605cc6a3bcb90a3d0e63b5f857e3149dc0d47eea3d92.

- Note (lg-gddt): Criminal appeals: Virgin Islands Supreme Court rules, covered by sources/vi-supreme-civil/ if it carries them. Statutory: 5 V.I.C. (Criminal Procedure) not vendored.

### `vi-criminal-trial-court/vi-rules-of-criminal-procedure-2019-sectioned.txt` — Virgin Islands Rules of Criminal Procedure, sectioned

- Derived in the pod (lg-gddt.10) from `vi-criminal-trial-court/vi-rules-of-criminal-procedure-2019.txt` (extract sha256 f3cac264beeac51f4e6f605cc6a3bcb90a3d0e63b5f857e3149dc0d47eea3d92); nothing fetched. In the body (after the table of contents), one marker line `## V.I. R. Crim. P. <n>` is inserted before each rule heading line `Rule <n>. <Title>` (63 headings, Rule 1 through Rule 57, including the hyphenated local Rules 5-1, 16-1 and 40-1 and the reserved Rules 9, 27, 32.2, 50 and 54). Lines in the reporter's notes that begin with a rule number but are not headings (for example `Rule 4.1 spells out ...`, `Rule 12.1(a)-(c).`, `Rule 16.`) carry no marker. Every other line is the extract unchanged, so deleting the marker lines (`sed '/^## V.I. R. Crim. P. /d'`) gives back the extract byte for byte.
- Edition: as the extract; CURRENCY UNCONFIRMED stands. `lit/vi-criminal-trial-court` cites it for structure only and takes no value from it.
- Upstream: https://www.vicourts.org//UserFiles/DBFiles/server_9784218/201911/16185775/vi_rules_of_criminal_procedure.pdf, retrieved 2026-10-28, as recorded on `vi-criminal-trial-court/vi-rules-of-criminal-procedure-2019.txt` above; sectioning was done from the vendored extract.
- Sectioned sha256 8ad12401a485121583b4df7cc44b0c5e250a9daadaa0a4e028c981fb507bca88 (the `path_sha256` pinned by `lit/vi-criminal-trial-court/Lit.toml`).
