
### `wd-okla/wd-okla-local-rules-2026.txt` — W.D. Okla. Local Court Rules (LCvR)

- Source: https://www.okwd.uscourts.gov/sites/okwd/files/LocalRules2026.pdf (official site; 108 pages, sha256 ec602e29b4e91d0f87f3a0549dcb0141e0f4cd40cdaecbe6ccd95daf3fdeee81).
- Edition: Local Court Rules, revised as of April 23, 2026 (title page), linked as "Local Rules - Effective April 2026" from https://www.okwd.uscourts.gov/local-rules; fetched with plain curl (no browser identity); robots.txt allows the path (Crawl-delay 10 honored).
- Also the criminal local rules (lg-gddt): 'LOCAL CRIMINAL RULES' part, from extract line 1790.
- Criminal forum (lg-gddt): okwd-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 5d8863c249291655b7d7bde6b3520c0b18fe064a6d74d92926d4a574a8d04c9e. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `wd-okla/original/wd-okla-local-rules-2026.pdf` as served (sha256 ec602e29b4e91d0f87f3a0549dcb0141e0f4cd40cdaecbe6ccd95daf3fdeee81); extraction sha256 5d8863c249291655b7d7bde6b3520c0b18fe064a6d74d92926d4a574a8d04c9e.

### `wd-okla/wd-okla-local-rules-2026-criminal-sectioned.txt` — W.D. Okla. Local Court Rules, criminal rules sectioned

- Derived in the pod (lg-gddt.24) from `wd-okla/wd-okla-local-rules-2026.txt` (extract sha256 5d8863c249291655b7d7bde6b3520c0b18fe064a6d74d92926d4a574a8d04c9e); nothing fetched. In the body of the Local Criminal Rules (after the table of contents), one marker line `## <n>` is inserted before each rule's own heading line `<n> <Title>`, where `<n>` is the rule's designation as written (`LCrR1.1` through `LCrR58.2`, 50 rules). The table of contents, the civil rules and the appendices carry no markers (LCrR58.2's section runs to the end of the file). Rule text was not edited: deleting the marker lines (each `## LCrR` line with its newline) restores the extract byte for byte.
- Edition: as the extract, Local Court Rules revised as of April 23, 2026.
- Sectioned sha256 72b239eadea3d599fab4091afa78c82f566d49b5f97985c4a9741fcfedfcde06 (the `path_sha256` pinned by `lit/okwd-criminal/Lit.toml`).
