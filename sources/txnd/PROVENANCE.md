
### `txnd/tx-nd-local-civil-rules.txt` — N.D. Tex. Local Civil Rules

- Source: https://www.txnd.uscourts.gov/sites/default/files/documents/CIVRULES.pdf (official site; 36 pages, sha256 46a817694c28c41732b91b2b672fe23ee08ad28f8c633a97534312a1b1f29dba).
- Edition: Local Civil Rules, "Effective September 1, 2026" per document cover; linked from https://www.txnd.uscourts.gov/rules-and-orders (robots.txt Crawl-delay 10 honored; no automated-access ban found in privacy-and-security-policy). Fetch method: curl with UA litgraph-rules-vendor/1.0, no bot challenge.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 329e0a1b010d56726debec13a00b19dd866bc4f886d036a44e5efa9eee7e6a77. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `txnd/original/tx-nd-local-civil-rules.pdf` as served (sha256 46a817694c28c41732b91b2b672fe23ee08ad28f8c633a97534312a1b1f29dba); extraction sha256 329e0a1b010d56726debec13a00b19dd866bc4f886d036a44e5efa9eee7e6a77.

### `txnd/txnd-local-criminal-rules-2026-09-01.txt` — N.D. Tex. Local Criminal Rules (effective September 1, 2026)

- Source: https://www.txnd.uscourts.gov/sites/default/files/documents/CRIMRULES.pdf (official site; 22 pages, sha256 033d693354587ef18e9a6eeeb0642db915425c5fd6ec7dba11244b8b0cf9e8d0).
- Edition: Local Criminal Rules 'Effective September 1, 2026' (title page), the 'Criminal Rules (PDF)' linked at https://www.txnd.uscourts.gov/rules-and-orders. robots.txt of www.txnd.uscourts.gov read (Crawl-delay 10 honored where stated; no Disallow covers the fetched path); the judiciary-wide Privacy & Security Policy (https://www.uscourts.gov/privacy-security-policy) read: no prohibition of automated access. Fetched with curl and the identifying UA lit-lang-provenance/1.0, at most 1 request per 10 s.
- Criminal forum (lg-gddt): txnd-criminal.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 800d8cb671875dff48e9ebcf252d2fb2e88a32c32d17298d10e441cce93472fd. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `txnd/original/txnd-local-criminal-rules-2026-09-01.pdf` as served (sha256 033d693354587ef18e9a6eeeb0642db915425c5fd6ec7dba11244b8b0cf9e8d0); extraction sha256 800d8cb671875dff48e9ebcf252d2fb2e88a32c32d17298d10e441cce93472fd.

### `txnd/txnd-local-criminal-rules-2026-09-01-sectioned.txt` — N.D. Tex. Local Criminal Rules, sectioned

- Derived in the pod (lg-gddt.14) from `txnd/txnd-local-criminal-rules-2026-09-01.txt` (extract sha256 800d8cb671875dff48e9ebcf252d2fb2e88a32c32d17298d10e441cce93472fd); nothing fetched. In the body (after the table of contents), one marker line `## LCrR <n>` is inserted before each rule's own heading line (`LCrR 1.1 Definitions.` through `LCrR 59.2 Briefing Practice ...`, 42 rules, repealed rules included). Every other line is the extract unchanged, so deleting the `## LCrR` lines gives back the extract byte for byte.
- Edition: as the extract, "Effective September 1, 2026".
- Sectioned sha256 71c01d4ff9d104fc207dad508d798109df0e0a947033212ab1aed039dbfd789c (the `path_sha256` pinned by `lit/txnd-criminal/Lit.toml`).
