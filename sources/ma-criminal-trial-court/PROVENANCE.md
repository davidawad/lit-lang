
### `ma-criminal-trial-court/ma-rules-of-criminal-procedure-2025-11-01.txt` — Massachusetts Rules of Criminal Procedure (429 pp; Rule 36 is case management / speedy trial)

- Source: https://www.mass.gov/doc/massachusetts-rules-of-criminal-procedure/download (official site; 429 pages, sha256 1269f9b86ec5094638d7348d445966a884731804983dd3f0ee03ca92e22f3847).
- Edition: Massachusetts Rules of Criminal Procedure, 'Including amendments effective November 1, 2025', published by the Massachusetts Trial Court Law Libraries (served as jud-lib-rules-of-criminal-procedure-nov-2025.pdf), linked from https://www.mass.gov/law-library/massachusetts-rules-of-criminal-procedure. www.mass.gov serves no robots.txt (no robots file returned); home page not retrievable (403); terms not read; fetched on David's explicit ruling 2026-10-28 despite robots/terms (barrier: a bot challenge, not a robots or terms refusal: plain curl answered with a WAF filter that returned HTTP 403 to curl on the first request (mass.gov) (HTTP 403); the pages were fetched in BrowserOS neo with an in-page fetch from the loaded origin at about 1 request per 1.1 s); tool: BrowserOS neo with the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 b4651ba12ce09aadd95d6df67b1463c881fc94376935fef3029e6527d39b5479. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `ma-criminal-trial-court/original/ma-rules-of-criminal-procedure-2025-11-01.pdf` as served (sha256 1269f9b86ec5094638d7348d445966a884731804983dd3f0ee03ca92e22f3847); extraction sha256 b4651ba12ce09aadd95d6df67b1463c881fc94376935fef3029e6527d39b5479.

- Note (lg-gddt): Criminal appeals: Massachusetts Rules of Appellate Procedure, covered by sources/ma-appeals-sjc/ if it carries them.

### `ma-criminal-trial-court/ma-rules-of-criminal-procedure-2025-11-01-sectioned.txt` — Massachusetts Rules of Criminal Procedure, sectioned

- Derived in the pod (lg-gddt.6) from `ma-criminal-trial-court/ma-rules-of-criminal-procedure-2025-11-01.txt` (extract sha256 b4651ba12ce09aadd95d6df67b1463c881fc94376935fef3029e6527d39b5479); nothing fetched. One `## Rule <n>` line is inserted right after each rule's own heading line `Rule <n>: <Title>` (54 rules, Rule 1 through Rule 49 with 3.1 and 14.1 to 14.4; each section runs through that rule's history and Reporter's Notes); after, not before, because the cite corpus also opens a section at a `Rule <n>:` line, as in `ma-appeals-sjc/ma-rules-appellate-procedure-sectioned.txt`. Cross-references that begin a line without a colon (`Rule 14.4.`, `Rule 14. This amendment ...`) are not headings and are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Edition: as the extract, including amendments effective November 1, 2025.
- Upstream: https://www.mass.gov/doc/massachusetts-rules-of-criminal-procedure/download, retrieved 2026-10-28, as recorded on `ma-criminal-trial-court/ma-rules-of-criminal-procedure-2025-11-01.txt` above; sectioning was done from the vendored extract.
- Sectioned sha256 e8e01b5566775a4eab96a9d3cebab847d63ac97b8c0760aec343b4c56c7d5f4e (the `path_sha256` pinned by `lit/ma-criminal-trial-court/Lit.toml`).
