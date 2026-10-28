
### `in-criminal-trial-court/in-rules-of-criminal-procedure.txt` — Indiana Rules of Criminal Procedure (36 pp; Criminal Rule 4 is the speedy-trial rule)

- Source: https://rules.incourts.gov/pdf/PDF%20-%20Criminal/criminal.pdf (official site; 36 pages, sha256 d4e812541e120c088465402e218bd8bb4ed1310e123069be11500125a98a083e).
- Edition: Indiana Rules of Criminal Procedure, 'Updated, Effective January 1, 2025' (cover), the official PDF on rules.incourts.gov (later amendment orders, if any, were not checked). rules.incourts.gov serves no robots.txt (no robots file returned); home page read: no terms or privacy link found; fetched on David's explicit ruling 2026-10-28 despite robots/terms (barrier: the Indiana courts' Terms of Use (https://www.in.gov/courts/policies/tou) bar robots and apply to the courts' web presence as a whole (per the lg-f3q8.77 row)); tool: curl with the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 ad37354d3fc708c0a40b90dcfe1bfafa8f1a03936ad7305b0c8343435a5ec702. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `in-criminal-trial-court/original/in-rules-of-criminal-procedure.pdf` as served (sha256 d4e812541e120c088465402e218bd8bb4ed1310e123069be11500125a98a083e); extraction sha256 ad37354d3fc708c0a40b90dcfe1bfafa8f1a03936ad7305b0c8343435a5ec702.

- Note (lg-gddt): Criminal appeals: Indiana Rules of Appellate Procedure, covered by sources/indctapp/ (in-appellate-rules-2026-07-01.txt).

### `in-criminal-trial-court/in-rules-of-criminal-procedure-sectioned.txt` — Indiana Rules of Criminal Procedure, sectioned

- Derived in the pod (lg-gddt.18) from `in-criminal-trial-court/in-rules-of-criminal-procedure.txt` (extract sha256 ad37354d3fc708c0a40b90dcfe1bfafa8f1a03936ad7305b0c8343435a5ec702); nothing fetched. After the table of contents, one marker line `## Ind. Crim. Rule <n>` is inserted before each rule's own heading line `Rule <n>. ...` (some open with a form feed; Rule 1.1 through Rule 6.2, 24 rules). The table of contents is unmarked. Every other line is the extract unchanged, so deleting the marker lines (`sed '/^## Ind. Crim. Rule /d'`) gives back the extract byte for byte.
- Edition: as the extract, "Updated, Effective January 1, 2025".
- Sectioned sha256 7db08ac4c67ad49564804a44f2ad60725cf0fe3eb6f3b8fa8a3b193081de8860 (the `path_sha256` pinned by `lit/in-criminal-trial-court/Lit.toml`).
