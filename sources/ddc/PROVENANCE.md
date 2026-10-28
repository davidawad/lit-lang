# `ddc` sources: provenance

### `ddc/ddc-local-rules.txt` — Local Rules of the U.S. District Court for the District of Columbia

- Source: https://www.dcd.uscourts.gov/sites/dcd/files/local_rules/Local%20Rules%20SEP_2026.pdf (official court site; 212 pages, sha256 e836e8706dc824294b73933ece7429cdbfc321104305501629b994631ebf5a86).
- Edition: cover: "Effective as of September 2015, Updated: September 2026"; listed as the current edition on https://www.dcd.uscourts.gov/court-info/local-rules-and-orders/local-rules (newest listed change: Local Rule 6.2, 9/14/2026).
- Also the criminal local rules (lg-gddt): LCrR 1.1 and following (contents list at extract line 529).
- Criminal forum (lg-gddt): dcd-criminal.
- Retrieved 2026-10-27 on carbon (the court site refuses agent pods: lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 e6084a6dc48c050f96035345573a3de47f57eadd0ff4e3d04039f0fb8e537928. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `ddc/original/ddc-local-rules.pdf` as served (sha256 e836e8706dc824294b73933ece7429cdbfc321104305501629b994631ebf5a86); extraction sha256 e6084a6dc48c050f96035345573a3de47f57eadd0ff4e3d04039f0fb8e537928.

### `ddc/ddc-local-rules-sectioned.txt` — D.D.C. Local Rules, criminal rules sectioned

- Derived in the pod (lg-gddt.4) from `ddc/ddc-local-rules.txt` (extract sha256 e6084a6dc48c050f96035345573a3de47f57eadd0ff4e3d04039f0fb8e537928); nothing fetched. In the body of the criminal rules (after their table of contents, before the bankruptcy rules), one marker line `## LCrR <n>` is inserted before each rule's own centered heading line `LCrR <n>` (LCrR 1.1 through LCrR 59.3, 57 rules, matching the contents list at extract line 529). The civil and other parts carry no markers. Every other line is the extract unchanged, so deleting the `## LCrR` lines (`sed '/^## LCrR /d'`) gives back the extract byte for byte.
- Upstream: https://www.dcd.uscourts.gov/sites/dcd/files/local_rules/Local%20Rules%20SEP_2026.pdf, retrieved 2026-10-27, as recorded on the source extract entry this file is derived from.
- Edition: as the extract ("Effective as of September 2015, Updated: September 2026").
- Sectioned sha256 0e04784a9c977eb2e470365b39310b429af07b64bdfe4a73ea31c95655d0d95b (the `path_sha256` pinned by `lit/dcd-criminal/Lit.toml`).
