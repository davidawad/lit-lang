
### `mi-circuit-court/mcr-chapters-1-9.txt` — Michigan Court Rules of 1985, Chapters 1-9

- Source: https://www.courts.michigan.gov/siteassets/rules-instructions-administrative-orders/michigan-court-rules/michigan-court-rules.pdf (official site; 875 pages, sha256 31ed59fbef8d2f177f6e8b50ff976a06ebd14aef16706a76987e03469f278a02).
- Edition: Michigan Court Rules of 1985, Chapters 1-9, "Updated September 2, 2026" as stated on the document; the court rules page https://www.courts.michigan.gov/rules-administrative-orders-and-jury-instructions/current-rules-and-jury-instructions/michigan-court-rules/ links this PDF as "PDF: Michigan Court Rules Chapters 1-9" (hash-prefixed path /4a4a71/siteassets/...; the unprefixed path serves the same file). SHARED with the Michigan appellate forum (mi-appellate): Chapter 7 covers MCR 7.2xx/7.3xx, so it is vendored once here. robots.txt allows; no terms page was found on the site.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Fetched via curl (single document, first request succeeded, no block encountered).
- Extract sha256 28f7b3760121d16db45005986414520aa94018c45d101d8d011b382d48a153bc. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `mi-circuit-court/original/mcr-chapters-1-9.pdf` as served (sha256 31ed59fbef8d2f177f6e8b50ff976a06ebd14aef16706a76987e03469f278a02); extraction sha256 28f7b3760121d16db45005986414520aa94018c45d101d8d011b382d48a153bc.
- Also the criminal rules (lg-gddt): MCR Chapter 6, Criminal Procedure (subchapters 6.000 to 6.600, including MCR 6.004 speedy trial) and the criminal appeal rules of Chapter 7 (MCR 7.200 et seq.); no separate criminal source vendored for Michigan. The statutory speedy-trial provision MCL 768.1 is not vendored.

### `mi-circuit-court/mcr-chapters-1-9-sectioned.txt` — Michigan Court Rules, Chapters 1-9, sectioned

- Derived in the pod (lg-f3q8.60) from `mi-circuit-court/mcr-chapters-1-9.txt` (extract sha256 28f7b3760121d16db45005986414520aa94018c45d101d8d011b382d48a153bc); nothing fetched. One `## MCR <n>` line is inserted right after each rule's own body heading line (`Rule <n> <title>`, including headings that open a page with a form feed, and `Rule 6.202.` with its stray period): 625 markers, MCR 1.101 through the last rule of Chapter 9. The table of contents is unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## MCR /d'`) restores the extract byte for byte. Shared by `lit/mi-circuit-court` and `lit/mi-appellate` (Chapter 7).
- Upstream: https://www.courts.michigan.gov/siteassets/rules-instructions-administrative-orders/michigan-court-rules/michigan-court-rules.pdf, retrieved 2026-10-27, as recorded on the `mi-circuit-court/mcr-chapters-1-9.txt` entry this file is derived from.
- Edition: as the extract, the Michigan Court Rules of 1985, Chapters 1-9, "Updated September 2, 2026".
- Sectioned sha256 34a24b105ffcab26ce2fe4d4f6a0b8d3fb11024e7716e7546d9a8d4d123242a3 (the `path_sha256` pinned by `lit/mi-circuit-court/Lit.toml` and `lit/mi-appellate/Lit.toml`).
