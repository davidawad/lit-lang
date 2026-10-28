
### `fed-dhaw/dhaw-local-rules.txt` — District of Hawaii Local Rules (civil and general)

- Source: https://www.hid.uscourts.gov/files/order532/2019_08_26_administrative_Order%20Amending%20the%20Local%20Rules%20eff%202019_09_01(1).pdf (official site; 76 pages, sha256 7d2ec4298f1478105779562485971cc03754b2cbe42c98a832b8fb1de542ef1f).
- Edition: District of Hawaii Local Rules (civil and general chapters), administrative order of 2019-08-26 amending the Local Rules effective September 1, 2019 (LR1.2), linked as "Civil Local Rules" from https://www.hid.uscourts.gov/rules/local-rules (the page lists no later revision; the site is a JavaScript app, so the link was read in BrowserOS, one tab, since closed; the PDF itself was fetched with plain curl; no BrowserOS download).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 47e02f2e80565f033be9dd7f0fd5b024f4fceb9fb625931b53d037b3d8a37681. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `fed-dhaw/original/dhaw-local-rules.pdf` as served (sha256 7d2ec4298f1478105779562485971cc03754b2cbe42c98a832b8fb1de542ef1f); extraction sha256 47e02f2e80565f033be9dd7f0fd5b024f4fceb9fb625931b53d037b3d8a37681.

### `fed-dhaw/dhaw-criminal-local-rules-2023-01-01.txt` — D. Haw. Criminal Local Rules (effective January 1, 2023)

- Source: https://www.hid.uscourts.gov/docs/localrules/USDC%20HID%20Criminal%20Local%20Rules%20Final%2012.16.2022.pdf (official site; 58 pages, sha256 12d2ec99eec2289197a10b0f89bf79ee66c1401e96f0cb1ed7eaebbaea579af6).
- Edition: Criminal Local Rules (CrimLR) 'Effective January 1, 2023' per the listing, the PDF filed December 16, 2022, linked as 'Criminal Local Rules (Effective January 1, 2023)' at https://www.hid.uscourts.gov/rules/local-rules (a JavaScript site, read through BrowserOS neo; the PDF itself fetched with curl); a 2024-2026 amendment order was not found on that page. robots.txt of www.hid.uscourts.gov read (Crawl-delay 10 honored where stated; no Disallow covers the fetched path); the judiciary-wide Privacy & Security Policy (https://www.uscourts.gov/privacy-security-policy) read: no prohibition of automated access. Fetched with curl and the identifying UA lit-lang-provenance/1.0, at most 1 request per 10 s.
- Criminal forum (lg-gddt): hid-criminal.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 a70eb14678c7c85bd294523f6a7d7aa950f6e4e37b82e3521359aaca8aa8825a. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `fed-dhaw/original/dhaw-criminal-local-rules-2023-01-01.pdf` as served (sha256 12d2ec99eec2289197a10b0f89bf79ee66c1401e96f0cb1ed7eaebbaea579af6); extraction sha256 a70eb14678c7c85bd294523f6a7d7aa950f6e4e37b82e3521359aaca8aa8825a.

### `fed-dhaw/dhaw-local-rules-sectioned.txt` — D. Haw. Local Rules (civil and general), sectioned

- Derived in the pod (lg-f3q8.42) from `fed-dhaw/dhaw-local-rules.txt` (extract sha256 47e02f2e80565f033be9dd7f0fd5b024f4fceb9fb625931b53d037b3d8a37681); nothing fetched. One `## LR <n>` line is inserted before each Local Rule's own heading line in the body (`LR1.1.` through `LR8009.1.`, 88 rules; `LR1070.1(a)` through `(e)` share the one marker `## LR 1070.1`); the table of contents is unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## LR /d'`) restores the extract byte for byte.
- Upstream: https://www.hid.uscourts.gov/files/order532/2019_08_26_administrative_Order%20Amending%20the%20Local%20Rules%20eff%202019_09_01(1, retrieved 2026-10-27, as recorded on the `fed-dhaw/dhaw-local-rules.txt` entry this file is derived from.
- Edition: as the extract, the Local Rules as amended by the administrative order of 2019-08-26, effective September 1, 2019.
- Sectioned sha256 6fb5b0546e0094dae365ecdfdfa4fce3de96fd6b95c7bba9c22a4afa07d684f2 (the `path_sha256` pinned by `lit/hid/Lit.toml`).

### `fed-dhaw/dhaw-criminal-local-rules-2023-01-01-sectioned.txt` — D. Haw. Criminal Local Rules (effective January 1, 2023), sectioned

- Derived in the pod (lg-gddt.22) from `fed-dhaw/dhaw-criminal-local-rules-2023-01-01.txt` (extract sha256 a70eb14678c7c85bd294523f6a7d7aa950f6e4e37b82e3521359aaca8aa8825a); nothing fetched. One marker line `## CrimLR <n>` is inserted before each rule's own body heading line `CrimLR<n>. <Title>` (CrimLR 2.1 through 85.1, 57 rules; CrimLR 85.1's heading has no period after the number); the table of contents carries no markers. Every other line is the extract unchanged, so deleting the marker lines gives back the extract byte for byte.
- Edition: as the extract, effective January 1, 2023.
- Sectioned sha256 3edc553a908be81301db8f8ce3cd479423cbf3d75916345ffc8644af85caf031 (the `path_sha256` pinned by `lit/hid-criminal/Lit.toml`).
