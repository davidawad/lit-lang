
### `bankr-or/orb-lbr-2025-12-01.txt` — Local Rules of Bankruptcy Procedure, U.S. Bankruptcy Court for the District of Oregon (effective December 1, 2025)

- Source: https://www.orb.uscourts.gov/sites/orb/files/documents/local_rules/LBR.120125%20clean.pdf (official site; 105 pages, sha256 147a528f3ae2eb10e70975c6ec4c259c73ad36c3e5ed7518afbbf09fb96b7846).
- Edition: Local Rules of Bankruptcy Procedure 'EFFECTIVE DECEMBER 1, 2025' (cover), linked as 'Local Bankruptcy Rules (LBRs) effective 12/1/25' from https://www.orb.uscourts.gov/rules. The same page posts a 2026 proposals-for-public-comment file, not vendored. robots.txt (Crawl-delay 10 honored; no rule covering these paths) and the Privacy Policy checked: no prohibition of automated access. Fetched with curl and the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 6ef48bfc3d1f031599e819c6c09423581fede9bd3dbee5abf2ef8f44b0afddd3. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-or/original/orb-lbr-2025-12-01.pdf` as served (sha256 147a528f3ae2eb10e70975c6ec4c259c73ad36c3e5ed7518afbbf09fb96b7846); extraction sha256 6ef48bfc3d1f031599e819c6c09423581fede9bd3dbee5abf2ef8f44b0afddd3.

### `bankr-or/orb-lbr-2025-12-01-sectioned.txt` — Bankr. D. Or. Local Rules of Bankruptcy Procedure, sectioned

- Derived in the pod (lg-u561.25) from `bankr-or/orb-lbr-2025-12-01.txt` (extract sha256 6ef48bfc3d1f031599e819c6c09423581fede9bd3dbee5abf2ef8f44b0afddd3); nothing fetched. One `## LBR <n>` line is inserted before each rule's own heading line in the body (an indented line, after an optional form feed, opening with `Rule <nnnn>-<n>.` or `Rule <nnnn>.<n>-<n>.` (the centered heading; the running page headers are not indented); the set is every such heading after the table of contents, from `1001-1` through `9070-1`, 112 rules); rule text was not edited: deleting the marker lines (`sed '/^## LBR /d'`) restores the extract byte for byte.
- Source: as the extract `bankr-or/orb-lbr-2025-12-01.txt`, https://www.orb.uscourts.gov/sites/orb/files/documents/local_rules/LBR.120125%20clean.pdf, retrieved 2026-10-28; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract.
- Sectioned sha256 a5ba665375cf92f03d7aa435bbeb1d438a721c6a6f641fe2ba95bb7a930bf5d1 (the `path_sha256` pinned by `lit/bankr-or/Lit.toml`).
