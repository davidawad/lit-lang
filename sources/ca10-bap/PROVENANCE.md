
### `ca10-bap/bap-local-rules-and-frbp-part-viii-2026.txt` — Tenth Circuit BAP Local Rules and FRBP Part VIII (2026 print edition)

- Source: https://www.bap10.uscourts.gov/sites/bap10/files/clerk/Bankruptcy%20Rules%20and%20BAP%20Local%20Rules%20%5BPrint%5D.pdf (official site; 97 pages, sha256 87b15eee973d27cbc6682c0d06f88c305f10d27b3d87f3ccfc84e24b5c71951e).
- Edition: BAP Local Rules effective January 1, 2026 and Part VIII of the Federal Rules of Bankruptcy Procedure effective December 1, 2024, as stated on the title page; linked from https://www.bap10.uscourts.gov/rules ("2026 FRBP and BAP Local Rules", which says FRBP changes take effect December 1, 2025); fetched with plain curl (no browser identity); robots.txt allows /sites/ and the site has no automated-access prohibition.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 c98e539d9f8db10376652be1222291ce49231761c5eef99d3ec831b38a490cfd. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `ca10-bap/original/bap-local-rules-and-frbp-part-viii-2026.pdf` as served (sha256 87b15eee973d27cbc6682c0d06f88c305f10d27b3d87f3ccfc84e24b5c71951e); extraction sha256 c98e539d9f8db10376652be1222291ce49231761c5eef99d3ec831b38a490cfd.

### `ca10-bap/bap-local-rules-sectioned.txt` — Tenth Circuit BAP Local Rules and FRBP Part VIII, sectioned

- Derived in the pod (lg-f3q8.64) from `ca10-bap/bap-local-rules-and-frbp-part-viii-2026.txt` (extract sha256 c98e539d9f8db10376652be1222291ce49231761c5eef99d3ec831b38a490cfd); nothing fetched. One `## 10th Cir. BAP L.R. <n>` line (the citation form of L.R. 8001-3(b)) is inserted before each local rule's own heading line in the body (38 markers), and one `## BAP print of FRBP <n> (December 1, 2024)` line before each FRBP Part VIII rule printed with them (30 markers), so a local rule's section ends where the next FRBP rule begins; no cite binds to the FRBP markers (FRBP cites bind to `sources/frbp.txt`, the December 1, 2025 edition). The table of contents is unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## 10th Cir. BAP L.R. /d;/^## BAP print of FRBP /d'`) restores the extract byte for byte.
- Source: as the extract `ca10-bap/bap-local-rules-and-frbp-part-viii-2026.txt`, https://www.bap10.uscourts.gov/sites/bap10/files/clerk/Bankruptcy%20Rules%20and%20BAP%20Local%20Rules%20%5BPrint%5D.pdf, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, BAP Local Rules effective January 1, 2026.
- Sectioned sha256 e691d77aa7ed5d4820a3f077cdd2bd371993f79baafb2dc56ed61f2d94d2de4f (the `path_sha256` pinned by `lit/bap10/Lit.toml`).

### `ca10-bap/bap10-home-2026-10-28.txt` — Tenth Circuit BAP home page (court location)

- Source: https://www.bap10.uscourts.gov/ (official site), the page's "Court Location" block: Byron White Court House, 1823 Stout Street, Denver, CO 80257.
- Why: 10th Cir. BAP L.R. 8026-8(b) makes a holiday of "the state in which this Court's clerk's office is located" a legal holiday; the rules do not say where that office is (lg-q28z, `colorado-legal-holidays` in lit/bap10).
- Retrieved 2026-10-28 in the pod (lg-q28z) with one curl fetch (User-Agent 'litgraph-rules-vendor/1.0 (public court rules)'); HTML converted to text (scripts and styles dropped, tags stripped, whitespace collapsed).
- Extract sha256 c29bb012f957bdc48ac63a74aef0043b9d83fe3aa2b3a325a39492d4e2ace77f.
- Original: `ca10-bap/original/bap10-home-2026-10-28.html` as served (sha256 53f6895ea5087a15e633092b004d6f3bc8aa311984eb69a5135ce142c48b1279); extraction sha256 c29bb012f957bdc48ac63a74aef0043b9d83fe3aa2b3a325a39492d4e2ace77f. Served at https://www.bap10.uscourts.gov/; fetched with curl (User-Agent litgraph-rules-vendor/1.0 (public court rules)), retrieved 2026-10-28.
