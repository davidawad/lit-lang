
### `vid/local-rules-civ-2026-02-01.txt` — D.V.I. Local Rules of Civil Procedure

- Source: https://www.vid.uscourts.gov/sites/vid/files/local_rules/260130_Local%20Rules%20Civil%20Procedure_effective%202.1.26[ECF%2010-1].pdf (official site; 45 pages, sha256 0d9d6129e7a85ce24f03f6dfd0805940c8d2faa8b1b990589081c085d2076e5a).
- Edition: Local Rules of Civil Procedure effective February 1, 2026 (with the Jan. 30, 2026 adopting order, Misc. No. 3:25-mc-0003, as page 1), as linked "Local Rules of Civil Procedure" on https://www.vid.uscourts.gov/court-info/local-rules-and-orders/local-rules (retrieved 2026-10-27).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 e5369189343fab8d7e3538fd223fdbf2d9ae49fc4f71d07588c6667fec3e262e. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `vid/original/local-rules-civ-2026-02-01.pdf` as served (sha256 0d9d6129e7a85ce24f03f6dfd0805940c8d2faa8b1b990589081c085d2076e5a); extraction sha256 e5369189343fab8d7e3538fd223fdbf2d9ae49fc4f71d07588c6667fec3e262e.

### `vid/vid-local-criminal-rules-2026-02-01.txt` — D.V.I. Local Rules of Criminal Procedure (effective February 1, 2026)

- Source: https://www.vid.uscourts.gov/sites/vid/files/local_rules/260130_Local%20Rules%20Criminal%20Procedure_effective%202.1.26%5BECF%2010-2%5D.pdf (official site; 11 pages, sha256 06db72fa21bfc64d469e4cc423ff8f27842bf56a083f39e694fef96622637b53).
- Edition: Local Rules of Criminal Procedure effective February 1, 2026, issued with the District Court's order of January 30, 2026 (Misc. No. 3:25-mc-0003, ECF 10-2), linked at https://www.vid.uscourts.gov/court-info/local-rules-and-orders/local-rules. robots.txt of www.vid.uscourts.gov read (Crawl-delay 10 honored where stated; no Disallow covers the fetched path); the judiciary-wide Privacy & Security Policy (https://www.uscourts.gov/privacy-security-policy) read: no prohibition of automated access. Fetched with curl and the identifying UA lit-lang-provenance/1.0, at most 1 request per 10 s.
- Criminal forum (lg-gddt): vid-criminal.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 cf0c5337ddf0d91d89b1c1618e3b3aa3f14930f709fb109eabc1d986ce434200. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `vid/original/vid-local-criminal-rules-2026-02-01.pdf` as served (sha256 06db72fa21bfc64d469e4cc423ff8f27842bf56a083f39e694fef96622637b53); extraction sha256 cf0c5337ddf0d91d89b1c1618e3b3aa3f14930f709fb109eabc1d986ce434200.

### `vid/local-rules-civ-2026-02-01-sectioned.txt` — D.V.I. Local Rules of Civil Procedure, sectioned

- Derived in the pod (lg-f3q8.76) from `vid/local-rules-civ-2026-02-01.txt` (extract sha256 e5369189343fab8d7e3538fd223fdbf2d9ae49fc4f71d07588c6667fec3e262e); nothing fetched. In the body (after each part's table of contents), one marker line `## LRCi <n>` is inserted before each rule's own heading line `Rule <n>. <title>` (LRCi 1.1 through LRCi 83.2, 54 rules). Every other line is the extract unchanged, so deleting the `## LRCi` lines (`sed '/^## LRCi /d'`) gives back the extract byte for byte.
- Upstream: https://www.vid.uscourts.gov/sites/vid/files/local_rules/260130_Local%20Rules%20Civil%20Procedure_effective%202.1.26[ECF%2010-1].pdf, retrieved 2026-10-27, as recorded on the `vid/local-rules-civ-2026-02-01.txt` entry this file is derived from.
- Edition: as the extract, "Effective February 1, 2026".
- Sectioned sha256 f681ae577068a35019bb2203e10d01e771feaa11789e66b8ad2f6019e7eb4843 (the `path_sha256` pinned by `lit/vid/Lit.toml`).

### `vid/vid-local-criminal-rules-2026-02-01-sectioned.txt` — D.V.I. Local Rules of Criminal Procedure, sectioned

- Derived in the pod (lg-gddt.10) from `vid/vid-local-criminal-rules-2026-02-01.txt` (extract sha256 cf0c5337ddf0d91d89b1c1618e3b3aa3f14930f709fb109eabc1d986ce434200); nothing fetched. In the body (after the table of contents), one marker line `## LRCr <n>` is inserted before each rule's own heading line `Rule <n>. <title>` (LRCr 1.1 through LRCr 59.1, 18 rules; Rule 1.4's heading has no period after the number). Every other line is the extract unchanged, so deleting the `## LRCr` lines (`sed '/^## LRCr /d'`) gives back the extract byte for byte.
- Edition: as the extract, "Effective February 1, 2026".
- Upstream: https://www.vid.uscourts.gov/sites/vid/files/local_rules/260130_Local%20Rules%20Criminal%20Procedure_effective%202.1.26%5BECF%2010-2%5D.pdf, retrieved 2026-10-28, as recorded on `vid/vid-local-criminal-rules-2026-02-01.txt` above; sectioning was done from the vendored extract.
- Sectioned sha256 f1dcf6ab0c89821e7ebaf12dda2b1f46e5b6b1b1165936037f628e1c91339b60 (the `path_sha256` pinned by `lit/vid-criminal/Lit.toml`).
