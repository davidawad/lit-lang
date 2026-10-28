
### `pr-tribunal-apelaciones/pr-reglamento-tribunal-apelaciones.txt` — Reglamento del Tribunal de Apelaciones de Puerto Rico (Spanish)

- Source: https://poderjudicial.pr/documentos/Leyes-Reglamentos/Leyes/Apelaciones.pdf (official site; 130 pages, sha256 e4b748ed5a8719597d14b73e0fd278eb42628dd535b37ce2bf0a595d36b16ca3).
- Edition: Reglamento del Tribunal de Apelaciones (Resolución ER-2004-0010, según enmendado), footnote lists amendments through ER-2025-0003 of 19 Dec 2025 (PDF created 2026-07-16); linked from https://poderjudicial.pr/tribunal-apelaciones/.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 803d31ba29050b4c27c921ef36bb32ad508a5610119489f2d8d4dd594f5d63a0. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `pr-tribunal-apelaciones/original/pr-reglamento-tribunal-apelaciones.pdf` as served (sha256 e4b748ed5a8719597d14b73e0fd278eb42628dd535b37ce2bf0a595d36b16ca3); extraction sha256 803d31ba29050b4c27c921ef36bb32ad508a5610119489f2d8d4dd594f5d63a0.

### `pr-tribunal-apelaciones/pr-reglamento-tribunal-apelaciones-sectioned.txt` — Reglamento del Tribunal de Apelaciones, sectioned

- Derived in the pod (lg-f3q8.25) from `pr-tribunal-apelaciones/pr-reglamento-tribunal-apelaciones.txt` (extract sha256 803d31ba29050b4c27c921ef36bb32ad508a5610119489f2d8d4dd594f5d63a0); nothing fetched. One `## Regla <n>` line is inserted right after each rule's own body heading line `Regla <n> - <title>`: 94 markers, Regla 1 through Regla 89 (with 2.1, 12.1, 30.1, 80.1, 83.1 and the like). The table of contents is unmarked. Rule text was not edited: deleting the marker lines (`## Regla <n>` plus its newline) restores the extract byte for byte.
- Upstream: https://poderjudicial.pr/documentos/Leyes-Reglamentos/Leyes/Apelaciones.pdf, retrieved 2026-10-27, as recorded on the `pr-tribunal-apelaciones/pr-reglamento-tribunal-apelaciones.txt` entry this file is derived from.
- Edition: as the extract, the Reglamento as amended through Resolución ER-2025-0003 of December 19, 2025.
- Sectioned sha256 922270e4114d4ec335d784912470c8389fff7f9c3c2619ea9e61089b29f3930b (the `path_sha256` pinned by `lit/pr-tribunal-apelaciones/Lit.toml`).
