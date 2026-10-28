
### `lamd/lamd-local-rules.txt` — M.D. La. Local Rules

- Source: https://www.lamd.uscourts.gov/sites/default/files/pdf/2022%20Local%20Rules%20Revisions%208-18-2022.pdf (official site; 78 pages, sha256 cdcfeeed1072e6fb368c5203995f4d247f8735b342a4155ee1762c9087a4e11e).
- Edition: Effective August 18, 2022, as stated on the document and linked from https://www.lamd.uscourts.gov/court-info/local-rules-and-orders. MDLA-only text (civil and criminal); laed and lawd have separate local rules. robots.txt checked; site publishes no terms page; no prohibition found. Page also notes proposed amendments published for comment until Feb 15 2027 (not in force, not vendored).
- Also the criminal local rules (lg-gddt): 'LOCAL CRIMINAL RULES' part, from extract line 3501.
- Criminal forum (lg-gddt): lamd-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 b4212f0329c52a639c4409da6a3720620a1b479fb15d3e36ec8c699b33635246. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `lamd/original/lamd-local-rules.pdf` as served (sha256 cdcfeeed1072e6fb368c5203995f4d247f8735b342a4155ee1762c9087a4e11e); extraction sha256 b4212f0329c52a639c4409da6a3720620a1b479fb15d3e36ec8c699b33635246.

### `lamd/lamd-local-rules-sectioned.txt` — M.D. La. Local Rules, sectioned

- Derived in the pod (lg-f3q8.18) from `lamd/lamd-local-rules.txt` (extract sha256 b4212f0329c52a639c4409da6a3720620a1b479fb15d3e36ec8c699b33635246); nothing fetched. In the body (after the table of contents), one marker line is inserted before each rule's own heading line: `## LR <n>` before each `LOCAL CIVIL RULE <n> - ...` heading (LR 3 through LR 83, 38 rules), `## LAR <n>` before each `LOCAL ADMIRALTY RULE <n> - ...` heading (3 rules) and `## LCrR <n>` before each `LOCAL CRIMINAL RULE <n> - ...` heading (10 rules). Every other line is the extract unchanged, so deleting the `## LR`, `## LAR` and `## LCrR` lines gives back the extract byte for byte.
- Edition: as the extract, "EFFECTIVE AUGUST 18, 2022".
- Sectioned sha256 f61e7174a7c38e7e7e54bc2c4e1d247325437212482383b45249c217d7297954 (the `path_sha256` pinned by `lit/lamd/Lit.toml`).
