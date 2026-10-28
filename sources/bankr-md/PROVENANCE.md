
### `bankr-md/md-local-rules-2025-12-01.txt` — D. Md. Local Bankruptcy Rules (revised December 1, 2025, Ver. 25.01)

- Source: https://www.mdb.uscourts.gov/files/localrules.pdf (official site; 231 pages, sha256 a1cd5426d50d3eb9c59b8f4ccfdb7ed5f825e7a52cdadb3aa6a6078bafd66d8d).
- Edition: Local Bankruptcy Rules 'As Revised December 1, 2025 (Ver. 25.01)' (title page), the 'Local Rules' link at https://www.mdb.uscourts.gov/administrative-orders; the administrative orders page lists no later order amending the rules (the 2026 administrative orders concern sealed documents, intake hours and BNC). robots.txt (no Crawl-delay stated; only /search/, /core/ and the like disallowed) and the court site has no usable policy page of its own (its only privacy page is the SSN-redaction notice), so the judiciary-wide Privacy & Security Policy (https://www.uscourts.gov/privacy-security-policy, with the robots.txt of www.uscourts.gov) was checked: no prohibition of automated access.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 e44e5d8578af7325b06e5061f8df25a8f4e816f46ebf0523f03a91e267a54827. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-md/original/md-local-rules-2025-12-01.pdf` as served (sha256 a1cd5426d50d3eb9c59b8f4ccfdb7ed5f825e7a52cdadb3aa6a6078bafd66d8d); extraction sha256 e44e5d8578af7325b06e5061f8df25a8f4e816f46ebf0523f03a91e267a54827.

### `bankr-md/md-local-rules-2025-12-01-sectioned.txt` — D. Md. Local Bankruptcy Rules, sectioned

- Derived in the pod (lg-u561.15) from `bankr-md/md-local-rules-2025-12-01.txt` (extract sha256 e44e5d8578af7325b06e5061f8df25a8f4e816f46ebf0523f03a91e267a54827); nothing fetched. One `## Rule <n>` line is inserted before each rule's own heading line in the body of the rules (`RULE 1001-1` through `RULE 9070-1`, 113 rules; extract lines 400-4399); the tables of contents and the appendices are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Upstream: https://www.mdb.uscourts.gov/files/localrules.pdf, retrieved 2026-10-28, as recorded on the `bankr-md/md-local-rules-2025-12-01.txt` entry this file is derived from.
- Edition: as the extract, revised December 1, 2025 (Ver. 25.01).
- Sectioned sha256 962e3f82aae1f665a15cbc6aca824d53179d05369f15e598db2687949de62a0e (the `path_sha256` pinned by `lit/bankr-md/Lit.toml`).
