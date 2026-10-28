
### `ncmd/ncmd-local-civil-rules.txt` — U.S. District Court for the Middle District of North Carolina Local Civil Rules

- Source: https://www.ncmd.uscourts.gov/sites/ncmd/files/mdnc%20local%20civil%20rules%2020260601.pdf (official site; 73 pages, sha256 038c3a7f8b990469ed768200dd2f0fc4ed321331b1cb29642b987ef25bb22018).
- Edition: Rules of Practice and Procedure (Local Civil Rules), effective June 1, 2026 (cover date; linked as "Local Civil Rules - Effective June 1, 2026" on https://www.ncmd.uscourts.gov/local-rules-and-orders).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 4e10d0ff97155d7c1bcf6e36dd90abc62b6ecaad0e4278ed4204242e54d2274f. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `ncmd/original/ncmd-local-civil-rules.pdf` as served (sha256 038c3a7f8b990469ed768200dd2f0fc4ed321331b1cb29642b987ef25bb22018); extraction sha256 4e10d0ff97155d7c1bcf6e36dd90abc62b6ecaad0e4278ed4204242e54d2274f.

### `ncmd/ncmd-local-criminal-rules-2026-06-01.txt` — M.D.N.C. Local Rules of Criminal Practice (effective June 1, 2026)

- Source: https://www.ncmd.uscourts.gov/sites/ncmd/files/mdnc%20local%20criminal%20rules%2020260601.pdf (official site; 12 pages, sha256 1c993d7ed91417d93af21ed75855f9e89facbe4d064a7d5d603772c608ab34dc).
- Edition: Part Two, Local Rules of Criminal Practice (LCrR), 'Effective June 1, 2026' (title page), linked as 'Local Criminal Rules' at https://www.ncmd.uscourts.gov/local-rules-and-orders (a marked version and the civil-rules order of May 28, 2026 are not vendored). robots.txt of www.ncmd.uscourts.gov read (Crawl-delay 10 honored where stated; no Disallow covers the fetched path); the judiciary-wide Privacy & Security Policy (https://www.uscourts.gov/privacy-security-policy) read: no prohibition of automated access. Fetched with curl and the identifying UA lit-lang-provenance/1.0, at most 1 request per 10 s.
- Criminal forum (lg-gddt): ncmd-criminal.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 da180ab0f0c0c1af3d5c64a7240f97770a670c5bd2e0b89795de1fadb5b467e3. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `ncmd/original/ncmd-local-criminal-rules-2026-06-01.pdf` as served (sha256 1c993d7ed91417d93af21ed75855f9e89facbe4d064a7d5d603772c608ab34dc); extraction sha256 da180ab0f0c0c1af3d5c64a7240f97770a670c5bd2e0b89795de1fadb5b467e3.

### `ncmd/ncmd-local-civil-rules-sectioned.txt` — M.D.N.C. Local Civil Rules (effective June 1, 2026), sectioned

- Derived in the pod (lg-f3q8.29) from `ncmd/ncmd-local-civil-rules.txt` (extract sha256 4e10d0ff97155d7c1bcf6e36dd90abc62b6ecaad0e4278ed4204242e54d2274f); nothing fetched. One `## LR <n>` line is inserted right before each rule's own body heading line `LR <n>  <TITLE>` (some after a form feed; from extract line 320 on): 79 markers, LR 1.1 through LR 83.12. The table of contents and the in-body cross-references are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## LR /d'`) restores the extract byte for byte.
- Edition: as the extract, the Local Civil Rules effective June 1, 2026.
- Sectioned sha256 7c67c21dbe347f03c46adc55f9b43edb79da8e5cf52919d9c00322dac37bf801 (the `path_sha256` pinned by `lit/ncmd/Lit.toml`).
