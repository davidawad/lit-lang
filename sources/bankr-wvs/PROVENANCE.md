
### `bankr-wvs/wvs-local-rules-2012-11.txt` — S.D. W. Va. Local Bankruptcy Rules (November 2012; court: effective January 2, 2013)

- Source: https://www.wvsb.uscourts.gov/sites/wvsb/files/localrules.pdf (official site; 138 pages, sha256 73d7d414afd0ff72b772690dbe4f02f758839dff8754f3d653083c42216775aa).
- Edition: Local Bankruptcy Rules 'November 2012' (title page), listed by the court as 'Current Local Bankruptcy Rules (effective 1/2/2013)' at https://www.wvsb.uscourts.gov/court-info/local-rules-and-orders/local-rules. OLD EDITION: later general orders at https://www.wvsb.uscourts.gov/court-info/local-rules-and-orders/general-orders rescind rules: GO 23-05 (vendored as wvs-go-23-05.txt) and GO 25-03 (rescinding S.D.W.Va. LBR 3015-1.3; the court's link returns 404, not found, not vendored). robots.txt (Crawl-delay 10 honored) and the court's privacy/security policy page (https://www.wvsb.uscourts.gov/privacy-guidelines) checked: no prohibition of automated access.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 5f7389305622ba98f33cb9f085be078d40487da622ded7194968e35ddb2fcf12. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-wvs/original/wvs-local-rules-2012-11.pdf` as served (sha256 73d7d414afd0ff72b772690dbe4f02f758839dff8754f3d653083c42216775aa); extraction sha256 5f7389305622ba98f33cb9f085be078d40487da622ded7194968e35ddb2fcf12.

### `bankr-wvs/wvs-go-23-05.txt` — S.D. W. Va. General Order 23-05 (rescinding LBR 3011-1)

- Source: https://www.wvsb.uscourts.gov/sites/wvsb/files/general-ordes/General%20Order%2023-05.pdf (official site; 1 pages, sha256 fd31f5c4bdf8ff6cc2e9fa9215982b05ecdd05670a7c7c5695586c503d01b78d).
- Edition: General Order 23-05 (2023, no date on the order itself) rescinding General Order 08-06 and S.D.W.Va. LBR 3011-1. Vendored separately: apply on top of wvs-local-rules-2012-11.txt from its 2023 entry (date not stated on the order; CURRENCY UNCONFIRMED as to the exact day). robots.txt (Crawl-delay 10 honored) and the court's privacy/security policy page (https://www.wvsb.uscourts.gov/privacy-guidelines) checked: no prohibition of automated access.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 392227050cd81c12f70616d708594b2c3681c1f049cb893c7f27cb6a19b4a0c1. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-wvs/original/wvs-go-23-05.pdf` as served (sha256 fd31f5c4bdf8ff6cc2e9fa9215982b05ecdd05670a7c7c5695586c503d01b78d); extraction sha256 392227050cd81c12f70616d708594b2c3681c1f049cb893c7f27cb6a19b4a0c1.

### `bankr-wvs/wvs-local-rules-2012-11-sectioned.txt` — S.D. W. Va. Local Bankruptcy Rules, sectioned

- Derived in the pod (lg-u561.15) from `bankr-wvs/wvs-local-rules-2012-11.txt` (extract sha256 5f7389305622ba98f33cb9f085be078d40487da622ded7194968e35ddb2fcf12); nothing fetched. One `## Rule <n>` line is inserted before each rule's own `<n>   Title` heading line in the body of the rules (`1007-1` through `9076-1`, 107 rules, including the decimal rules such as 1019-1.1 and 3015-1.3, and the rescinded 3011-1 and 3015-1.3, which keep their sections; the heading `1007–2`, printed with an en dash, is keyed with an ASCII hyphen); the table of contents is unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Edition: as the extract, November 2012 (court: effective January 2, 2013).
- Sectioned sha256 cadb7f5fa10d6de52afbce7c4edc53c9c173ff9f6aea8f47ff9e09089f703af5 (the `path_sha256` pinned by `lit/bankr-wvs/Lit.toml`).
