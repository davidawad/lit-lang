
### `caaf/caaf-rules.txt` — CAAF Rules of Practice and Procedure

- Source: https://www.armfor.uscourts.gov/newcaaf/library/Rules/Rules2026Oct.pdf (official site; 47 pages, sha256 842ad17234343902c16d7e302eaf2d3b8418ef44efc0dbaeea5e837dc3c84426).
- Edition: CAAF Rules of Practice and Procedure, effective October 1, 2026 (cover; linked as "2026 Rules (effective 1 Oct 2026)" from https://www.armfor.uscourts.gov/newcaaf/rules.htm). Fetched with curl..
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 e831177cdfc3bac65fdbe40554f8502ee3fd5a5633050fad5a86e2d77b4d9575. Sectioned for cite resolution (lg-f3q8.132): a `## CAAF R. N` marker line was inserted before the body heading of each rule (53 rules, Rule 1 through Rule 45 with the lettered rules; the table of contents is unmarked). Rule text was not edited: deleting the marker lines (`sed '/^## CAAF R\. /d'`) restores the extract byte for byte.
- Sectioned file sha256 7b58729fb2d782a6b592acfb6aa69a15a06eef6c40de44f886306ce76e330f20 (the `path_sha256` pinned by `lit/caaf/Lit.toml`).
- Original: `caaf/original/caaf-rules.pdf` as served (sha256 842ad17234343902c16d7e302eaf2d3b8418ef44efc0dbaeea5e837dc3c84426); extraction sha256 e831177cdfc3bac65fdbe40554f8502ee3fd5a5633050fad5a86e2d77b4d9575.
