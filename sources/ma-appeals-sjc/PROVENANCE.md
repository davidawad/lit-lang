
### `ma-appeals-sjc/ma-rules-appellate-procedure.txt` — Massachusetts Rules of Appellate Procedure

- Source: https://www.mass.gov/doc/massachusetts-rules-of-appellate-procedure/download (official site; 178 pages, sha256 0811b8698254509d2830120cc2b1596e76d7d24d913603842617efd6ac391b34).
- Edition: "Including amendments effective September 1, 2026", Massachusetts Trial Court Law Libraries compilation, linked from https://www.mass.gov/law-library/massachusetts-rules-of-appellate-procedure (page last updated Sept 1, 2026). Fetched via browser fetch (curl got 403).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 37b3ad56623d12046aad57d33647d38923aa840296931840187cbd4594c0d28e. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `ma-appeals-sjc/original/ma-rules-appellate-procedure.pdf` as served (sha256 0811b8698254509d2830120cc2b1596e76d7d24d913603842617efd6ac391b34); extraction sha256 37b3ad56623d12046aad57d33647d38923aa840296931840187cbd4594c0d28e.

### `ma-appeals-sjc/ma-rules-appellate-procedure-sectioned.txt` — Massachusetts Rules of Appellate Procedure, sectioned

- Derived in the pod (lg-f3q8.23) from `ma-appeals-sjc/ma-rules-appellate-procedure.txt` (extract sha256 37b3ad56623d12046aad57d33647d38923aa840296931840187cbd4594c0d28e); nothing fetched. One `## Rule <n>` line is inserted after the first line of each rule's own heading (`RULE 1: SCOPE OF RULES: DEFINITIONS` through `RULE 32: TITLE`, 35 rules, including 11.1, 24.1 and 27.1); after, not before, because the cite corpus also opens a section at a `RULE <n>:` line, so a marker before it would hold an empty section. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Source: as the extract `ma-appeals-sjc/ma-rules-appellate-procedure.txt`, https://www.mass.gov/doc/massachusetts-rules-of-appellate-procedure/download, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, including amendments effective September 1, 2026.
- Sectioned sha256 1e933a8ddaecfe30dac949319d8f1f78f3a2ecc2a03e6815b10545072338eb5b (the `path_sha256` pinned by `lit/ma-superior-civil/Lit.toml` and `lit/ma-appeals-sjc/Lit.toml`).
