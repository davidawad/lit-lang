
### `ilcd/ilcd-local-rules.txt` — C.D. Illinois Local Rules

- Source: https://www.ilcd.uscourts.gov/sites/ilcd/files/September%202026%20Version%20.pdf (official site; 135 pages, sha256 d7135a3d60e8a2cbfa16f5fe513007602626de34340b0a07815cb3a70d2a8c30).
- Edition: Local Rules, effective September 3, 2026 (current row of https://www.ilcd.uscourts.gov/local-rules); fetched by curl.
- Also the criminal local rules (lg-gddt): 'CRIMINAL RULES' part, from extract line 3172.
- Criminal forum (lg-gddt): ilcd-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 0960fbf4a9780f19ea31e898d70fe20888c3a337eb18fc376f0b995eadb145a9. Raw extract, not yet sectioned into `## <ref>` form.

- Fetch method: first fetched with curl and a desktop browser User-Agent before the no-browser-identity rule (PM 2026-10-27); re-fetched on 2026-10-27 with plain curl (its default User-Agent, no browser headers): byte-identical (same sha256).
- Original: `ilcd/original/ilcd-local-rules.pdf` as served (sha256 d7135a3d60e8a2cbfa16f5fe513007602626de34340b0a07815cb3a70d2a8c30); extraction sha256 0960fbf4a9780f19ea31e898d70fe20888c3a337eb18fc376f0b995eadb145a9.

### `ilcd/ilcd-local-rules-sectioned.txt` — C.D. Illinois Local Rules (civil rules), sectioned

- Derived in the pod (lg-f3q8.54) from `ilcd/ilcd-local-rules.txt` (extract sha256 0960fbf4a9780f19ea31e898d70fe20888c3a337eb18fc376f0b995eadb145a9); nothing fetched. One `## Rule <n>` line is inserted before each civil rule's own heading line, between the table of contents and the `CRIMINAL RULES` heading (`Rule 1.1` through `Rule 83.16`, 57 rules); the criminal rules, whose numbers repeat civil ones, are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Upstream: https://www.ilcd.uscourts.gov/sites/ilcd/files/September%202026%20Version%20.pdf, retrieved 2026-10-27, as recorded on the `ilcd/ilcd-local-rules.txt` entry this file is derived from.
- Edition: as the extract, effective September 3, 2026.
- Sectioned sha256 2bdc7aa2fdba1320384af050e6ad4d271a1f466233eaa62537a912b68cfb7b26 (the `path_sha256` pinned by `lit/ilcd/Lit.toml`).

### `ilcd/ilcd-criminal-rules-sectioned.txt` — C.D. Illinois Local Rules, criminal rules sectioned

- Derived in the pod (lg-gddt.18) from `ilcd/ilcd-local-rules.txt` (extract sha256 0960fbf4a9780f19ea31e898d70fe20888c3a337eb18fc376f0b995eadb145a9); nothing fetched. In the body of the criminal rules (after the `CRIMINAL RULES` heading at extract line 3077, before the `APPENDIX TO RULES` forms), one marker line `## Criminal Local Rule <n>` is inserted before each rule's own heading line `RULE <n> ...` (Rule 12.1 through Rule 58.2, 19 rules; the rules cite one another as "Criminal Local Rule <n>"). The civil rules and the table of contents carry no such markers. Every other line is the extract unchanged, so deleting the marker lines (`sed '/^## Criminal Local Rule /d'`) gives back the extract byte for byte.
- Edition: as the extract, effective September 3, 2026.
- Upstream: https://www.ilcd.uscourts.gov/sites/ilcd/files/September%202026%20Version%20.pdf, retrieved 2026-10-27, as recorded on `ilcd/ilcd-local-rules.txt` above; sectioning was done from the vendored extract.
- Sectioned sha256 9e638b2497c76ed5072a306463c9d8cdcbf68bd55b95eaafe964b4e3311fa6d8 (the `path_sha256` pinned by `lit/ilcd-criminal/Lit.toml`).
