
### `indctapp/in-appellate-rules-2026-07-01.txt` — Indiana Rules of Appellate Procedure (adopted effective January 1, 2001; updated, effective July 1, 2026)

- Source: https://rules.incourts.gov/pdf/PDF%20-%20Appellate/appellate.pdf (official site; 116 pages, sha256 60c64f89d3831e2f6dc5fe0ec474ca663019d258ec4bdf5d8fb3e620649705bb).
- Edition: Indiana Rules of Appellate Procedure, 'Adopted, Effective January 1, 2001; Updated, Effective July 1, 2026' (cover), the official PDF on rules.incourts.gov. fetched on David's explicit ruling 2026-10-28 despite robots/terms; barrier: the Indiana courts' Terms of Use (https://www.in.gov/courts/policies/tou) bar robots and apply to the courts' web presence as a whole; tool: curl with the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 70ff2d2e7c24794b46bfc1214345b12b34c8b26e4cc4f8ed5b4d6cee4bfec7b4. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `indctapp/original/in-appellate-rules-2026-07-01.pdf` as served (sha256 60c64f89d3831e2f6dc5fe0ec474ca663019d258ec4bdf5d8fb3e620649705bb); extraction sha256 70ff2d2e7c24794b46bfc1214345b12b34c8b26e4cc4f8ed5b4d6cee4bfec7b4.

### `indctapp/in-appellate-rules-sectioned.txt` — Indiana Rules of Appellate Procedure, sectioned

- Derived in the pod (lg-f3q8.55) from `indctapp/in-appellate-rules-2026-07-01.txt` (extract sha256 70ff2d2e7c24794b46bfc1214345b12b34c8b26e4cc4f8ed5b4d6cee4bfec7b4); nothing fetched. One `## Ind. Appellate Rule <n>` line is inserted before each rule's own `Rule <n>.` heading line in the body, matched in table-of-contents order (69 rules, Rule 1 through Rule 68, Rule 14.1 included); the table of contents and cross-references that begin a line (e.g. the stray `Rule 24.` line inside Rule 11) are unmarked. Rule text was not edited: deleting the marker lines (`sed -E '/^## Ind\. Appellate Rule /d'`) restores the extract byte for byte.
- Source: as the extract, https://rules.incourts.gov/pdf/PDF%20-%20Appellate/appellate.pdf, retrieved 2026-10-28; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, Indiana Rules of Appellate Procedure, updated, effective July 1, 2026.
- Sectioned sha256 2cf5535875cdf99f49b2f9f4754e75f420d21879303a03afb2c858fbdfa1856e (the `path_sha256` pinned by `lit/in-trial/Lit.toml` and `lit/indctapp/Lit.toml`).
