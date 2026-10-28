
### `pr-criminal-trial-court/pr-reglas-de-procedimiento-criminal-1963.txt` — Reglas de Procedimiento Criminal de 1963 (segun enmendadas), Puerto Rico (126 pp; Spanish; Regla 64(n) is the speedy-trial dismissal ground)

- Source: https://poderjudicial.pr/Documentos/Leyes-Reglamentos/Reglamentos//Reglas-de-Procedimiento-Criminal.pdf (official site; 126 pages, sha256 af8b32598344091a1c63058ffe152312d0f2b4514e4c5dc8ed4c5f773238c8c4).
- Edition: Reglas de Procedimiento Criminal de 1963 (according to amendments), the 'Reglas-de-Procedimiento-Criminal' PDF on https://poderjudicial.pr/leyes-y-reglamentos-de-la-rama-judicial/ (PDF metadata modified 2020-03-09; the document does not say through which amendment it is current, so later amendments are NOT confirmed), CURRENCY UNCONFIRMED. poderjudicial.pr serves no robots.txt (no robots file returned); home page read: no terms or privacy link found; fetched on David's explicit ruling 2026-10-28 despite robots/terms; tool: curl with the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 e8ff46b3ddc70f227e6782c2641cd67d7e25928a0643d639ffc49fbed0a41e61. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `pr-criminal-trial-court/original/pr-reglas-de-procedimiento-criminal-1963.pdf` as served (sha256 af8b32598344091a1c63058ffe152312d0f2b4514e4c5dc8ed4c5f773238c8c4); extraction sha256 e8ff46b3ddc70f227e6782c2641cd67d7e25928a0643d639ffc49fbed0a41e61.

- Note (lg-gddt): Spanish-language official text. Criminal appeals: Reglamento del Tribunal de Apelaciones, covered by sources/pr-tribunal-apelaciones/.

### `pr-criminal-trial-court/pr-reglas-de-procedimiento-criminal-1963-sectioned.txt` — Reglas de Procedimiento Criminal de 1963, sectioned

- Derived in the pod (lg-gddt.6) from `pr-criminal-trial-court/pr-reglas-de-procedimiento-criminal-1963.txt` (extract sha256 e8ff46b3ddc70f227e6782c2641cd67d7e25928a0643d639ffc49fbed0a41e61); nothing fetched. One `## Regla <n>` line is inserted before each rule's own body heading line `Regla <n>. <Título>` after the table of contents, in ascending order (261 rules, Regla 1 through Regla 255 with decimal rules such as 4.1, 6.1, 162.1 and 192.1). The closing explanatory comments on Reglas 171 and 185, which repeat those headings out of order, are unmarked and fall in Regla 255's section. Rule text was not edited: deleting the marker lines (`sed '/^## Regla /d'`) restores the extract byte for byte.
- Edition: as the extract (statutory amendments through February 2020); CURRENCY UNCONFIRMED stands. `lit/pr-criminal-trial-court` cites it for structure only and takes no value from it.
- Sectioned sha256 8abe43617cafa139f6cdfc2a94499941c5d658cf60cdbb7bc1613c7d67350259 (the `path_sha256` pinned by `lit/pr-criminal-trial-court/Lit.toml`).
