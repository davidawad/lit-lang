
### `ca-criminal-trial-court/ca-rules-of-court-title-4-criminal.txt` — California Rules of Court, Title 4, Criminal Rules (127 pp)

- Source: https://courts.ca.gov/system/files?file=file/roc-title-4.pdf (official site; 127 pages, sha256 28800aa0f531d61ad9cd71898bc244b191ab170f9d5cda14a411a24872cec6f8).
- Edition: California Rules of Court Title 4 (Criminal Rules), the 'roc-title-4' PDF linked from https://courts.ca.gov/cms/rules/index/four; no edition date on the document (PDF metadata modified 2026-02-27), CURRENCY UNCONFIRMED; the Penal Code (including section 1382, speedy trial) is a separate source. courts.ca.gov robots.txt read; no Disallow covering this path; Accessibility page (https://courts.ca.gov/about/accessibility) read: no automated-access prohibition found; fetched on David's explicit ruling 2026-10-28 despite robots/terms; tool: curl with the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 099579728f93ba89972bf815ab13e26485d66b50b2fbbebf86a6a3a0ec6acc39. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `ca-criminal-trial-court/original/ca-rules-of-court-title-4-criminal.pdf` as served (sha256 28800aa0f531d61ad9cd71898bc244b191ab170f9d5cda14a411a24872cec6f8); extraction sha256 099579728f93ba89972bf815ab13e26485d66b50b2fbbebf86a6a3a0ec6acc39.

- Note (lg-gddt): Criminal appeals are governed by Title 8 of the California Rules of Court, covered by sources/ca-court-of-appeal/ if it carries Title 8 (check its entry); the Penal Code text (Part 2, Of Criminal Procedure, with section 1382) was not retrievable by curl (Cloudflare challenge) and is queued for BrowserOS.

### `ca-criminal-trial-court/ca-penal-code-1050-1381-1382-1473.txt` — California Penal Code sections 1050, 1381, 1381.5, 1382 (speedy trial / dismissal for want of prosecution) and 1473 (habeas), from leginfo

- Source: https://leginfo.legislature.ca.gov/faces/codes_displaySection.xhtml?lawCode=PEN (5 official HTML page(s), one GET each).
- Edition: California Penal Code sections as served by the Legislative Counsel's leginfo site on the retrieval date; the pages carry no edition date beyond the section's own history lines, CURRENCY UNCONFIRMED. TARGETED SET: only the speedy-trial and cited sections were fetched; the rest of Penal Code Part 2 is not vendored. leginfo.legislature.ca.gov serves no robots.txt (no robots file returned); home page not retrievable (403); terms not read; fetched on David's explicit ruling 2026-10-28 despite robots/terms (barrier: a bot challenge, not a robots or terms refusal: plain curl answered with a Cloudflare/WAF challenge (HTTP 403); the pages were fetched in BrowserOS neo with an in-page fetch from the loaded origin at about 1 request per 1.1 s); tool: BrowserOS neo with the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); each page extracted with html2txt v2 (stdlib html.parser, text restricted to the element with id codeLawSectionNoHead), unedited, concatenated under one `## <heading>` per page.
- Extract sha256 9ffe136449bac7fe3e0512b24b544975ae4771b96ba393ad7a2b779326d6ad56. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `ca-criminal-trial-court/original/penal-code-pages/0001.html` as served (sha256 8b668bca6add9ff191c050a58e882d7328ef3b9a6577d7faab68f90ab6024f5e). Served at https://leginfo.legislature.ca.gov/faces/codes_displaySection.xhtml?lawCode=PEN&sectionNum=1050.
- Original: `ca-criminal-trial-court/original/penal-code-pages/0002.html` as served (sha256 377b2225685997d1fdb95d6c56e93ba419ba2a182617f5707298a3025f82e59a). Served at https://leginfo.legislature.ca.gov/faces/codes_displaySection.xhtml?lawCode=PEN&sectionNum=1381.
- Original: `ca-criminal-trial-court/original/penal-code-pages/0003.html` as served (sha256 0e5d40bd56191746f77b69463a130c9ebf6c24b3f34cf8b33197aef78409b086). Served at https://leginfo.legislature.ca.gov/faces/codes_displaySection.xhtml?lawCode=PEN&sectionNum=1381.5.
- Original: `ca-criminal-trial-court/original/penal-code-pages/0004.html` as served (sha256 6f71134fc8cbd1165c4dabb8dd45728f265242fe5925380961d9682192ba20d6). Served at https://leginfo.legislature.ca.gov/faces/codes_displaySection.xhtml?lawCode=PEN&sectionNum=1382.
- Original: `ca-criminal-trial-court/original/penal-code-pages/0005.html` as served (sha256 bd18ad28a5d9705737c74e3d8d60edfb514dea00aa734a05e72ae054f6420728). Served at https://leginfo.legislature.ca.gov/faces/codes_displaySection.xhtml?lawCode=PEN&sectionNum=1473.
- Extraction sha256 9ffe136449bac7fe3e0512b24b544975ae4771b96ba393ad7a2b779326d6ad56.

- Note (lg-gddt): The earlier PARTIAL status of this forum (Penal Code pending) is resolved for the speedy-trial sections: Title 4 Criminal Rules of Court (rules text) plus Penal Code 1050, 1381, 1381.5, 1382 and 1473. Other Penal Code parts the outline names generically ('arraignment', 'discovery', etc.) are not vendored.

### `ca-criminal-trial-court/ca-rules-of-court-title-4-criminal-sectioned.txt` — California Rules of Court, Title 4 (Criminal Rules), sectioned

- Derived in the pod (lg-gddt.22) from `ca-criminal-trial-court/ca-rules-of-court-title-4-criminal.txt` (extract sha256 099579728f93ba89972bf815ab13e26485d66b50b2fbbebf86a6a3a0ec6acc39); nothing fetched. One marker line `## Cal. Rules of Court, rule <n>` is inserted before each rule's own body heading line `Rule 4.<n>. <Title>` at the left margin (99 rules, rule 4.1 through 4.700); the indented contents lines of each division carry no markers. Every other line is the extract unchanged, so deleting the marker lines gives back the extract byte for byte.
- Edition: as the extract; CURRENCY UNCONFIRMED stands. `lit/ca-criminal-trial-court` cites it for structure only and takes no value from it.
- Sectioned sha256 dba7a1f830790c37fab4276a7d8e390d72ecf6e8568409a8a04bde7bfd30c5a3 (the `path_sha256` pinned by `lit/ca-criminal-trial-court/Lit.toml`).
