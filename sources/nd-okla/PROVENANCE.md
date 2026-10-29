
### `nd-okla/nd-okla-local-rules-2026.txt` — N.D. Okla. Local Rules (LCvR)

- Source: https://www.oknd.uscourts.gov/sites/default/files/2026-07-31%20Local%20Rules.pdf (official site; 95 pages, sha256 acb17193b5b6a6c0651484c14fc850f17e8affac0679dbffe4ff5d9b49d69400).
- Edition: Local Rules, Effective Date 7/31/2026 (title page), linked as "Local Rules" from https://www.oknd.uscourts.gov/local-rules; fetched with plain curl (no browser identity); robots.txt allows the path (Crawl-delay 10 honored).
- Also the criminal local rules (lg-gddt): 'CRIMINAL RULES' part, from extract line 2382.
- Criminal forum (lg-gddt): oknd-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 23a3bc7a926356cc1e063f4b823cbeb3487728e8ce0a98d83346bff30c461db1. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `nd-okla/original/nd-okla-local-rules-2026.pdf` as served (sha256 acb17193b5b6a6c0651484c14fc850f17e8affac0679dbffe4ff5d9b49d69400); extraction sha256 23a3bc7a926356cc1e063f4b823cbeb3487728e8ce0a98d83346bff30c461db1.

### `nd-okla/nd-okla-local-rules-2026-criminal-sectioned.txt` — N.D. Okla. Local Rules, criminal rules sectioned

- Derived in the pod (lg-gddt.24) from `nd-okla/nd-okla-local-rules-2026.txt` (extract sha256 23a3bc7a926356cc1e063f4b823cbeb3487728e8ce0a98d83346bff30c461db1); nothing fetched. In the body of the Criminal Rules (after the table of contents), one marker line `## <n>` is inserted before each rule's own heading line `<n> <Title>.`, where `<n>` is the rule's designation as the rules write it (`LCrR1-1` through `LCrR58-1`, with the `LCvR49.1-1` the criminal part carries), 56 rules. The table of contents, the `LCrR<n> – <Title>` group lines and the civil rules carry no markers. Rule text was not edited: deleting the marker lines (each `## LCrR` or `## LCvR` line with its newline) restores the extract byte for byte.
- Source: as the extract `nd-okla/nd-okla-local-rules-2026.txt`, https://www.oknd.uscourts.gov/sites/default/files/2026-07-31%20Local%20Rules.pdf; Retrieved 2026-10-27 with the extract (this file was derived from it in the pod, not fetched itself).
- Edition: as the extract, Local Rules "Effective Date: 7/31/2026".
- Sectioned sha256 e3dd4216a76092003426b54a5c61db73a62d511749c324a0dc5612999837293a (the `path_sha256` pinned by `lit/oknd-criminal/Lit.toml`).

### `nd-okla/nd-okla-local-rules-2026-civil-sectioned.txt` — N.D. Okla. Local Rules, civil rules sectioned

- Derived in the pod (lg-f3q8.88) from `nd-okla/nd-okla-local-rules-2026.txt` (extract sha256 23a3bc7a926356cc1e063f4b823cbeb3487728e8ce0a98d83346bff30c461db1); nothing fetched. In the body of the Civil Rules (after the table of contents and the general rules, before the Criminal Rules), one marker line `## <n>` is inserted before each rule's own heading line `<n> <Title>.`, where `<n>` is the rule's designation as the rules write it (`LCvR3-1` through `LCvR84-1`, 53 rules). The table of contents, the `LCvR<n> – <Title>` group lines, the general rules and the criminal rules carry no markers (LCvR84-1's section runs to the end of the file). Rule text was not edited: deleting the marker lines (each `## LCvR` line with its newline) restores the extract byte for byte.
- Source: as the extract `nd-okla/nd-okla-local-rules-2026.txt`, https://www.oknd.uscourts.gov/sites/default/files/2026-07-31%20Local%20Rules.pdf; Retrieved 2026-10-27 with the extract (this file was derived from it in the pod, not fetched itself).
- Edition: as the extract, Local Rules "Effective Date: 7/31/2026".
- Sectioned sha256 79dfc08efc079799d595432f08e26973d9aa2f7edfeb3836855ec83917f941cd (the `path_sha256` pinned by `lit/oknd/Lit.toml`).
