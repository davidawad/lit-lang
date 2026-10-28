
### `oh-common-pleas/ohio-civil-rules.txt` — Ohio Rules of Civil Procedure

- Source: https://www.supremecourt.ohio.gov/docs/LegalResources/Rules/civil/CivilProcedure.pdf (official site; 927 pages, sha256 49201001c188b1545e68567cdfdcfd011d0840b4811d74b5ab4c4be83c74e040).
- Edition: Ohio Rules of Civil Procedure; the court's rules page https://www.supremecourt.ohio.gov/laws-rules/ohio-rules-of-court/ lists 'Rules of Civil Procedure' amended effective July 1, 2026 and the PDF carries the July 1, 2026 amendments and staff notes.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Fetched via curl (single document, first request succeeded, no block encountered).
- Extract sha256 7d9aa7ea241c99cfed7a285a032dbda4c009d22f1347a5fde703347f203d3d51. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `oh-common-pleas/original/ohio-civil-rules.pdf` as served (sha256 49201001c188b1545e68567cdfdcfd011d0840b4811d74b5ab4c4be83c74e040); extraction sha256 7d9aa7ea241c99cfed7a285a032dbda4c009d22f1347a5fde703347f203d3d51.

### `oh-common-pleas/ohio-civil-rules-sectioned.txt` — Ohio Rules of Civil Procedure, sectioned

- Derived in the pod (lg-f3q8.61) from `oh-common-pleas/ohio-civil-rules.txt` (extract sha256 7d9aa7ea241c99cfed7a285a032dbda4c009d22f1347a5fde703347f203d3d51); nothing fetched. One `## Civ.R. <n>` line is inserted right after each rule's own heading line `RULE <n>. <title>` (123 markers, Civ.R. 1 through Civ.R. 86; the combined `RULE 77 TO 80. [RESERVED]` line is unmarked, and the forms after Civ.R. 86 fall in its section). After, not before, because the cite corpus also opens a section at a `RULE <n>:` line, which would leave a marker placed before it empty. Rule text was not edited: deleting the marker lines (`sed '/^## Civ\.R\. /d'`) restores the extract byte for byte.
- Edition: as the extract, the Rules of Civil Procedure as amended effective July 1, 2026.
- Sectioned sha256 c160d248cef571563999330b31759050e39d8ee5da4b3542a95a4a73edb3c5a1 (the `path_sha256` pinned by `lit/oh-common-pleas/Lit.toml`).
