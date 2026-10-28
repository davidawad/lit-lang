
### `bankr-nyn/nyn-local-rules-2024-12-01.txt` — N.D.N.Y. Local Bankruptcy Rules (effective December 1, 2024)

- Source: https://www.nynb.uscourts.gov/sites/nynb/files/LBR_GenOrders/Local%20Bankruptcy%20Rules_Effective%2012-01-2024%20Final.pdf (official site; 161 pages, sha256 42cf58029ce1b15f5df588fae5dd701a51993b9d43c151cdfaaa041b5ccdd4e4).
- Edition: Local Bankruptcy Rules 'Effective December 1, 2024' (title page), linked as 'Local Rules' at https://www.nynb.uscourts.gov/local-rules-general-orders; no later amendment is listed there. robots.txt (Crawl-delay 10 honored) and the court's privacy/security policy page (https://www.nynb.uscourts.gov/privacy-policy) checked: no prohibition of automated access.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 6b8070bd12a4015e4a7dd05ddb594dab4a7399306839867157bc66607b4d8a64. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-nyn/original/nyn-local-rules-2024-12-01.pdf` as served (sha256 42cf58029ce1b15f5df588fae5dd701a51993b9d43c151cdfaaa041b5ccdd4e4); extraction sha256 6b8070bd12a4015e4a7dd05ddb594dab4a7399306839867157bc66607b4d8a64.

### `bankr-nyn/nyn-local-rules-2024-12-01-sectioned.txt` — N.D.N.Y. Local Bankruptcy Rules, sectioned

- Derived in the pod (lg-u561.11) from `bankr-nyn/nyn-local-rules-2024-12-01.txt` (extract sha256 6b8070bd12a4015e4a7dd05ddb594dab4a7399306839867157bc66607b4d8a64); nothing fetched. One `## LBR <n>` line is inserted before each rule's own heading line in the body (`RULE 1001-1  SHORT TITLE – APPLICABILITY` through `RULE 9037-2`, including the spaced `RULE 7026- 2` and the lower-case `Rule 7040-1`; 100 sections, matching the table of contents); the table of contents and the appendices' own text are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## LBR /d'`) restores the extract byte for byte.
- Source: as the extract `bankr-nyn/nyn-local-rules-2024-12-01.txt`, https://www.nynb.uscourts.gov/sites/nynb/files/LBR_GenOrders/Local%20Bankruptcy%20Rules_Effective%2012-01-2024%20Final.pdf, retrieved 2026-10-28; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, effective December 1, 2024.
- Sectioned sha256 25212ec01be96753b15348ccfcd3bae15a0e09d419b1e434bebbc4d5577d06d3 (the `path_sha256` pinned by `lit/bankr-nyn/Lit.toml` for source `nynb-lbr`).
