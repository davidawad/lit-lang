
### `fed-sdcal/casd-local-rules-2026-06-01.txt` — S.D. Cal. Local Rules (revised June 1, 2026)

- Source: https://www.casd.uscourts.gov/_assets/pdf/rules/2026.06.01%20Local%20Rules.pdf (official site; 112 pages, sha256 a8dcb48ef9e7be52e84f0156bdfce23d4ca58b91411b44cb4e86a9cad6ee6746).
- Edition: Local Rules of the U.S. District Court for the Southern District of California, 'Revised as of: June 1, 2026' (title page; Civil Local Rules and Criminal, Magistrate, Bankruptcy and other parts). Later general orders were not looked for (the court's rules landing page returned an error to curl). fetched on David's explicit ruling 2026-10-28 despite robots/terms (barrier: robots.txt Disallow:/_assets/ for all agents on www.casd.uscourts.gov, confirmed in robots.txt read today; tool: curl with the identifying UA lit-lang-provenance/1.0).
- Also the criminal local rules (lg-gddt): the criminal rules (cited 'CrimLR') are part of this document; citation rule at extract line 150.
- Criminal forum (lg-gddt): casd-criminal.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 4ad9cbadcc5f9a3bac8ec6a3c82c8a8c6de1b43c1e0169171c74ad16df3bcee7. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `fed-sdcal/original/casd-local-rules-2026-06-01.pdf` as served (sha256 a8dcb48ef9e7be52e84f0156bdfce23d4ca58b91411b44cb4e86a9cad6ee6746); extraction sha256 4ad9cbadcc5f9a3bac8ec6a3c82c8a8c6de1b43c1e0169171c74ad16df3bcee7.

### `fed-sdcal/casd-local-rules-2026-06-01-sectioned.txt` — S.D. Cal. Local Rules (revised June 1, 2026), sectioned

- Derived in the pod (lg-f3q8.39) from `fed-sdcal/casd-local-rules-2026-06-01.txt` (extract sha256 4ad9cbadcc5f9a3bac8ec6a3c82c8a8c6de1b43c1e0169171c74ad16df3bcee7); nothing fetched. One `## S.D. Cal. CivLR <n>` line is inserted right before each civil rule's own body heading line (`Civil Rule <n>` or `Civil Local Rule <n>` at the left margin; 62 markers, 1.1 through 83.11, including 65.1.2). The table of contents and the criminal, magistrate and bankruptcy parts are unmarked (the last marker's section runs to the end of the file). Rule text was not edited: deleting the marker lines (each `## S.D. Cal. CivLR ` line with its newline) restores the extract byte for byte.
- Edition: as the extract, Local Rules revised as of June 1, 2026.
- Sectioned sha256 61112f367b39c26fdcc4ac0f66eeb5a5d438341d21ceb6f1449e25a226bb94c7 (the `path_sha256` pinned by `lit/casd/Lit.toml`).
