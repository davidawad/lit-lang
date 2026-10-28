
### `bankr-alm/almb-lbr-2024-04-01.txt` — Local Rules, U.S. Bankruptcy Court for the Middle District of Alabama (effective April 1, 2024)

- Source: https://www.almb.uscourts.gov/sites/almb/files/ALMB%20Local%20Rules%2C%20April%201%2C%202024.pdf (official site; 57 pages, sha256 86b99199bb5e33aa9ac9aeab4d83884df3a640eff55b78ffdc56916fa6c8f12f).
- Edition: Local Rules 'Effective April 1, 2024' (cover), linked as 'Local Rules Effective April 1, 2024' from https://www.almb.uscourts.gov/local-rules-and-administrative-orders. robots.txt (Crawl-delay 10 honored; no rule covering these paths) and the Privacy Policy checked: no prohibition of automated access. Fetched with curl and the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 96c09d024434a7f8f869fac92ee723c2af88e77b492875343b8576eece3bebce. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-alm/original/almb-lbr-2024-04-01.pdf` as served (sha256 86b99199bb5e33aa9ac9aeab4d83884df3a640eff55b78ffdc56916fa6c8f12f); extraction sha256 96c09d024434a7f8f869fac92ee723c2af88e77b492875343b8576eece3bebce.

### `bankr-alm/almb-lbr-2024-04-01-sectioned.txt` — M.D. Ala. Local Rules, sectioned

- Derived in the pod (lg-u561.29) from `bankr-alm/almb-lbr-2024-04-01.txt` (extract sha256 96c09d024434a7f8f869fac92ee723c2af88e77b492875343b8576eece3bebce); nothing fetched. One `## Rule <n>` line is inserted before each rule's own heading line in the body (`RULE 1002-1 PETITION - GENERAL` through `RULE 9072-1 ORDERS - PROPOSED`, each following a form feed; 41 sections, matching the table of contents); the table of contents and the preamble (extract lines 1-75) are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Source: as the extract `bankr-alm/almb-lbr-2024-04-01.txt`, https://www.almb.uscourts.gov/sites/almb/files/ALMB%20Local%20Rules%2C%20April%201%2C%202024.pdf, retrieved 2026-10-28; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, the Local Rules effective April 1, 2024.
- Sectioned sha256 0bc752f4274dda1d3698f34834066fbba8316e42991da79d3dd814b332910fce (the `path_sha256` pinned by `lit/bankr-alm/Lit.toml` for source `almb-lbr`).
