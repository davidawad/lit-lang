
### `bankr-lae/lae-local-rules-2013-05-01.txt` — E.D. La. Bankruptcy Local Rules (adopted May 1, 2013)

- Source: https://www.laeb.uscourts.gov/sites/laeb/files/local_rules/LocalRules050113.pdf (official site; 56 pages, sha256 912a4b2cb2478c589b0b9f4daf308296a3cb3c3dd2fb7a482a5a2dfcc8f83f4e).
- Edition: Local Rules 'Adopted May 1, 2013' (title page), the only edition linked as Local Rules at https://www.laeb.uscourts.gov/court-info/local-rules-and-orders/local-rules. OLD EDITION: the court still presents it as current; its Orders page (https://www.laeb.uscourts.gov/orders) lists no later order amending the rules (General Order 2013-1 of April 25, 2013 precedes the adoption date). robots.txt (Crawl-delay 10 honored) and the court's privacy/security policy page (https://www.laeb.uscourts.gov/privacy-policy) checked: no prohibition of automated access.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 6d3b041e61bc10cdf90ffa9a38315453c46149c4b42ac8152c57dad1fe996ad6. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-lae/original/lae-local-rules-2013-05-01.pdf` as served (sha256 912a4b2cb2478c589b0b9f4daf308296a3cb3c3dd2fb7a482a5a2dfcc8f83f4e); extraction sha256 6d3b041e61bc10cdf90ffa9a38315453c46149c4b42ac8152c57dad1fe996ad6.

### `bankr-lae/lae-local-rules-2013-05-01-sectioned.txt` — Bankr. E.D. La. Local Rules, sectioned

- Derived in the pod (lg-u561.17) from `bankr-lae/lae-local-rules-2013-05-01.txt` (extract sha256 6d3b041e61bc10cdf90ffa9a38315453c46149c4b42ac8152c57dad1fe996ad6); nothing fetched. One `## LR <n>` line is inserted before each rule's own heading line in the body (a line opening, after an optional form feed, with `RULE <nnnn>-<n>`, from `RULE 1001-1` through `RULE 9037-1`, 61 rules, the same set as the table of contents); the table of contents is unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## LR /d'`) restores the extract byte for byte.
- Source: as the extract `bankr-lae/lae-local-rules-2013-05-01.txt`, https://www.laeb.uscourts.gov/sites/laeb/files/local_rules/LocalRules050113.pdf, retrieved 2026-10-28; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract.
- Sectioned sha256 bc80012ccbf9773a0d99fc39820d70e4d643cc80f71ea9ba3a29a1d1dbc24a69 (the `path_sha256` pinned by `lit/bankr-lae/Lit.toml`).
