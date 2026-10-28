# `dc-state-appellate` sources: provenance

### `dc-state-appellate/dcca-rules.txt` — Rules of the District of Columbia Court of Appeals

- Source: https://www.dccourts.gov/sites/default/files/coa-rules/DCCA-Rules.pdf (official court site; 135 pages, sha256 5b71b874d774c74dc7f98b2873d3393363d259dd7edbaf315a18b7962d0197bf).
- Edition: title page: "Reflecting amendments in effect as of May 11, 2026"; linked from https://www.dccourts.gov/court-of-appeals/court-of-appeals-resources/rules-of-the-dc-court-of-appeals (page last updated 02/04/2026).
- Retrieved 2026-10-27 on carbon (the court site refuses agent pods: lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 6ebf6a2ad4b75f15c0acd71ef1a3ce7adf02913d5e41183086c04449f5caeea4. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `dc-state-appellate/original/dcca-rules.pdf` as served (sha256 5b71b874d774c74dc7f98b2873d3393363d259dd7edbaf315a18b7962d0197bf); extraction sha256 6ebf6a2ad4b75f15c0acd71ef1a3ce7adf02913d5e41183086c04449f5caeea4.

### `dc-state-appellate/dcca-rules-sectioned.txt` — Rules of the District of Columbia Court of Appeals, sectioned

- Derived in the pod (lg-gddt.4) from `dc-state-appellate/dcca-rules.txt` (extract sha256 6ebf6a2ad4b75f15c0acd71ef1a3ce7adf02913d5e41183086c04449f5caeea4); nothing fetched. One marker line `## Rule <n>` is inserted before each rule's heading line `Rule <n>. <Title>` (50 rules, Rule 1 through Rule 50). Every other line is the extract unchanged, so deleting the `## Rule` lines (`sed '/^## Rule /d'`) gives back the extract byte for byte.
- Upstream: https://www.dccourts.gov/sites/default/files/coa-rules/DCCA-Rules.pdf, retrieved 2026-10-27, as recorded on the source extract entry this file is derived from.
- Edition: as the extract ("Reflecting amendments in effect as of May 11, 2026").
- Sectioned sha256 17ed6746cb9c0e76cf59794b5f203a6e03a85e16838865ecf46b46b98c5d6555 (the `path_sha256` pinned by `lit/dc-criminal-trial-court/Lit.toml`).
