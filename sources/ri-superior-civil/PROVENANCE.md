
### `ri-superior-civil/ri-superior-rules-civil-procedure.txt` — R.I. Superior Court Rules of Civil Procedure

- Source: https://www.courts.ri.gov/Courts/superiorcourt/Documents/SuperiorCourtRulesOfCivilProcedure.pdf (official site; 101 pages, sha256 01dea708aa73f7816bd816f6ae5d29701b38e5b0571d8c2e1ea01e17acc700f6).
- Edition: No edition line on the document (PDF created 2024-04-10); linked as "Rules of Civil Procedure" from https://www.courts.ri.gov/Legal-Resources/Pages/Superior-Court-Rules.aspx, which states no edition or later amendment.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 ae125faa41bba2587748ae3a258a4b0d4dd1715f234474dad3eb822e6b1e4f8f. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `ri-superior-civil/original/ri-superior-rules-civil-procedure.pdf` as served (sha256 01dea708aa73f7816bd816f6ae5d29701b38e5b0571d8c2e1ea01e17acc700f6); extraction sha256 ae125faa41bba2587748ae3a258a4b0d4dd1715f234474dad3eb822e6b1e4f8f.

### `ri-superior-civil/ri-superior-rules-civil-procedure-sectioned.txt` — R.I. Superior Court Rules of Civil Procedure, sectioned

- Derived in the pod (lg-f3q8.26) from `ri-superior-civil/ri-superior-rules-civil-procedure.txt` (extract sha256 ae125faa41bba2587748ae3a258a4b0d4dd1715f234474dad3eb822e6b1e4f8f); nothing fetched. One `## Rule <n>` line is inserted before each rule's own heading line in the body (`<n>. <Title>. — ...`, some after a form feed): 85 markers, Rule 1 through Rule 86 (Rules 73-76 are not in the text). Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Edition: as the extract; every page is footed "February 2024" (the PDF was created 2024-04-10).
- Sectioned sha256 0a2de5b753150babc82991dd7064124a011191ac8f61803d2a8b79c3cc1c8230 (the `path_sha256` pinned by `lit/ri-superior-civil/Lit.toml`).
