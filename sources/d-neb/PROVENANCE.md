
### `d-neb/ne-civil-rules.txt` — D. Neb. Civil Rules (NECivR)

- Source: https://www.ned.uscourts.gov/sites/default/files/documents/NECivR.2025.pdf (official site; 63 pages, sha256 b2016037ba41ec3cd6f161d7d395d23c45c5cc6d1047b674d3bafee6b1e0ea4d).
- Edition: Nebraska Civil Rules (NECivR), "Effective December 1, 2025" per cover; listed "In Effect on and after December 1, 2025" at https://www.ned.uscourts.gov/local-rules (a 2026 proposed amendment for comment is not vendored). robots.txt read first (no Disallow of /sites/default/files/); privacy policy link on the court page is an internal host that does not resolve, no terms page reachable. Fetch method: curl with UA litgraph-rules-vendor/1.0, no bot challenge.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 903a924152c81694a3e69e2eb8aa7e4df2669c0b90bea71ecf73359d926c2bef. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `d-neb/original/ne-civil-rules.pdf` as served (sha256 b2016037ba41ec3cd6f161d7d395d23c45c5cc6d1047b674d3bafee6b1e0ea4d); extraction sha256 903a924152c81694a3e69e2eb8aa7e4df2669c0b90bea71ecf73359d926c2bef.

### `d-neb/ne-general-rules.txt` — D. Neb. General Rules (NEGenR)

- Source: https://www.ned.uscourts.gov/sites/default/files/documents/NEGenR.2025.pdf (official site; 38 pages, sha256 7aeb14b6339776dd5d1a3b9815495c93f12c608b0b1d5c9629d0c22bc40fb361).
- Edition: Nebraska General Rules (NEGenR), "Effective December 1, 2025"; listed at https://www.ned.uscourts.gov/local-rules. robots.txt read first. Fetch method: curl with UA litgraph-rules-vendor/1.0, no bot challenge.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 e545892094f1cc4df67c7e25757cf62e7900c1ddbaefa940c2b876532fe27c04. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `d-neb/original/ne-general-rules.pdf` as served (sha256 7aeb14b6339776dd5d1a3b9815495c93f12c608b0b1d5c9629d0c22bc40fb361); extraction sha256 e545892094f1cc4df67c7e25757cf62e7900c1ddbaefa940c2b876532fe27c04.

### `d-neb/dneb-necrimr-2025-12-01.txt` — D. Neb. Nebraska Criminal Rules (NECrimR, effective December 1, 2025)

- Source: https://www.ned.uscourts.gov/sites/default/files/documents/NECrimR.2025.pdf (official site; 77 pages, sha256 7ad46d57cd1da95f8a135b8847b36ffe456cd0d64e23df25580b43ff996c7d69).
- Edition: Nebraska Criminal Rules 'Effective December 1, 2025' (first page), NECrimR.2025 listed under 'In Effect on and after December 1, 2025' at https://www.ned.uscourts.gov/local-rules; the 2026 proposed rules on that page are not in force and not vendored. robots.txt of www.ned.uscourts.gov read (Crawl-delay 10 honored where stated; no Disallow covers the fetched path); the judiciary-wide Privacy & Security Policy (https://www.uscourts.gov/privacy-security-policy) read: no prohibition of automated access. Fetched with curl and the identifying UA lit-lang-provenance/1.0, at most 1 request per 10 s.
- Criminal forum (lg-gddt): ned-criminal.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 5d4fb10fee85aaa9b8d248c0a679eeeed6a482658d855a5da289197d9c9f7556. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `d-neb/original/dneb-necrimr-2025-12-01.pdf` as served (sha256 7ad46d57cd1da95f8a135b8847b36ffe456cd0d64e23df25580b43ff996c7d69); extraction sha256 5d4fb10fee85aaa9b8d248c0a679eeeed6a482658d855a5da289197d9c9f7556.

### `d-neb/ne-civil-rules-sectioned.txt` — D. Neb. Civil Rules (NECivR), sectioned

- Derived in the pod (lg-f3q8.9) from `d-neb/ne-civil-rules.txt` (extract sha256 903a924152c81694a3e69e2eb8aa7e4df2669c0b90bea71ecf73359d926c2bef); nothing fetched. One `## NECivR <n>` line is inserted before each rule's own heading line in the body after the table of contents (`NECivR 1.1` through `NECivR 85.1`, 62 rules, `NECivR 65.1.1` included). Rule text was not edited: deleting the marker lines (`sed '/^## NECivR /d'`) restores the extract byte for byte.
- Edition: as the extract, effective December 1, 2025.
- Sectioned sha256 cb0d2937e9bb6fa2e7fd65709557d84da6a61dfac80246b104b060e1895fb2cd (the `path_sha256` pinned by `lit/ned/Lit.toml`).
