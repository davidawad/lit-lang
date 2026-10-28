
### `de-criminal-trial-court/de-superior-court-criminal-rules.txt` — Rules of Criminal Procedure for the Superior Court of the State of Delaware (68 pp)

- Source: https://courts.delaware.gov/forms/download.aspx?id=304498 (official site; 68 pages, sha256 953877ac94ae7b00b440c209c39ebe3f917df952694baf0edb3d8ff7bdfb90e7).
- Edition: Rules of Criminal Procedure, Superior Court of Delaware, the PDF linked from https://courts.delaware.gov/rules/ (PDF metadata created 2026-03-23); no edition date on the document, CURRENCY UNCONFIRMED. Speedy trial is Rule 48 in this set. courts.delaware.gov robots.txt read; no Disallow covering this path; Accessibility page (https://courts.delaware.gov/aoc/ada.aspx) read: no automated-access prohibition found; fetched on David's explicit ruling 2026-10-28 despite robots/terms; tool: curl with the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 323f719a38c913db1851d655a48c540a930893169da98f9f96ead2eed93c9e84. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `de-criminal-trial-court/original/de-superior-court-criminal-rules.pdf` as served (sha256 953877ac94ae7b00b440c209c39ebe3f917df952694baf0edb3d8ff7bdfb90e7); extraction sha256 323f719a38c913db1851d655a48c540a930893169da98f9f96ead2eed93c9e84.

- Note (lg-gddt): Delaware Court of Common Pleas and Family Court criminal rules are separate documents (courts.delaware.gov/forms/download.aspx?id=174058 and id=39318) and are not vendored here; criminal appeals are in the Delaware Supreme Court Rules, covered by sources/de-supreme-civil/ if it carries them.

### `de-criminal-trial-court/de-superior-court-criminal-rules-sectioned.txt` — Delaware Superior Court Rules of Criminal Procedure, sectioned

- Derived in the pod (lg-gddt.10) from `de-criminal-trial-court/de-superior-court-criminal-rules.txt` (extract sha256 323f719a38c913db1851d655a48c540a930893169da98f9f96ead2eed93c9e84); nothing fetched. One marker line `## Del. Super. Ct. Crim. R. <n>` is inserted before each rule heading line `Rule <n>. <Title>` (78 rules, Rule 1 through Rule 64, with the special pretrial-release Rules 5.2 to 5.4 at the end of the document). The placeholder heading `Rule 5.2. to 5.4.` after Rule 5.1 is marked `## Del. Super. Ct. Crim. R. 5.2-5.4 [placeholder]`, and extract line 1733, which begins with "Rule 12." mid-paragraph inside Rule 41(f), is not a heading and carries no marker. Every other line is the extract unchanged, so deleting the marker lines (`sed '/^## Del. Super. Ct. Crim. R. /d'`) gives back the extract byte for byte.
- Edition: as the extract; CURRENCY UNCONFIRMED stands. `lit/de-criminal-trial-court` cites it for structure only and takes no value from it.
- Source: as the extract, https://courts.delaware.gov/forms/download.aspx?id=304498, retrieved 2026-10-28; the sectioning was done from the vendored extract.
- Sectioned sha256 9fe56d886e34be142af1b3ba1a2ed1154bb7f79a9964ef804d002c0a06277961 (the `path_sha256` pinned by `lit/de-criminal-trial-court/Lit.toml`).
