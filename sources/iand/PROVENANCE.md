# `iand` sources: provenance

### `iand/local-rules-joint.txt` — Joint Local Rules, N.D. and S.D. Iowa

- Source: https://www.iand.uscourts.gov/sites/iand/files/Local%20Rules%20-%20Final%20Website%20Version.pdf (official site; 106 pages, sha256 e810de7fd321eea597eeaaf97a38bb0f507061f82e5cc771e0991fa964ca5370).
- Edition: Joint Local Rules of the U.S. District Courts for the Northern and Southern Districts of Iowa, dated December 7, 2020 (N.D. AO 20-AO-0013-P; S.D. AO 20-AO-0023-P); linked as the current rules from https://www.iand.uscourts.gov/local-rules-federal-rules-administrative-orders (page also lists proposed amendments, comment period closing Nov. 13, 2026, not yet in force).
- Also the criminal local rules (lg-gddt): 'CRIMINAL RULES' part, from extract line 2911. Same document governs both districts.
- Criminal forum (lg-gddt): iand-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 f21b8f3f4df6312590be7cc4c801b02391715ebd58017e315347487f3249c4c5. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `iand/original/local-rules-joint.pdf` as served (sha256 e810de7fd321eea597eeaaf97a38bb0f507061f82e5cc771e0991fa964ca5370); extraction sha256 f21b8f3f4df6312590be7cc4c801b02391715ebd58017e315347487f3249c4c5.

### `iand/local-rules-civil-sectioned.txt` — Joint Local Rules, N.D. and S.D. Iowa, civil rules sectioned

- Derived in the pod (lg-f3q8.6) from `iand/local-rules-joint.txt` (extract sha256 f21b8f3f4df6312590be7cc4c801b02391715ebd58017e315347487f3249c4c5); nothing fetched. One `## LR <n>` line is inserted before each civil rule's own heading line (`LR 1` through `LR 8001`, 41 rules, numbers such as `LR 5A`, `LR 7.1` and `LR 16A` kept as printed; a heading after a page's form feed keeps its form feed on the heading line, after the marker). The table of contents and the criminal rules (LCrR) are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## LR /d'`) restores the extract byte for byte.
- Source: as the extract `iand/local-rules-joint.txt`, https://www.iand.uscourts.gov/sites/iand/files/Local%20Rules%20-%20Final%20Website%20Version.pdf, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, December 7, 2020.
- Sectioned sha256 2be9a39b0ef0b4e1b635f401708329d883069ab5612805dcd89b6ca4974095a0 (the `path_sha256` pinned by `lit/iand/Lit.toml`).
