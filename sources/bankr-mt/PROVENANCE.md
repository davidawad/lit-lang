
### `bankr-mt/mtb-lbr-2025-12-01.txt` — Local Bankruptcy Rules, U.S. Bankruptcy Court for the District of Montana (effective December 1, 2025)

- Source: https://www.mtb.uscourts.gov/sites/mtb/files/LocalBankruptcyRules_12012025.pdf (official site; 84 pages, sha256 006216a0ddf68e5e718e8dc4809b773fe769bdeb98efb5fd7919ba76690d8bb5).
- Edition: Local Bankruptcy Rules 'Effective December 1, 2025' (cover), linked as 'U.S. Bankruptcy Court Local Rules' from https://www.mtb.uscourts.gov/rules-statutes. The same page posts 2026 proposed amendments (effective December 1, 2026 if adopted) for public comment only; they are proposals and are not vendored. robots.txt (Crawl-delay 10 honored; no rule covering these paths) and the Privacy Policy checked: no prohibition of automated access. Fetched with curl and the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 033bed723dab175d5b0d57235db169fc0a0427d9dc26d9ce3c6b19a0e183ed30. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-mt/original/mtb-lbr-2025-12-01.pdf` as served (sha256 006216a0ddf68e5e718e8dc4809b773fe769bdeb98efb5fd7919ba76690d8bb5); extraction sha256 033bed723dab175d5b0d57235db169fc0a0427d9dc26d9ce3c6b19a0e183ed30.

### `bankr-mt/mtb-lbr-2025-12-01-sectioned.txt` — Bankr. D. Mont. Local Bankruptcy Rules, sectioned

- Derived in the pod (lg-f3q8.44) from `bankr-mt/mtb-lbr-2025-12-01.txt` (extract sha256 033bed723dab175d5b0d57235db169fc0a0427d9dc26d9ce3c6b19a0e183ed30); nothing fetched. One `## LBR <n>` line is inserted before each rule's own heading line in the body (a line opening, after an optional form feed, with `RULE <nnnn>-<n>`, from `RULE 1001-1.` through `RULE 9037-1.`, 113 rules, the same set as the table of contents); the table of contents and the running footers (a `RULE ...` line directly after `Mont. LBR December 1, 2025`) are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## LBR /d'`) restores the extract byte for byte.
- Source: as the extract `bankr-mt/mtb-lbr-2025-12-01.txt`, https://www.mtb.uscourts.gov/sites/mtb/files/LocalBankruptcyRules_12012025.pdf, retrieved 2026-10-28; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, Local Bankruptcy Rules effective December 1, 2025.
- Sectioned sha256 3bc1c3ffcccb3c9c06abee1c0e17fd49a30194d6196aa55320d11b43542fa541 (the `path_sha256` pinned by `lit/bankr-mt/Lit.toml`).
