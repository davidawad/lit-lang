
### `nm-state-civil/nmra-rule-set-1-civil-2026.txt` — NMRA Rule Set 1, Rules of Civil Procedure for the District Courts (unannotated)

- Source: https://nmonesource.com/nmos/nmra-unanno/en/18468/1/document.do (official site; 314 pages, sha256 59a97c6f3e896ba0850de08d39d2b33c2cfb71d3e829e9eec84572c2a532e5eb).
- Edition: NMRA "Unannotated" collection on NMOneSource (New Mexico Compilation Commission, https://www.nmcompcomm.us/scope-of-coverage-2/; site banner "Website last modified: 10/26/2026"); the PDF carries no edition line of its own, its latest amendment notes are effective December 31, 2025; the current "nmra-unanno" collection, not "nmra-historical"; item page https://nmonesource.com/nmos/nmra-unanno/en/item/18468/index.do. Host note: nmonesource.com runs on the Lexum Decisia platform, so the PM may want to confirm it counts as the official State source. Fetched with plain curl using the identifying User-Agent (robots.txt disallows only two other items and the BUbiNG crawler).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 40504a8c243d38d3d5babb3349bdb801cff172d83dee0e9117760359988bdbf4. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `nm-state-civil/original/nmra-rule-set-1-civil-2026.pdf` as served (sha256 59a97c6f3e896ba0850de08d39d2b33c2cfb71d3e829e9eec84572c2a532e5eb); extraction sha256 40504a8c243d38d3d5babb3349bdb801cff172d83dee0e9117760359988bdbf4.

### `nm-state-civil/nmra-rule-set-1-civil-2026-sectioned.txt` — NMRA Rule Set 1, sectioned

- Derived in the pod (lg-f3q8.68) from `nm-state-civil/nmra-rule-set-1-civil-2026.txt` (extract sha256 40504a8c243d38d3d5babb3349bdb801cff172d83dee0e9117760359988bdbf4); nothing fetched. One `## NMRA <n>` line is inserted before each rule's own heading line (`<n>. <title>`, optionally after a form feed), `NMRA 1-001` through the last rule of the set, 175 rules. Rule text was not edited: deleting the marker lines (`perl -ne 'print unless /^## NMRA /'`) restores the extract byte for byte.
- Edition: as the extract (NMRA "Unannotated" collection, retrieved 2026-10-27).
- Sectioned sha256 3a0c71824c0bb61518f0e471fa352f041992926938eecc55b51b382b23622851 (the `path_sha256` pinned by `lit/nm-state-civil/Lit.toml`).
