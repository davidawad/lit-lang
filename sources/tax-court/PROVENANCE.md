
### `tax-court/tax-court-rules.txt` — Tax Court Rules of Practice and Procedure

- Source: https://www.ustaxcourt.gov/files/documents/Complete-Rules-of-Practice-and-Procedure.pdf (official site; 315 pages, sha256 60040032eaed0001c3fabeb100bcec4ae5ddb14a4add628c563ef8c3ad956eb3).
- Edition: Tax Court Rules of Practice and Procedure, "as amended through August 2024" (cover); https://www.ustaxcourt.gov/rules/ links it as Complete Rules of Practice and Procedure and https://www.ustaxcourt.gov/notices-of-rule-amendments/ lists the August 8, 2024 amendments as the latest. Fetched with curl..
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 f688b3d65f05e93359950bbda10a3409feb9dbb554e6973255ce5aa9af6f7ad5. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `tax-court/original/tax-court-rules.pdf` as served (sha256 60040032eaed0001c3fabeb100bcec4ae5ddb14a4add628c563ef8c3ad956eb3); extraction sha256 f688b3d65f05e93359950bbda10a3409feb9dbb554e6973255ce5aa9af6f7ad5.

### `tax-court/tax-court-rules-sectioned.txt` — Tax Court Rules of Practice and Procedure, sectioned

- Derived in the pod (lg-f3q8.34) from `tax-court/tax-court-rules.txt` (extract sha256 f688b3d65f05e93359950bbda10a3409feb9dbb554e6973255ce5aa9af6f7ad5); nothing fetched. One `## Rule <n>` line is inserted before each rule's own heading line in the body (`RULE 1. RULEMAKING AUTHORITY, ...` through Title XXXIV, 204 rules including 151.1, 229A and 255.1-255.7; the table of contents is left unmarked), so each section holds its rule's heading and text. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Edition: as the extract, as amended through August 2024 (the latest amendments effective August 8, 2024, 163 T.C.).
- Sectioned sha256 6d2998bbbf3de127c53557ae53ddb5e4087720a35463d80b232ff181bb380f1a (the `path_sha256` pinned by `lit/tax-court/Lit.toml`).
