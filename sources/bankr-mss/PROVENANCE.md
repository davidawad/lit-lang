
### `bankr-mss/mss-uniform-local-rules-2026-03-01.txt` — N.D. and S.D. Miss. Uniform Local Rules (as amended March 1, 2026)

- Source: https://www.mssb.uscourts.gov/sites/mssb/files/RulesOrdersProcs/LocalRules/Final%20Uniform%20Local%20Rules%20%28Clean%20Version%29%20Eff.%203-1-2026.pdf (official site; 75 pages, sha256 ecaeb5f8d4c156ec7940e51499731cac301f4e2310a236e5acce070b6b2f26d4).
- Edition: Uniform Local Rules of the U.S. Bankruptcy Courts for the Northern and Southern Districts of Mississippi, 'originally adopted February 1, 2010, as amended March 1, 2026' (title page), the clean version linked at https://www.mssb.uscourts.gov/local-rules (same document as bankr-msn). A pending Rule 9019-1 amendment effective December 1, 2026 (summary on that page) is not in force and is not vendored. robots.txt (Crawl-delay 10 honored) and the court's privacy/security policy page (https://www.mssb.uscourts.gov/privacy-policy/) checked: no prohibition of automated access.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 15a4fc38f57592a7ec67418d78b4c4f7340a45ef6190d0a91c9ab11edee66ed9. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-mss/original/mss-uniform-local-rules-2026-03-01.pdf` as served (sha256 ecaeb5f8d4c156ec7940e51499731cac301f4e2310a236e5acce070b6b2f26d4); extraction sha256 15a4fc38f57592a7ec67418d78b4c4f7340a45ef6190d0a91c9ab11edee66ed9.

### `bankr-mss/mss-uniform-local-rules-2026-03-01-sectioned.txt` — Uniform Local Rules (N.D. and S.D. Miss.), sectioned

- Derived in the pod (lg-u561.17) from `bankr-mss/mss-uniform-local-rules-2026-03-01.txt` (extract sha256 15a4fc38f57592a7ec67418d78b4c4f7340a45ef6190d0a91c9ab11edee66ed9); nothing fetched. One `## Rule <n>` line is inserted before each rule's own heading line in the body (a line opening, after an optional form feed, with `Rule <nnnn>-<n>` or `Rule <nnnn>.<n>-<n>` at the margin; indented cross-references and the appended forms are unmarked, from `Rule 1001-1.` through `Rule 9037-1.`, 58 rules, the same set as the table of contents); the table of contents is unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Edition: as the extract.
- Sectioned sha256 a545c639410afef70780a2b42c6e86a191cbaa012518f504c50c90a432d55cdd (the `path_sha256` pinned by `lit/bankr-mss/Lit.toml`).
