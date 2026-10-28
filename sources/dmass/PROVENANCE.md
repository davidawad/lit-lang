
### `dmass/dmass-local-rules.txt` — D. Mass. Local Rules (combined)

- Source: https://www.mad.uscourts.gov/general/pdf/local-rules/Combined%20Local%20Rules.pdf (official site; 174 pages, sha256 1a8052b28e225059bc92a16420e9e4c637bd036c60f500b16861d339f888e1af).
- Edition: "Including Amendments Through September 8, 2026" (as stated on the document and on https://www.mad.uscourts.gov/general/rules-home.htm).
- Also the criminal local rules (lg-gddt): Criminal Cases rules (Rule 112.1 and following), from extract line 4409.
- Criminal forum (lg-gddt): mad-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 6e2e2a7c271b58941adb67ad32a82994121a7139b767d2a6acee6c8ce01f64c3. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `dmass/original/dmass-local-rules.pdf` as served (sha256 1a8052b28e225059bc92a16420e9e4c637bd036c60f500b16861d339f888e1af); extraction sha256 6e2e2a7c271b58941adb67ad32a82994121a7139b767d2a6acee6c8ce01f64c3.

### `dmass/dmass-local-rules-sectioned.txt` — D. Mass. Local Rules, sectioned

- Derived in the pod (lg-f3q8.23) from `dmass/dmass-local-rules.txt` (extract sha256 6e2e2a7c271b58941adb67ad32a82994121a7139b767d2a6acee6c8ce01f64c3); nothing fetched. One `## Rule <n>` line is inserted before each rule's own heading line in the body (`RULE 1.1 TITLE` through `RULE 206`, 105 rules); the table of contents is unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Source: as the extract `dmass/dmass-local-rules.txt`, https://www.mad.uscourts.gov/general/pdf/local-rules/Combined%20Local%20Rules.pdf, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, including amendments through September 8, 2026.
- Sectioned sha256 e0862acc529800fda4629e3067b1321f7edfec4585c778cb238e781bef47a8ba (the `path_sha256` pinned by `lit/mad/Lit.toml`).
