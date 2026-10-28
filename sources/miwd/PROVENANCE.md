
### `miwd/miwd-local-rules.txt` — Local Rules, W.D. Michigan

- Source: https://www.miwd.uscourts.gov/sites/miwd/files/MIWD_Local_Rules.pdf (official site; 94 pages, sha256 61027d97dc83d2d8ea6c198f285912fa8e47d6dc5ad18b3f1a8d07d252468040).
- Edition: W.D. Mich. Local Rules of Practice and Procedure, PDF dated Aug 13, 2026; page https://www.miwd.uscourts.gov/court-info/local-rules-and-orders states amendments adopted by Administrative Order 26-RL-067 (Aug 13, 2026) are effective October 12, 2026.
- Also the criminal local rules (lg-gddt): 'Local Criminal Rules' part, from extract line 2207.
- Criminal forum (lg-gddt): miwd-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Fetched via curl (single document, first request succeeded, no block encountered).
- Extract sha256 e109f9bd7511adf428460db58bd09c120e5a68558bc2fdde4f604169ed3796f5. Raw extract, not yet sectioned into `## <ref>` form.

- Fetch method: first fetched with curl and a desktop browser User-Agent before the no-browser-identity rule (PM 2026-10-27); re-fetched on 2026-10-27 with plain curl (default User-Agent, no browser headers): byte-identical (same sha256).

- Currency confirmed 2026-10-28 (lg-f3q8.60): this file IS the amended text in force from October 12, 2026. Administrative Order 26-RL-067 (Aug. 13, 2026; vendored below as miwd-ao-26-rl-067.txt) adopts the rules "as set forth in the attachments" and makes them effective sixty days after its date (October 12, 2026, as the court's local-rules page states). Its 94-page attachment matches this PDF word for word (pdftotext of both, page numbers and rule-leader underscores ignored: 29,573 words each, 0 differences), and the PDF the court serves on 2026-10-28 is byte-identical to this original (same sha256). Bind values with in_force 2026-10-12; text before that date is the prior edition, not vendored.
- Original: `miwd/original/miwd-local-rules.pdf` as served (sha256 61027d97dc83d2d8ea6c198f285912fa8e47d6dc5ad18b3f1a8d07d252468040); extraction sha256 e109f9bd7511adf428460db58bd09c120e5a68558bc2fdde4f604169ed3796f5.

### `miwd/miwd-ao-26-rl-067.txt` — W.D. Mich. Administrative Order 26-RL-067 (adopting the October 12, 2026 Local Rules)

- Source: https://www.miwd.uscourts.gov/sites/miwd/files/Admin%20Order%2026-RL-067.pdf (official site; 95 pages, sha256 81eb19f489b8de956f387b7a53a30d5c9f2b5a0f3e928cd10745e3c11807c561).
- Edition: Administrative Order No. 26-RL-067, dated August 13, 2026 (Chief Judge Hala Y. Jarbou), adopting the amended Local General, Civil and Criminal Rules as set forth in its attachments, effective sixty days from its date (October 12, 2026, per https://www.miwd.uscourts.gov/court-info/local-rules-and-orders). Its attachment is the same text as miwd-local-rules.txt (word-for-word comparison 2026-10-28); vendored as the dated adoption record, not as a separate layer. robots.txt (no rule covering /sites/) checked: no prohibition of automated access.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 1d8e9bc66e352d97ccf3bcc62e144f790304ffdfab868dc2652b1b53de69ff88. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `miwd/original/miwd-ao-26-rl-067.pdf` as served (sha256 81eb19f489b8de956f387b7a53a30d5c9f2b5a0f3e928cd10745e3c11807c561); extraction sha256 1d8e9bc66e352d97ccf3bcc62e144f790304ffdfab868dc2652b1b53de69ff88.

### `miwd/miwd-local-civil-rules-sectioned.txt` — W.D. Mich. Local Civil Rules, sectioned

- Derived in the pod (lg-f3q8.60) from `miwd/miwd-local-rules.txt` (extract sha256 e109f9bd7511adf428460db58bd09c120e5a68558bc2fdde4f604169ed3796f5); nothing fetched. One `## LCivR <n>` line is inserted right before each Local Civil Rule's numbered subrule line (`<n>  <title>`, e.g. `7.2   Dispositive motions`): 65 markers, LCivR 1.1 through LCivR 83.2. The table of contents, the Local General Rules and the Local Criminal Rules are unmarked (LCivR 83.2's section runs to the end of the file). Rule text was not edited: deleting the marker lines (`sed '/^## LCivR /d'`) restores the extract byte for byte.
- Upstream: https://www.miwd.uscourts.gov/sites/miwd/files/MIWD_Local_Rules.pdf, retrieved 2026-10-27, as recorded on the `miwd/miwd-local-rules.txt` entry this file is derived from.
- Edition: as the extract, the Local Rules as amended by Administrative Order 26-RL-067 (August 13, 2026), in force from October 12, 2026.
- Sectioned sha256 b009aa700a37360e9ed6e024bc3e273546ea34423dd5c7cbe3fd6ea745f5ff2a (the `path_sha256` pinned by `lit/miwd/Lit.toml`).

### `miwd/miwd-local-criminal-rules-sectioned.txt` — W.D. Mich. Local Criminal Rules, sectioned
- Derived in the pod (lg-gddt.16) from `miwd/miwd-local-rules.txt` (extract sha256 e109f9bd7511adf428460db58bd09c120e5a68558bc2fdde4f604169ed3796f5); nothing fetched. One `## LCrR <n>` line is inserted right before each Local Criminal Rule's numbered subrule line (`<n>  <title>`, e.g. `47.1   Dispositive motions`) from the "Local Criminal Rule 1" heading (extract line 2152) on: 50 markers, LCrR 1.1 through LCrR 58.2 (58.2's section runs to the end of the file). The table of contents, the Local General Rules and the Local Civil Rules are unmarked (the civil rules are sectioned separately in `miwd-local-civil-rules-sectioned.txt`). Rule text was not edited: deleting the marker lines (`sed '/^## LCrR /d'`) restores the extract byte for byte.
- Edition: as the extract, the Local Rules as amended by Administrative Order 26-RL-067 (August 13, 2026), in force from October 12, 2026.
- Sectioned sha256 8f226ec6a907a2fbc5bf9817acae5890ecf50a58a7eeb278b05cf37b2aa3a3b5 (the `path_sha256` pinned by `lit/miwd-criminal/Lit.toml`).
