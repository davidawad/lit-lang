
### `msnd/local-uniform-civil-rules.txt` — N.D. and S.D. Miss. Local Uniform Civil Rules

- Source: https://www.msnd.uscourts.gov/sites/msnd/files/forms/2024%20Local%20Civil%20Rules%20MSSD%20MSND_12.1.2024.pdf (official site; 164 pages, sha256 e538403825e3f44c5112e6f929ce266b3f39e31387df9ce57823f76bf9df055d).
- Edition: Local Uniform Civil Rules of the N.D. and S.D. Mississippi, dated December 1, 2024 (cover); linked from https://www.msnd.uscourts.gov/forms/uniform-local-civil-rules and, as the same-titled file, from https://www.mssd.uscourts.gov/court-info/local-rules-and-orders/local-rules; one joint document, vendored once under msnd and equally the S.D. Miss. (mssd) local rules; fetched with curl (plain User-Agent), no bot refusal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 953f3a272cbcab3917a2e99beb95f13b8df0eb49dd281f265cd22c6a1a8b6087. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `msnd/original/local-uniform-civil-rules.pdf` as served (sha256 e538403825e3f44c5112e6f929ce266b3f39e31387df9ce57823f76bf9df055d); extraction sha256 953f3a272cbcab3917a2e99beb95f13b8df0eb49dd281f265cd22c6a1a8b6087.

### `msnd/msnd-uniform-local-criminal-rules-2019-12-01.txt` — N.D. and S.D. Miss. Local Uniform Criminal Rules (effective December 1, 2019)

- Source: https://www.msnd.uscourts.gov/sites/msnd/files/forms/2019MASTERCOPYCRIMINAL.pdf (official site; 8 pages, sha256 9cc5d3ebfb0ea63ed87a35b4fe84f2cadf30580c3acfab79b84d8575e5a47619).
- Edition: Local Uniform Criminal Rules of the Northern and Southern Districts of Mississippi 'EFFECTIVE December 1, 2019' (title page), the 2019MASTERCOPYCRIMINAL file linked as 'Uniform Local Criminal Rules' at https://www.msnd.uscourts.gov/court-info/local-rules-and-orders (same text as the S.D. Miss. file). robots.txt of www.msnd.uscourts.gov read (Crawl-delay 10 honored where stated; no Disallow covers the fetched path); the judiciary-wide Privacy & Security Policy (https://www.uscourts.gov/privacy-security-policy) read: no prohibition of automated access. Fetched with curl and the identifying UA lit-lang-provenance/1.0, at most 1 request per 10 s.
- Criminal forum (lg-gddt): msnd-criminal.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 d7b754294b86aec547fb4d55d49367664809e3f46e34b14955031ca674249abc. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `msnd/original/msnd-uniform-local-criminal-rules-2019-12-01.pdf` as served (sha256 9cc5d3ebfb0ea63ed87a35b4fe84f2cadf30580c3acfab79b84d8575e5a47619); extraction sha256 d7b754294b86aec547fb4d55d49367664809e3f46e34b14955031ca674249abc.

### `msnd/local-uniform-civil-rules-sectioned.txt` — N.D. and S.D. Miss. Local Uniform Civil Rules, sectioned

- Derived in the pod (lg-f3q8.19) from `msnd/local-uniform-civil-rules.txt` (extract sha256 953f3a272cbcab3917a2e99beb95f13b8df0eb49dd281f265cd22c6a1a8b6087); nothing fetched. In the body (after the table of contents), one `## L.U. Civ. R. <n>` line is inserted before each rule's own heading line (`Rule 1.` through `Rule 83.8.`, 33 rules), one `## L.U. Adm. R. <letter>` line before each local admiralty rule heading (`Rule A.` through `Rule G.`), and one `## Official Forms` line before the forms. Every other line is the extract unchanged, so deleting the `## ` lines gives back the extract byte for byte.
- Upstream: https://www.msnd.uscourts.gov/sites/msnd/files/forms/2024%20Local%20Civil%20Rules%20MSSD%20MSND_12.1.2024.pdf, retrieved 2026-10-27, as recorded on the `msnd/local-uniform-civil-rules.txt` entry this file is derived from.
- Edition: as the extract, Local Uniform Civil Rules "CURRENT AS OF DECEMBER 1, 2024" (cover). The same extract is vendored as `mssd/mssd-uniform-local-civil-rules-2024-12-01.txt` (identical extract sha256); lit/msnd and lit/mssd both pin this sectioned copy.
- Sectioned sha256 12c17e9935fa4522baf28e69db7c9878420bd5fa902cd4ba7a526b889a88f330 (the `path_sha256` pinned by `lit/msnd/Lit.toml`).

### `msnd/msnd-uniform-local-criminal-rules-2019-12-01-sectioned.txt` — N.D. and S.D. Miss. Local Uniform Criminal Rules, sectioned

- Derived in the pod (lg-gddt.14) from `msnd/msnd-uniform-local-criminal-rules-2019-12-01.txt` (extract sha256 d7b754294b86aec547fb4d55d49367664809e3f46e34b14955031ca674249abc); nothing fetched. In the body (after the table of contents), one marker line `## U.L.Cr.R. <n>` (the citation form U.L.Cr.R. 60 gives) is inserted before each rule's own heading line (`Rule 1. SCOPE.` through `Rule 60. TITLE.`, 9 rules). Every other line is the extract unchanged, so deleting the `## U.L.Cr.R.` lines gives back the extract byte for byte.
- Edition: as the extract, "EFFECTIVE December 1, 2019".
- Criminal forum (lg-gddt): msnd-criminal.
- Sectioned sha256 43a6852ec1eb974869aa39cac10c05f51c0667aecf89caa92fcbe4d7b189e36b (the `path_sha256` pinned by `lit/msnd-criminal/Lit.toml`).
