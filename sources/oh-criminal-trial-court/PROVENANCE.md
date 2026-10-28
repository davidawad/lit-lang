
### `oh-criminal-trial-court/oh-rules-of-criminal-procedure.txt` — Ohio Rules of Criminal Procedure (193 pp)

- Source: https://www.supremecourt.ohio.gov/docs/LegalResources/Rules/criminal/CriminalProcedure.pdf (official site; 193 pages, sha256 eb872c8559b55b6dcddd526ac3001026fb459d98706b78c45783c0a1cdfa95cf).
- Edition: Ohio Rules of Criminal Procedure, 'Effective Date: July 1, 1973; Amended: July 1, 1975; July 1, 1996; July 1, 2023' as stated in the document (the Supreme Court of Ohio's compiled PDF); later amendments, if any, not checked. Speedy trial is statutory (R.C. 2945.71 to 2945.73, not vendored); Crim.R. 45 governs time computation. www.supremecourt.ohio.gov robots.txt read; no Disallow covering this path; home page read: no terms or privacy link found; fetched on David's explicit ruling 2026-10-28 despite robots/terms; tool: curl with the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 de6101b1d2a852e99d2a95a87caebad7d2b01f52ff60f3812ee949f79281b896. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `oh-criminal-trial-court/original/oh-rules-of-criminal-procedure.pdf` as served (sha256 eb872c8559b55b6dcddd526ac3001026fb459d98706b78c45783c0a1cdfa95cf); extraction sha256 de6101b1d2a852e99d2a95a87caebad7d2b01f52ff60f3812ee949f79281b896.

- Note (lg-gddt): Criminal appeals: Ohio Rules of Appellate Procedure, covered by sources/oh-appellate/ if it carries them.

### `oh-criminal-trial-court/oh-rules-of-criminal-procedure-sectioned.txt` — Ohio Rules of Criminal Procedure, sectioned
- Derived in the pod (lg-gddt.16) from `oh-criminal-trial-court/oh-rules-of-criminal-procedure.txt` (extract sha256 de6101b1d2a852e99d2a95a87caebad7d2b01f52ff60f3812ee949f79281b896); nothing fetched. One `## Crim.R. <n>` line is inserted right after each rule's own heading line (`RULE <n>. <Title>.`, including the reserved rules): 69 markers, Crim.R. 1 through Crim.R. 60. The table of contents is unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## Crim\.R\. /d'`) restores the extract byte for byte.
- Edition: the extract's rule histories run through amendments effective July 1, 2026 (e.g. Crim.R. 32: "Amended: ... July 1, 2025; July 1, 2026"), so the compiled PDF is the edition in force from July 1, 2026; the cover's amendment list stops at July 1, 2023.
- Sectioned sha256 c45c66fef631f302b1cd64337115e2e0599f357e8f94d20b383e21a1b11c38f9 (the `path_sha256` pinned by `lit/oh-criminal-trial-court/Lit.toml`).
