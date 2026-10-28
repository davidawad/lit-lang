
### `mn-district/mnrcp-2025-07-01.txt` — Minnesota Rules of Civil Procedure (amendments effective July 1, 2025)

- Source: https://mncourts.gov/_media/migration/appellate/supreme-court/court-rules/MN-Rules-of-Civil-Procedure.pdf (official site; 160 pages, sha256 2044ecee1b41aab5d58ee14427e8b6404f091ff2eecc14459ee4447900f0b276).
- Edition: Minnesota Rules of Civil Procedure 'With amendments effective July 1, 2025' (title page), linked at https://mncourts.gov/supremecourt/court-rules. The court's Recent Rules Orders page (https://mncourts.gov/supremecourt/court-rules/recent-rules-orders) lists no later order amending these rules (the latest, 05/20/25 effective July 1, 2025, is incorporated). fetched on David's explicit ruling 2026-10-28 despite robots/terms (barrier: Cloudflare challenge on mncourts.gov, cf-mitigated: challenge, against plain curl; tool: BrowserOS neo, a real browser, one download per document, intake copy trashed).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 748265a1b436591f3b5a5a9949f90c801cb05e5541e5b71281c7c00cc5ffdd1b. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `mn-district/original/mnrcp-2025-07-01.pdf` as served (sha256 2044ecee1b41aab5d58ee14427e8b6404f091ff2eecc14459ee4447900f0b276); extraction sha256 748265a1b436591f3b5a5a9949f90c801cb05e5541e5b71281c7c00cc5ffdd1b.

### `mn-district/mnrcp-2025-07-01-sectioned.txt` — Minnesota Rules of Civil Procedure, sectioned

- Derived in the pod (lg-f3q8.7) from `mn-district/mnrcp-2025-07-01.txt` (extract sha256 748265a1b436591f3b5a5a9949f90c801cb05e5541e5b71281c7c00cc5ffdd1b); nothing fetched. One `## Rule <n>` line is inserted before each rule's own heading in the body: before each `RULE <n>.` title line (80 rules, `RULE 1.` through `RULE 86.`, including `RULE 5A.`), and before the first line, after the table of contents and in its order, that heads each of the 266 numbered subdivisions the table of contents lists (`3.01.` through `86.01.`, including abrogated and deleted ones and headings that follow a form feed). Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Upstream: https://mncourts.gov/_media/migration/appellate/supreme-court/court-rules/MN-Rules-of-Civil-Procedure.pdf, retrieved 2026-10-28, as recorded on the `mn-district/mnrcp-2025-07-01.txt` entry this file is derived from.
- Edition: as the extract, amendments effective July 1, 2025.
- Sectioned sha256 983acc5f8dc41340bca690419e279fbcda4e58aef6110f4797cf7e21d890fbd2 (the `path_sha256` pinned by `lit/mn-district/Lit.toml` for source `mnrcp`).
