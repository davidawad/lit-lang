
### `bankr-miw/miw-local-rules-2026-10-11.txt` — W.D. Mich. Local Bankruptcy Rules (effective October 11, 2026)

- Source: https://www.miwb.uscourts.gov/sites/miwb/files/local_rules/Final%20-%20Clean_1.pdf (official site; 98 pages, sha256 08d50ea717428eedb60e3960ff67369ed21e144e06bc9509203fde2d2d0fb9f6).
- Edition: Local Bankruptcy Rules 'Effective: October 11, 2026' (title page), linked as 'Local Bankruptcy Rules - Effective October 11, 2026' (Final - Clean) at https://www.miwb.uscourts.gov/court-info/local-rules-and-orders/local-rules; announced at https://www.miwb.uscourts.gov/news/amendments-local-rules-effective-october-11-2026. robots.txt (Crawl-delay 10 honored) and the court site has no usable policy page of its own (its /privacy-policy/ returned 404 and no policy page is linked), so the judiciary-wide Privacy & Security Policy (https://www.uscourts.gov/privacy-security-policy, with the robots.txt of www.uscourts.gov) was checked: no prohibition of automated access.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 c38d0e9fae0b8acee79f0fc61a6c96cda3cae179140073cf27ed41d8068d40e0. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-miw/original/miw-local-rules-2026-10-11.pdf` as served (sha256 08d50ea717428eedb60e3960ff67369ed21e144e06bc9509203fde2d2d0fb9f6); extraction sha256 c38d0e9fae0b8acee79f0fc61a6c96cda3cae179140073cf27ed41d8068d40e0.

### `bankr-miw/miw-local-rules-2026-10-11-sectioned.txt` — W.D. Mich. Local Bankruptcy Rules, sectioned

- Derived in the pod (lg-u561.19) from `bankr-miw/miw-local-rules-2026-10-11.txt` (extract sha256 c38d0e9fae0b8acee79f0fc61a6c96cda3cae179140073cf27ed41d8068d40e0); nothing fetched. One `## Rule <n>` line is inserted before each rule's own heading line in the body (`LBR <n>: <title>`, `LBR 1001` through `LBR 9037-3`, including the headings that begin with a form feed; 84 sections, matching the table of contents); the title page and table of contents (extract lines 1-144) are unmarked, and the appended exhibits after LBR 9037-3 fall within its section. Rule text was not edited: deleting the marker lines (`sed '/^## /d'`) restores the extract byte for byte.
- Source: as the extract `bankr-miw/miw-local-rules-2026-10-11.txt`, https://www.miwb.uscourts.gov/sites/miwb/files/local_rules/Final%20-%20Clean_1.pdf, retrieved 2026-10-28; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, the Local Bankruptcy Rules effective October 11, 2026.
- Sectioned sha256 bb33a02a0ebfbad60086f79d1e6f6458e01ebf9eefd1bcb047d5efbc7852db04 (the `path_sha256` pinned by `lit/bankr-miw/Lit.toml` for source `miwb-lbr`).
