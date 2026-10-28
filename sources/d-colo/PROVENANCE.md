
### `d-colo/d-colo-local-rules-2025.txt` — D. Colo. Local Rules of Practice (D.C.COLO.LCivR)

- Source: http://www.cod.uscourts.gov/Portals/0/Documents/LocalRules/2025_Final_Local_Rules.pdf (official site; 102 pages, sha256 7606869ea84c00157957ee77e6b1d6329ed785ce6723a54a1cfa412ccee2882d).
- Edition: Local Rules of Practice effective December 1, 2025 as stated on https://www.cod.uscourts.gov/CourtOperations/RulesProcedures/LocalRules.aspx ("Local Rules eff. 12-1-2025"; the page lists no pending proposed amendments); fetched with plain curl (no browser identity); host has no robots.txt (404) and its Privacy page has no automated-access prohibition.
- Also the criminal local rules (lg-gddt): 'SECTION II - CRIMINAL RULES' part, from extract line 1775.
- Criminal forum (lg-gddt): cod-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 1cdfa66a1c54140cf9683fe26f77a6253d327379e4984b37ba3bffdf80ab6e38. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `d-colo/original/d-colo-local-rules-2025.pdf` as served (sha256 7606869ea84c00157957ee77e6b1d6329ed785ce6723a54a1cfa412ccee2882d); extraction sha256 1cdfa66a1c54140cf9683fe26f77a6253d327379e4984b37ba3bffdf80ab6e38.

### `d-colo/d-colo-local-rules-civil-sectioned.txt` — D. Colo. Local Rules of Practice, civil rules sectioned

- Derived in the pod (lg-f3q8.65) from `d-colo/d-colo-local-rules-2025.txt` (extract sha256 1cdfa66a1c54140cf9683fe26f77a6253d327379e4984b37ba3bffdf80ab6e38); nothing fetched. One `## D.C.COLO.LCivR <n>` line is inserted right after each civil rule's own body heading line (`D.C.COLO.LCivR <n>`, some preceded by a form feed) in Section I - Civil Rules (extract lines 356-1774): 50 markers, D.C.COLO.LCivR 1.1 through 84.1. The table of contents and the criminal, patent, AP and attorney rules are unmarked (LCivR 84.1's section runs to the end of the file). Rule text was not edited: deleting the marker lines (`sed '/^## D\.C\.COLO\.LCivR /d'`) restores the extract byte for byte.
- Edition: as the extract, "Effective December 1, 2025" (cover "Revised 3/4/2026"); the court's local-rules page lists "Local Rules eff. 12-1-2025" as current.
- Sectioned sha256 da28ecd36b0d7a365c95dfb11419b058835ed5b8f79ea3957d814a1c151d1991 (the `path_sha256` pinned by `lit/cod/Lit.toml`).
