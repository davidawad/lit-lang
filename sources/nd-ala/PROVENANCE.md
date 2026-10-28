
### `nd-ala/local-rules.txt` — N.D. Ala. Local Rules

- Source: https://alnd.uscourts.gov/sites/alnd/files/ALND%20Local%20Rules%20Revised%2012-04-2019.pdf (official site; 14 pages, sha256 b9295a56444a7d46926807423bca614fae4ae5ce04f5c6031e82a7eb6870e4a2).
- Edition: Effective December 4, 2019 (cover); listed as "Local Rules as of December 4, 2019" on https://alnd.uscourts.gov/local-rules; fetched with curl (plain User-Agent), no bot refusal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 5fefae2739c468bdaf46051017e6b8ed7e8cb524bf8169cb68e25b3fce949f8f. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `nd-ala/original/local-rules.pdf` as served (sha256 b9295a56444a7d46926807423bca614fae4ae5ce04f5c6031e82a7eb6870e4a2); extraction sha256 5fefae2739c468bdaf46051017e6b8ed7e8cb524bf8169cb68e25b3fce949f8f.

### `nd-ala/local-rules-sectioned.txt` — N.D. Ala. Local Rules, sectioned

- Derived in the pod (lg-f3q8.12) from `nd-ala/local-rules.txt` (extract sha256 5fefae2739c468bdaf46051017e6b8ed7e8cb524bf8169cb68e25b3fce949f8f); nothing fetched. One `## LR <n>` line is inserted before each rule's own heading line (`LR 3.1` through `LR 83.2`, 16 rules; a heading after a page's form feed keeps its form feed on the rule line). Rule text was not edited: deleting the marker lines (`sed '/^## LR /d'`) restores the extract byte for byte.
- Edition: as the extract, effective December 4, 2019.
- Sectioned sha256 9bf70d789baa46d67301338f131242f182416f7f92dacf80830930067931e132 (the `path_sha256` pinned by `lit/alnd/Lit.toml`).
