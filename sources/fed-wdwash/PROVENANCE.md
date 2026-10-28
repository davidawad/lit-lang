
### `fed-wdwash/wdwash-local-civil-rules.txt` — Western District of Washington Local Civil Rules

- Source: https://www.wawd.uscourts.gov/sites/wawd/files/WAWD%20Local%20Civil%20Rules%20%282026%29%20-%20Clean.pdf (official site; 157 pages, sha256 d9befea781ff1b8d449e6b0bf598097fbf570e64f7234a56fb3de1c9ef704562).
- Edition: Western District of Washington Local Civil Rules (LCR), "last revised effective January 13, 2026" (LCR 86), linked as "Local Civil Rules" (Effective 1/13/2026) from https://www.wawd.uscourts.gov/local-rules-and-orders; the proposed amendments (comments due 2026-11-02) on the same page are not in force and not vendored.
- Also the criminal local rules (lg-gddt): the Local Criminal Rules (cited 'CrR') are part of this combined document; citation rule at extract line 4702.
- Criminal forum (lg-gddt): wawd-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 26669aad3709b58b19c3d1ebbb1f90a2c94cd202294c33bac13d443847e08037. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `fed-wdwash/original/wdwash-local-civil-rules.pdf` as served (sha256 d9befea781ff1b8d449e6b0bf598097fbf570e64f7234a56fb3de1c9ef704562); extraction sha256 26669aad3709b58b19c3d1ebbb1f90a2c94cd202294c33bac13d443847e08037.

### `fed-wdwash/wdwash-local-civil-rules-sectioned.txt` — W.D. Washington Local Civil Rules, sectioned

- Derived in the pod (lg-f3q8.48) from `fed-wdwash/wdwash-local-civil-rules.txt` (extract sha256 26669aad3709b58b19c3d1ebbb1f90a2c94cd202294c33bac13d443847e08037); nothing fetched. One `## LCR <n>` line is inserted before each rule's own centered heading line in the body (a line holding only `LCR <n>` or `LCR <n>.<n>`, from `LCR 1` through `LCR 103`, 71 rules); the table of contents, the introduction, the reserved ranges headed `LCR <n> THROUGH <n>` or `LCR <n> AND <n>` and the appendices are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## LCR /d'`) restores the extract byte for byte.
- Edition: as the extract, Local Civil Rules last revised effective January 13, 2026.
- Sectioned sha256 6e36487f123d77750b4503501397b9ff436c3ee1667e22c19e1717f5523f9fc3 (the `path_sha256` pinned by `lit/wawd/Lit.toml`).
