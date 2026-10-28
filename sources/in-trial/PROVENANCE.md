
### `in-trial/in-trial-rules-2026-08-01.txt` — Indiana Rules of Trial Procedure (adopted January 1, 1970; updated, effective August 1, 2026)

- Source: https://rules.incourts.gov/pdf/PDF%20-%20Trial/trial.pdf (official site; 216 pages, sha256 34afd2958651379db88f18df64487773aa61a39c076fcef0edd5ea9190953e3d).
- Edition: Indiana Rules of Trial Procedure, 'Adopted, January 1, 1970; Updated, Effective August 1, 2026' (cover), the official PDF on rules.incourts.gov. fetched on David's explicit ruling 2026-10-28 despite robots/terms; barrier: the Indiana courts' Terms of Use (https://www.in.gov/courts/policies/tou) bar robots and apply to the courts' web presence as a whole; tool: curl with the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 b35ed75477deb9e6cfdabf3a61f94e0196a37980b06583fdac398dc2de2173a0. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `in-trial/original/in-trial-rules-2026-08-01.pdf` as served (sha256 34afd2958651379db88f18df64487773aa61a39c076fcef0edd5ea9190953e3d); extraction sha256 b35ed75477deb9e6cfdabf3a61f94e0196a37980b06583fdac398dc2de2173a0.

### `in-trial/in-trial-rules-sectioned.txt` — Indiana Rules of Trial Procedure, sectioned

- Derived in the pod (lg-f3q8.55) from `in-trial/in-trial-rules-2026-08-01.txt` (extract sha256 b35ed75477deb9e6cfdabf3a61f94e0196a37980b06583fdac398dc2de2173a0); nothing fetched. One `## Ind. Trial Rule <n>` line is inserted before each rule's own `Rule <n>.` heading line in the body, matched in table-of-contents order (123 rules, Rule 1 through Rule 88, the numbered sub-rules such as 4.1 and 53.3 included); the table of contents and cross-references that begin a line are unmarked. Rule text was not edited: deleting the marker lines (`sed -E '/^## Ind\. Trial Rule /d'`) restores the extract byte for byte.
- Source: as the extract, https://rules.incourts.gov/pdf/PDF%20-%20Trial/trial.pdf, retrieved 2026-10-28; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, Indiana Rules of Trial Procedure, updated, effective August 1, 2026.
- Sectioned sha256 92ba49510cdaa0189c416ba35f18cbb4c46eb3107cf6a25dde81f11a1dd7091c (the `path_sha256` pinned by `lit/in-trial/Lit.toml`).
