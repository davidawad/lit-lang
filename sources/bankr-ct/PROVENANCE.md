
### `bankr-ct/ct-local-rules-2026-03-16.txt` — D. Conn. Bankruptcy Local Rules and Appendices (effective March 16, 2026)

- Source: https://www.ctb.uscourts.gov/sites/ctb/files/Local%20Rules_2026_03_16with%20appdncs.pdf (official site; 202 pages, sha256 f8877ab74b73ae64cff1eb1183116a8d979849d9369cc7d92ef32ea1c46b7b4f).
- Edition: Local Rules of Bankruptcy Procedure with appendices, 'Effective March 16, 2026' (document header), linked from https://www.ctb.uscourts.gov/local-rules-bankruptcy-procedure; supersedes the rules effective August 2, 2021. robots.txt (Crawl-delay 10 honored) and the court's privacy/security policy page (https://www.ctb.uscourts.gov/privacy-policy/) checked: no prohibition of automated access.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 bff247f5a3e021ec79fb3cd6c32b27358cf6b00254097a76ada1c9ef87686b07. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-ct/original/ct-local-rules-2026-03-16.pdf` as served (sha256 f8877ab74b73ae64cff1eb1183116a8d979849d9369cc7d92ef32ea1c46b7b4f); extraction sha256 bff247f5a3e021ec79fb3cd6c32b27358cf6b00254097a76ada1c9ef87686b07.

### `bankr-ct/ct-local-rules-2026-03-16-sectioned.txt` — D. Conn. Local Rules of Bankruptcy Procedure, sectioned

- Derived in the pod (lg-u561.11) from `bankr-ct/ct-local-rules-2026-03-16.txt` (extract sha256 bff247f5a3e021ec79fb3cd6c32b27358cf6b00254097a76ada1c9ef87686b07); nothing fetched. One `## LBR <n>` line is inserted before the first body line (after the table of contents, before Appendix A) opening `Local Bankr. R. <n>`, the rule's own heading or the running page header naming it (`LBR 1001-1` through `LBR 9083-6`; 95 sections); the table of contents and the appendices are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## LBR /d'`) restores the extract byte for byte.
- Edition: as the extract, effective March 16, 2026.
- Sectioned sha256 da2242972b4d43ccd7786c9b51b7f26ed935e902b3381031130da570bb359926 (the `path_sha256` pinned by `lit/bankr-ct/Lit.toml` for source `ctb-lbr`).
