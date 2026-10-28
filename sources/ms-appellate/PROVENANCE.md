
### `ms-appellate/rules-of-appellate-procedure.txt` — Mississippi Rules of Appellate Procedure

- Source: https://courts.ms.gov/research/rules/msrulesofcourt/Rules%20of%20Appellate%20Procedure%20Current.pdf (official site; 183 pages, sha256 3615b91c68ce90481d2fe54d275aff762e03486127af3a0e10c0e95406d9b942).
- Edition: Mississippi Rules of Appellate Procedure, file named "Current" on https://courts.ms.gov/research/rules/rules.php; latest amendment in the text June 23, 2025; fetched with curl (plain User-Agent) from the PDF URL (link found by reading the rules index once in BrowserOS neo, tab closed); site has no terms page and robots.txt does not disallow /research/rules/.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 fc399122e8f718c61f2a110c76e9d3a12c5cd9f102ecf692d3c188232b9da750. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `ms-appellate/original/rules-of-appellate-procedure.pdf` as served (sha256 3615b91c68ce90481d2fe54d275aff762e03486127af3a0e10c0e95406d9b942); extraction sha256 fc399122e8f718c61f2a110c76e9d3a12c5cd9f102ecf692d3c188232b9da750.

### `ms-appellate/rules-of-appellate-procedure-sectioned.txt` — Mississippi Rules of Appellate Procedure, sectioned

- Derived in the pod (lg-f3q8.19) from `ms-appellate/rules-of-appellate-procedure.txt` (extract sha256 fc399122e8f718c61f2a110c76e9d3a12c5cd9f102ecf692d3c188232b9da750); nothing fetched. After the table of contents, one `## M.R.A.P. <n>` line is inserted before each rule's own heading line (`RULE 1. SCOPE OF RULES` through `RULE 50. APPELLATE MEDIATION`, 54 headings including 35-A, 35-B, 48A, 48B and 48C). Every other line is the extract unchanged, so deleting the `## ` lines gives back the extract byte for byte.
- Edition: as the extract (the "Current" file; latest amendment in the text June 23, 2025).
- Sectioned sha256 0b719c334e842c8d3b7912af0e4ec6adb2bdc04d225d77b3c47ad16637032e28 (the `path_sha256` pinned by `lit/ms-appellate/Lit.toml`).
