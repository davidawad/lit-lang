# `mnd` sources: provenance

### `mnd/local-rules.txt` — Local Rules, D. Minn.

- Source: https://www.mnd.uscourts.gov/sites/mnd/files/Local-Rules-Master.pdf (official site; 142 pages, sha256 01aa8892b4d346f976b203d66a6739fa8b1bb028d3b005092ddd003da796eda1).
- Edition: Local Rules of the U.S. District Court for the District of Minnesota, document dated June 10, 2026 (complete set; linked from https://www.mnd.uscourts.gov/local-rules-complete-set, "current Local Rules").
- Also the criminal local rules (lg-gddt): no separate criminal part; criminal provisions include LR 12.1 Criminal Discovery, Pretrial Motions (extract line 2721) and LR 49.1.
- Criminal forum (lg-gddt): mnd-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 0ec9d04835b62cfe8eb3311701377d6a42aaeba62d121912039c7dc88e6eb2e6. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `mnd/original/local-rules.pdf` as served (sha256 01aa8892b4d346f976b203d66a6739fa8b1bb028d3b005092ddd003da796eda1); extraction sha256 0ec9d04835b62cfe8eb3311701377d6a42aaeba62d121912039c7dc88e6eb2e6.

### `mnd/local-rules-sectioned.txt` — Local Rules, D. Minn., sectioned

- Derived in the pod (lg-f3q8.7) from `mnd/local-rules.txt` (extract sha256 0ec9d04835b62cfe8eb3311701377d6a42aaeba62d121912039c7dc88e6eb2e6); nothing fetched. One `## LR <n>` line is inserted before each rule's own heading line in the body (`LR 1.1 SCOPE OF THE RULES` through `LR 83.13 COURT APPOINTEES`, 60 rules, including abrogated and renumbered rules and the headings that follow a form feed); the table of contents is unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## LR /d'`) restores the extract byte for byte.
- Edition: as the extract, the complete set dated June 10, 2026.
- Sectioned sha256 55de9589101301bed6b7ee7835d50d4f628b1dc51f9ce2c3365e845bfa1b50cf (the `path_sha256` pinned by `lit/mnd/Lit.toml` for source `mnd-lr`).
