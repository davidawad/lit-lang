
### `sd-fla/local-rules.txt` — S.D. Fla. Local Rules

- Source: https://www.flsd.uscourts.gov/sites/flsd/files/25-09-22%202025%20Local%20Rules%20effective%20120125%20-%20FINAL.pdf (official site; 130 pages, sha256 bac0fe34c01be93a05af039c967c903f0bd738503483a7aef56c55a93ac8f5df).
- Edition: 2025 Local Rules effective December 1, 2025 (PDF title and filename); linked from https://www.flsd.uscourts.gov/local-rules-and-procedures; fetched with curl (plain User-Agent), no bot refusal.
- Also the criminal local rules (lg-gddt): Rule 88.1 and following, Criminal Proceedings, from extract line 3065.
- Criminal forum (lg-gddt): flsd-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 a76f60b49290a20c38377decc1cf48f972cad228d95f67f9a754db26aa02a791. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `sd-fla/original/local-rules.pdf` as served (sha256 bac0fe34c01be93a05af039c967c903f0bd738503483a7aef56c55a93ac8f5df); extraction sha256 a76f60b49290a20c38377decc1cf48f972cad228d95f67f9a754db26aa02a791.

### `sd-fla/local-rules-sectioned.txt` — S.D. Fla. Local Rules, sectioned

- Derived in the pod (lg-f3q8.14) from `sd-fla/local-rules.txt` (extract sha256 a76f60b49290a20c38377decc1cf48f972cad228d95f67f9a754db26aa02a791); nothing fetched. One `## Rule <n>` line is inserted before each rule's own heading line in the body (`RULE 1.1 SCOPE OF THE LOCAL RULES` through `RULE 88.11 AFTER HOURS CRIMINAL DUTY PROCEDURES`, 41 rules); the table of contents and the table of repealed and relocated rules are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Source: as the extract `sd-fla/local-rules.txt`, https://www.flsd.uscourts.gov/sites/flsd/files/25-09-22%202025%20Local%20Rules%20effective%20120125%20-%20FINAL.pdf, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, 2025 Local Rules effective December 1, 2025.
- Sectioned sha256 afdad87f96c91b544073c847c470cd5e09bf4842c44f5ec5ff1f8681cb3dd12f (the `path_sha256` pinned by `lit/flsd/Lit.toml`).
