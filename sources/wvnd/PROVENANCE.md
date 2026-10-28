
### `wvnd/wvnd-local-rules-2018-03.txt` — N.D. W. Va. Local Rules (effective March 2018; civil and criminal)

- Source: http://www.wvnd.uscourts.gov/sites/wvnd/files/NDWV%20Local%20Rules%202018%20MASTER.pdf (official site; 162 pages, sha256 7b5520cb858b057cd98a1b54a6954b7e4a9ccf590a8f96934dfbc647abe249e9).
- Edition: Local Rules 'EFFECTIVE: MARCH, 2018' (title page), the single master file linked as 'Local Rules' at http://www.wvnd.uscourts.gov/ (home page); it holds the district's civil and criminal local rules, no separate criminal document was found. OLD EDITION: the court presents it as current. Fetched because sources/wvnd did not exist. robots.txt of www.wvnd.uscourts.gov read (Crawl-delay 10 honored where stated; no Disallow covers the fetched path); the judiciary-wide Privacy & Security Policy (https://www.uscourts.gov/privacy-security-policy) read: no prohibition of automated access. Fetched with curl and the identifying UA lit-lang-provenance/1.0, at most 1 request per 10 s.
- Criminal forum (lg-gddt): wvnd-criminal.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 2cb6b762a3f170542b33fa7adb251106a01ef5647b15365743aee6b471c02e64. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `wvnd/original/wvnd-local-rules-2018-03.pdf` as served (sha256 7b5520cb858b057cd98a1b54a6954b7e4a9ccf590a8f96934dfbc647abe249e9); extraction sha256 2cb6b762a3f170542b33fa7adb251106a01ef5647b15365743aee6b471c02e64.

### `wvnd/wvnd-local-rules-2018-03-sectioned.txt` — N.D. W. Va. Local Rules (March 2018), civil rules sectioned

- Derived in the pod (lg-f3q8.32) from `wvnd/wvnd-local-rules-2018-03.txt` (extract sha256 2cb6b762a3f170542b33fa7adb251106a01ef5647b15365743aee6b471c02e64); nothing fetched. One `## LR Civ P <n>` line is inserted right before each Local Rule of Civil Procedure's own body heading line `LR Civ P <n>. <title>` (some after a form feed): 35 markers, LR Civ P 1.01 through LR Civ P 78.01. The table of contents and the General, Criminal and Prisoner Litigation rules are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## LR Civ P /d'`) restores the extract byte for byte.
- Edition: as the extract, effective March 2018.
- Sectioned sha256 2ac98eb1735026721b26eb16aecdeef4cccd43ef801943ebb4b697180329fff5 (the `path_sha256` pinned by `lit/wvnd/Lit.toml`).
