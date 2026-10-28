
### `fl-criminal-trial-court/fl-rules-of-criminal-procedure-2026-10-01.txt` — Florida Rules of Criminal Procedure (497 pp; includes Rule 3.191, speedy trial)

- Source: https://www-media.floridabar.org/uploads/2026/10/2027_04-Oct-Criminal-Procedure-Rules-10-1-2026.pdf (official site; 497 pages, sha256 9f6d0a95acf1ec480c9e89a0a68f875adb5a29db7721b0d52d9eb773dfc904aa).
- Edition: Florida Rules of Criminal Procedure, file '2027_04-Oct-Criminal-Procedure-Rules-10-1-2026', linked as 'Chapter 3 – Rules of Criminal Procedure' from https://www.floridabar.org/rules/ctproc/ (the Florida Bar's posting of the Supreme Court's rules; edition 10/1/2026 per the file name). www-media.floridabar.org robots.txt read; no Disallow covering this path; home page not retrievable (403); terms not read; fetched on David's explicit ruling 2026-10-28 despite robots/terms; tool: curl with the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 01fea499d8d9a69b89564437dd517eb18b34f04a55fbc4dcf072db60734b01a8. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `fl-criminal-trial-court/original/fl-rules-of-criminal-procedure-2026-10-01.pdf` as served (sha256 9f6d0a95acf1ec480c9e89a0a68f875adb5a29db7721b0d52d9eb773dfc904aa); extraction sha256 01fea499d8d9a69b89564437dd517eb18b34f04a55fbc4dcf072db60734b01a8.

- Note (lg-gddt): Criminal appeals: Florida Rules of Appellate Procedure, covered by sources/fl-state-appellate/.

### `fl-criminal-trial-court/fl-rules-of-criminal-procedure-2026-10-01-sectioned.txt` — Florida Rules of Criminal Procedure, sectioned

- Derived in the pod (lg-gddt.26) from `fl-criminal-trial-court/fl-rules-of-criminal-procedure-2026-10-01.txt` (extract sha256 01fea499d8d9a69b89564437dd517eb18b34f04a55fbc4dcf072db60734b01a8); nothing fetched. One `## Rule <n>` line is inserted before each rule's own heading line in the body (`RULE 3.010.` through the forms `RULE 3.9876.`, 156 rules, the same 156 the table of contents lists; three form headings open with a form feed); the table of contents is unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Source: as the extract, https://www-media.floridabar.org/uploads/2026/10/2027_04-Oct-Criminal-Procedure-Rules-10-1-2026.pdf, retrieved 2026-10-28; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, the October 1, 2026 edition.
- Sectioned sha256 0bfe9f774dcd8e56e9bc17ef5adc1ac45d2d99c3ac89d2c5d601745832c0a300 (the `path_sha256` pinned by `lit/fl-criminal-trial-court/Lit.toml`).
