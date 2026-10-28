
### `nmcca/nmcca-rules.txt` — NMCCA Rules of Appellate Procedure with JRAP

- Source: https://www.jag.navy.mil/documents/4691/2022_NMCCA_Rules_of_App_Proc.pdf (official site; 118 pages, sha256 2ae7408f6a85bb739894cac79421e85250d2dc35f1c8004399b817182f625b9c).
- Edition: NMCCA Rules of Appellate Procedure, effective 1 June 2022, with the Joint Rules (cover; linked as "NMCCA Rules of Appellate Procedure" from https://www.jag.navy.mil/about/organization/ojag/code-05/nmcca/ on 2026-10-27; the page lists no later edition). Fetched with curl; robots.txt allows..
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 5c99707bcc6b95e2a88e177d830a2346c406bf2f9456343b802aafdc91e1bfa6. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `nmcca/original/nmcca-rules.pdf` as served (sha256 2ae7408f6a85bb739894cac79421e85250d2dc35f1c8004399b817182f625b9c); extraction sha256 5c99707bcc6b95e2a88e177d830a2346c406bf2f9456343b802aafdc91e1bfa6.

### `nmcca/nmcca-rules-sectioned.txt` — NMCCA Rules of Appellate Procedure with JRAP, sectioned

- Derived in the pod (lg-f3q8.130) from `nmcca/nmcca-rules.txt` (extract sha256 5c99707bcc6b95e2a88e177d830a2346c406bf2f9456343b802aafdc91e1bfa6); nothing fetched. One `## NMCCA R. <n>` line is inserted before each rule's own `Rule <n>. <TITLE>` heading line in the body (32 rules, Rule 1 through Rule 32), and one `## NMCCA R. App. L` line before the body heading of Appendix L (Summary of Filing Time Standards); the table of rules is unmarked. Each Service Court rule (e.g. the unnumbered "Answer and Reply Briefs." heading the rules call Rule 18.4) stays inside its Joint Rule's section. Rule text was not edited: deleting the marker lines (`sed -E '/^## NMCCA R\. /d'`) restores the extract byte for byte.
- Source: as the extract, https://www.jag.navy.mil/documents/4691/2022_NMCCA_Rules_of_App_Proc.pdf, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, NMCCA Rules of Appellate Procedure, effective 1 June 2022, with the Joint Rules.
- Sectioned sha256 df704eee30133e0f1dd2efa0ed9b907a5c9501a71c7c8017c154c6a5d2bd9d2b (the `path_sha256` pinned by `lit/nmcca/Lit.toml`).
