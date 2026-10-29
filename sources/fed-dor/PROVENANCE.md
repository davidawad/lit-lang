
### `fed-dor/dor-local-criminal-rules-printable.txt` — D. Or. Local Rules of Criminal Procedure (printable page, undated)

- Source: https://www.ord.uscourts.gov/index.php/rules-orders-and-notices/local-rules/criminal-procedure (printable version https://www.ord.uscourts.gov/index.php/printable-criminal-procedure?tmpl=component&print=1) (1 pages, each fetched with one GET).
- Edition: Local Rules of Criminal Procedure (Crim. LR 5, 11, 17, 32, 43, 49 and others) as served on the court's printable page on 2026-10-28; the page states no edition date, each rule carries its own amendment history; CURRENCY UNCONFIRMED beyond that. robots.txt of www.ord.uscourts.gov read (Disallow lists only cache/cli/components-style system paths, none covering the fetched path); the judiciary-wide Privacy & Security Policy (https://www.uscourts.gov/privacy-security-policy) read: no prohibition of automated access. No robots or terms barrier found.
- Criminal forum (lg-gddt): ord-criminal.
- Retrieved 2026-10-28 on carbon (lg-gddt.2); fetched with curl and the identifying UA lit-lang-provenance/1.0; extracted with `htmlx2.py 1.1` (stdlib html.parser, mode body, one `## <page>` block per page, unedited text).
- Extract sha256 dac68cf88696bc92a215f045ae7c8c51f74353e8356a65318b3ecdf9e8f8caf6. Raw extract, not yet sectioned into `## <ref>` form beyond the per-page blocks.
- Original: `fed-dor/original/ord-criminal-procedure/crim.html` as served (sha256 5ef5ebe42c34aa9ad1946db3170a56f3262f0cbe8b9f0fcdaa23c480d7e69b2b).
- Extraction sha256 dac68cf88696bc92a215f045ae7c8c51f74353e8356a65318b3ecdf9e8f8caf6.

### `fed-dor/dor-local-criminal-rules-printable-sectioned.txt` — D. Or. Local Rules of Criminal Procedure (printable page), sectioned

- Derived in the pod (lg-gddt.22) from `fed-dor/dor-local-criminal-rules-printable.txt` (extract sha256 dac68cf88696bc92a215f045ae7c8c51f74353e8356a65318b3ecdf9e8f8caf6); nothing fetched. One marker line `## Crim. LR <n>` is inserted before each rule's own heading line in the page text (Crim. LR 5, 11, 11-1, 17, 17-1, 17-2, 32, 32-1, 32-2, 43, 43-1, 43-2 (printed "Crim. OR 43-2"), 43-3, 49, 49-1 to 49-12, 49.1, 58 (printed "Crim LR. 58") and 58-1; 29 markers); amendment-history and practice-tip lines that merely name a rule carry no markers. The page's own `## Printable Criminal Procedure` block line is unchanged. Every other line is the extract unchanged, so deleting the marker lines gives back the extract byte for byte.
- Source: as the extract `fed-dor/dor-local-criminal-rules-printable.txt`, https://www.ord.uscourts.gov/index.php/rules-orders-and-notices/local-rules/criminal-procedure; Retrieved 2026-10-28 with the extract (this file was derived from it in the pod, not fetched itself).
- Edition: as the extract; CURRENCY UNCONFIRMED stands. `lit/ord-criminal` cites it for structure only and takes no value from it.
- Sectioned sha256 b6d0a140f21a3fd7f50576303e6361e3d9cc606cfa86084027b20a054a43f92e (the `path_sha256` pinned by `lit/ord-criminal/Lit.toml`).
