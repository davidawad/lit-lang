
### `lawd/lawd-local-rules.txt` — W.D. La. Civil and Criminal Local Rules

- Source: https://www.lawd.uscourts.gov/sites/lawd/files/UPLOADS/CivilCrimLRs.Barbara.updated.Ver3.030526.ppm.pdf (official site; 45 pages, sha256 80e6ba6dcea5200b390b51e7ffaff8cf3549f36e5de3786d3b72639bab89fbb5).
- Edition: Effective May 7, 2024 (current version, file dated Mar 5 2026), as stated on the document and linked as "Local Rules (PDF)(Current Version)" at https://www.lawd.uscourts.gov/local-rules. WDLA-only text; laed and lamd have separate local rules. Amending order (TAD.Order.LRchanges.attachedrules.050724.pdf) is already incorporated and not vendored separately. robots.txt (Crawl-delay 10 honored) and privacy policy checked: no prohibition.
- Also the criminal local rules (lg-gddt): 'LOCAL CRIMINAL RULES' part, from extract line 1749.
- Criminal forum (lg-gddt): lawd-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 019eb5a35283abebbce5ba0404b57b0faa141d40b8c2865e3490444724a59588. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `lawd/original/lawd-local-rules.pdf` as served (sha256 80e6ba6dcea5200b390b51e7ffaff8cf3549f36e5de3786d3b72639bab89fbb5); extraction sha256 019eb5a35283abebbce5ba0404b57b0faa141d40b8c2865e3490444724a59588.

### `lawd/lawd-local-rules-sectioned.txt` — W.D. La. Civil and Criminal Local Rules, sectioned

- Derived in the pod (lg-f3q8.18) from `lawd/lawd-local-rules.txt` (extract sha256 019eb5a35283abebbce5ba0404b57b0faa141d40b8c2865e3490444724a59588); nothing fetched. In the body (after the table of contents), one marker line is inserted before each rule's own heading line: `## LR <n>` before each local civil rule heading (`LR3.1 Collateral Proceedings and Refiled Cases` through `LR83.4.4 Record Transmitted to the District Court`, 108 rules), `## LAR <n>` before each `LOCAL ADMIRALTY RULE <n> - ...` heading (3 rules) and `## LCrR <n>` before each `LOCAL CRIMINAL RULE <n> - ...` heading (10 rules). Every other line is the extract unchanged, so deleting the `## LR`, `## LAR` and `## LCrR` lines gives back the extract byte for byte.
- Upstream: https://www.lawd.uscourts.gov/sites/lawd/files/UPLOADS/CivilCrimLRs.Barbara.updated.Ver3.030526.ppm.pdf, retrieved 2026-10-27, as recorded on the `lawd/lawd-local-rules.txt` entry this file is derived from.
- Edition: as the extract, "Effective: May 7, 2024".
- Sectioned sha256 98d5dc04a3b3f2956976bc5d7f877837c858c95afb11b2e4f733b611e47efa1a (the `path_sha256` pinned by `lit/lawd/Lit.toml` and, for the `## LCrR` and `## LR 7.1` sections, by `lit/lawd-criminal/Lit.toml`, lg-gddt.14).
