# `vt-supreme` sources: provenance

### `vt-supreme/vrap.txt` — Vermont Rules of Appellate Procedure (V.R.A.P.)

- Source: https://advance.lexis.com/ (Vermont Statutes, Court Rules and Administrative Code public access) (official public-access publisher).
- Edition: As the site states: "Current with rule changes received through March 16, 2026." 60 rules plus 4 structural/appendix documents. The Lexis page footer ("Vermont State & Federal Court Rules Copyright © 2026 All rights reserved.") appears in the original records and is not rule text.
- Retrieved 2026-10-28 on carbon by David's BrowserOS neo pass (Method: BrowserOS neo, Lexis public-access viewer text, David's explicit ruling 2026-10-28; Lexis annotations excluded); the viewer's per-document records were captured as JSON (BrowserOS tool framing stripped, content unedited) and saved as the original; the extraction is the PM session's plain-text rendering of those records, in document order. No other served original exists for this text. Barrier: LexisNexis public access (reCAPTCHA on document pages; lg-f3q8.77 row); fetched on David's explicit ruling 2026-10-28 despite robots/terms.
- Extract sha256 3512d3500edbe1c72ad8d51add6578f91da26910588159c6eabcd1b448fbf9a4.
- Original: `vt-supreme/original/vrap.json` as served to the viewer (sha256 3d59c45082c9aaefa1489895bc461f8d4fe510374caeba1423b7ee7f93b8fa53); extraction sha256 3512d3500edbe1c72ad8d51add6578f91da26910588159c6eabcd1b448fbf9a4.

### `vt-supreme/vrap-sectioned.txt` — Vermont Rules of Appellate Procedure, sectioned

- Derived in the pod (lg-f3q8.52) from `vt-supreme/vrap.txt` (extract sha256 3512d3500edbe1c72ad8d51add6578f91da26910588159c6eabcd1b448fbf9a4); nothing fetched. One `## Rule <n>` line is inserted before each `===== V.R.A.P. Rule <n> =====` record line (Rule 1 through Rule 49, 60 rules; the structural and appendix records are unmarked). Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Source: as the extract `vt-supreme/vrap.txt`, https://advance.lexis.com/, retrieved 2026-10-28; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, current with rule changes received through March 16, 2026.
- Sectioned sha256 fb1c5b25a8a63cfa372f669dfbc78884b15a7fb681e3f081e553a7c120814b05 (the `path_sha256` pinned by `lit/vt-supreme/Lit.toml` and `lit/vt-superior-civil/Lit.toml`).
