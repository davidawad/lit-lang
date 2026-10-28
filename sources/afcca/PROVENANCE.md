
### `afcca/afcca-rules.txt` — AFCCA Rules of Practice and Procedure with JRAP

- Source: https://afcca.law.af.mil/afcca_data/cp/afcca_rules_of_practice_and_procedure_wjrap_29_jun_2026_2888395.pdf (official site; 55 pages, sha256 d0dc191c3b7948c196629c7422b051d451584da21ef3c1d77e1ec748a6be3c95).
- Edition: AFCCA Rules of Practice and Procedure, effective 29 June 2026, published together with the Joint Rules of Appellate Procedure for Courts of Criminal Appeals (cover; linked from https://afcca.law.af.mil/resources_1_4.html). Fetched with curl; the site has no robots.txt..
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 c75fb3d25c4f3fe8f7b58b4d2c27f3ee01052ec41cc14c672b18e99a9d5c4c5e. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `afcca/original/afcca-rules.pdf` as served (sha256 d0dc191c3b7948c196629c7422b051d451584da21ef3c1d77e1ec748a6be3c95); extraction sha256 c75fb3d25c4f3fe8f7b58b4d2c27f3ee01052ec41cc14c672b18e99a9d5c4c5e.

### `afcca/afcca-rules-sectioned.txt` — AFCCA Rules of Practice and Procedure with JRAP, sectioned

- Derived in the pod (lg-f3q8.135) from `afcca/afcca-rules.txt` (extract sha256 c75fb3d25c4f3fe8f7b58b4d2c27f3ee01052ec41cc14c672b18e99a9d5c4c5e); nothing fetched. In the body (after the table of contents, through the signature block before the appendices), one `## JRAP <n>` line is inserted before each Joint Rule's own `Rule <n>. TITLE` heading line (32 rules, JRAP 1 through JRAP 32), and one `## AFCCA R. <n>.<m>` line before each Court rule's own `Rule <n>.<m> Title` heading line (66 rules); cross-references that begin a line (the `Rule 17.2 for additional guidance.` and `Rule 17. Any briefs` lines) are unmarked. Rule text was not edited: deleting the marker lines (`sed -E '/^## (JRAP|AFCCA R\.) /d'`) restores the extract byte for byte.
- Source and edition: as the extract (effective 29 June 2026; JRAP as amended 17 May 2024, effective 27 May 2024, JRAP 4).
- Sectioned sha256 18fb2401db6acd46e11471a2b7c66c56b3d5bd4acd691517c118879c8f90b809 (the `path_sha256` pinned by `lit/afcca/Lit.toml`).
