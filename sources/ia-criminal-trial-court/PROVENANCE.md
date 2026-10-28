
### `ia-criminal-trial-court/ia-court-rules-chapter-2-criminal-procedure.txt` — Iowa Court Rules, Chapter 2, Rules of Criminal Procedure: Indictable Offenses (103 pp; Rule 2.33 is speedy trial)

- Source: https://www.legis.iowa.gov/docs/ACO/CourtRulesChapter/2.pdf (official site; 103 pages, sha256 4a64068b4d059421929afaf7ee8ef25f00f8e962d852f4cf7184d91c852e0eda).
- Edition: Iowa Court Rules Chapter 2 'Rules of Criminal Procedure: Indictable Offenses', page header 'May 2025', the Iowa Legislature's posting (legis.iowa.gov/docs/ACO/CourtRulesChapter/2.pdf); later amendments, if any, not checked. www.legis.iowa.gov robots.txt read; no Disallow covering this path; Online Privacy Policy page (https://www.legis.iowa.gov/misc/privacy) read: no automated-access prohibition found; fetched on David's explicit ruling 2026-10-28 despite robots/terms; tool: curl with the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 fdc9d52711a0bf355fe0b846c9d3738d3986d2064e277325d7b5a8100d18d1e3. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `ia-criminal-trial-court/original/ia-court-rules-chapter-2-criminal-procedure.pdf` as served (sha256 4a64068b4d059421929afaf7ee8ef25f00f8e962d852f4cf7184d91c852e0eda); extraction sha256 fdc9d52711a0bf355fe0b846c9d3738d3986d2064e277325d7b5a8100d18d1e3.

- Note (lg-gddt): Criminal appeals: Iowa Court Rules Chapter 6 (appellate procedure), covered by sources/iowactapp/ if it carries them.

### `ia-criminal-trial-court/ia-rules-criminal-procedure-sectioned.txt` — Iowa Court Rules, Chapter 2, Rules of Criminal Procedure, sectioned

- Derived in the pod (lg-gddt.20) from `ia-criminal-trial-court/ia-court-rules-chapter-2-criminal-procedure.txt` (extract sha256 fdc9d52711a0bf355fe0b846c9d3738d3986d2064e277325d7b5a8100d18d1e3); nothing fetched. One `## Rule <n>` line is inserted before each rule's own heading line in the body (`Rule <n> <Title>`, some after a page's form feed, which stays on the heading line after the marker): `Rule 2.1` through `Rule 2.86`, 70 rules, reserved rules included; the form headings (`Rule 2.36 — Form 1` and the like) and the table of contents are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Edition: as the extract, running header May 2025; newest amendment note effective May 26, 2025.
- Sectioned sha256 22f3efa6757d7dfcf0050bf958a668cb90d39c9dfcd8f5f6317b3a104ef1af77 (the `path_sha256` pinned by `lit/ia-criminal-trial-court/Lit.toml`).
