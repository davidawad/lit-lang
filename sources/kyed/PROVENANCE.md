
### `kyed/kyed-joint-local-civil-rules.txt` — Joint Local Rules of Civil Practice, E.D. and W.D. Kentucky

- Source: https://www.kyed.uscourts.gov/sites/kyed/files/KY_Amended_Civil_Rules_August_2026.pdf (official site; 30 pages, sha256 f7942d10800716fe5cc62401a47f6ef71d8db68cbc5516a504df2263597d1086).
- Edition: Joint Local Rules of Civil Practice (E.D. and W.D. Ky.), file "KY_Amended_Civil_Rules_August_2026" (no in-document date; LR 86.1: amendments take effect on each Joint General Order); linked from https://www.kyed.uscourts.gov/local-rules. SHARED with W.D. Ky. (kywd): the W.D. page https://www.kywd.uscourts.gov/court-info/local-rules-and-orders/local-rules links a same-named 30-page PDF whose extracted text is byte-identical, so it is vendored once here.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Fetched via curl (single document, first request succeeded, no block encountered).
- Extract sha256 8f0a980379167512682342755ac946da441141b4df4cf195406343945ca92a04. Raw extract, not yet sectioned into `## <ref>` form.

- Fetch method: first fetched with curl and a desktop browser User-Agent before the no-browser-identity rule (PM 2026-10-27); re-fetched on 2026-10-27 with plain curl (default User-Agent, no browser headers): byte-identical (same sha256).
- Original: `kyed/original/kyed-joint-local-civil-rules.pdf` as served (sha256 f7942d10800716fe5cc62401a47f6ef71d8db68cbc5516a504df2263597d1086); extraction sha256 8f0a980379167512682342755ac946da441141b4df4cf195406343945ca92a04.

### `kyed/kyed-joint-local-criminal-rules-2026-08.txt` — E.D. and W.D. Ky. Joint Local Criminal Rules (August 2026)

- Source: https://www.kyed.uscourts.gov/sites/kyed/files/KY_Amended_Criminal_Rules_August_2026.pdf (official site; 21 pages, sha256 87b3e8f92ce42c9385bd2bacf8b1daff81d1208ce3431e138279f2ee1259354b).
- Edition: Joint Local Rules of Criminal Practice (LCrR), the PDF 'KY_Amended_Criminal_Rules_August_2026' linked as 'Criminal Rules' at https://www.kyed.uscourts.gov/local-rules (same text as the W.D. Ky. file). robots.txt of www.kyed.uscourts.gov read (Crawl-delay 10 honored where stated; no Disallow covers the fetched path); the judiciary-wide Privacy & Security Policy (https://www.uscourts.gov/privacy-security-policy) read: no prohibition of automated access. Fetched with curl and the identifying UA lit-lang-provenance/1.0, at most 1 request per 10 s.
- Criminal forum (lg-gddt): kyed-criminal.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 a719630de26b3872e11d0a69ab5135a0df7a9b010b35195da591d5aaaa45adb3. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `kyed/original/kyed-joint-local-criminal-rules-2026-08.pdf` as served (sha256 87b3e8f92ce42c9385bd2bacf8b1daff81d1208ce3431e138279f2ee1259354b); extraction sha256 a719630de26b3872e11d0a69ab5135a0df7a9b010b35195da591d5aaaa45adb3.

### `kyed/kyed-joint-local-criminal-rules-2026-08-sectioned.txt` — E.D. and W.D. Ky. Joint Local Criminal Rules (August 2026), sectioned
- Derived in the pod (lg-gddt.16) from `kyed/kyed-joint-local-criminal-rules-2026-08.txt` (extract sha256 a719630de26b3872e11d0a69ab5135a0df7a9b010b35195da591d5aaaa45adb3); nothing fetched. One `## LCrR <n>` line is inserted right after each rule's own body heading line `LCrR <n> <title>` (32 markers, LCrR 1.1 through LCrR 60.1; the table of contents and the introduction are unmarked, and LCrR 60.1's section runs through the index to the end of the file). Rule text was not edited: deleting the marker lines (`sed '/^## LCrR /d'`) restores the extract byte for byte.
- Edition: as the extract, the Joint Local Rules of Criminal Practice served as "KY_Amended_Criminal_Rules_August_2026"; the document states no effective date and the adopting Joint General Order (LCrR 59.1) is not vendored.
- Upstream: https://www.kyed.uscourts.gov/sites/kyed/files/KY_Amended_Criminal_Rules_August_2026.pdf, retrieved 2026-10-28, as recorded on `kyed/kyed-joint-local-criminal-rules-2026-08.txt` above; sectioning was done from the vendored extract.
- Sectioned sha256 765455429286794288bfecd3aa26fa70a71c8b932350abb4e4b40499e94bc2e4 (the `path_sha256` pinned by `lit/kyed-criminal/Lit.toml`).

### `kyed/kyed-joint-local-civil-rules-sectioned.txt` — E.D. and W.D. Ky. Joint Local Civil Rules (August 2026), sectioned
- Derived in the pod (lg-f3q8.91) from `kyed/kyed-joint-local-civil-rules.txt` (extract sha256 8f0a980379167512682342755ac946da441141b4df4cf195406343945ca92a04); nothing fetched. One `## LR <n>` line is inserted right after each rule's own body heading line `LR <n> <title>` (51 markers, LR 1.1 through LR 86.1, LR 7.1.1 and LR 65.1.1 among them; the table of contents is unmarked, and LR 86.1's section runs through the index to the end of the file). Rule text was not edited: deleting the marker lines (`sed '/^## LR /d'`) restores the extract byte for byte.
- Edition: as the extract, the Joint Local Rules of Civil Practice served as "KY_Amended_Civil_Rules_August_2026"; the document states no effective date (LR 86.1: amendments take effect on each Joint General Order, none vendored).
- Sectioned sha256 57d7a49d9a6bc2a6538632417ed1d42b36ccd97bca95654c1752bc5bf967b91a (the `path_sha256` pinned by `lit/kyed/Lit.toml`).
