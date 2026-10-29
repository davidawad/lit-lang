
### `nced/nced-local-civil-rules.txt` — U.S. District Court for the Eastern District of North Carolina Local Civil Rules

- Source: https://www.nced.uscourts.gov/pdfs/Local%20Civil%20Rules%202023.pdf (official site; 98 pages, sha256 fdfd8a2f25ad587e6cdf99c4bcd4f4a238b72f6827ca7251b480c2fb3792e7ea).
- Edition: Local Civil Rules of Practice and Procedure, May 2023 (amendments effective May 31, 2023 per 23-SO-2; linked as "Local Civil Rules (effective May 31, 2023)" on https://www.nced.uscourts.gov/rules/Default.aspx, the latest the page lists).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 4cdb81a6a3898e676ee357fd5a28ada685c23fb50a13a668f7f8b4071a1d33e0. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `nced/original/nced-local-civil-rules.pdf` as served (sha256 fdfd8a2f25ad587e6cdf99c4bcd4f4a238b72f6827ca7251b480c2fb3792e7ea); extraction sha256 4cdb81a6a3898e676ee357fd5a28ada685c23fb50a13a668f7f8b4071a1d33e0.

### `nced/nced-local-criminal-rules-2023-05.txt` — E.D.N.C. Local Criminal Rules of Practice and Procedure (May 2023)

- Source: https://www.nced.uscourts.gov/pdfs/Local%20Criminal%20Rules%202023.pdf (official site; 55 pages, sha256 3f825feeafc92cb1f71b3c9761d99971d50ff9ae5336d235fe137bd2bd485464).
- Edition: Local Criminal Rules of Practice and Procedure 'May 2023' (title page), the 'Clean Version' under the 2023 criminal rules at https://www.nced.uscourts.gov/rules/Default.aspx. no robots.txt served (404); the judiciary-wide Privacy & Security Policy (https://www.uscourts.gov/privacy-security-policy) read: no prohibition of automated access. Fetched with curl and the identifying UA lit-lang-provenance/1.0, at most 1 request per 10 s.
- Criminal forum (lg-gddt): nced-criminal.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 0f1a3f457d9faaeda6bc02fbe3878da6a9008404c6d46dcea166755fb099b0b7. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `nced/original/nced-local-criminal-rules-2023-05.pdf` as served (sha256 3f825feeafc92cb1f71b3c9761d99971d50ff9ae5336d235fe137bd2bd485464); extraction sha256 0f1a3f457d9faaeda6bc02fbe3878da6a9008404c6d46dcea166755fb099b0b7.

### `nced/nced-local-civil-rules-sectioned.txt` — E.D.N.C. Local Civil Rules (May 2023), sectioned

- Derived in the pod (lg-f3q8.29) from `nced/nced-local-civil-rules.txt` (extract sha256 4cdb81a6a3898e676ee357fd5a28ada685c23fb50a13a668f7f8b4071a1d33e0); nothing fetched. One `## Rule <n>` line is inserted right before each Local Civil Rule's own body heading line `Rule <n>  <title>` (from extract line 515 on): 128 markers, Rule 1.1 through Rule 100.4. The table of contents and the in-body cross-references are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Upstream: https://www.nced.uscourts.gov/pdfs/Local%20Civil%20Rules%202023.pdf, retrieved 2026-10-27, as recorded on the `nced/nced-local-civil-rules.txt` entry this file is derived from.
- Edition: as the extract, the Local Civil Rules of Practice and Procedure, May 2023 (amendments effective May 31, 2023).
- Sectioned sha256 7ea7072dbbf52629c8a7f15c0692de2e9d002d8eb6530d783acdb0b20506ef98 (the `path_sha256` pinned by `lit/nced/Lit.toml`).

### `nced/nced-local-criminal-rules-2023-05-sectioned.txt` — E.D.N.C. Local Criminal Rules (May 2023), sectioned

- Derived in the pod (lg-gddt.12) from `nced/nced-local-criminal-rules-2023-05.txt` (extract sha256 0f1a3f457d9faaeda6bc02fbe3878da6a9008404c6d46dcea166755fb099b0b7); nothing fetched. One `## Local Criminal Rule <n>` line is inserted right before each rule's own body heading line `Rule <n>  <title>` (from extract line 330 on): 88 markers, Rule 1.1 through Rule 100.2. The table of contents and the in-body cross-references are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## Local Criminal Rule /d'`) restores the extract byte for byte.
- Source: as the extract `nced/nced-local-criminal-rules-2023-05.txt`, https://www.nced.uscourts.gov/pdfs/Local%20Criminal%20Rules%202023.pdf; Retrieved 2026-10-28 with the extract (this file was derived from it in the pod, not fetched itself).
- Edition: as the extract, the Local Criminal Rules of Practice and Procedure, May 2023.
- Sectioned sha256 d42904459ea530131e7bf92536261d78ef08c358b586bbbc2fbcf7c087159c14 (the `path_sha256` pinned by `lit/nced-criminal/Lit.toml`).
