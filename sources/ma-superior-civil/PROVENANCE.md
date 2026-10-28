
### `ma-superior-civil/ma-rules-civil-procedure.txt` — Massachusetts Rules of Civil Procedure

- Source: https://www.mass.gov/doc/massachusetts-rules-of-civil-procedure/download (official site; 271 pages, sha256 995d92caabfe01b72050a0d1f0960fc12be62795f57410a0824d9120a4aeae75).
- Edition: "Including amendments effective March 10, 2025", Massachusetts Trial Court Law Libraries compilation (its own disclaimer says not an official source) linked from https://www.mass.gov/law-library/massachusetts-rules-of-civil-procedure (page last updated Sept 3, 2026, lists no later amendment; SJC order amending Rules 5 and 11 is effective Jan 1, 2027 and not yet in force). Fetched via browser fetch (curl got 403).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 768042d9d4b46f5d0dda403c018fadd297e4637aaaf606ba51fc03f2d1690a8a. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `ma-superior-civil/original/ma-rules-civil-procedure.pdf` as served (sha256 995d92caabfe01b72050a0d1f0960fc12be62795f57410a0824d9120a4aeae75); extraction sha256 768042d9d4b46f5d0dda403c018fadd297e4637aaaf606ba51fc03f2d1690a8a.

### `ma-superior-civil/ma-rules-civil-procedure-sectioned.txt` — Massachusetts Rules of Civil Procedure, sectioned

- Derived in the pod (lg-f3q8.23) from `ma-superior-civil/ma-rules-civil-procedure.txt` (extract sha256 768042d9d4b46f5d0dda403c018fadd297e4637aaaf606ba51fc03f2d1690a8a); nothing fetched. One `## Rule <n>` line is inserted before each rule's own heading line (`Rule 1: Scope of Rules` through `Rule 85: Title`, 96 rules, including the lettered and decimal rules such as 1A, 4.1 and 55.1); reporter's-note lines that begin with a rule reference are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Source: as the extract `ma-superior-civil/ma-rules-civil-procedure.txt`, https://www.mass.gov/doc/massachusetts-rules-of-civil-procedure/download, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, including amendments effective March 10, 2025.
- Sectioned sha256 3368e76c097f4f8dd3560ba731b88d6f6ca00fb93824103f2555d140a1952a1d (the `path_sha256` pinned by `lit/ma-superior-civil/Lit.toml`).
