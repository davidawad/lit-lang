# `ared` sources: provenance

### `ared/ared-arwd-local-rules.txt` — Local Rules of the U.S. District Courts for the Eastern and Western Districts of Arkansas

- Source: https://www.are.uscourts.gov/sites/are/files/local_rules/All_LR.pdf (official court site; 84 pages, sha256 9fc211b94a6b06e77953f28f1b88c5e61f8d567dc68d71a78a91eee7715fa459).
- Edition: title page: "Local Rules of the United States District Court for the Eastern and Western Districts of Arkansas", last amendment listed September 21, 2017; the "Complete Copy" on https://www.are.uscourts.gov/court-info/local-rules-and-orders/local-rules, last modified 2020-11-17, newer than every individual rule PDF the page lists. One joint text: it also governs forum arwd (the W.D. Ark. site copy is a 2016 file, not vendored).
- Also the criminal local rules (lg-gddt): 'IV.       CRIMINAL PRETRIAL' part, from extract line 1498.
- Criminal forum (lg-gddt): ared-criminal, arwd-criminal.
- Retrieved 2026-10-27 on carbon (the court site refuses agent pods: lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 0bc073ca4ceb05726d03a65d05a6294c20199591be8b23dc465ff14d724c8950. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `ared/original/ared-arwd-local-rules.pdf` as served (sha256 9fc211b94a6b06e77953f28f1b88c5e61f8d567dc68d71a78a91eee7715fa459); extraction sha256 0bc073ca4ceb05726d03a65d05a6294c20199591be8b23dc465ff14d724c8950.

### `ared/ared-arwd-local-rules-sectioned.txt` — Local Rules of the E.D. and W.D. Arkansas, sectioned

- Derived in the pod (lg-f3q8.3) from `ared/ared-arwd-local-rules.txt` (extract sha256 0bc073ca4ceb05726d03a65d05a6294c20199591be8b23dc465ff14d724c8950); nothing fetched. One `## L.R. <n>` line is inserted before the first heading line `LOCAL RULE <n>` of each rule (`L.R. 4.1` through `L.R. 83.8`, 37 rules; the second and third `LOCAL RULE 83.6` headings, the rule's appendix forms, are unmarked). Rule text was not edited: deleting the marker lines (`sed '/^## L.R. /d'`) restores the extract byte for byte.
- Upstream: https://www.are.uscourts.gov/sites/are/files/local_rules/All_LR.pdf, retrieved 2026-10-27, as recorded on the `ared/ared-arwd-local-rules.txt` entry this file is derived from.
- Edition: as the extract, the "Complete Copy" last modified 2020-11-17 (newest amendment note: L.R. 5.1, November 5, 2020).
- Sectioned sha256 9ffe4767e0a2f226a2c05119182219877a27b0c28c3edda6bd4442ab6c411cd4 (the `path_sha256` pinned by `lit/ared/Lit.toml`, `lit/arwd/Lit.toml` and `lit/ared-criminal/Lit.toml`).
