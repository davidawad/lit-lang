
### `vt-superior-civil/vt-vrcp-55a-order-2026-10-06.txt` — Vermont order amending V.R.C.P. 55(a)

- Source: https://www.vtcourts.gov/sites/default/files/documents/PROMULGATED-VRCP55%28a%29--STAMPED.pdf (official site; 1 pages, sha256 e175a02c1a2059f56e2e480b5629656c1e8c1950c490f859e759381ed159f016).
- Edition: Vermont Supreme Court promulgation order amending V.R.C.P. 55(a), promulgated October 6, 2026, effective January 1, 2027 (Reporter's Notes 2027 Amendment), listed at https://www.vtcourts.gov/attorneys/rules/promulgated. Amendment order only: the base V.R.C.P. text is not vendored (the judiciary site hosts no per-rule PDFs; the official code is on LexisNexis only). Apply on top of the base rule once obtained; not yet in force until 2027-01-01. robots.txt (vtcourts.gov, formerly vermontjudiciary.org) and Terms of Use checked: no prohibition of automated access.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 a25685a2973dec0f59d4aebff0397df7d1ac8e517645a9a4f0724b6c2d10111f. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `vt-superior-civil/original/vt-vrcp-55a-order-2026-10-06.pdf` as served (sha256 e175a02c1a2059f56e2e480b5629656c1e8c1950c490f859e759381ed159f016); extraction sha256 a25685a2973dec0f59d4aebff0397df7d1ac8e517645a9a4f0724b6c2d10111f.

### `vt-superior-civil/vrcp.txt` — Vermont Rules of Civil Procedure (V.R.C.P.)

- Source: https://advance.lexis.com/ (Vermont Statutes, Court Rules and Administrative Code public access) (official public-access publisher).
- Edition: As the site states: "Current with rule changes received through March 16, 2026." 115 rules plus 1 structural document. Later orders are vendored separately as dated layers (vt-vrcp-55a-order-2026-10-06.txt). The Lexis page footer ("Vermont State & Federal Court Rules Copyright © 2026 All rights reserved.") appears in the original records and is not rule text.
- Retrieved 2026-10-28 on carbon by David's BrowserOS neo pass (Method: BrowserOS neo, Lexis public-access viewer text, David's explicit ruling 2026-10-28; Lexis annotations excluded); the viewer's per-document records were captured as JSON (BrowserOS tool framing stripped, content unedited) and saved as the original; the extraction is the PM session's plain-text rendering of those records, in document order. No other served original exists for this text. Barrier: LexisNexis public access (reCAPTCHA on document pages; lg-f3q8.77 row); fetched on David's explicit ruling 2026-10-28 despite robots/terms.
- Extract sha256 97c37263b450dad65df451e34be464d814ca7eb2390b10bdae1696d36375080f.
- Original: `vt-superior-civil/original/vrcp.json` as served to the viewer (sha256 e6750c9690c20c1da0385432958b6407ce1a9669ab7009c4bc853bd7bf281f71); extraction sha256 97c37263b450dad65df451e34be464d814ca7eb2390b10bdae1696d36375080f.

### `vt-superior-civil/vrcp-sectioned.txt` — Vermont Rules of Civil Procedure, sectioned

- Derived in the pod (lg-f3q8.52) from `vt-superior-civil/vrcp.txt` (extract sha256 97c37263b450dad65df451e34be464d814ca7eb2390b10bdae1696d36375080f); nothing fetched. One `## Rule <n>` line is inserted before each `===== V.R.C.P. Rule <n> =====` record line (Rule 1 through Rule 86, 115 rules; the appendix of forms record is unmarked). Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Source: as the extract `vt-superior-civil/vrcp.txt`, https://advance.lexis.com/, retrieved 2026-10-28; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, current with rule changes received through March 16, 2026. The October 6, 2026 Rule 55(a) promulgation order (effective January 1, 2027) is a separate dated document and is not merged into it.
- Sectioned sha256 8583220322f0a9df87238db26261c190ade801d0ff4e6d0656b84fe20bb4c004 (the `path_sha256` pinned by `lit/vt-superior-civil/Lit.toml` and `lit/vt-supreme/Lit.toml`).
