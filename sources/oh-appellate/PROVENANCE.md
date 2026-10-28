
### `oh-appellate/ohio-appellate-rules.txt` — Ohio Rules of Appellate Procedure

- Source: https://www.supremecourt.ohio.gov/docs/LegalResources/Rules/appellate/AppellateProcedure.pdf (official site; 83 pages, sha256 ac120dddbcaaad3c4423e40f03f1d312dc03ce6bf743a95bdbfa91ebacc94df6).
- Edition: Ohio Rules of Appellate Procedure; the court's rules page https://www.supremecourt.ohio.gov/laws-rules/ohio-rules-of-court/ lists 'Rules of Appellate Procedure' amended effective July 1, 2026 and the PDF carries the July 1, 2026 amendments and staff notes.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Fetched via curl (single document, first request succeeded, no block encountered).
- Extract sha256 4a9c3614c017b97507235fc9244abdbb8ac6d0ed814f0ba09c6502fd13c43a3a. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `oh-appellate/original/ohio-appellate-rules.pdf` as served (sha256 ac120dddbcaaad3c4423e40f03f1d312dc03ce6bf743a95bdbfa91ebacc94df6); extraction sha256 4a9c3614c017b97507235fc9244abdbb8ac6d0ed814f0ba09c6502fd13c43a3a.

### `oh-appellate/ohio-supreme-court-rules-of-practice.txt` — Supreme Court of Ohio Rules of Practice (S.Ct.Prac.R.)

- Source: https://www.supremecourt.ohio.gov/docs/LegalResources/Rules/practice/rulesofpractice.pdf (official site; 128 pages, sha256 b87d4df8a0f966b8cb7679e2f30b2fdeb0f54182299a95f4a1c3c4497c364a37).
- Edition: Supreme Court of Ohio Rules of Practice, 'Effective May 1, 2025 (Including amendments effective May 1, 2025)' as stated on the document; https://www.supremecourt.ohio.gov/laws-rules/ohio-rules-of-court/ lists 'Supreme Court Rules of Practice' amended effective May 1, 2025 (current).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Fetched via curl (single document, first request succeeded, no block encountered).
- Extract sha256 e18d8eb5e24542f9e2ab0d18f8322f0bda2e9cb83a4f2c08bf90921e2ab03793. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `oh-appellate/original/ohio-supreme-court-rules-of-practice.pdf` as served (sha256 b87d4df8a0f966b8cb7679e2f30b2fdeb0f54182299a95f4a1c3c4497c364a37); extraction sha256 e18d8eb5e24542f9e2ab0d18f8322f0bda2e9cb83a4f2c08bf90921e2ab03793.

### `oh-appellate/ohio-appellate-rules-sectioned.txt` — Ohio Rules of Appellate Procedure, sectioned

- Derived in the pod (lg-f3q8.61) from `oh-appellate/ohio-appellate-rules.txt` (extract sha256 4a9c3614c017b97507235fc9244abdbb8ac6d0ed814f0ba09c6502fd13c43a3a); nothing fetched. One `## App.R. <n>` line is inserted right after each rule's own heading line `RULE <n>. <title>` (36 markers, App.R. 1 through App.R. 43; the heading printed `Rule 26.` is marked `## App.R. 26`; the `RULES 31-33.` and `RULES 35-40.` reserved lines are unmarked). Rule text was not edited: deleting the marker lines (`sed '/^## App\.R\. /d'`) restores the extract byte for byte.
- Source: as the extract `oh-appellate/ohio-appellate-rules.txt`, https://www.supremecourt.ohio.gov/docs/LegalResources/Rules/appellate/AppellateProcedure.pdf, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, the Rules of Appellate Procedure as amended effective July 1, 2026.
- Sectioned sha256 140f97309db187996c93d42dada5defa09fe1bfd4ef12c3fa521c08027f66034 (the `path_sha256` pinned by `lit/oh-common-pleas/Lit.toml` and `lit/oh-court-of-appeals/Lit.toml`).

### `oh-appellate/ohio-supreme-court-rules-of-practice-sectioned.txt` — Supreme Court of Ohio Rules of Practice, sectioned

- Derived in the pod (lg-f3q8.61) from `oh-appellate/ohio-supreme-court-rules-of-practice.txt` (extract sha256 e18d8eb5e24542f9e2ab0d18f8322f0bda2e9cb83a4f2c08bf90921e2ab03793); nothing fetched. One `## S.Ct.Prac.R. <n>` line is inserted right after each rule's own body heading line `Rule <n>. <title>` (131 markers, S.Ct.Prac.R. 1.01 through the last rule; the table of contents is unmarked). Rule text was not edited: deleting the marker lines (`sed '/^## S\.Ct\.Prac\.R\. /d'`) restores the extract byte for byte.
- Source: as the extract `oh-appellate/ohio-supreme-court-rules-of-practice.txt`, https://www.supremecourt.ohio.gov/docs/LegalResources/Rules/practice/rulesofpractice.pdf, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, the Rules of Practice effective May 1, 2025 (including amendments effective May 1, 2025).
- Sectioned sha256 ab366ec5b9b3093c03c5bfe2720724becaee9666c86cf511e89ca98a4f4cb164 (the `path_sha256` pinned by `lit/oh-court-of-appeals/Lit.toml`).
