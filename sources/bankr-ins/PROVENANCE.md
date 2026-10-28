
### `bankr-ins/insb-lbr-2025-12-01.txt` — Local Rules, U.S. Bankruptcy Court for the Southern District of Indiana (effective June 1, 2010, as amended through December 1, 2025)

- Source: https://www.insb.uscourts.gov/sites/insb/files/Local-Rules.pdf (official site; 99 pages, sha256 7bdbda10109b1a8a8ae0967ce7d4e9b756acae3470dd585074b5318a14473997).
- Edition: Local Rules 'Effective June 1, 2010; As Amended Through December 1, 2025' (cover), linked as 'Local Rules PDF version' from https://www.insb.uscourts.gov/content/local-rules. robots.txt (Crawl-delay 10 honored; no rule covering these paths) and the Privacy Policy checked: no prohibition of automated access. Fetched with curl and the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 44a57c5ff8f7e3affb8df674546c12d77f8b4f369cd13f85da391e5209c6a060. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-ins/original/insb-lbr-2025-12-01.pdf` as served (sha256 7bdbda10109b1a8a8ae0967ce7d4e9b756acae3470dd585074b5318a14473997); extraction sha256 44a57c5ff8f7e3affb8df674546c12d77f8b4f369cd13f85da391e5209c6a060.

### `bankr-ins/insb-lbr-2025-12-01-sectioned.txt` — S.D. Ind. Local Rules, sectioned

- Derived in the pod (lg-u561.21) from `bankr-ins/insb-lbr-2025-12-01.txt` (extract sha256 44a57c5ff8f7e3affb8df674546c12d77f8b4f369cd13f85da391e5209c6a060); nothing fetched. One `## LBR <n>` line is inserted before each rule's body heading `B-<n>.  <TITLE>` (after the table of contents; the court's `B-` prefix dropped, so `LBR 1000-1` heads rule B-1000-1, through `LBR 9070-1`, with `LBR 3002.1-1` keeping its point; 110 sections); the cover and the table of contents are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## LBR /d'`) restores the extract byte for byte.
- Edition: as the extract, effective June 1, 2010, as amended through December 1, 2025.
- Sectioned sha256 87dbad10ea1900f6d25884a4edcf764a8d34308ff9857490b17b47e78b1154e8 (the `path_sha256` pinned by `lit/bankr-ins/Lit.toml` for source `insb-lbr`).
