
### `bankr-lam/lam-local-rules-2022-01-01.txt` — M.D. La. Bankruptcy Local Rules (amended January 1, 2022)

- Source: https://www.lamb.uscourts.gov/sites/lamb/files/2022_Local_Rules.pdf (official site; 68 pages, sha256 688e97e8869b1d335353cced1f94a2c6e98f26ed831ba25114558954125b2945).
- Edition: Local Rules 'Amended January 1, 2022' (title page), linked from https://www.lamb.uscourts.gov/rules-and-forms (adopted by Standing Order 2022-1). OLD EDITION: later standing orders at https://www.lamb.uscourts.gov/standing-general-orders: SO 2024-2 and SO 2025-1 are vendored as lam-so-2024-2.txt and lam-so-2025-1.txt; SO 2026-1 (adversary cover sheet), 2026-2 (petition identification documents) and 2026-3 (consent orders on motions to convert) state practice requirements without amending a local rule's text and are not vendored. robots.txt (Crawl-delay 10 honored) and the court's privacy/security policy page (https://www.lamb.uscourts.gov/privacy-policy/) checked: no prohibition of automated access.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 d256c677be3b20c249ab6dbf1d27bc319a17a6a84a94b90bb95c0e565b142813. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-lam/original/lam-local-rules-2022-01-01.pdf` as served (sha256 688e97e8869b1d335353cced1f94a2c6e98f26ed831ba25114558954125b2945); extraction sha256 d256c677be3b20c249ab6dbf1d27bc319a17a6a84a94b90bb95c0e565b142813.

### `bankr-lam/lam-so-2024-2.txt` — M.D. La. Standing Order 2024-2 (amendment to Local Rule 4001-1)

- Source: https://www.lamb.uscourts.gov/sites/lamb/files/so_2024_2.pdf (official site; 1 pages, sha256 828c2f26d1b3485ddb4f149620b95e731a0d87cacc9b2b5823eb1295984ae1a9).
- Edition: Standing Order 2024-2, December 3, 2024, amending Local Rule 4001-1(f), listed at https://www.lamb.uscourts.gov/standing-general-orders. Vendored separately: apply on top of lam-local-rules-2022-01-01.txt from December 3, 2024. robots.txt (Crawl-delay 10 honored) and the court's privacy/security policy page (https://www.lamb.uscourts.gov/privacy-policy/) checked: no prohibition of automated access.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 c79d95a43ac4f571207114eebb76c3ff889cdd82e0caa97172d31c095da1a9f4. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-lam/original/lam-so-2024-2.pdf` as served (sha256 828c2f26d1b3485ddb4f149620b95e731a0d87cacc9b2b5823eb1295984ae1a9); extraction sha256 c79d95a43ac4f571207114eebb76c3ff889cdd82e0caa97172d31c095da1a9f4.

### `bankr-lam/lam-so-2025-1.txt` — M.D. La. Standing Order 2025-1 (objection deadline, modifying Local Rule 9014-1(a)(1))

- Source: https://www.lamb.uscourts.gov/sites/lamb/files/Standing%20Order%202025-1.pdf (official site; 1 pages, sha256 1ca55ff4c166733ae4fc5e3f4b0d8af98dff28b18b42c509ec71aaad470738de).
- Edition: Standing Order 2025-1, November 25, 2025, 'Notwithstanding Local Rule 9014-1(a)(1)' sets the objection deadline for motions set Mondays at 2:00 p.m., listed at https://www.lamb.uscourts.gov/standing-general-orders. Vendored separately: apply on top of lam-local-rules-2022-01-01.txt from November 25, 2025. robots.txt (Crawl-delay 10 honored) and the court's privacy/security policy page (https://www.lamb.uscourts.gov/privacy-policy/) checked: no prohibition of automated access.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 b616be57607029b006b9f6b9374c33061d954025ebf64216b4f8b63fd6481813. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-lam/original/lam-so-2025-1.pdf` as served (sha256 1ca55ff4c166733ae4fc5e3f4b0d8af98dff28b18b42c509ec71aaad470738de); extraction sha256 b616be57607029b006b9f6b9374c33061d954025ebf64216b4f8b63fd6481813.

### `bankr-lam/lam-local-rules-2022-01-01-sectioned.txt` — Bankr. M.D. La. Local Rules, sectioned

- Derived in the pod (lg-u561.17) from `bankr-lam/lam-local-rules-2022-01-01.txt` (extract sha256 d256c677be3b20c249ab6dbf1d27bc319a17a6a84a94b90bb95c0e565b142813); nothing fetched. One `## LR <n>` line is inserted before each rule's own heading line in the body (a line opening, after an optional form feed, with the rule number `<nnnn>-<n>` and its title, from `1002-1` through `9037-1`, 70 rules, the same set as the table of contents); the table of contents is unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## LR /d'`) restores the extract byte for byte.
- Source: as the extract `bankr-lam/lam-local-rules-2022-01-01.txt`, https://www.lamb.uscourts.gov/sites/lamb/files/2022_Local_Rules.pdf, retrieved 2026-10-28; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract.
- Sectioned sha256 07159974c25de67ec99d854575c712d2ef911b17bed2edd5c7249a697aed536e (the `path_sha256` pinned by `lit/bankr-lam/Lit.toml`).

### `bankr-lam/lam-so-2025-1-sectioned.txt` — M.D. La. Standing Order 2025-1, sectioned

- Derived in the pod (lg-u561.17) from `bankr-lam/lam-so-2025-1.txt` (extract sha256 b616be57607029b006b9f6b9374c33061d954025ebf64216b4f8b63fd6481813); nothing fetched. One `## SO 2025-1 Rule 9014-1` line is inserted before the order's operative text (the line opening `Notwithstanding Local Rule 9014-1(a)(1)`), so a cite to the rule it displaces locates it. Text was not edited: deleting the marker line (`sed '/^## SO 2025-1 /d'`) restores the extract byte for byte.
- Source: as the extract `bankr-lam/lam-so-2025-1.txt`, https://www.lamb.uscourts.gov/sites/lamb/files/Standing%20Order%202025-1.pdf, retrieved 2026-10-28; the sectioning was done in the pod from the vendored extract.
- Sectioned sha256 948a1d8b521f2eb08bcd54c8ce895322a5c1eb58c3d2514e68eab42635268975 (pinned by `lit/bankr-lam/Lit.toml`).
