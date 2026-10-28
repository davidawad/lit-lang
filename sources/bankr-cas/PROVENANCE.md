
### `bankr-cas/casb-lbr-2025-12-01.txt` — Local Bankruptcy Rules and Administrative Procedures, S.D. Cal. Bankruptcy Court (December 1, 2025)

- Source: https://www.casb.uscourts.gov/sites/casb/files/2025_LBR_and_AdminProc_2025-12-01_R2.pdf (official site; 124 pages, sha256 9af0effcd09df1001e7ce6e0cdfdb613f1091148b8fcf47dbb51b3b869bcd16b).
- Edition: Local Bankruptcy Rules and Administrative Procedures 'December 1, 2025' (cover), linked as 'Local Rules and Administrative Procedures' from https://www.casb.uscourts.gov/rules_procedures (the site's /rules-procedures path returned 404 and the rules page is /rules_procedures). robots.txt (Crawl-delay 10 honored; no rule covering these paths) and the Privacy Policy checked: no prohibition of automated access. Fetched with curl and the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 ba09f6281960a98f72c46e16255f71c6c077a337312392b95d4e237cd7268eea. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-cas/original/casb-lbr-2025-12-01.pdf` as served (sha256 9af0effcd09df1001e7ce6e0cdfdb613f1091148b8fcf47dbb51b3b869bcd16b); extraction sha256 ba09f6281960a98f72c46e16255f71c6c077a337312392b95d4e237cd7268eea.

### `bankr-cas/casb-lbr-2025-12-01-sectioned.txt` — Bankr. S.D. Cal. Local Bankruptcy Rules, sectioned

- Derived in the pod (lg-u561.25) from `bankr-cas/casb-lbr-2025-12-01.txt` (extract sha256 ba09f6281960a98f72c46e16255f71c6c077a337312392b95d4e237cd7268eea); nothing fetched. One `## LBR <n>` line is inserted before each rule's own heading line in the body (a line opening, after an optional form feed and without indentation, with `<nnnn>-<n>.`; the Administrative Procedures and forms bound after the rules fall in the last section; the set is every such heading after the table of contents, from `1001-1` through `9034-2`, 177 rules); rule text was not edited: deleting the marker lines (`sed '/^## LBR /d'`) restores the extract byte for byte.
- Edition: as the extract.
- Sectioned sha256 b6ae868f4f58bb688466e8bb07b79ca5d2afe0436c81570d60de2039a01178e6 (the `path_sha256` pinned by `lit/bankr-cas/Lit.toml`).
