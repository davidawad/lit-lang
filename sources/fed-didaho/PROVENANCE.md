
### `fed-didaho/didaho-local-criminal-rules-packet-2026-10-27.txt` — D. Idaho Local Criminal Rules, complete rule packet (printed October 27, 2026)

- Source: https://www.id.uscourts.gov/content_fetcher/print_pdf_packet.cfml?Court_Unit=District&Content_Type=Rule&Content_Sub_Type=Criminal (official site; 20 pages, sha256 22544af39f2b5e5d17d30b1c72fad06eb9a02f207d70cf5d0a30fff8d283c3f5).
- Edition: Idaho Local District Criminal Rules Packet 'Printed: Oct 27, 2026' (first page), the 'Get Complete Rule Packet' PDF generated at https://www.id.uscourts.gov/clerks/rules_orders/Criminal_Local_Rules.cfm?lni=318 (no edition date other than the print date; the live rules database is the court's current text). robots.txt answered with a redirect, no rules read; the judiciary-wide Privacy & Security Policy (https://www.uscourts.gov/privacy-security-policy) read: no prohibition of automated access. Fetched with curl and the identifying UA lit-lang-provenance/1.0, at most 1 request per 10 s.
- Criminal forum (lg-gddt): idd-criminal.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 783f17646824794fabb7d301c86eed5dd92fcd0d5be5532bded8611c75ae152b. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `fed-didaho/original/didaho-local-criminal-rules-packet-2026-10-27.pdf` as served (sha256 22544af39f2b5e5d17d30b1c72fad06eb9a02f207d70cf5d0a30fff8d283c3f5); extraction sha256 783f17646824794fabb7d301c86eed5dd92fcd0d5be5532bded8611c75ae152b.

### `fed-didaho/didaho-local-civil-rules-packet-2026-10-27.txt` — D. Idaho Local Civil Rules, complete rule packet (printed October 27, 2026)

- Source: https://www.id.uscourts.gov/content_fetcher/print_pdf_packet.cfml?Court_Unit=District&Content_Type=Rule&Content_Sub_Type=Civil (official site; 77 pages, sha256 8e7e03dc3e8cb94999855f56aa707d3f7460d64f4b813d2c6d144f1bc48aeefc).
- Edition: Idaho Local District Civil Rules Packet 'Printed: Oct 27, 2026' (first page), the 'Get Complete Rule Packet' PDF generated at https://www.id.uscourts.gov/clerks/rules_orders/Civil_Local_Rules.cfm?lni=317 (fetched because sources/fed-didaho did not exist). robots.txt answered with a redirect, no rules read; the judiciary-wide Privacy & Security Policy (https://www.uscourts.gov/privacy-security-policy) read: no prohibition of automated access. Fetched with curl and the identifying UA lit-lang-provenance/1.0, at most 1 request per 10 s.
- Criminal forum (lg-gddt): idd-criminal.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 8558c08009284fdf5967fa722a162e2160428cb2a102deffe631886265ea20b3. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `fed-didaho/original/didaho-local-civil-rules-packet-2026-10-27.pdf` as served (sha256 8e7e03dc3e8cb94999855f56aa707d3f7460d64f4b813d2c6d144f1bc48aeefc); extraction sha256 8558c08009284fdf5967fa722a162e2160428cb2a102deffe631886265ea20b3.

### `fed-didaho/didaho-local-civil-rules-packet-2026-10-27-sectioned.txt` — D. Idaho Local Civil Rules packet (printed October 27, 2026), sectioned

- Derived in the pod (lg-f3q8.43) from `fed-didaho/didaho-local-civil-rules-packet-2026-10-27.txt` (extract sha256 8558c08009284fdf5967fa722a162e2160428cb2a102deffe631886265ea20b3); nothing fetched. One `## Loc. Civ. R. <n>` line is inserted right before each rule's own body heading line `District Local Rule Civ <n> (Civil)` (each led by a form feed): 58 markers, Loc. Civ. R. 1.1 through Loc. Civ. R. 83.8. The table of contents and the announcement page are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## Loc\. Civ\. R\. /d'`) restores the extract byte for byte.
- Upstream: https://www.id.uscourts.gov/content_fetcher/print_pdf_packet.cfml?Court_Unit=District&Content_Type=Rule&Content_Sub_Type=Civil, retrieved 2026-10-28, as recorded on the `fed-didaho/didaho-local-civil-rules-packet-2026-10-27.txt` entry this file is derived from.
- Edition: as the extract; the packet's announcement page reads "Revised and adopted August 17, 2026".
- Sectioned sha256 cd724975f097a38678edcd34535f4600ebf75163e12bb0d4e817c1a81c7194ec (the `path_sha256` pinned by `lit/idd/Lit.toml`).

### `fed-didaho/didaho-local-criminal-rules-packet-2026-10-27-sectioned.txt` — D. Idaho Local Criminal Rules packet (printed October 27, 2026), sectioned

- Derived in the pod (lg-gddt.22) from `fed-didaho/didaho-local-criminal-rules-packet-2026-10-27.txt` (extract sha256 783f17646824794fabb7d301c86eed5dd92fcd0d5be5532bded8611c75ae152b); nothing fetched. One marker line `## Loc. Crim. R. <n>` is inserted before each rule's own heading line `District Local Rule Crim <n> (Criminal)` (each led by a form feed; Loc. Crim. R. 1.1 through 59.1, 16 rules, matching the packet's list); the list and announcement carry no markers. Every other line is the extract unchanged, so deleting the marker lines gives back the extract byte for byte.
- Edition: as the extract; the packet's announcement page reads "Revised and adopted August 17, 2026".
- Sectioned sha256 ac21449f118fd0715f141064db150c3f22380e28c1896f156747f198ce171fb3 (the `path_sha256` pinned by `lit/idd-criminal/Lit.toml`).
