
### `bankr-ut/utb-lbr-2025-12-01.txt` — Local Rules of Practice, U.S. Bankruptcy Court for the District of Utah (effective December 1, 2025)

- Source: https://www.utb.uscourts.gov/sites/utb/files/2025_local_rules-apr26.pdf (official site; 220 pages, sha256 1ace00fb52224b5c08824c556d48a4890947a896f0a9599029ddcd6b986ae419).
- Edition: Local Rules of Practice 'Effective Date: December 1, 2025' (cover; file name '2025_local_rules-apr26', so a revision posted in April 2026), linked from https://www.utb.uscourts.gov/local-rules-2025; the court's Local Rules and Orders page says the December 1, 2025 rules supersede all prior versions and Standing Orders. robots.txt (Crawl-delay 10 honored; no rule covering these paths) and the Privacy Policy checked: no prohibition of automated access. Fetched with curl and the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 070598f0b265a2454b43477b1790314770ab59230b1f0aafd2c39381e2c251e5. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-ut/original/utb-lbr-2025-12-01.pdf` as served (sha256 1ace00fb52224b5c08824c556d48a4890947a896f0a9599029ddcd6b986ae419); extraction sha256 070598f0b265a2454b43477b1790314770ab59230b1f0aafd2c39381e2c251e5.
- Amending-order check (lg-u561.30): order page read 2026-10-27 (https://www.utb.uscourts.gov/local-rules-and-orders): no amending order after 2025-12-01; General Order 25-003 (Adoption of Amended Local Rules of the Bankruptcy Court) is the Dec 1, 2025 adoption itself. The vendored file is named 2025_local_rules-apr26 (a later posting), and no order or note on the page explains a post-December revision.

### `bankr-ut/utb-lbr-2025-12-01-sectioned.txt` — Bankr. D. Utah Local Rules of Practice, sectioned

- Derived in the pod (lg-u561.27) from `bankr-ut/utb-lbr-2025-12-01.txt` (extract sha256 070598f0b265a2454b43477b1790314770ab59230b1f0aafd2c39381e2c251e5); nothing fetched. One `## LBR <n>` line is inserted before each rule's own heading line in the rules body (a line opening at the margin, after an optional form feed, with `RULE <nnnn>-<n>` or `RULE <nnnn>.<n>-<n>`), from `RULE 1001-1` through `RULE 9073-1`, 71 rules; the table of contents and the local forms after the rules are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## LBR /d'`) restores the extract byte for byte.
- Edition: as the extract.
- Sectioned sha256 18951a0cfb998a11252586f384d525ffe25bd8052b0c15e139975119253b5848 (the `path_sha256` pinned by `lit/bankr-ut/Lit.toml`).
