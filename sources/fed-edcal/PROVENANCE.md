
### `fed-edcal/edcal-local-rules.txt` — E.D. Cal. Local Rules

- Source: https://www.caed.uscourts.gov/caednew/assets/File/EDCA%20LOCAL%20RULES%20EFF_%2002-23-26.pdf (official site; 280 pages, sha256 280a7078af778ed24ce067dd5a9ac0122347b62140c1eca71c49e9f72ffd75f5).
- Edition: E.D. Cal. Local Rules, effective February 23, 2026, listed as current on https://www.caed.uscourts.gov/caednew/index.cfm/rules/local-rules/ (no proposed amendments pending). Fetched with curl (robots.txt disallows only /caed/DOCUMENTS/, not this path).
- Also the criminal local rules (lg-gddt): 'CRIMINAL RULES' part, from extract line 6503.
- Criminal forum (lg-gddt): caed-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 ca7b71025ec07ae5ae1598a8699d4748770c49875b7603440cb720e8e3437d20. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `fed-edcal/original/edcal-local-rules.pdf` as served (sha256 280a7078af778ed24ce067dd5a9ac0122347b62140c1eca71c49e9f72ffd75f5); extraction sha256 ca7b71025ec07ae5ae1598a8699d4748770c49875b7603440cb720e8e3437d20.

### `fed-edcal/edcal-local-rules-sectioned.txt` — E.D. Cal. Local Rules, sectioned

- Derived in the pod (lg-f3q8.39) from `fed-edcal/edcal-local-rules.txt` (extract sha256 ca7b71025ec07ae5ae1598a8699d4748770c49875b7603440cb720e8e3437d20); nothing fetched. One `## E.D. Cal. L.R. <n>` line is inserted right after each rule's own body heading line (`RULE <n>`, alone or followed by its Federal Rule cross-reference; 119 markers, 100 through 590, including 141.1, 162.1 and the like). The table of contents and the index are unmarked. Rule text was not edited: deleting the marker lines (each `## E.D. Cal. L.R. ` line with its newline) restores the extract byte for byte.
- Edition: as the extract, Local Rules effective February 23, 2026.
- Sectioned sha256 2687cac2861239ae26dffd16fce684409b0c852691175126e9b9f709fbde344c (the `path_sha256` pinned by `lit/caed/Lit.toml`).
