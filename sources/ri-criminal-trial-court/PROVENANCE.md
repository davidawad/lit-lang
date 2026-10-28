
### `ri-criminal-trial-court/ri-superior-court-rules-of-criminal-procedure.txt` — Rhode Island Superior Court Rules of Criminal Procedure

- Source: https://www.courts.ri.gov/Courts/superiorcourt/Documents/SuperiorCourtRulesOfCriminalProcedure.pdf (official site; 47 pages, sha256 8db09dbd04061108d3adbcbb5cb8c7fce7ed1ebc34be05059a1ea4a1e6124454).
- Edition: Rhode Island Superior Court Rules of Criminal Procedure, the 'Rules of Criminal Procedure' PDF linked from https://www.courts.ri.gov/Legal-Resources/Pages/Superior-Court-Rules.aspx; the document states no edition date on its first page, CURRENCY UNCONFIRMED. Speedy trial: Rule 48 (dismissal) and R.I. Gen. Laws 12-13 (not vendored). www.courts.ri.gov robots.txt read; no Disallow covering this path; home page read: no terms or privacy link found; fetched on David's explicit ruling 2026-10-28 despite robots/terms; tool: curl with the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 85393ee45ae1b46c1822d9045b0e847a6251c840455f28d9465ad7d0f2b7ac8c. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `ri-criminal-trial-court/original/ri-superior-court-rules-of-criminal-procedure.pdf` as served (sha256 8db09dbd04061108d3adbcbb5cb8c7fce7ed1ebc34be05059a1ea4a1e6124454); extraction sha256 85393ee45ae1b46c1822d9045b0e847a6251c840455f28d9465ad7d0f2b7ac8c.

- Note (lg-gddt): Criminal appeals: Rhode Island Supreme Court Rules of Appellate Procedure, covered by sources/ri-supreme/.

### `ri-criminal-trial-court/ri-superior-court-rules-of-criminal-procedure-sectioned.txt` — Rhode Island Superior Court Rules of Criminal Procedure, sectioned

- Derived in the pod (lg-gddt.6) from `ri-criminal-trial-court/ri-superior-court-rules-of-criminal-procedure.txt` (extract sha256 85393ee45ae1b46c1822d9045b0e847a6251c840455f28d9465ad7d0f2b7ac8c); nothing fetched. One `## Rule <n>` line is inserted before each rule's own heading line (`<n>. <Title>. — ...`, `Rule 26.1. ...`, or `<n>. [Reserved.]`), in ascending order: 66 markers, Rule 1 through Rule 60 with 6.1, 9.1, 26.1 to 26.3 and 30.1 (Rule 60's section runs through the Appendix of Forms). A cross-reference that begins a line (`Rule 9.1 to dismiss ...`) is not a heading and is unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Edition: as the extract (Rule 59 states that the amended rules took effect September 5, 2017; the document states no edition date); CURRENCY UNCONFIRMED stands. `lit/ri-criminal-trial-court` cites it for structure only and takes no value from it.
- Upstream: https://www.courts.ri.gov/Courts/superiorcourt/Documents/SuperiorCourtRulesOfCriminalProcedure.pdf, retrieved 2026-10-28, as recorded on `ri-criminal-trial-court/ri-superior-court-rules-of-criminal-procedure.txt` above; sectioning was done from the vendored extract.
- Sectioned sha256 f8bcfb15f679b9b394949b71003e5e05d67a5558e403bde6d3cc2d1548076819 (the `path_sha256` pinned by `lit/ri-criminal-trial-court/Lit.toml`).
