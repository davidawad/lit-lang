
### `ohsd/ohsd-local-rules.txt` — Local Civil and Criminal Rules, S.D. Ohio

- Source: https://www.ohsd.uscourts.gov/sites/ohsd/files//2024-10-21%20Local%20Rules.pdf (official site; 58 pages, sha256 9e68d137b17d93a176c7d9d0834ad77affa2faa5621dd775590c23aeb5cc0bbf).
- Edition: S.D. Ohio Local Civil and Criminal Rules, "October 21, 2024" as stated on the document; https://www.ohsd.uscourts.gov/local-rules lists "Local Rules as amended October 21, 2024" as the newest edition.
- Also the criminal local rules (lg-gddt): 'LOCAL CRIMINAL RULES' part, from extract line 1991.
- Criminal forum (lg-gddt): ohsd-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Fetched via curl (single document, first request succeeded, no block encountered).
- Extract sha256 c585d5c074b5f98c949dcd687b8df331c46ab62984c6041bf1bc6c936e379118. Raw extract, not yet sectioned into `## <ref>` form.

- Fetch method: first fetched with curl and a desktop browser User-Agent before the no-browser-identity rule (PM 2026-10-27); re-fetched on 2026-10-27 with plain curl (default User-Agent, no browser headers): byte-identical (same sha256).
- Original: `ohsd/original/ohsd-local-rules.pdf` as served (sha256 9e68d137b17d93a176c7d9d0834ad77affa2faa5621dd775590c23aeb5cc0bbf); extraction sha256 c585d5c074b5f98c949dcd687b8df331c46ab62984c6041bf1bc6c936e379118.

### `ohsd/ohsd-local-rules-sectioned.txt` — S.D. Ohio Local Civil Rules, sectioned

- Derived in the pod (lg-f3q8.61) from `ohsd/ohsd-local-rules.txt` (extract sha256 c585d5c074b5f98c949dcd687b8df331c46ab62984c6041bf1bc6c936e379118); nothing fetched. One `## S.D. Ohio Civ. R. <n>` line is inserted right after each Local Civil Rule's own body heading line `<n> <title>` (60 markers, 1.1 through 83.6; the body heading printed `8.1.A Social Security Cases` is marked `## S.D. Ohio Civ. R. 8.1`, the number the rules' own text cites). The table of contents and the Local Criminal Rules are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## S\.D\. Ohio Civ\. R\. /d'`) restores the extract byte for byte.
- Source: as the extract `ohsd/ohsd-local-rules.txt`, https://www.ohsd.uscourts.gov/sites/ohsd/files//2024-10-21%20Local%20Rules.pdf, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, Local Civil and Criminal Rules dated October 21, 2024.
- Sectioned sha256 a405583a3a8bc4312d0d697c948f1dcc0dc64f01847f4617208673316ed58732 (the `path_sha256` pinned by `lit/ohsd/Lit.toml`).
