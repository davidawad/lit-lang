
### `bankr-ils/ilsb-lbr-2015-12-01.txt` — Local Rules of Bankruptcy Procedure, S.D. Ill. (June 1, 2012, revised December 1, 2015)

- Source: https://www.ilsb.uscourts.gov/sites/ilsb/files/LocalRules-BkSoDistrict_12-01-15.pdf (official site; 42 pages, sha256 146f9f0632cb4d4b329f0574e32dbf4a091647cae4695fc19504271375580e95).
- Edition: Local Rules of Bankruptcy Procedure 'June 1, 2012; Revised December 1, 2015 to Reflect Updated Form Numbers and Names' (cover), the current-version link 'ILSB Local Rules - Revised December 1, 2015' on https://www.ilsb.uscourts.gov/local-rules (the page's other two links are the June 1, 2012 edition and a rescinded set). robots.txt (Crawl-delay 10 honored; no rule covering these paths) and the Privacy Policy checked: no prohibition of automated access. Fetched with curl and the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 728dffb66e1f0328937bb8f085b079ebbe5d67ec965f97dc1c19f0fe7f16c2bd. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-ils/original/ilsb-lbr-2015-12-01.pdf` as served (sha256 146f9f0632cb4d4b329f0574e32dbf4a091647cae4695fc19504271375580e95); extraction sha256 728dffb66e1f0328937bb8f085b079ebbe5d67ec965f97dc1c19f0fe7f16c2bd.

### `bankr-ils/ilsb-lbr-2015-12-01-sectioned.txt` — S.D. Ill. Local Rules of Bankruptcy Procedure, sectioned

- Derived in the pod (lg-u561.21) from `bankr-ils/ilsb-lbr-2015-12-01.txt` (extract sha256 728dffb66e1f0328937bb8f085b079ebbe5d67ec965f97dc1c19f0fe7f16c2bd); nothing fetched. One `## LBR <n>` line is inserted before each rule's body heading (`L.R. <n> – <Title>`, after the table of contents; `LBR 1001` through `LBR 9070`, `LBR 3002.1` keeping its point; 83 sections); the cover, the order approving the rules, the committee list, the table of contents and Appendix A are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## LBR /d'`) restores the extract byte for byte.
- Source: as the extract, https://www.ilsb.uscourts.gov/sites/ilsb/files/LocalRules-BkSoDistrict_12-01-15.pdf (official site), retrieved 2026-10-28 on carbon; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, June 1, 2012, revised December 1, 2015.
- Sectioned sha256 6543bdc901c48b26cbcb4f3a1a988d142aaef7fd3aacd3fe8e0fd633d8e12046 (the `path_sha256` pinned by `lit/bankr-ils/Lit.toml` for source `ilsb-lbr`).
