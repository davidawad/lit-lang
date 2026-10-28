
### `ilnd/ilnd-local-rules.txt` — N.D. Illinois Local Rules

- Source: https://www.ilnd.uscourts.gov/_assets/_documents/_rules/LRRULES.pdf (official site; 159 pages, sha256 033bacf6dd51b004d9869c1ed25a6dea2c620d8ae0d5c6370c88489b8d9ae96c).
- Edition: Local Rules of the N.D. Ill., "effective September 1, 1999 with amendments through May 4, 2026" (document title page), linked from https://www.ilnd.uscourts.gov/LandingPage.php?page=local_rules; fetched by curl.
- Also the criminal local rules (lg-gddt): 'CRIMINAL RULES' part, from extract line 4156.
- Criminal forum (lg-gddt): ilnd-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 11c6f33721819a2d069e85d9898e2fc1b56ceb350583f95607bb307ae378069c. Raw extract, not yet sectioned into `## <ref>` form.

- Fetch method: first fetched with curl and a desktop browser User-Agent before the no-browser-identity rule (PM 2026-10-27); re-fetched on 2026-10-27 with plain curl (its default User-Agent, no browser headers): byte-identical (same sha256).
- Original: `ilnd/original/ilnd-local-rules.pdf` as served (sha256 033bacf6dd51b004d9869c1ed25a6dea2c620d8ae0d5c6370c88489b8d9ae96c); extraction sha256 11c6f33721819a2d069e85d9898e2fc1b56ceb350583f95607bb307ae378069c.

### `ilnd/ilnd-local-rules-sectioned.txt` — N.D. Illinois Local Rules, sectioned

- Derived in the pod (lg-f3q8.54) from `ilnd/ilnd-local-rules.txt` (extract sha256 11c6f33721819a2d069e85d9898e2fc1b56ceb350583f95607bb307ae378069c); nothing fetched. One `## LR <n>` line is inserted before each rule's own heading line after the table of contents (`LR 3.1` through `LR 83.58.5`, 108 rules; the first heading of each number only); in-text references at the start of a line (`LR 5.3(b) and ...`, `LR 83.16 is ...`) are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## LR /d'`) restores the extract byte for byte.
- Edition: as the extract, effective September 1, 1999 with amendments through May 4, 2026.
- Sectioned sha256 89f817ee22099c4a2e7c7ce5440900990f6db5e63128a1f41c09910fca078d39 (the `path_sha256` pinned by `lit/ilnd/Lit.toml`).
