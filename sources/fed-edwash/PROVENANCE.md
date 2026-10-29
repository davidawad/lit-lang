
### `fed-edwash/edwash-local-civil-rules.txt` — Eastern District of Washington Local Civil Rules

- Source: https://www.waed.uscourts.gov/sites/cst/waed/files/local-rules/LocalCivilRules---November-2025.pdf (official site; 40 pages, sha256 832f0fbecae7328a602ebcabe3aef9a2afc9509490cfe471da85cc197f542b42).
- Edition: Eastern District of Washington Local Civil Rules (LCivR), "effective November 17, 2025" (LCivR 86), linked as "Local Civil Rules" from https://www.waed.uscourts.gov/rules-policies/local-rules.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 fd9445ae2e9d1c2d26c33bb482e9060fc5b2e973fe5091b080e6dfad84fdb0e5. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `fed-edwash/original/edwash-local-civil-rules.pdf` as served (sha256 832f0fbecae7328a602ebcabe3aef9a2afc9509490cfe471da85cc197f542b42); extraction sha256 fd9445ae2e9d1c2d26c33bb482e9060fc5b2e973fe5091b080e6dfad84fdb0e5.

### `fed-edwash/edwash-local-criminal-rules-2015-02-20.txt` — E.D. Wash. Local Criminal Rules (effective February 20, 2015)

- Source: https://www.waed.uscourts.gov/sites/cst/waed/files/local-rules/LocalCriminalRules.pdf (official site; 17 pages, sha256 b9115c4078491a7bb4a5f8b37e228a4f57e3a045e9e4077a21d1c5d97b6968a4).
- Edition: Local Criminal Rules (LCrR) 'effective February 20, 2015' (text), the 'PDF Version' at https://www.waed.uscourts.gov/about-court/rules-policies/local-rules/local-criminal-rules. OLD EDITION: the court presents it as current. robots.txt of www.waed.uscourts.gov read (Crawl-delay 10 honored where stated; no Disallow covers the fetched path); the judiciary-wide Privacy & Security Policy (https://www.uscourts.gov/privacy-security-policy) read: no prohibition of automated access. Fetched with curl and the identifying UA lit-lang-provenance/1.0, at most 1 request per 10 s.
- Criminal forum (lg-gddt): waed-criminal.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 66063b8bc957c37a4732495370b9df2676fcf16711c34d38764e33ae20a857f6. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `fed-edwash/original/edwash-local-criminal-rules-2015-02-20.pdf` as served (sha256 b9115c4078491a7bb4a5f8b37e228a4f57e3a045e9e4077a21d1c5d97b6968a4); extraction sha256 66063b8bc957c37a4732495370b9df2676fcf16711c34d38764e33ae20a857f6.

### `fed-edwash/edwash-local-civil-rules-sectioned.txt` — E.D. Washington Local Civil Rules, sectioned

- Derived in the pod (lg-f3q8.48) from `fed-edwash/edwash-local-civil-rules.txt` (extract sha256 fd9445ae2e9d1c2d26c33bb482e9060fc5b2e973fe5091b080e6dfad84fdb0e5); nothing fetched. One `## LCivR <n>` line is inserted before each rule's own centered heading line (a line holding only `LCivR <n>` or `LCivR <n>.<n>`, from `LCivR 1` through `LCivR 86`, 102 rules); the introduction is unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## LCivR /d'`) restores the extract byte for byte.
- Source: as the extract `fed-edwash/edwash-local-civil-rules.txt`, https://www.waed.uscourts.gov/sites/cst/waed/files/local-rules/LocalCivilRules---November-2025.pdf, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, Local Civil Rules effective November 17, 2025.
- Sectioned sha256 6ca79396cb160e969bb8dd5bdbba675dd6b3b726a3205648555481b23f5ca095 (the `path_sha256` pinned by `lit/waed/Lit.toml`).

### `fed-edwash/edwash-local-criminal-rules-2015-02-20-sectioned.txt` — E.D. Wash. Local Criminal Rules (effective February 20, 2015), sectioned

- Derived in the pod (lg-gddt.22) from `fed-edwash/edwash-local-criminal-rules-2015-02-20.txt` (extract sha256 66063b8bc957c37a4732495370b9df2676fcf16711c34d38764e33ae20a857f6); nothing fetched. One marker line `## LCrR <n>` is inserted before each rule's own body heading line `LCrR <n>` (indented or centered, alone on its line, from extract line 82; LCrR 1 through 61, 74 rules including reserved ones, matching the table of rules); the table carries no markers. Every other line is the extract unchanged, so deleting the marker lines gives back the extract byte for byte.
- Source: as the extract `fed-edwash/edwash-local-criminal-rules-2015-02-20.txt`, https://www.waed.uscourts.gov/sites/cst/waed/files/local-rules/LocalCriminalRules.pdf; Retrieved 2026-10-28 with the extract (this file was derived from it in the pod, not fetched itself).
- Edition: as the extract, effective February 20, 2015 (LCrR 57(b)). OLD EDITION presented by the court as current.
- Sectioned sha256 aaa4633f837117f9920bdb1ca5c967c0d15757b2c5ca066bfd8bb2f7043ed1b0 (the `path_sha256` pinned by `lit/waed-criminal/Lit.toml`).
