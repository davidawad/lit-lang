
### `ca5/rules-and-iops.txt` — Fifth Circuit Rules and Internal Operating Procedures (bound with FRAP)

- Source: https://www.ca5.uscourts.gov/docs/C9B3642D-BBBF-4D20-859C-C5B0875D75E8/federalrulesofappellateprocedure (official site; 174 pages, sha256 babf8e516c16984035591c0a6ca6f3f12973771adb23a4805967b3606f3a93ab).
- Edition: FRAP as amended to December 1, 2025, with Fifth Circuit Rules and IOPs as amended through December 2025 (cover); linked as "Federal and 5th Cir Rules and IOPs" from https://www.ca5.uscourts.gov/rules-procedures; fetched with curl (plain User-Agent), no bot refusal; robots.txt absent (404), privacy policy has no automated-access prohibition.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 5441b4b66e6740fea13ab9a5833ff14839668d569c37fbe012443bcfff35051b. Sectioned for cite resolution (lg-f3q8.17): a `## 5th Cir. R. N` marker line was inserted before the body of each numbered circuit rule paragraph (`N.M`, `N.M.K`, ...) and before the heading of each rule with no numbered paragraphs (3, 5, 12, 17, 21, 22); 214 markers, the table of contents is unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## 5th Cir\. R\. /d'`) restores the extract byte for byte.
- Sectioned file sha256 19e2c73c6404d44dfabeddf7c6302b8930c80b2134406e881c76e65203dd0e91 (the `path_sha256` pinned by `lit/ca5/Lit.toml`).
- Original: `ca5/original/rules-and-iops.pdf` as served (sha256 babf8e516c16984035591c0a6ca6f3f12973771adb23a4805967b3606f3a93ab); extraction sha256 5441b4b66e6740fea13ab9a5833ff14839668d569c37fbe012443bcfff35051b.
