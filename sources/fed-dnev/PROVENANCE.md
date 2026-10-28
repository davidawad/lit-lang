
### `fed-dnev/dnev-local-rules-2020-04-17.txt` — D. Nev. Local Rules of Practice (April 17, 2020)

- Source: https://www.nvd.uscourts.gov/wp-content/uploads/2020/04/Local-Rules-of-Practice-Amended-2020.pdf (official site; 115 pages, sha256 f90ed909fa5aa88cf17fc97853803b834aa791003d2894707b3fd93c5ae64346).
- Edition: Local Rules of Practice of the U.S. District Court for the District of Nevada, 'April 17, 2020' (title page, with General Order 2020-07 adopting the further amendments), the only edition linked as Local Rules at https://www.nvd.uscourts.gov/court-information/rules-and-orders/. OLD EDITION presented by the court as current; the general orders table at https://www.nvd.uscourts.gov/court-information/rules-and-orders/general-orders/ lists no later order amending the Local Rules (GO 2021-05 sets procedural rules for non-habeas civil rights filings and is not vendored). fetched on David's explicit ruling 2026-10-28 despite robots/terms (barrier: robots.txt 'User-agent: * / Disallow: /' on www.nvd.uscourts.gov; tool: curl with the identifying UA lit-lang-provenance/1.0, at most 1 request per 10 s).
- Also the criminal local rules (lg-gddt): LCR 1-1 and following (criminal, administrative proceedings, probation), from extract line 4280.
- Criminal forum (lg-gddt): nvd-criminal.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 6537ebf98ea289b3393e895132b80bc71d5633e2fdcc6f5c03b7ce618844cc1b. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `fed-dnev/original/dnev-local-rules-2020-04-17.pdf` as served (sha256 f90ed909fa5aa88cf17fc97853803b834aa791003d2894707b3fd93c5ae64346); extraction sha256 6537ebf98ea289b3393e895132b80bc71d5633e2fdcc6f5c03b7ce618844cc1b.

### `fed-dnev/dnev-local-rules-2020-04-17-sectioned.txt` — D. Nev. Local Rules of Practice (April 17, 2020), sectioned

- Derived in the pod (lg-f3q8.45) from `fed-dnev/dnev-local-rules-2020-04-17.txt` (extract sha256 6537ebf98ea289b3393e895132b80bc71d5633e2fdcc6f5c03b7ce618844cc1b); nothing fetched. One `## <rule>` line is inserted right before each rule's own body heading line in Parts IA, IB, IC and II (`LR IA 1-1.` through `LR 81-1.`, some led by a form feed): 119 markers, `## LR IA <n>`, `## LR IB <n>`, `## LR IC <n>` and `## LR <n>`. The table of contents and Parts III to V (patent, criminal, special proceedings) are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## LR /d'`) restores the extract byte for byte.
- Upstream: https://www.nvd.uscourts.gov/wp-content/uploads/2020/04/Local-Rules-of-Practice-Amended-2020.pdf, retrieved 2026-10-28, as recorded on the `fed-dnev/dnev-local-rules-2020-04-17.txt` entry this file is derived from.
- Edition: as the extract; "These rules, as amended, take effect on April 17, 2020" (LR IA 1-5).
- Sectioned sha256 1f0f9aab6084f6c537612da46e869a18e018b047ff5136dbef92fbd712b78058 (the `path_sha256` pinned by `lit/nvd/Lit.toml`).
