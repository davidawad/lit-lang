# L0 source corpus — provenance

These files are the **L0 layer** in `docs/ARCHITECTURE.md`'s tower: verbatim
plain-text extracts of primary law (rule texts, statutes, regulations), each
with a retrieval date and a sha256 recorded below so the extraction is
auditable. `litgraph lint` fuzzy-matches pack `cite`/`authority` strings (and
quoted phrases in `note`) against these files once a pack's `sources[].path`
points at one (see `docs/PACK_SCHEMA.md`'s authoring rule 1 and AGENTS.md's
pack-edit rule 4 for the mechanism; every pack's `sources[].path` is wired as
of this file's own entries below).

## Why vendor plain text instead of fetching at run/test time

- **Public domain.** Every document here is a U.S. government work (court
  rules prescribed under 28 U.S.C. § 2072, the U.S. Code, the Code of Federal
  Regulations) — no copyright restriction on redistributing the text.
- **Small where it is embedded.** The binary embeds only the `.txt`
  extractions, the same way `packs/*.json` is. The as-served originals sit
  beside them in `original/` (see "Originals" below) as plain git blobs,
  and are never embedded.
- **Deterministic, offline tests.** `cargo nextest run` must not need network
  access (repo quality bar). A `litgraph sources fetch` command hitting
  live .gov/Cornell endpoints in CI would be slow and flaky (rate limits,
  layout changes, outages) for content that changes on the order of years,
  not days. Vendoring converts "fetch reliability" into an ordinary content
  review, the same way `packs/*.json` themselves are reviewed.
- **Matches how packs are already authored.** `packs/cafc-SOURCES-NOTES.md`
  and `packs/cofc-SOURCES-NOTES.md` document a `pdftotext -layout` +
  sha256 authoring pass already; this corpus is the same pattern, just
  checked into the repo instead of staying in a private KB so cite
  verification can run in CI.

Re-deriving any file here (to refresh after a rule amendment) means
re-running the extraction described in its entry below and diffing the
result — that is the "reproducible" bar, not a live network call at lint
time.

## D.C. Circuit Rules and Handbook

### `cadc-circuit-rules-2025-12-04.txt`

- Official D.C. Circuit combined Circuit Rules and Federal Rules of Appellate
  Procedure, Circuit Rules amended through Dec. 4, 2025, and FRAP through
  Dec. 1, 2025:
  <https://www.cadc.uscourts.gov/sites/cadc/files/rules-RulesFRAP20251204.pdf>
- Retrieved 2026-10-27. Extracted with `pdftotext -layout`; form-feed page
  separators were normalized to newlines and `## Rule N` / `## Circuit Rule
  N` markers were added for cite resolution. Rule text was not edited.
- Original PDF sha256:
  `182a2597fb2d30fcbc69f5ce7c4146ac8f716a72ee4476d844354fbd30c9f4cf`.
- Vendored text sha256:
  `9a5eb420c77590c7a918df6b344e427293584cef13af7f7db35bc0200746729e`.
- Original: `original/cadc-circuit-rules-2025-12-04.pdf` as served (sha256 182a2597fb2d30fcbc69f5ce7c4146ac8f716a72ee4476d844354fbd30c9f4cf); extraction sha256 9a5eb420c77590c7a918df6b344e427293584cef13af7f7db35bc0200746729e. Served at https://www.cadc.uscourts.gov/sites/cadc/files/rules-RulesFRAP20251204.pdf; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.

### `cadc-handbook-2026-09-17.txt`

- Official D.C. Circuit Handbook of Practice and Internal Procedures:
  <https://www.cadc.uscourts.gov/sites/cadc/files/rules-Handbook20260917.pdf>
- Retrieved 2026-10-27. Extracted with `pdftotext -layout`.
- Original PDF sha256:
  `1fd1eebbba92611c2b99ae08d5078b841ae6b9a76eef99ced920fa9bc9bb4d5e`.
- Vendored text sha256:
  `e541d181253bd50730add3b957ecb760cbb537ae3eb6020aef842fd896fe60d1`.
- Original: `original/cadc-handbook-2026-09-17.pdf` as served (sha256 1fd1eebbba92611c2b99ae08d5078b841ae6b9a76eef99ced920fa9bc9bb4d5e); extraction sha256 e541d181253bd50730add3b957ecb760cbb537ae3eb6020aef842fd896fe60d1. Served at https://www.cadc.uscourts.gov/sites/cadc/files/rules-Handbook20260917.pdf; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.

## Format

Each `.txt` file is one or more sections, each starting at column 0 with
`## <canonical ref>` (matching the ref strings `litigate_engine::cite::normalize`
produces — see `crates/litigate-engine/src/cite/normalize.rs`), optionally followed
by a heading line, then the verbatim body text up to the next `## ` line.

## Originals

Every vendored extraction keeps the bytes the court or agency served
(lg-f3q8.80): `<dir>/original/<extraction stem>.<pdf|html|docx>`, as
plain git blobs (no LFS, lg-f3q8.85). The extraction's entry records it
with one line, ``- Original: `<path>` as served (sha256 <original>);
extraction sha256 <extraction>.`` Paths are relative to `sources/`.
`scripts/ci-scoped provenance` checks an added original against that line.
Entries live in the forum's own
`sources/<forum>/PROVENANCE.md`; this file records top-level files only.

## Files

### `frcp.txt` — Federal Rules of Civil Procedure (112 rules)

- Ported from `civ-pro-the-gathering/content/reference/frcp-full.json`
  (sibling project, same author, also public-domain rule text with its own
  provenance block: "official 2024 committee print... extracted locally
  (pdftotext) from David's library copy", retrieved 2026-07-26, canonical
  `https://www.uscourts.gov/rules-policies/current-rules-practice-procedure`).
  Re-formatted into this file's `## Rule N` section convention; no wording
  changed.
- sha256 (this file): `cb9a19a0ad99acdbd04f46acd3893bb96fbd34c9a8f66de57f5d171dbd268b44`
- Original: `original/frcp.pdf` as served (sha256 bd8705fc038d87e4fe222a7ea2e4324222c9430e2373fce56826bd2dfa2f8baf); extraction sha256 cb9a19a0ad99acdbd04f46acd3893bb96fbd34c9a8f66de57f5d171dbd268b44. Served at https://www.uscourts.gov/sites/default/files/document/federal-rules-of-civil-procedure.pdf; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27. CURRENCY: served document differs from the vendored extraction (sha256 bd8705fc038d87e4fe222a7ea2e4324222c9430e2373fce56826bd2dfa2f8baf); re-vendor pending.
- Comparison (lg-f3q8.82, 2026-10-27): the served file is the House Judiciary Committee print "Federal Rules of Civil Procedure, December 1, 2025" (142 pages). Each of the 97 vendored rules was compared word for word with the print's text (page headers, title lines and GPO hyphenation ignored). The vendored text predates the amendments effective December 1, 2025 (order of April 23, 2025, which "affected Rules 16 and 26, and added new Rule 16.1"). Differences: (1) Rule 16(b)(3)(B)(iv) gains the words "the timing and method for complying with Rule 26(b)(5)(A) and" before "any agreements the parties reach for asserting claims of privilege or of protection as trial-preparation material after information is produced, including agreements reached under Federal Rule of Evidence 502"; (2) Rule 26(f)(3)(D) gains the same words ("...method for complying with Rule 26(b)(5)(A) and—if the parties agree on a procedure..."); (3) Rule 16.1 (Multidistrict Litigation) is absent from frcp.txt; (4) the amendment credits of Rules 16 and 26 gain "Apr. 23, 2025, eff. Dec. 1, 2025" and Rule 16.1's reads "As added Apr. 23, 2025, eff. Dec. 1, 2025". No other rule's wording differs. frcp.txt is not edited here.

### `frap.txt` — Federal Rules of Appellate Procedure (55 rules)

- Ported from `civ-pro-the-gathering/content/reference/frap-full.json`,
  whose provenance records each rule "extracted from the Cornell LII...
  pages and then verified line by line against the official Administrative
  Office edition (uscourts.gov, December 1, 2024)" (Rules 6/39 verified
  against the Apr. 23, 2025 Supreme Court amendment order instead),
  retrieved 2026-08-15.
- sha256 (this file): `b370cc0558698ff88b62b6cec212742fde8afbdd2fe653a3b255d69dec893172`
- Original: `original/frap.pdf` as served (sha256 a6f417607a71c05a3a54df68a43ec67e691dcf276c46a415e78038ae4f9305f4); extraction sha256 b370cc0558698ff88b62b6cec212742fde8afbdd2fe653a3b255d69dec893172. Served at https://www.uscourts.gov/sites/default/files/document/federal-rules-of-appellate-procedure.pdf; same bytes as frap/original/frap.pdf (the House Judiciary Committee print of December 1, 2025, fetched 2026-10-27), copied not refetched; retrieved 2026-10-27.
- Comparison (lg-f3q8.82, 2026-10-27): frap.txt compared rule by rule with the OCR extract of that print (sources/frap/frap.txt; the PDF has no usable text layer, so the check is only as good as the OCR). 52 of 55 rules located; running heads, amendment credits and OCR fragments aside, no wording difference was found. FRAP 3.1, 5.1 and 35 (abrogated or transferred stubs) were not compared. One defect in frap.txt itself, not a currency change: FRAP 42 stops after "(b) Dismissal in the Court of Appeals." and omits the text of 42(b) that the print has ("(1) Stipulated Dismissal. The circuit clerk must dismiss a docketed appeal if the parties file a signed dismissal agreement specifying how costs are to be paid and pay any court fees that are due. (2) Appellant's Motion to Dismiss. ..."). frap.txt is not edited here.
- Repair (lg-f3q8.84, 2026-10-27): FRAP 42(b)(1)–(3), (c) and (d) restored verbatim from that print (printed p. 48, PDF page 63), read from the OCR extract sources/frap/frap.txt and checked against the page image; straight apostrophes kept to match the rest of this file. No other rule was changed.

### `usc-cfr-prosecution-fragments.txt` — 28 U.S.C. (4 sections), 35 U.S.C.
(7 sections), 37 C.F.R. Part 1 prosecution rules (17 sections)

- Ported from `civ-pro-the-gathering/content/reference/uscfr-full.json`.
  35 U.S.C./37 C.F.R. text there was "extracted locally from the USPTO
  Consolidated Patent Laws/Rules (July 2025)"; the 28 U.S.C. sections were
  "appended from uscode.house.gov (official)" and, in the source JSON,
  embedded a full uscode.house.gov page dump (edition-history sidebar,
  nav chrome) around the actual statute text. This port strips that
  boilerplate mechanically: keeps only the span from the `§NNNN. Title`
  heading line to the `[Excerpt ...]` marker the JSON's own author added.
- sha256 (this file, post-cleanup): `5bbba22c27cd4f0957c799a8a1f48b6ba9d0670be03a9fec796a3c8ec394c1b3`
- Original: none as served (assembled from a port of the sibling project's uscfr-full.json (USPTO Consolidated Patent Laws/Rules, July 2025, and uscode.house.gov pages for the 28 U.S.C. sections)); see the sources listed above.

### `usc-19-1337.txt` — 19 U.S.C. § 1337 (ITC unfair-import-practices statute)

- Fetched directly 2026-09-28 from `uscode.house.gov`
  (`https://uscode.house.gov/view.xhtml?req=(title:19%20section:1337%20edition:prelim)`,
  official U.S. House Office of the Law Revision Counsel text), full text
  including subsections (a)-(n). HTML chrome/nav stripped mechanically
  (kept the span from the `§1337.` heading to the `Editorial Notes`
  marker), tags stripped, entities decoded.
- sha256 (this file): `3865d7c3c1b4d9c0883d3eb29fd160c3c789d8cb8ffed675311e5dccd0892408`
- Original source (lg-f3q8.82, 2026-10-27): uscode.house.gov now serves its pages in a temporary format and the `view.xhtml?req=(...)` URLs 404, so the originals were fetched from the same Code at its other official host, GPO's govinfo (`https://www.govinfo.gov/link/uscode/<title>/<section>?link-type=html`, a 302 to the granule of United States Code, 2024 Edition, current through January 6, 2025). Verified word for word against this extraction: the section text is identical (apart from GPO end-of-line hyphenation and this file's added title line).
- Original: `original/usc-19-1337.html` as served (sha256 31b4ab0dd3b107a3e1086c6bf6e24c026e540357f499ab7c8f20d162b22ee034); extraction sha256 3865d7c3c1b4d9c0883d3eb29fd160c3c789d8cb8ffed675311e5dccd0892408. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title19/html/USCODE-2024-title19-chap4-subtitleII-partII-sec1337.htm (via https://www.govinfo.gov/link/uscode/19/1337?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.

### `cfr-37-part42.txt` — 37 C.F.R. Part 42 (PTAB trial practice, 84 sections)

- Fetched directly 2026-09-28 from `www.govinfo.gov`'s official annual CFR
  XML (`CFR-2024-title37-vol1-part42.xml`, title 37 "Patents, Trademarks,
  and Copyrights", 2024-07-01 edition), the authoritative machine-readable
  form GPO publishes for the printed CFR. Parsed with Python's
  `xml.etree.ElementTree`: every `<SECTION>` becomes one `## 37 C.F.R. §
  N` block from its `<SECTNO>`/`<SUBJECT>`/`<P>` children, whitespace
  collapsed, tags stripped. Covers `§ 42.108` (institution of *inter
  partes* review) cited as the worked example in this bead's brief.
- sha256 (this file): `06786d129e12353754288989ddad0ef9866bb068eb0a7a35e25465b65276e4a1`
- Original: `original/cfr-37-part42.xml` as served (sha256 53d2ae9852561c67f0707b4fa1e2a62a576d9d77ad986aa2a05a107c2f1e66fa); extraction sha256 06786d129e12353754288989ddad0ef9866bb068eb0a7a35e25465b65276e4a1. Served at https://www.govinfo.gov/content/pkg/CFR-2024-title37-vol1/xml/CFR-2024-title37-vol1-part42.xml; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.

### `rcfc.txt` — Rules of the U.S. Court of Federal Claims (112 rules, RCFC 1-87)

- The RCFC main-rules PDF is the same one `packs/cofc-court-of-federal-claims.json`'s
  `rcfc-rules` source pinned (`Rules%207.27.2026.pdf`, as amended through
  July 27, 2026; sha256 `ff02c342f055be5e5f51df4de1dcec3c1264bb9907895d379d5baa796ec30a97`).
  That URL now 404s; the court serves the compiled rules as
  `https://www.uscfc.uscourts.gov/sites/cfc/files/Rules%2010.08.26.pdf`
  (same bytes as `original/rcfc-appendices.pdf`, fetched 2026-10-24).
  Re-vendored 2026-10-27 from that file (lg-f3q8.82): same extraction method,
  and the 122 sections' wording is unchanged from the July 27 text (only
  blank-line spacing differs); RCFC 86 no longer carries the stray start of
  Appendix A that the old window cut included.
  `pdftotext` (no `-layout`; the two-column committee-notes sidebar in
  `-layout` mode interleaves with rule text mid-sentence, `pdftotext`'s
  content-stream reading order does not) against the main-rules window only
  (`Rule 1. Scope and Purpose` through `Rule 87`, before Appendix A/C/H/J and
  the Vaccine/Patent Rules — which restart their own `Rule 1..N` numbering
  under headings this corpus does not distinguish from the main rules — begin).
  Split into `## RCFC N` sections on a `^Rule N\.` line start (stripping the
  form-feed byte `pdftotext` emits at the top of each page first — several
  rules that happen to start a page, including RCFC 41, were silently
  dropped before this fix since the form-feed sat before `Rule` at column
  0). Committee-notes prose sometimes bleeds into the tail of the preceding
  rule's body (a known, accepted imprecision — it only affects optional
  quoted-phrase fuzzy matching, never whether a citation resolves at all).
- sha256 (this file): `2cd0ad815721d3234273142db6f8b32e5235d395c98273f94a51a2e8d2e81be9`
- Original: `original/rcfc.pdf` as served (sha256 943733526f5391cad22562adf31411148fecfb03f01ae7bcf8bd6ec051a0c971); extraction sha256 2cd0ad815721d3234273142db6f8b32e5235d395c98273f94a51a2e8d2e81be9. Served at https://www.uscfc.uscourts.gov/sites/cfc/files/Rules%2010.08.26.pdf; same bytes as original/rcfc-appendices.pdf (fetched 2026-10-24), copied not refetched; retrieved 2026-10-27.

### `fedcir-rules.txt` — Federal Circuit Rules of Practice (55 numbered local rules)

- `FederalCircuitRulesofPractice.pdf` (`www.cafc.uscourts.gov`) as served
  2026-10-27, edition "October 6, 2026". The pack's `fedcir-rules` source and
  the first vendoring pinned the December 1, 2025 edition (sha256
  `b6346ed56a0ef66e8a681765873caf69239cda273f8946f1625571e013712c05`); the
  court overwrote the file at the same URL. Re-vendored 2026-10-27
  (lg-f3q8.82) with the same method; the wording of all 55 sections is
  unchanged except that one paragraph of Fed. Cir. R. 29 ("If the appendix is
  filed in two versions pursuant to Fed. Cir. R. 25.1(e)(1), the confidential
  version must ...") now falls under Fed. Cir. R. 30, where this edition's
  page order places it. `pdftotext` (no `-layout`). This PDF
  combines FRAP-numbered local rules, Attorney Discipline Rules, and
  practice notes in one document; only the `FEDERAL CIRCUIT RULE N` running
  page headers are extracted (Attorney Discipline Rules and IOPs are a
  separate numbering scheme no pack currently cites). A rule spanning
  several pages repeats its own running header each page; those repeats are
  merged into one section rather than treated as a new one.
- sha256 (this file): `96a9d0aa7276d24383c00a5c3c6c107f5359bff6e7bf926907e6c28748dbe878`
- Original: `original/fedcir-rules.pdf` as served (sha256 8355ba375177b344e90da55a4fe720b2b90279c248e8a089b30a8b564a4e5b9b); extraction sha256 96a9d0aa7276d24383c00a5c3c6c107f5359bff6e7bf926907e6c28748dbe878. Served at https://www.cafc.uscourts.gov/wp-content/uploads/RulesProceduresAndForms/FederalCircuitRules/FederalCircuitRulesofPractice.pdf; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.

### `cfr-19-part210.txt` — 19 C.F.R. Part 210 (USITC Rules of Adjudication and Enforcement, 79 sections)

- Fetched 2026-09-29 from `www.govinfo.gov`'s official annual CFR XML
  (`CFR-2024-title19-vol3-part210.xml`, title 19 "Customs Duties", chapter
  II "United States International Trade Commission", 2024-04-01 edition),
  same method and same `<SECTION>`/`<SECTNO>`/`<SUBJECT>`/`<P>` parse as
  `cfr-37-part42.txt` below. Resolves `itc-337.json`'s internal
  `cfr210.N.x` shorthand (see `crates/litigate-engine/src/cite/normalize.rs`'s
  `try_itc_cfr_slug`), which needs the pack's `forum: "itc"` field to
  disambiguate — set as part of this same pass.
- sha256 (this file): `92d29ab6a95dc52eaaa5b4fa5b1d0991131546e01612bc83f8e365f40420c99e`
- Original: `original/cfr-19-part210.xml` as served (sha256 f97b7e87df3d5a56bd1afba06ab4d5e2c172ab50d24f8bf9209fef00ae5a8c6e); extraction sha256 92d29ab6a95dc52eaaa5b4fa5b1d0991131546e01612bc83f8e365f40420c99e. Served at https://www.govinfo.gov/content/pkg/CFR-2024-title19-vol3/xml/CFR-2024-title19-vol3-part210.xml; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.

### `cfr-37-part1.txt` — 37 C.F.R. Part 1 (Rules of Practice in Patent Cases, all 363 sections)

- Fetched 2026-09-29 from `www.govinfo.gov` (`CFR-2024-title37-vol1-part1.xml`,
  2024-07-01 edition), same method as `cfr-37-part42.txt`. Supersedes the
  17-section Part 1 subset already embedded piecemeal in
  `usc-cfr-prosecution-fragments.txt` (both are kept; a citation resolves
  against whichever file a given pack's `sources[].path` lists, so there is
  no conflict — see that test in `corpus.rs`, `duplicate_headings_across_files_both_appear`).
- sha256 (this file): `e5765fec0e8c5f295d4f230ed9743b7972dbd41340693833abf990fe1918d1ec`
- Original: `original/cfr-37-part1.xml` as served (sha256 7e024b8a8d1f7b4b32939771ec577d62e5b4dace4c40ca0f8b92f9d8c6f76637); extraction sha256 e5765fec0e8c5f295d4f230ed9743b7972dbd41340693833abf990fe1918d1ec. Served at https://www.govinfo.gov/content/pkg/CFR-2024-title37-vol1/xml/CFR-2024-title37-vol1-part1.xml; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.

### `cfr-37-part41.txt` — 37 C.F.R. Part 41 (Board of Patent Appeals and Interferences / PTAB ex parte appeal practice, 75 sections)

- Fetched 2026-09-29 from `www.govinfo.gov` (`CFR-2024-title37-vol1-part41.xml`,
  2024-07-01 edition), same method as `cfr-37-part42.txt`/`cfr-19-part210.txt`.
  Resolves `mpep-prosecution.json`'s citations into the examiner's-answer/
  appeal-to-PTAB region (§§ 41.20, 41.45).
- sha256 (this file): `912ba8c173b834a4030c979c0ff3bc6f96d393dcff717c6d14812daba23934ee`
- Original: `original/cfr-37-part41.xml` as served (sha256 39fabbd0edde889c0a3a50bbf3b2f2f6ecd03ad1c752a6f99c15edd0211050ec); extraction sha256 912ba8c173b834a4030c979c0ff3bc6f96d393dcff717c6d14812daba23934ee. Served at https://www.govinfo.gov/content/pkg/CFR-2024-title37-vol1/xml/CFR-2024-title37-vol1-part41.xml; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.

### `frcrimp.txt` — Federal Rules of Criminal Procedure (62 rules)

- Fetched 2026-09-29 from `www.uscourts.gov`'s current-rules page
  (`federal-rules-of-criminal-procedure-dec-1-2024_0.pdf`, as amended to
  Dec. 1, 2024 — the same PDF `packs/frcrimp-criminal-procedure.json`'s
  `frcrimp-rules` source pins; sha256
  `c89161b8a258b62af4d5298de832c43bd72f2eea1104c7625a5b561658e5875c`
  confirmed identical). `pdftotext` (no `-layout`); this PDF has no
  appendices, so the whole body after the table of contents is main-rules
  text. Split on `^Rule N\.` line starts (form-feed byte stripped first,
  same fix as `rcfc.txt` above), page running headers/footers stripped
  mechanically.
- sha256 (this file): `cf92787ff6bf24cb4d853cb347925f5f37dba41c1bc1cac26da0ffcfb6cf19ed`
- Original: `original/frcrimp.pdf` as served (sha256 c89161b8a258b62af4d5298de832c43bd72f2eea1104c7625a5b561658e5875c); extraction sha256 cf92787ff6bf24cb4d853cb347925f5f37dba41c1bc1cac26da0ffcfb6cf19ed. Served at https://www.uscourts.gov/sites/default/files/2025-02/federal-rules-of-criminal-procedure-dec-1-2024_0.pdf; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.

### `mpep.txt` — MPEP chapters 200 (partial), 600, 700, 800 (partial), 1200, 1300, 1400 (partial), 2200 (partial) — 466 sections

- Chapters 600/700/1200/1300 (456 sections) ported from
  `civ-pro-the-gathering/content/reference/mpep-sections.json` (sibling
  project; that file's own provenance: "Extracted locally from David's
  library conversion of the official USPTO MPEP", canonical
  `https://www.uspto.gov/web/offices/pac/mpep/`, retrieved 2026-07-26),
  reformatted into this file's `## MPEP § N` convention (that JSON's own
  `ref` field, e.g. `"MPEP 601"`, becomes heading `"MPEP § 601"` to match
  `normalize.rs`'s `Family::Mpep` heading form) — no wording changed.
- §§ 1401/1412/1460 (chapter 1400, reissue) and §§ 2209/2249/2254 (chapter
  2200, ex parte reexam) — 6 sections `mpep-prosecution.json` cites that the
  sibling repo's JSON doesn't cover — fetched 2026-09-29 directly from
  `www.uspto.gov` (`mpep-1400.pdf`, `mpep-2200.pdf`, 9th ed. Rev. 01.2024).
- §§ 201.06/201.07 (chapter 200) and §§ 818.01/821.04 (chapter 800) — 4 more
  sections the same pack cites — fetched the same way from `mpep-0200.pdf`/
  `mpep-0800.pdf`.
- All 10 of the above: `pdftotext -layout`, sections located by line search
  and extracted as bounded windows around each heading. Lower extraction
  precision than the JSON-ported chapters (the two-column committee-note-
  style sidebar this edition uses bleeds into the window at points, and one
  section — MPEP § 803 — could not be cleanly isolated this way at all, so
  it stays unvendored; see "Deliberately not vendored yet" below); the
  imprecision is acceptable for the same reason as `rcfc.txt` above — it
  only affects optional quote-fuzzy-matching, never whether a citation
  resolves at all.
- Two `mpep-prosecution.json` citations (`MPEP 818.03`, `MPEP 818.03(a)`)
  were corrected to `MPEP 818.01`/`MPEP 818.01(d)` in the same pass: § 818.03
  does not exist in the current (9th ed., Rev. 01.2024) MPEP — confirmed by
  reading the actual chapter 800 text end to end looking for it — and the
  content those two edges describe (electing without/with traverse of a
  restriction requirement) is exactly § 818.01's own subject matter (which
  does exist, with an `(d) Traverse of Restriction` subsection).
- sha256 (this file): `110b56e793d244d0f16d1f9f00afe18a1f3d6fe2338daba8e000fc297fe3d524`
- Original: none as served (assembled from a port of the sibling project's mpep-sections.json plus windows cut with pdftotext -layout from the USPTO MPEP chapter PDFs mpep-0200, mpep-0800, mpep-1400 and mpep-2200); see the sources listed above.

### `usc-additional.txt` — 57 more 28/35/18/5 U.S.C. sections

- The sections every pack's `sources` cite beyond the four 28 U.S.C. and
  seven 35 U.S.C. sections `usc-cfr-prosecution-fragments.txt` already had:
  28 U.S.C. §§ 46, 1254, 1291, 1295, 1331, 1332, 1391, 1491, 1498, 1500,
  1914, 2101, 2106, 2107, 2412, 2501, 2522; 35 U.S.C. §§ 6, 120, 121, 134,
  142, 143, 144, 154, 251, 286, 302-306, 307, 311, 312, 313-319, 321, 322,
  323-328; 18 U.S.C. §§ 3142, 3161, 3162, 3282, 3501, 3552, 3553; 5 U.S.C.
  §§ 556, 706. Fetched 2026-09-29 directly from
  `uscode.house.gov`'s per-section XHTML view (official U.S. House Office
  of the Law Revision Counsel text, `view.xhtml?req=(title:T section:S
  edition:prelim)`), the same official source `usc-19-1337.txt` used.
  Extracted mechanically between that page's own `<!-- field-start:head
  -->`/`<!-- field-end:head -->` and `<!-- field-start:statute -->`/`<!--
  field-end:statute -->` HTML comment markers (a machine-readable
  boundary the page itself provides, cleaner than the boilerplate-stripping
  `usc-cfr-prosecution-fragments.txt` needed), tags stripped, entities
  decoded.
- sha256 (this file): `5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8`
- Original source (lg-f3q8.82, 2026-10-27): uscode.house.gov now serves its pages in a temporary format and the `view.xhtml?req=(...)` URLs 404, so the originals were fetched from the same Code at its other official host, GPO's govinfo (`https://www.govinfo.gov/link/uscode/<title>/<section>?link-type=html`, a 302 to the granule of United States Code, 2024 Edition, current through January 6, 2025). Verified word for word against this extraction: all 57 sections are identical (apart from GPO end-of-line hyphenation).
- Original: `original/usc-additional/001-28-usc-46.html` as served (sha256 6c4b3ac69ad43d525dfa3ffd1940f535f1568e0311a1b25616976f6112eabb77); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-partI-chap3-sec46.htm (via https://www.govinfo.gov/link/uscode/28/46?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/002-28-usc-1254.html` as served (sha256 a971f37f0ba0073816ffa573cea652e5881a0ce9b1249f5f31024a423cb2be58); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-partIV-chap81-sec1254.htm (via https://www.govinfo.gov/link/uscode/28/1254?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/003-28-usc-1291.html` as served (sha256 62cef84db60bd8e9a3beae2e40da5d3f67849b4237f6f6965fadf05f26477b6e); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-partIV-chap83-sec1291.htm (via https://www.govinfo.gov/link/uscode/28/1291?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/004-28-usc-1295.html` as served (sha256 7abaf36b5a24b9153d95004108c6549fd8b65fc3357610a843620169c11ccc18); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-partIV-chap83-sec1295.htm (via https://www.govinfo.gov/link/uscode/28/1295?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/005-28-usc-1331.html` as served (sha256 11696c46e5ce7524c7d16d80e986bae97968d7eb1f412e31c01011a637dcb865); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-partIV-chap85-sec1331.htm (via https://www.govinfo.gov/link/uscode/28/1331?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/006-28-usc-1491.html` as served (sha256 8b5f335e67c60228364601f9a8d32871c047f70669d891cc5d8564367b70a676); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-partIV-chap91-sec1491.htm (via https://www.govinfo.gov/link/uscode/28/1491?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/007-28-usc-1498.html` as served (sha256 5c29b3ece2dd609c1d57555cf91524ce750e0a335d6cda4fc56e8ccddcab5899); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-partIV-chap91-sec1498.htm (via https://www.govinfo.gov/link/uscode/28/1498?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/008-28-usc-1500.html` as served (sha256 7a0f217874b683126d0410d21462ba6e22e8aaa5cd5944f14defeaeb266f3f2a); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-partIV-chap91-sec1500.htm (via https://www.govinfo.gov/link/uscode/28/1500?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/009-28-usc-1914.html` as served (sha256 326e048a248340388906815e26bae6a6964c38dd4c03df89da95f82e4c60a942); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-partV-chap123-sec1914.htm (via https://www.govinfo.gov/link/uscode/28/1914?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/010-28-usc-2101.html` as served (sha256 dbde265b0f16b65306900a4449ed5a87379055cd5203700fec38417e36ba8b50); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-partV-chap133-sec2101.htm (via https://www.govinfo.gov/link/uscode/28/2101?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/011-28-usc-2106.html` as served (sha256 1bd177319b551d52c6b6458c26d0af30c06a7c808d197385fc5dab5457fa7a9b); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-partV-chap133-sec2106.htm (via https://www.govinfo.gov/link/uscode/28/2106?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/012-28-usc-2107.html` as served (sha256 d92f602aa1f864a8aac95d37026aa09e86f8b5fa51c07e36bf4969d9da2f1cee); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-partV-chap133-sec2107.htm (via https://www.govinfo.gov/link/uscode/28/2107?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/013-28-usc-2412.html` as served (sha256 790720821a2482f459ffcdc71806083ce9bf05c77f42ada5be8b8b0780ec9612); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-partVI-chap161-sec2412.htm (via https://www.govinfo.gov/link/uscode/28/2412?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/014-28-usc-2501.html` as served (sha256 0284eeb38d502157c1e41d38d558127a62997284a2f9b94b6efac61855cc6f72); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-partVI-chap165-sec2501.htm (via https://www.govinfo.gov/link/uscode/28/2501?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/015-28-usc-2522.html` as served (sha256 d222585f1ce03ab9b730c9aba46a4f2c925f8bc9a135082cb6cf427324e5180a); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-partVI-chap165-sec2522.htm (via https://www.govinfo.gov/link/uscode/28/2522?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/016-35-usc-6.html` as served (sha256 5c0c54cf1bdc91cf4e4362f7abaa30c0938b31ad7c920804b8a8d06a2d18d07e); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title35/html/USCODE-2024-title35-partI-chap1-sec6.htm (via https://www.govinfo.gov/link/uscode/35/6?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/017-35-usc-120.html` as served (sha256 14d946cbc42fc37f77cd60d97997e78fd16d638a0b6fc683036d8684ebc9c062); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title35/html/USCODE-2024-title35-partII-chap11-sec120.htm (via https://www.govinfo.gov/link/uscode/35/120?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/018-35-usc-121.html` as served (sha256 fcfb9a4dfb59c82917f6893342bb0c2de24dfb7b035afc62676d9b4809682fae); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title35/html/USCODE-2024-title35-partII-chap11-sec121.htm (via https://www.govinfo.gov/link/uscode/35/121?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/019-35-usc-142.html` as served (sha256 be5c62ef8c5ceca26f2df22173e2361b141b6056427180f42ae03a36fb103fd9); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title35/html/USCODE-2024-title35-partII-chap13-sec142.htm (via https://www.govinfo.gov/link/uscode/35/142?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/020-35-usc-251.html` as served (sha256 f169cd614202412bb23de050ed650da92ee3182791c098bfe9fd1b166444e30e); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title35/html/USCODE-2024-title35-partIII-chap25-sec251.htm (via https://www.govinfo.gov/link/uscode/35/251?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/021-35-usc-286.html` as served (sha256 c2133d8c1ba6a831ca5544fcf2be6a908643b80aab0f110d530c5fcf63f1c1be); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title35/html/USCODE-2024-title35-partIII-chap29-sec286.htm (via https://www.govinfo.gov/link/uscode/35/286?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/022-35-usc-302.html` as served (sha256 dd247e101b673163cf0ae07b8859767e7ab28448d33abf261c344e5f3bde20d1); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title35/html/USCODE-2024-title35-partIII-chap30-sec302.htm (via https://www.govinfo.gov/link/uscode/35/302?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/023-35-usc-303.html` as served (sha256 224e73abc74ed6f584554e715f1bd4661aec250e206e9b5f58d11dec4f900c88); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title35/html/USCODE-2024-title35-partIII-chap30-sec303.htm (via https://www.govinfo.gov/link/uscode/35/303?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/024-35-usc-304.html` as served (sha256 23e726e5ecbb3bb43e7d4cc4983ee0bb58d80999993ee3adfcc5d734004a752d); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title35/html/USCODE-2024-title35-partIII-chap30-sec304.htm (via https://www.govinfo.gov/link/uscode/35/304?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/025-35-usc-305.html` as served (sha256 2ac119068e8a222a3f524b20f0417e4541e5a7668459fe96ab071f5cd8e9e658); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title35/html/USCODE-2024-title35-partIII-chap30-sec305.htm (via https://www.govinfo.gov/link/uscode/35/305?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/026-35-usc-307.html` as served (sha256 6aec983c95ea93e7d23715514e22a0938b1ac4fbf509b2877aab0c45baa71d3a); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title35/html/USCODE-2024-title35-partIII-chap30-sec307.htm (via https://www.govinfo.gov/link/uscode/35/307?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/027-35-usc-311.html` as served (sha256 22ed6dab3ad7419a36afdb85ea7f57de55222438c9d16add829aaa4071a9f8f8); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title35/html/USCODE-2024-title35-partIII-chap31-sec311.htm (via https://www.govinfo.gov/link/uscode/35/311?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/028-35-usc-313.html` as served (sha256 5dce83147e9dc87c9aa72e9773391c0aa531160172663c5448a930b341a7a382); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title35/html/USCODE-2024-title35-partIII-chap31-sec313.htm (via https://www.govinfo.gov/link/uscode/35/313?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/029-35-usc-314.html` as served (sha256 94665f33a3b8c84633c36680dffda0d1c2c2b6cf19e63025d9ff3848e27bc323); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title35/html/USCODE-2024-title35-partIII-chap31-sec314.htm (via https://www.govinfo.gov/link/uscode/35/314?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/030-35-usc-315.html` as served (sha256 65e3a8e65da2d2d6d3347b9b8f1fa2b9d3ae3610e252bf4f598438d48f910ef0); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title35/html/USCODE-2024-title35-partIII-chap31-sec315.htm (via https://www.govinfo.gov/link/uscode/35/315?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/031-35-usc-316.html` as served (sha256 3ce7ebf06e8439b98d8a9cfc6653eea07090f33f0463d2889e0b7a64a1beaaa5); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title35/html/USCODE-2024-title35-partIII-chap31-sec316.htm (via https://www.govinfo.gov/link/uscode/35/316?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/032-35-usc-317.html` as served (sha256 38b65f172a04c87e66551c31bbdb6eab3cc182c8c4897f47befad2d1847c49c2); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title35/html/USCODE-2024-title35-partIII-chap31-sec317.htm (via https://www.govinfo.gov/link/uscode/35/317?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/033-35-usc-318.html` as served (sha256 65df0c1d90ae8ba7ea2b92a9f339040af7b467e43997ffdb4327ac0429c08fa4); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title35/html/USCODE-2024-title35-partIII-chap31-sec318.htm (via https://www.govinfo.gov/link/uscode/35/318?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/034-35-usc-319.html` as served (sha256 72bc4e79228b4579966d0813cf62576ce049a0a3f56f218cc3fca9c27b6ccda8); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title35/html/USCODE-2024-title35-partIII-chap31-sec319.htm (via https://www.govinfo.gov/link/uscode/35/319?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/035-35-usc-321.html` as served (sha256 a357029fb866a390d33aff8b84ec30836e27526a93cbc5acd2f15f384b668a8d); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title35/html/USCODE-2024-title35-partIII-chap32-sec321.htm (via https://www.govinfo.gov/link/uscode/35/321?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/036-35-usc-323.html` as served (sha256 00dbd9cc3a8ad534d42257fd8e071db931f810abe3f55e448e5d37d7e1b000cc); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title35/html/USCODE-2024-title35-partIII-chap32-sec323.htm (via https://www.govinfo.gov/link/uscode/35/323?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/037-35-usc-324.html` as served (sha256 fddb89c53c1debd49912d070aba5740e71dbf8adb1843958bf6ecd95b6c35d78); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title35/html/USCODE-2024-title35-partIII-chap32-sec324.htm (via https://www.govinfo.gov/link/uscode/35/324?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/038-35-usc-325.html` as served (sha256 301ea7f25d6337042d7a7a171d969afb3a349379b2ac31c3f5abb14350f56931); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title35/html/USCODE-2024-title35-partIII-chap32-sec325.htm (via https://www.govinfo.gov/link/uscode/35/325?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/039-35-usc-326.html` as served (sha256 5716b681d8605103971d72f031c02cc336c7e3c19dedf632b80e97fd7367c468); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title35/html/USCODE-2024-title35-partIII-chap32-sec326.htm (via https://www.govinfo.gov/link/uscode/35/326?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/040-35-usc-327.html` as served (sha256 0cd8b0d3c23c740383326f9be77a4efad4be6022eb9126df52f6fde1ec064444); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title35/html/USCODE-2024-title35-partIII-chap32-sec327.htm (via https://www.govinfo.gov/link/uscode/35/327?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/041-35-usc-328.html` as served (sha256 8f8be0d17b793ad977271697cf9f601e3a41997668a7f9e7d12ca70508b7024c); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title35/html/USCODE-2024-title35-partIII-chap32-sec328.htm (via https://www.govinfo.gov/link/uscode/35/328?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/042-18-usc-3161.html` as served (sha256 378fbffbbae22ac29f78e05ad3eee85925422c5d72b7e963e34cc433c6b08565); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title18/html/USCODE-2024-title18-partII-chap208-sec3161.htm (via https://www.govinfo.gov/link/uscode/18/3161?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/043-18-usc-3282.html` as served (sha256 da915b42536a7752c3e2eabc59b4f80349e217d168e31e4227d1fbd2ca99f175); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title18/html/USCODE-2024-title18-partII-chap213-sec3282.htm (via https://www.govinfo.gov/link/uscode/18/3282?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/044-5-usc-556.html` as served (sha256 a475e9af5e9ba903d47eed12e1e3dc2352b987e6230cc31247d5b72e985e45bf); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title5/html/USCODE-2024-title5-partI-chap5-subchapII-sec556.htm (via https://www.govinfo.gov/link/uscode/5/556?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/045-35-usc-312.html` as served (sha256 99f9fdb773f7b754e11481b98898f66cd8127dd408817b666769fd3d44620d6e); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title35/html/USCODE-2024-title35-partIII-chap31-sec312.htm (via https://www.govinfo.gov/link/uscode/35/312?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/046-18-usc-3142.html` as served (sha256 f4a6c4ca5dd7c89b9a76609b9c626708700fbc36a90505717311776fabe52362); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title18/html/USCODE-2024-title18-partII-chap207-sec3142.htm (via https://www.govinfo.gov/link/uscode/18/3142?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/047-18-usc-3162.html` as served (sha256 a990ac7a4efbe7447419312692daaf5d73c300caff93e03b6ea904932074a818); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title18/html/USCODE-2024-title18-partII-chap208-sec3162.htm (via https://www.govinfo.gov/link/uscode/18/3162?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/048-18-usc-3501.html` as served (sha256 e7ebb78db0ecf25924f4ea49fe64e5c037c49365e0ddc45fa7cf680feb2dbbb9); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title18/html/USCODE-2024-title18-partII-chap223-sec3501.htm (via https://www.govinfo.gov/link/uscode/18/3501?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/049-18-usc-3552.html` as served (sha256 0bd4530469e99ef9900f3f0806b58a3f2722b755b0ba7f5e466a401a41fd571d); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title18/html/USCODE-2024-title18-partII-chap227-subchapA-sec3552.htm (via https://www.govinfo.gov/link/uscode/18/3552?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/050-18-usc-3553.html` as served (sha256 2ca903a1b7d0f865b65a56fd9d0005e1d86f03aabf4302cf442e04f6a1a22eb2); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title18/html/USCODE-2024-title18-partII-chap227-subchapA-sec3553.htm (via https://www.govinfo.gov/link/uscode/18/3553?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/051-28-usc-1332.html` as served (sha256 b2ba7987c5c0bf3413b96124a17ad9a6238d47a6bc61c0330beee67136741791); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-partIV-chap85-sec1332.htm (via https://www.govinfo.gov/link/uscode/28/1332?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/052-28-usc-1391.html` as served (sha256 56661b65f32cecbd15277248cc7e1029264056b16cf97644c7ed8d3507e628cb); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-partIV-chap87-sec1391.htm (via https://www.govinfo.gov/link/uscode/28/1391?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/053-5-usc-706.html` as served (sha256 f4728d146a9408440e422e45d8217c698c7828abcc15242a07011a712ed8582d); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title5/html/USCODE-2024-title5-partI-chap7-sec706.htm (via https://www.govinfo.gov/link/uscode/5/706?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/054-35-usc-143.html` as served (sha256 f38a0fe2d899df3b538c476783000d1d125ac9917f7ce10f67f6a6fd5d7efc9a); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title35/html/USCODE-2024-title35-partII-chap13-sec143.htm (via https://www.govinfo.gov/link/uscode/35/143?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/055-35-usc-144.html` as served (sha256 237c79f3c11c74d79b50208c0e877267fe302e1ca50afdd88691b8c5af41bbb3); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title35/html/USCODE-2024-title35-partII-chap13-sec144.htm (via https://www.govinfo.gov/link/uscode/35/144?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/056-35-usc-306.html` as served (sha256 c20691546e6af231462a52102c5fdbfb44ee86270ccc094054a57c3502ab3e5a); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title35/html/USCODE-2024-title35-partIII-chap30-sec306.htm (via https://www.govinfo.gov/link/uscode/35/306?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-additional/057-35-usc-322.html` as served (sha256 df1a57f4c1c4715128871d7a26f63be5f020ca11861476a471e6ae0a997cd173); extraction sha256 5dedd90ad2e3e67185a2ae4b1ddfa68f7d7695b092666521fb9478c5c23852d8. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title35/html/USCODE-2024-title35-partIII-chap32-sec322.htm (via https://www.govinfo.gov/link/uscode/35/322?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.

### `usc-5-6103.txt` — 5 U.S.C. § 6103 (legal public holidays), with its notes

- Fetched 2026-10-22 from `www.govinfo.gov`'s official United States Code,
  2023 Edition (GPO), granule
  `USCODE-2023-title5-partIII-subpartE-chap61-subchapI-sec6103.htm`
  (sha256 of the fetched HTML:
  `0c3ef0793a5a8106354d3d3f3d4dc0e9aa9de7803325d01ca95048ff9c4cd806`). Tags
  stripped, entities decoded, runs of spaces collapsed; everything from the
  `§6103. Holidays` heading to the end of the page is kept, because the
  section's own source credits, Historical and Revision Notes, Amendments
  and effective-date notes are what date each holiday (`lit/core`'s
  calendar effective intervals cite them).
- sha256 (this file): `c1c9c35af37ca67e297aa61bd83e90d34ffed96045097fc97b99dfc342903dbf`
- Original: `original/usc-5-6103.html` as served (sha256 0c3ef0793a5a8106354d3d3f3d4dc0e9aa9de7803325d01ca95048ff9c4cd806); extraction sha256 c1c9c35af37ca67e297aa61bd83e90d34ffed96045097fc97b99dfc342903dbf. Served at https://www.govinfo.gov/content/pkg/USCODE-2023-title5/html/USCODE-2023-title5-partIII-subpartE-chap61-subchapI-sec6103.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.

### `cfr-19-part201.txt` — 19 C.F.R. §§ 201.14, 201.16 (USITC computation of time, service)

- Fetched 2026-10-22 from `www.govinfo.gov`'s official annual CFR XML
  (`CFR-2024-title19-vol3-part201.xml`, 2024-04-01 edition; sha256 of the
  fetched XML:
  `68669957569c7e7071252a1202cddde538173f019665e38e9965ae9a9741f4e2`), the
  same edition and `<SECTION>`/`<SECTNO>`/`<SUBJECT>`/`<P>` parse as
  `cfr-19-part210.txt` above, for the two sections § 210.6(a) incorporates.
- sha256 (this file): `436ec5247f1328fe6d6e2c15bcefd8f627cc3fcd44f0a0c034c7884609015879`
- Original: `original/cfr-19-part201.xml` as served (sha256 68669957569c7e7071252a1202cddde538173f019665e38e9965ae9a9741f4e2); extraction sha256 436ec5247f1328fe6d6e2c15bcefd8f627cc3fcd44f0a0c034c7884609015879. Served at https://www.govinfo.gov/content/pkg/CFR-2024-title19-vol3/xml/CFR-2024-title19-vol3-part201.xml; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.

### `usc-28-app-time-history.txt` — FRCP 6 and FRAP 26 as printed in the 2008 and 2011 editions of 28 U.S.C. App.

- The superseded (pre-December 1, 2009) time-computation texts and the
  amendment history `lit/core`'s earlier counting-rule versions cite:
  `## FRCP (2008 ed.) Rule 6`, `## FRCP (2011 ed.) Rule 6`, `## FRAP (2008
  ed.) Rule 26`, `## FRAP (2011 ed.) Rule 26` (each rule's text, its
  amendment credits, and its Advisory Committee Notes). The headings are
  deliberately not canonical `litigate_engine::cite::normalize` refs, so pack cites
  of the current rules never match these superseded texts.
- Fetched 2026-10-22 from `www.govinfo.gov`'s official United States Code
  (GPO), 2008 and 2011 Editions, granules
  `USCODE-2008-title28-app-rulesofci-rule6.htm` (sha256
  `256bd5122ec679035921b2740daa3fa29ccd0f21d3d60356113306e7ad609ebd`),
  `USCODE-2011-title28-app-federalru-dup1-rule6.htm` (`4661c642ad737aa66685cf08896ab474fe407f1efad0c526291d8f44772499cc`),
  `USCODE-2008-title28-app-federalru-rule26.htm` (`c9a9e2174f36cce7706030fa5fb55be3c0661960736086b1de57afc8475d8cdc`),
  `USCODE-2011-title28-app-federalru-rule26.htm` (`4917d6072a7e0d81eb26b2633656681265baa163a3fc3c977787482c3d190365`).
  Same extraction as `usc-5-6103.txt`, from each page's `Rule N.` heading to
  its end.
- sha256 (this file): `a55dd4c3b28e99491083474806c9e8bb621e4fd43b2fddf5c4359d1202f03f20`
- Original: `original/usc-28-app-time-history/001-content-pkg-USCODE-2008-title28-html-USCODE-2008-title28-app-rulesofci-rule6.html` as served (sha256 256bd5122ec679035921b2740daa3fa29ccd0f21d3d60356113306e7ad609ebd); extraction sha256 a55dd4c3b28e99491083474806c9e8bb621e4fd43b2fddf5c4359d1202f03f20. Served at https://www.govinfo.gov/content/pkg/USCODE-2008-title28/html/USCODE-2008-title28-app-rulesofci-rule6.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-28-app-time-history/002-nt-pkg-USCODE-2011-title28-html-USCODE-2011-title28-app-federalru-dup1-rule6.html` as served (sha256 4661c642ad737aa66685cf08896ab474fe407f1efad0c526291d8f44772499cc); extraction sha256 a55dd4c3b28e99491083474806c9e8bb621e4fd43b2fddf5c4359d1202f03f20. Served at https://www.govinfo.gov/content/pkg/USCODE-2011-title28/html/USCODE-2011-title28-app-federalru-dup1-rule6.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-28-app-time-history/003-ontent-pkg-USCODE-2008-title28-html-USCODE-2008-title28-app-federalru-rule26.html` as served (sha256 c9a9e2174f36cce7706030fa5fb55be3c0661960736086b1de57afc8475d8cdc); extraction sha256 a55dd4c3b28e99491083474806c9e8bb621e4fd43b2fddf5c4359d1202f03f20. Served at https://www.govinfo.gov/content/pkg/USCODE-2008-title28/html/USCODE-2008-title28-app-federalru-rule26.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-28-app-time-history/004-ontent-pkg-USCODE-2011-title28-html-USCODE-2011-title28-app-federalru-rule26.html` as served (sha256 4917d6072a7e0d81eb26b2633656681265baa163a3fc3c977787482c3d190365); extraction sha256 a55dd4c3b28e99491083474806c9e8bb621e4fd43b2fddf5c4359d1202f03f20. Served at https://www.govinfo.gov/content/pkg/USCODE-2011-title28/html/USCODE-2011-title28-app-federalru-rule26.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.

### `usc-28-app-2017-time.txt` — FRCP 6 and FRAP 26 as printed in the 2017 edition of 28 U.S.C. App.

- The texts `lit/core`'s 2016 versions of `frcp6` and `frap26` took effect
  with (each as amended December 1, 2016, the last amendment its credits
  list): `## FRCP (2017 ed.) Rule 6` and `## FRAP (2017 ed.) Rule 26`, each
  rule's text, amendment credits and Committee Notes. Not canonical refs,
  as in `usc-28-app-time-history.txt`.
- Fetched 2026-10-27 from `www.govinfo.gov`'s official United States Code
  (GPO), 2017 Edition (current through January 12, 2018), granules
  `USCODE-2017-title28-app-federalru-dup1-rule6.htm` (sha256
  `43524cf8ac34d6e27356597cf5307a61b5fb0e6fd3516addc1fd7f12d5a9a740`) and
  `USCODE-2017-title28-app-federalru-rule26.htm`
  (`6702848ddfeb87f0b06830920aa5ee4d71dd76249c5b0187cc50055a0b214c93`).
  Everything from the page's section head to its end, one line per
  paragraph, tags stripped, entities decoded, runs of spaces collapsed.
- sha256 (this file): `5d927c0a1b91b1a30a949a272c164ea6826cfebf6eac917da9a676732b22844d`
- Original: `original/usc-28-app-2017-time/001-nt-pkg-USCODE-2017-title28-html-USCODE-2017-title28-app-federalru-dup1-rule6.html` as served (sha256 43524cf8ac34d6e27356597cf5307a61b5fb0e6fd3516addc1fd7f12d5a9a740); extraction sha256 5d927c0a1b91b1a30a949a272c164ea6826cfebf6eac917da9a676732b22844d. Served at https://www.govinfo.gov/content/pkg/USCODE-2017-title28/html/USCODE-2017-title28-app-federalru-dup1-rule6.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-28-app-2017-time/002-ontent-pkg-USCODE-2017-title28-html-USCODE-2017-title28-app-federalru-rule26.html` as served (sha256 6702848ddfeb87f0b06830920aa5ee4d71dd76249c5b0187cc50055a0b214c93); extraction sha256 5d927c0a1b91b1a30a949a272c164ea6826cfebf6eac917da9a676732b22844d. Served at https://www.govinfo.gov/content/pkg/USCODE-2017-title28/html/USCODE-2017-title28-app-federalru-rule26.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.

### `rule-amendment-history.txt` — amendment histories of the sections `lit/core`'s current versions cite

- One `## versions <heading>` block per section (SPEC §4.3): an `as_of`
  line, then the section's amendments as a `content_versions` list
  (`date` and `amendment_date` the effective date, `issue_date` the order
  or publication date). Each is transcribed from the section's printed
  amendment credits, read as of the date the `as_of` line gives:
  - `Rule 6`, `Rule 14`, `Rule 81`, `Rule 86` (FRCP) and `Fed. R. Crim. P.
    45`: the "As amended ..., eff. ..." line closing each rule in
    `frcp.txt` and `frcrimp.txt` (as amended to December 1, 2024).
  - `FRAP 26`: the credits of the 2024 edition of 28 U.S.C. App. (current
    through January 6, 2025), granule
    `USCODE-2024-title28-app-federalru-rule26.htm` (sha256
    `1b1e1f0b76345262b3e275d5c2d5cfff2d47b813bdc1b0921ce96ed8a58fb878`),
    fetched 2026-10-27; `frap.txt` prints none.
  - `RCFC 6`: "As revised and reissued May 1, 2002; as amended June 20,
    2006, Nov. 3, 2008, Jan. 11, 2010, Aug. 1, 2017, July 29, 2024" in
    `rcfc.txt` (no issue dates printed).
  - `19 C.F.R. § 201.14`: the source note in `cfr-19-part201.txt` (41 FR
    17710; 56 FR 11923; 68 FR 32975; 76 FR 61941), each dated by the
    effective date its notice gives in `fr-usitc-time-history.txt` (May
    27, 1976; August 4, 2003; November 7, 2011), except 56 FR 11923, whose
    effective date is not vendored and which is dated by publication
    (March 21, 1991).
- sha256 (this file): `e6aef2e7672430dd698945d200c41a1988ef4f78ba7bfc0a2d7eae8e216362b8`
- Original: none as served (assembled from transcribed amendment credits read from frcp.txt, frcrimp.txt, rcfc.txt, cfr-19-part201.txt and the govinfo 2024 FRAP granule); see the sources listed above.

### `usc-28-app-frap4-history.txt` — FRAP 4 as printed in the 2008 and 2011 editions of 28 U.S.C. App.

- The pre-December 1, 2009 text of FRAP 4 and its amendment history, which
  date the earlier versions of the FRAP 4(a) appeal periods in `lit/frcp`,
  `lit/frap` and `lit/cafc`: `## FRAP (2008 ed.) Rule 4` (the text as
  amended through December 1, 2005, its amendment credits and every
  Advisory Committee Note from 1967 to 2005) and `## FRAP (2011 ed.) Rule
  4` (the text after the 2009, 2010 and 2011 amendments, with the 2009
  Committee Note that lengthens 4(a)(5)(C)'s 10 days to 14). The headings
  are deliberately not canonical refs, as in `usc-28-app-time-history.txt`.
- Fetched 2026-10-24 from `www.govinfo.gov`'s official United States Code
  (GPO), 2008 and 2011 Editions, granules
  `USCODE-2008-title28-app-federalru-rule4.htm` (sha256
  `9f1f9ca13c2e78bc28572e9abd1347434673846f35663fcec5c203a5d79593eb`) and
  `USCODE-2011-title28-app-federalru-rule4.htm`
  (`2a42d46c84b552edcf5385cfe73f41686a8ed0071bfe2c59f9ca101ac74234cb`).
  Same extraction as `usc-5-6103.txt`, from each page's `Rule 4.` heading
  to its end, one line per block element.
- sha256 (this file): `c246fa771851ef008653521977a993531d589881d85354e9c9f949f9c1b1a284`
- Original: `original/usc-28-app-frap4-history/001-content-pkg-USCODE-2008-title28-html-USCODE-2008-title28-app-federalru-rule4.html` as served (sha256 9f1f9ca13c2e78bc28572e9abd1347434673846f35663fcec5c203a5d79593eb); extraction sha256 c246fa771851ef008653521977a993531d589881d85354e9c9f949f9c1b1a284. Served at https://www.govinfo.gov/content/pkg/USCODE-2008-title28/html/USCODE-2008-title28-app-federalru-rule4.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/usc-28-app-frap4-history/002-content-pkg-USCODE-2011-title28-html-USCODE-2011-title28-app-federalru-rule4.html` as served (sha256 2a42d46c84b552edcf5385cfe73f41686a8ed0071bfe2c59f9ca101ac74234cb); extraction sha256 c246fa771851ef008653521977a993531d589881d85354e9c9f949f9c1b1a284. Served at https://www.govinfo.gov/content/pkg/USCODE-2011-title28/html/USCODE-2011-title28-app-federalru-rule4.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.

### `rcfc-appendices.txt` — RCFC Appendix H ¶ 3 (ADR ground rules) and Appendix J Patent Rules 16-17 (mandatory settlement discussions)

- `## RCFC App. H 3`: paragraph 3 ("Procedures") of Appendix H through
  ground rule (i), from the court's standalone `23. Appendix H.pdf`
  (`www.uscfc.uscourts.gov`, fetched 2026-10-24; sha256
  `41228005f2b5d60119d7b53f3bee122060a17f2234f91ab0b9cfa9f916b5ce2b`, the
  hash `lit/cofc`'s `rcfc-app-h` binding already recorded). `pdftotext
  -layout` text, its two columns read left then right (the 2020 Amendment
  note calls the referral clause "Paragraph 3(b)(1)", confirming the ground
  rules are paragraph 3's), lines joined and hyphenated line breaks
  closed; no wording changed. Ground rule (j) and the patent and copyright
  disclosure lists are omitted.
- `## PRCFC 16`, `## PRCFC 17`: Title V of the Patent Rules (Appendix J),
  from the court's compiled rules `Rules 10.08.26.pdf` (fetched 2026-10-24;
  sha256 `943733526f5391cad22562adf31411148fecfb03f01ae7bcf8bd6ec051a0c971`),
  whose title page reads "As amended through July 27, 2026", the expression
  `lit/cofc` is in force from. The standalone `Patent Rules 7.27.26.pdf` the
  `rcfc-app-j` binding records no longer resolves. Same extraction; page
  furniture dropped.
- sha256 (this file): `b49c8c942e95cbee386893d3899e4ac93b6b6bce1fea8eeff05e2ad44fa1800b`
- Original: `original/rcfc-appendices.pdf` as served (sha256 943733526f5391cad22562adf31411148fecfb03f01ae7bcf8bd6ec051a0c971); extraction sha256 b49c8c942e95cbee386893d3899e4ac93b6b6bce1fea8eeff05e2ad44fa1800b. Served at https://www.uscfc.uscourts.gov/sites/cfc/files/Rules%2010.08.26.pdf; retrieved 2026-10-24 (fetched as recorded above).

### `uspto-closures.txt` — the days the USPTO designated a "Federal holiday within the District of Columbia" (2009-2026)

- `## USPTO operating status`: every entry of the Office's "Major events
  affecting customers" page
  (`www.uspto.gov/learning-and-resources/operating-status`, fetched
  2026-10-24; sha256 of the HTML
  `c539783875b0937908137a59faae756d4c7b743350af4edb8aa6ec3c943f7f23`)
  that designates a day the Office was closed a federal holiday (under 35
  U.S.C. § 21 and 37 C.F.R. §§ 1.7, 1.9 and 2.196): executive-order and
  weather closures and the 2011 and 2015 power outages, each with its
  heading when it has one. `## USPTO closed December 24, 2020`: the
  Office's notice of that closure, which the page omits
  (`www.uspto.gov/subscription-center/2020/uspto-closed-thursday-dec-24-2020-0`,
  sha256 of the HTML
  `cb5ec84ea98d1c3d61a6ce0a83fbd85a8c2f79b3ebbbc63ecbc1d5a6e1fb9363`).
  Tags stripped, entities decoded, runs of spaces collapsed; other entries
  (postal interruptions, "extraordinary situation" relief, window hours)
  dropped. `lit/core`'s `uspto-closures` calendar lists these days.
- sha256 (this file): `23ebcf53195a7a766c7a69fd417e2e936612435709b3f1f99a23a14e6224ae86`
- Original: `original/uspto-closures/001-learning-and-resources-operating-status.html` as served (sha256 b7836ef928c0a743887c01798667e125b1819304b4f47f831631f1e6b1db49ce); extraction sha256 23ebcf53195a7a766c7a69fd417e2e936612435709b3f1f99a23a14e6224ae86. Served at https://www.uspto.gov/learning-and-resources/operating-status; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27. Page bytes differ from the first fetch's recorded sha, but the content was re-checked 2026-10-27 (lg-f3q8.82): every closure paragraph in the extraction is present in the served page and no later federal-holiday closure has been added, so no re-vendor is needed.
- Original: `original/uspto-closures/002-subscription-center-2020-uspto-closed-thursday-dec-24-2020-0.html` as served (sha256 212be76bac11a7c45538fce40f1a3db292a4c4c43689b1f386d6d5a2ab20f465); extraction sha256 23ebcf53195a7a766c7a69fd417e2e936612435709b3f1f99a23a14e6224ae86. Served at https://www.uspto.gov/subscription-center/2020/uspto-closed-thursday-dec-24-2020-0; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27. Page bytes differ from the first fetch's recorded sha, but the content was re-checked 2026-10-27 (lg-f3q8.82): every closure paragraph in the extraction is present in the served page and no later federal-holiday closure has been added, so no re-vendor is needed.

### `fr-usitc-time-history.txt` — Federal Register excerpts dating 19 C.F.R. §§ 201.14(a), 201.16(d)-(f) and 210.6 (USITC computation of time and service), 1976-2018

- Excerpts, one `## <vol> FR <page>` section per printed page, of every
  USITC rulemaking that made or touched the counting and service-time text
  `lit/core`'s `itc210` versions cite: the 1976 rules of practice (41 FR
  17710-17711, effective May 27, 1976), the 1984 calendar-days
  clarification (49 FR 32569-32571), 1991 (56 FR 11923, § 201.14(b) only),
  2002 (67 FR 68036-68037), 2003 (68 FR 32971, 32975), 2007 (72 FR
  13689), 2008 (73 FR 38316, 38320), 2011 (76 FR 61937, 61941-61942), 2013
  (78 FR 23474-23475, 23480) and 2018 (83 FR 21140, 21159). Each section
  opens with a bracketed line naming the issue, page and FR Doc. number;
  everything after it is the printed text, verbatim except for line
  joining. `* * * * *` marks omitted text, as the FR prints it.
- Fetched 2026-10-23. 1976 and 1984: `www.govinfo.gov`'s Federal Register
  archive issue PDFs `FR-1976-04-27.pdf` (sha256
  `b63c1a7cb2d9e068e0352b9b38c969f58cd7e1cfd4edbd20d237576b26048449`) and
  `FR-1984-08-15.pdf`
  (`247ce904f99b2a6740f796f376aa6e5c87a39d0623541177d0343ea4d155d32d`).
  The text layer was extracted with `pdftotext` and checked against the
  rendered page images (pp. 17710-17711); OCR soft hyphens are joined. 1991:
  `FR-1991-03-21.pdf`
  (`e7777501669824e7c3aaa67f710621a89f02df689fe2d03e78647b1377fbb03f`),
  same method. 2002-2018: `www.federalregister.gov`'s GPO full-text
  rendering of each document (`/documents/full_text/text/<yyyy>/<mm>/<dd>/<doc>.txt`),
  with `[[Page N]]` markers giving the page. Documents and sha256 of the
  fetched text: 02-28404 (`0b8c20459501fe9e99b1524d879e7812dd89b31af9e003ad859c69857b5b4ddb`),
  03-13688 (`fc5a7e481787c56ec6f1f5bdb9db7f28e2b2fbff291bef78e4a1729f1a7819dc`),
  E7-5177 (`d4b23cf9a173f4e2a673b370ed983ca943ece185807302ef2cb6a33a7c90be50`),
  E8-14872 (`bb9fa1cc9ae113b6215df1abdbdddff3e5bf22f6abd8e71ef9aa636a614d3dbb`),
  2011-25646 (`bbe5fa864d2d43c64cdc139cdcec254bbd4874997bee55f4033435eaa10dc4ca`),
  2013-08931 (`314e668faaaee24a62d71f1a90c21a0a1580d3996a8b73499052b0b8efd44a76`),
  2018-09268 (`8dd83746d9334792271a5108aa5b1151a0a7efd836ce79abbb163e09b4d3fd0f`).
  Each document's govinfo PDF is `https://www.govinfo.gov/content/pkg/FR-<date>/pdf/<doc>.pdf`.
- How the list was found: every amendment credit in the 2024 CFR source
  notes of § 201.14 (41 FR 17710; 56 FR 11923; 68 FR 32975; 76 FR 61941)
  and § 201.16 (41 FR 17711; 47 FR 6190; 47 FR 33682; 49 FR 32571; 67 FR
  68037; 73 FR 38320; 76 FR 61942; 78 FR 23480; 83 FR 21159), plus the
  federalregister.gov API's list of USITC rules for 1994-2018, which found
  the 2007 § 210.6 revision (72 FR 13689) that the 2024 § 210.6 note
  ([78 FR 23480]) no longer credits. The two 1982 documents (47 FR 6190,
  `FR-1982-02-10.pdf` sha256
  `ae7d608310f86df4858e1158ce8062f5e8d56596f356ab8abe89d7eeb9394b28`; 47 FR
  33682, `FR-1982-08-04.pdf`
  `b8936bd30d219c5c22b6b99fedb3b0c71eb40f99b1a37c09b0b9ddd99d5898cd`) were
  read and are not vendored: their amendatory instructions touch § 201.16(b)
  and (c) only. The 1994 Part 210 rewrite (59 FR 39020, August 1, 1994)
  is not vendored either (its issue PDF did not download within the
  fetch limit); its § 210.6, which only incorporates §§ 201.14 and 201.16(d),
  was read in the 1997 annual CFR edition
  (`CFR-1997-title19-vol3-sec210-6.xml`, sha256
  `683ebeb277b3b3efd4f177597d599d0991970bec4e90cd755c7d4ecdef972ac1`), and
  the 2007 and 2012 editions (sha256
  `ca9978f33fc1185ec296fd6fa40bd15324dc2fdde1fd08aaadc6c239fb27aa05`,
  `e8953336a7c43e311b2fdcfe21b1a75d3b8f97a1c5b1694190a127d99bd7bf24`)
  confirm the 2007 text stood until 2013.
- sha256 (this file): `02ce787e06ce9d46aa341d0aab319a23cda0eaf6c631077a7bd64c4a6a0faa86`
- Original: `original/fr-usitc-time-history/001-content-pkg-FR-1976-04-27-pdf-FR-1976-04-27.pdf` as served (sha256 b63c1a7cb2d9e068e0352b9b38c969f58cd7e1cfd4edbd20d237576b26048449); extraction sha256 02ce787e06ce9d46aa341d0aab319a23cda0eaf6c631077a7bd64c4a6a0faa86. Served at https://www.govinfo.gov/content/pkg/FR-1976-04-27/pdf/FR-1976-04-27.pdf; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/fr-usitc-time-history/002-content-pkg-FR-1984-08-15-pdf-FR-1984-08-15.pdf` as served (sha256 247ce904f99b2a6740f796f376aa6e5c87a39d0623541177d0343ea4d155d32d); extraction sha256 02ce787e06ce9d46aa341d0aab319a23cda0eaf6c631077a7bd64c4a6a0faa86. Served at https://www.govinfo.gov/content/pkg/FR-1984-08-15/pdf/FR-1984-08-15.pdf; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/fr-usitc-time-history/003-content-pkg-FR-1991-03-21-pdf-FR-1991-03-21.pdf` as served (sha256 e7777501669824e7c3aaa67f710621a89f02df689fe2d03e78647b1377fbb03f); extraction sha256 02ce787e06ce9d46aa341d0aab319a23cda0eaf6c631077a7bd64c4a6a0faa86. Served at https://www.govinfo.gov/content/pkg/FR-1991-03-21/pdf/FR-1991-03-21.pdf; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/fr-usitc-time-history/004-documents-full_text-text-2002-11-08-02-28404.raw` as served (sha256 0b8c20459501fe9e99b1524d879e7812dd89b31af9e003ad859c69857b5b4ddb); extraction sha256 02ce787e06ce9d46aa341d0aab319a23cda0eaf6c631077a7bd64c4a6a0faa86. Served at https://www.federalregister.gov/documents/full_text/text/2002/11/08/02-28404.txt; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/fr-usitc-time-history/005-documents-full_text-text-2003-06-03-03-13688.raw` as served (sha256 fc5a7e481787c56ec6f1f5bdb9db7f28e2b2fbff291bef78e4a1729f1a7819dc); extraction sha256 02ce787e06ce9d46aa341d0aab319a23cda0eaf6c631077a7bd64c4a6a0faa86. Served at https://www.federalregister.gov/documents/full_text/text/2003/06/03/03-13688.txt; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/fr-usitc-time-history/006-documents-full_text-text-2007-03-23-E7-5177.raw` as served (sha256 d4b23cf9a173f4e2a673b370ed983ca943ece185807302ef2cb6a33a7c90be50); extraction sha256 02ce787e06ce9d46aa341d0aab319a23cda0eaf6c631077a7bd64c4a6a0faa86. Served at https://www.federalregister.gov/documents/full_text/text/2007/03/23/E7-5177.txt; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/fr-usitc-time-history/007-documents-full_text-text-2008-07-07-E8-14872.raw` as served (sha256 bb9fa1cc9ae113b6215df1abdbdddff3e5bf22f6abd8e71ef9aa636a614d3dbb); extraction sha256 02ce787e06ce9d46aa341d0aab319a23cda0eaf6c631077a7bd64c4a6a0faa86. Served at https://www.federalregister.gov/documents/full_text/text/2008/07/07/E8-14872.txt; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/fr-usitc-time-history/008-documents-full_text-text-2011-10-06-2011-25646.raw` as served (sha256 bbe5fa864d2d43c64cdc139cdcec254bbd4874997bee55f4033435eaa10dc4ca); extraction sha256 02ce787e06ce9d46aa341d0aab319a23cda0eaf6c631077a7bd64c4a6a0faa86. Served at https://www.federalregister.gov/documents/full_text/text/2011/10/06/2011-25646.txt; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/fr-usitc-time-history/009-documents-full_text-text-2013-04-19-2013-08931.raw` as served (sha256 314e668faaaee24a62d71f1a90c21a0a1580d3996a8b73499052b0b8efd44a76); extraction sha256 02ce787e06ce9d46aa341d0aab319a23cda0eaf6c631077a7bd64c4a6a0faa86. Served at https://www.federalregister.gov/documents/full_text/text/2013/04/19/2013-08931.txt; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/fr-usitc-time-history/010-documents-full_text-text-2018-05-08-2018-09268.raw` as served (sha256 8dd83746d9334792271a5108aa5b1151a0a7efd836ce79abbb163e09b4d3fd0f); extraction sha256 02ce787e06ce9d46aa341d0aab319a23cda0eaf6c631077a7bd64c4a6a0faa86. Served at https://www.federalregister.gov/documents/full_text/text/2018/05/08/2018-09268.txt; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.

## Deliberately not vendored yet (follow-up)

- **RCFC Appendices A/C/H/J** (case management, bid-protest, ADR, patent
  rules) and the **Federal Circuit's Attorney Discipline Rules / Internal
  Operating Procedures / Mediation Guidelines** — no pack currently cites
  into these; `rcfc.txt`/`fedcir-rules.txt` above deliberately stop at the
  main numbered rules.
- **MPEP §§ 803 and 818** (top-level headings only — "Restriction — When
  Proper" and "Election and Reply") — cited by `mpep-prosecution.json`, but
  neither's own top-level body heading could be cleanly isolated in
  `mpep-0800.pdf`'s `pdftotext -layout` output in this pass (the two-column
  layout mixes each into surrounding statutory text without a clean `^803 `/
  `^818 ` line start the way their own lettered/decimal subsections have);
  §§ 818.01/821.04 from the same chapter *were* isolated cleanly and are in
  `mpep.txt`. A follow-up should try `pdftotext` without `-layout` (the
  approach that worked for `rcfc.txt`/`frcrimp.txt`) instead.
- **MPEP §§ 2210-2214** (ex parte reexam request formalities, cited as the
  range "MPEP 2209-2214" alongside § 2209, which *is* vendored) — same
  two-column extraction problem as § 803 above; only § 2209 and § 2254 from
  that chapter were cleanly isolated in this pass.
- **MPEP** chapters 2600 (inter partes reexam) and 2800 (supplemental
  examination) — not cited by any pack today.
- **Fed. R. Evid.** — no pack cites an FRE rule number today (only the
  `Daubert` case name, which is out-of-scope case law, not a rule cite);
  `normalize.rs` doesn't even have an FRE `Family` variant yet.
- Court/tribunal **fee schedules** and **statistical datapacks** (e.g.
  `uscfc-fee-schedule`, `cafc-fee-schedule`, `patentlyo-2024-datapack`,
  `ptab-tpg`) — these back a pack's dollar/probability figures, not a
  `cite`/`authority` string, so cite verification has no reason to vendor
  them; out of this corpus's scope entirely.

### `cfr-37-part2.txt` — 37 C.F.R. Part 2 (trademark rules of practice incl. TTAB Subpart A, 155 sections)

- Source: the eCFR text of 37 C.F.R. Part 2 as of 2026-10-19, read from David's
  library copy (`37 CFR Part 2 - Rules of Practice in Trademark Cases (eCFR
  2026-10-19).md`; official source `https://www.ecfr.gov/current/title-37/chapter-I/subchapter-B/part-2`).
  Split mechanically on the `### § 2.N` headings into one `## 37 C.F.R. § 2.N`
  block per section (section number and heading restated as the first body
  line, `#### ` subpart headings and `*[NN FR ...]*` amendment-history lines
  dropped). No wording changed. Used by `packs/ttab-trademark-trial-and-appeal-board.json`.
- sha256 (this file): `66402ce5e48bf2a9aa039058b3bfc181d3511b9e94137fac8b6a3d84a26b33a7`
- Original: none as served (assembled from David's library copy of the eCFR Part 2 text as of 2026-10-19, split mechanically); see the sources listed above.

### `cfr-37-part2-answer-time-history.txt` — 37 C.F.R. §§ 2.105, 2.106, 2.113 and 2.116 (TTAB answer time) as of January 13 and 14, 2017

- The dates `lit/ttab`'s answer-time deadlines are in force from. For each
  section, `## 37 C.F.R. (eCFR versions) § N` opens with a bracketed line
  listing every version the eCFR records for it (2017-01-01, its first
  recorded version, and 2017-01-14, the last amendment), then the section's
  text as of 2017-01-14; `## 37 C.F.R. (eCFR 2017-01-13) § N` is the text
  the day before. The headings are deliberately not canonical
  `litigate_engine::cite::normalize` refs, so cites of the current sections resolve
  to `cfr-37-part2.txt`, whose wording these 2017-01-14 texts match word for
  word.
- Fetched 2026-10-23 from the eCFR versioner API (`www.ecfr.gov`, Office of
  the Federal Register): the version list
  `/api/versioner/v1/versions/title-37.json?part=2` (sha256
  `d1a55830383f26753b161838a0e0512897ba35fac0e0c293cd2541b1c2cd4bff`), and
  `/api/versioner/v1/full/<date>/title-37.xml?part=2&section=<N>` for
  2017-01-14 (§ 2.105 `ee15a98638c69c675b069e160a0eefff7ef4732bb6314b65b1d592987a156779`,
  § 2.106 `84402a507a3cccf21ce99e1fd68bbeae1fd8cffc41c7f0c2c7390cb6e616f97f`,
  § 2.113 `49c60f5418ec5228616ad9f2d250559919dba1648e3f6d32695bb066b3a7b48c`;
  the 2026-10-19 responses are byte-identical) and 2017-01-13 (§ 2.105
  `e67cbbcc06c258c1c5b8af8e44621cf92abd5ae1634cf7c81f6b0874725c1e9a`,
  § 2.106 `2b48e49bf2cc100d9980228754be964b3d8ae664ae0267e801d99596ab8af263`,
  § 2.113 `465446e09efbf10073a05610c2e5aeab43ca72a8f2f6c5979e89dcc4b484539c`).
  Each `<HEAD>` and `<P>` became one line, tags stripped, entities decoded,
  whitespace collapsed. No wording changed.
- § 2.116 (the Federal Rules of Civil Procedure govern inter partes
  procedure "wherever applicable and appropriate", which `lit/ttab`'s
  answer-time extension reads FRCP 6(b)(1)(A) through), fetched 2026-10-24
  the same way: the version list
  `/api/versioner/v1/versions/title-37.json?section=2.116` (sha256
  `b30211620065f5d4a4612b29c270bb2ae6f3e80a55cb5e19d292906c55930910`;
  versions 2017-01-01 and 2017-01-14 only), and the section at 2017-01-14
  (`ce683532c79bc3168c1b103f204c4bada23fa3c32c09c368551b9a5790a65683`; the
  2026-10-19 response is byte-identical) and 2017-01-13
  (`44b401f931dd4498c0d6316539f4c778e551b8976deb8682338c2bbf62f75a41`).
  Paragraph (a) reads the same in both; the 2017 amendment (81 FR 69975)
  changed (c) and (e)-(g) only.
- sha256 (this file): `2e5f7e43fa7ffe053c139a3c49f967c0d996652a0db371859c55fd30d1bc1a66`
- Original: `original/cfr-37-part2-answer-time-history/001-CFR-2016-title37-vol1-part2.xml` as served (sha256 d675f0a50793c20ee0b11ba24420e9fbd8c98c8b6ef700acd8cff773212f7d1c); extraction sha256 2e5f7e43fa7ffe053c139a3c49f967c0d996652a0db371859c55fd30d1bc1a66. Served at https://www.govinfo.gov/content/pkg/CFR-2016-title37-vol1/xml/CFR-2016-title37-vol1-part2.xml; this is the govinfo annual CFR edition of 2016-07-01, not the eCFR point-in-time bytes the extraction was taken from; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/cfr-37-part2-answer-time-history/002-CFR-2017-title37-vol1-part2.xml` as served (sha256 7634dbd3816d83fd109cb085d230e0888ea61ea9044702d6330aa1273f37f56a); extraction sha256 2e5f7e43fa7ffe053c139a3c49f967c0d996652a0db371859c55fd30d1bc1a66. Served at https://www.govinfo.gov/content/pkg/CFR-2017-title37-vol1/xml/CFR-2017-title37-vol1-part2.xml; this is the govinfo annual CFR edition of 2017-07-01, not the eCFR point-in-time bytes the extraction was taken from; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original coverage (lg-f3q8.82, 2026-10-27): the eCFR point-in-time URL this extraction was fetched from is disallowed by ecfr.gov's robots.txt (`/api/versioner/v1/full/`), so the official originals are the govinfo annual editions around the dates recorded: the 2016 edition (title 37 revised as of 2016-07-01, 001) for the texts as of 2017-01-01 and 2017-01-13, and the 2017 edition (revised as of 2017-07-01, 002) for the texts as of 2017-01-14. Neither edition is a point-in-time copy; amendments between 2016-07-01 and 2017-01-13 would not show in 001. Compared word for word with the sections of this extraction: the 2017-01-14 texts and the 2017-01-01 § 2.196 are identical; every other difference is the annual edition's printed source note (`[81 FR ...]` history line) that the extraction omits.

### `cfr-37-part2-history.txt` — 37 C.F.R. §§ 2.119, 2.128 and 2.196 as in force on dated eCFR points in time

- The texts that date `lit/ttab`'s counting rule (37 C.F.R. § 2.196 with
  § 2.119(c)'s service rule), which `cfr-37-part2.txt` cannot, having
  dropped its amendment-history lines: `## 37 C.F.R. (eCFR 2017-01-01)
  § 2.119` (the last text adding "5 days" after service by first-class
  mail, Priority Mail Express, or overnight courier), `## 37 C.F.R. (eCFR
  2017-01-14) § 2.119` (the text in force from January 14, 2017, with no
  added time, and its source note `[81 FR 69975, Oct. 7, 2016]`), and `## 37
  C.F.R. (eCFR 2017-01-01) § 2.196`; and `## 37 C.F.R. (eCFR 2017-01-14)
  § 2.128`, which dates the TTAB briefing periods. The headings are deliberately not
  canonical refs, so cites of the current sections never match them.
- Fetched 2026-10-23 from the eCFR point-in-time API
  (`https://www.ecfr.gov/api/versioner/v1/full/<date>/title-37.xml?section=<n>`):
  § 2.119 at 2017-01-01 (sha256
  `baa801e7830e57f49aa20c0cf3d1953529bc932b0cd2290c348a99cacfe96a83`) and
  2017-01-14 (`ce4965c42d3239c42332c5d9636b35131b4e447259eb8b6a7fcf3ef1e578a17b`),
  § 2.196 at 2017-01-01
  (`96acb388ef12e7b0b4ea61e5eb62a88cee212fbdab516167a218bfaeb012dd56`), and
  § 2.128 at 2017-01-14
  (`9ec43724d120002679c5a2f68693211c33f5aaf1f445e0156c291e250a2e1d28`;
  fetched 2026-10-23 with `part=2`, compressed). § 2.128's versions are
  2017-01-01 and 2017-01-14 only; paragraph (a)(1) reads the same in both
  (2017-01-01 file sha256
  `3b05cab255163b5867057fc6a127e4cd511b888ae8885a47a19ee0ce1a6389a6`, not
  stored), and as `cfr-37-part2.txt` vendors it today. The
  versions API (`/api/versioner/v1/versions/title-37.json?section=<n>`)
  lists § 2.119 versions on 2017-01-01, 2017-01-14 and 2018-11-30 (the last
  leaves (c) unchanged) and § 2.196 only on 2017-01-01, the start of the
  eCFR's point-in-time coverage: § 2.196 is unchanged since at least then.
  Each `<P>` paragraph, the `<HEAD>` heading and the `<CITA>` source note are
  one line each, tags stripped and entities decoded; no wording changed.
- sha256 (this file): `ea972783c810a458f4754ce2597e2f29f08235d9d5cdcb8c0e8d1cd2508688e4`
- Original: `original/cfr-37-part2-history/001-CFR-2016-title37-vol1-part2.xml` as served (sha256 d675f0a50793c20ee0b11ba24420e9fbd8c98c8b6ef700acd8cff773212f7d1c); extraction sha256 ea972783c810a458f4754ce2597e2f29f08235d9d5cdcb8c0e8d1cd2508688e4. Served at https://www.govinfo.gov/content/pkg/CFR-2016-title37-vol1/xml/CFR-2016-title37-vol1-part2.xml; this is the govinfo annual CFR edition of 2016-07-01, not the eCFR point-in-time bytes the extraction was taken from; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/cfr-37-part2-history/002-CFR-2017-title37-vol1-part2.xml` as served (sha256 7634dbd3816d83fd109cb085d230e0888ea61ea9044702d6330aa1273f37f56a); extraction sha256 ea972783c810a458f4754ce2597e2f29f08235d9d5cdcb8c0e8d1cd2508688e4. Served at https://www.govinfo.gov/content/pkg/CFR-2017-title37-vol1/xml/CFR-2017-title37-vol1-part2.xml; this is the govinfo annual CFR edition of 2017-07-01, not the eCFR point-in-time bytes the extraction was taken from; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original coverage (lg-f3q8.82, 2026-10-27): the eCFR point-in-time URL this extraction was fetched from is disallowed by ecfr.gov's robots.txt (`/api/versioner/v1/full/`), so the official originals are the govinfo annual editions around the dates recorded: the 2016 edition (title 37 revised as of 2016-07-01, 001) for the texts as of 2017-01-01 and 2017-01-13, and the 2017 edition (revised as of 2017-07-01, 002) for the texts as of 2017-01-14. Neither edition is a point-in-time copy; amendments between 2016-07-01 and 2017-01-13 would not show in 001. Compared word for word with the sections of this extraction: the 2017-01-14 texts and the 2017-01-01 § 2.196 are identical; every other difference is the annual edition's printed source note (`[81 FR ...]` history line) that the extraction omits, except that § 2.119(b) in the 2016 edition contains the sentence "Whenever it shall be satisfactorily shown to the Director that none of the above modes of obtaining service or serving the paper is practicable, service may be by notice published in the Official Gazette.", which this extraction's 2017-01-01 text of § 2.119 does not (unresolved; both texts lack it from 2017-01-14).

### `ecfr-versions-title-37.txt` — eCFR versioner version lists of the 37 C.F.R. sections cited

- The amendment histories `cite-resolves` reads (SPEC §4.3): a cite of
  `cfr-37-part1.txt`, `cfr-37-part2.txt` or `cfr-37-part42.txt` (dated by
  its binding's `as_of`) by a declaration in force from an earlier date is
  that declaration's text only when its section's list shows no version
  effective after that date, up to the text's. One `## versions 37 C.F.R.
  § N` block per section `lit/mpep`, `lit/ptab` and `lit/ttab` cite: an
  `as_of 2026-10-24` line (the fetch date), then the response to
  `https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=N`
  (Office of the Federal Register), byte for byte on one line. The
  headings are deliberately not canonical refs, so no cite resolves to
  them. The lists start at 2017-01-01, the start of the eCFR's
  point-in-time coverage, so they prove nothing about a version in force
  before then.
- Fetched 2026-10-24. SHA-256 of each response (the block's last line):
  - § 1.7: `b05d1efdaec5cec72f84229707f964daf359e135632c8b4991b392f852559924`
  - § 1.16: `900c41f00037c23fa8b52824d40130736add9505ae93d6281a6ab1278dd877d2`
  - § 1.17: `b513e1f80b76adf6307496fdf47f9cd72ab1a9ab19ce32937bf6399fd256ac67`
  - § 1.18: `a4146e56fed715df85c88743dc5e4aa6082a443e862008370f20c5a77e6dd8d6`
  - § 1.20: `bdd9ed0d99fd816ba08cd73171f91f889ebdc7e5c7ff4b2e19808e4ce2d8ccf7`
  - § 1.104: `5fddd6eedd918f2df6f119fb47e121b5b1e7cebf3486640b3d9e9ee0a0160607`
  - § 1.133: `38b8935cde1f81a0cfcc14bcff451321c1123467502d77bfc551619479cfd85c`
  - § 1.134: `55620caa0644f3718d1bef52df4695e19615d327c22308653c59d4c427868dbb`
  - § 1.137: `f26dd480c778a76b648ba4c75a64f9f9f0fd39ef82c79557136eee5a5285f45a`
  - § 1.142: `0129bd8c6dfe95d6a4b9f368fa4fd8ed2dbfa69a3cf11bc7cbcfbe0f77754876`
  - § 1.143: `1d8c24c98826b1c45afa62e3d3beb0e9058f7f69dbc39f6f52894e265f3f43eb`
  - § 1.144: `ee0c94275014897f8f7cf1d79159279282f42ebb0409834b725d014698603e1f`
  - § 1.171: `38ca49f03e43511563a695250d4606dc97823f070c9ddd5360846a41f99fe9ce`
  - § 1.510: `e0bf431ace62f84139482f725a0744f75ddee66bf1e5a6ee4ed5045925f184db`
  - § 1.515: `08132fb4845e07a2700b1cd59d51c3fe8ef8d9459bf41e260c6ce648220b8652`
  - § 1.530: `bcf8895a4e81671153c042587d5998287b2861a5d28856aec6a7f97cff3ed19d`
  - § 1.550: `227ae4a84aafa256f300f8d0038cb00c11fe3a707811653aa4e121d4469cd599`
  - § 1.570: `2d8de876d87fc0959d7fc3b75a5474f29f656c2aaee5d94f6e170860a82faa40`
  - § 2.6: `1feac2b52422420078093d563df7306811bd966b277904d4ab0522222c2f55b7`
  - § 2.68: `d424543583ec96f886e0882c5a4ef0b009024808e7793623d658e771365ccae5`
  - § 2.88: `f398efe4ae636f678ab9f3ff310179ea17dff38893d7a7d329cc3d98711376cb`
  - § 2.101: `55690d36a068e612d3be376204f62a6f09348df1def93b51dcb7a70c51e4f856`
  - § 2.102: `cebb4ba1e7e3b5726dea84d9f63e2d87e473a228ec3fad3a8d9a4bfbcd2bc364`
  - § 2.105: `8196196e26e9e069cc98745fcf25745f8c1d89f0b60b2694c8f4c9e033d6e923`
  - § 2.106: `bafe6722400078cf4e3850cceb4b1d29103bbef827ca65187cf83f044339290e`
  - § 2.111: `fb478d7488ffb7792802e803e75cee79a136c7701fce99de636281dce5e7f45b`
  - § 2.113: `e37846c21e075a78123b969a47ebd61d86329f57978b4850b06a07aa5aaa07c0`
  - § 2.120: `7abdbb0667ea2eaaa70359c0aa3cb31322564c946465be1a43ad99fcafe11fb3`
  - § 2.121: `bd59901972a1e2b677dcba6440a8a1a0c42c64ed2e09c2918500fce51810f9c1`
  - § 2.123: `aaccb79f28ab614449c4dea8cf13dcfb11e37ed7623852b9177b8899e6f6abb5`
  - § 2.125: `6d928b5fd8ec7a6a2403373732bf3f87fe58420e117d140526ba13915fcec701`
  - § 2.128: `84c1c21d2f4b29b9a74f282c7cbce3a59d43d1300977eaa25185e8bd692fe267`
  - § 2.129: `46be73aa4f50e11629cfe8229278a767a4f587dc6aaad5c867e6928e4ab5f45d`
  - § 2.133: `83b7d43f1b32283d04c6a66c8c12ef47df669234152394d60a533b8df55c7808`
  - § 2.135: `57ea6363fff223478410917f2fbdea6cd1fa2362fb10f035ef894c44c79e1575`
  - § 2.136: `3883d7604e57461bca0e5e369061981d122190f76c99826ec4879e44161d9143`
  - § 2.145: `b8b24d0d7ec1cb15937362ee8757a8856587e90baf74d7b2a6fea0d8d34a007b`
  - § 2.196: `155ecb2188b4f43e202e074a05847754add55d6f03cc4acb73153c254be56ccb`
  - § 42.5: `ad8584b6f44f8258a42f04904bc65ad3e9b797c63f092a73bc25854f990b28c2`
  - § 42.23: `cd4d5610fbb61e14540badd51f9bb8e69f98dfe5c6e2b811dbe2b11b6562852a`
  - § 42.51: `96e272b43757047438bebd6040f3be92414702bdcb6a38cf072408e129ae8f59`
  - § 42.54: `d536aac43c230b3829f992d6094b3352ee27b407f1d6a256421887458b16d2b5`
  - § 42.70: `a992cb2349c371cd96924d00ccbae5007b1184c51d5dd88cbb5cbf7d25b5a1c5`
  - § 42.71: `8821cdf86cc686b2cb114d91fa18c47c09c4862768303fdeae5d885bcc89f7f5`
  - § 42.73: `a094844ec3755b2bfaae6307843923e5e71ec492392a173aedd3ed2d619d3918`
  - § 42.74: `c027a7caba2e40c7fd62d67abd987160040fdc71e5b3e72fbad397a8388581ca`
  - § 42.104: `2fb8632176a967fdbbfb54dda888edea6e206f4f8cda7cf8fbc45043d116d020`
  - § 42.106: `ebfcec77e715a797a50c95f84387f892a75e1f9566b56f50df4106f4c5188d78`
  - § 42.107: `d568d69556be8736e4317c4839fe5baf15335d3e12a60520769d0c7ef4ecc295`
  - § 42.108: `7e1d5e33e5c00dafa685a99cd18769674f5c5d62a7f8de1fde20d68decadb5b8`
  - § 42.120: `8a94bc1d501668bcacbb20e1f0328f21baf83bf23605c6a5f70639bd491c4269`
  - § 42.121: `fd40f74f4930ffbdfc57a64fd311b2d540a70692afb44d56885c862bf6e66567`
  - § 42.122: `97b014134fa28a48101dd37780b44c417558f5216b61fdca8cd493dfefcc7f8f`
  - § 42.204: `1e5dac78eda9bb275a1f82817a7a2da8811c8c28c6a0b2e3470d7f0526adb440`
  - § 42.206: `4e5bb6f2549185ec1066db76a6825964ad85208ceecb52d8c989e4242c24b018`
  - § 42.207: `492b8169f500af5691e238c57d13fc70a5bd49fb1bee44f0bdfc55519c354619`
- sha256 (this file): `083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058`
- Original: `original/ecfr-versions-title-37/001-api-versioner-v1-versions-title-37.json_section-1.7.json` as served (sha256 b05d1efdaec5cec72f84229707f964daf359e135632c8b4991b392f852559924); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=1.7; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/002-api-versioner-v1-versions-title-37.json_section-1.16.json` as served (sha256 900c41f00037c23fa8b52824d40130736add9505ae93d6281a6ab1278dd877d2); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=1.16; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/003-api-versioner-v1-versions-title-37.json_section-1.17.json` as served (sha256 b513e1f80b76adf6307496fdf47f9cd72ab1a9ab19ce32937bf6399fd256ac67); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=1.17; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/004-api-versioner-v1-versions-title-37.json_section-1.18.json` as served (sha256 a4146e56fed715df85c88743dc5e4aa6082a443e862008370f20c5a77e6dd8d6); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=1.18; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/005-api-versioner-v1-versions-title-37.json_section-1.20.json` as served (sha256 bdd9ed0d99fd816ba08cd73171f91f889ebdc7e5c7ff4b2e19808e4ce2d8ccf7); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=1.20; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/006-api-versioner-v1-versions-title-37.json_section-1.104.json` as served (sha256 5fddd6eedd918f2df6f119fb47e121b5b1e7cebf3486640b3d9e9ee0a0160607); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=1.104; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/007-api-versioner-v1-versions-title-37.json_section-1.133.json` as served (sha256 38b8935cde1f81a0cfcc14bcff451321c1123467502d77bfc551619479cfd85c); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=1.133; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/008-api-versioner-v1-versions-title-37.json_section-1.134.json` as served (sha256 55620caa0644f3718d1bef52df4695e19615d327c22308653c59d4c427868dbb); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=1.134; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/009-api-versioner-v1-versions-title-37.json_section-1.137.json` as served (sha256 f26dd480c778a76b648ba4c75a64f9f9f0fd39ef82c79557136eee5a5285f45a); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=1.137; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/010-api-versioner-v1-versions-title-37.json_section-1.142.json` as served (sha256 0129bd8c6dfe95d6a4b9f368fa4fd8ed2dbfa69a3cf11bc7cbcfbe0f77754876); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=1.142; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/011-api-versioner-v1-versions-title-37.json_section-1.143.json` as served (sha256 1d8c24c98826b1c45afa62e3d3beb0e9058f7f69dbc39f6f52894e265f3f43eb); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=1.143; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/012-api-versioner-v1-versions-title-37.json_section-1.144.json` as served (sha256 ee0c94275014897f8f7cf1d79159279282f42ebb0409834b725d014698603e1f); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=1.144; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/013-api-versioner-v1-versions-title-37.json_section-1.171.json` as served (sha256 38ca49f03e43511563a695250d4606dc97823f070c9ddd5360846a41f99fe9ce); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=1.171; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/014-api-versioner-v1-versions-title-37.json_section-1.510.json` as served (sha256 e0bf431ace62f84139482f725a0744f75ddee66bf1e5a6ee4ed5045925f184db); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=1.510; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/015-api-versioner-v1-versions-title-37.json_section-1.515.json` as served (sha256 08132fb4845e07a2700b1cd59d51c3fe8ef8d9459bf41e260c6ce648220b8652); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=1.515; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/016-api-versioner-v1-versions-title-37.json_section-1.530.json` as served (sha256 bcf8895a4e81671153c042587d5998287b2861a5d28856aec6a7f97cff3ed19d); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=1.530; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/017-api-versioner-v1-versions-title-37.json_section-1.550.json` as served (sha256 227ae4a84aafa256f300f8d0038cb00c11fe3a707811653aa4e121d4469cd599); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=1.550; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/018-api-versioner-v1-versions-title-37.json_section-1.570.json` as served (sha256 2d8de876d87fc0959d7fc3b75a5474f29f656c2aaee5d94f6e170860a82faa40); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=1.570; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/019-api-versioner-v1-versions-title-37.json_section-2.6.json` as served (sha256 1feac2b52422420078093d563df7306811bd966b277904d4ab0522222c2f55b7); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=2.6; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/020-api-versioner-v1-versions-title-37.json_section-2.68.json` as served (sha256 d424543583ec96f886e0882c5a4ef0b009024808e7793623d658e771365ccae5); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=2.68; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/021-api-versioner-v1-versions-title-37.json_section-2.88.json` as served (sha256 f398efe4ae636f678ab9f3ff310179ea17dff38893d7a7d329cc3d98711376cb); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=2.88; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/022-api-versioner-v1-versions-title-37.json_section-2.101.json` as served (sha256 55690d36a068e612d3be376204f62a6f09348df1def93b51dcb7a70c51e4f856); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=2.101; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/023-api-versioner-v1-versions-title-37.json_section-2.102.json` as served (sha256 cebb4ba1e7e3b5726dea84d9f63e2d87e473a228ec3fad3a8d9a4bfbcd2bc364); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=2.102; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/024-api-versioner-v1-versions-title-37.json_section-2.105.json` as served (sha256 8196196e26e9e069cc98745fcf25745f8c1d89f0b60b2694c8f4c9e033d6e923); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=2.105; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/025-api-versioner-v1-versions-title-37.json_section-2.106.json` as served (sha256 bafe6722400078cf4e3850cceb4b1d29103bbef827ca65187cf83f044339290e); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=2.106; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/026-api-versioner-v1-versions-title-37.json_section-2.111.json` as served (sha256 fb478d7488ffb7792802e803e75cee79a136c7701fce99de636281dce5e7f45b); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=2.111; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/027-api-versioner-v1-versions-title-37.json_section-2.113.json` as served (sha256 e37846c21e075a78123b969a47ebd61d86329f57978b4850b06a07aa5aaa07c0); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=2.113; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/028-api-versioner-v1-versions-title-37.json_section-2.120.json` as served (sha256 7abdbb0667ea2eaaa70359c0aa3cb31322564c946465be1a43ad99fcafe11fb3); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=2.120; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/029-api-versioner-v1-versions-title-37.json_section-2.121.json` as served (sha256 bd59901972a1e2b677dcba6440a8a1a0c42c64ed2e09c2918500fce51810f9c1); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=2.121; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/030-api-versioner-v1-versions-title-37.json_section-2.123.json` as served (sha256 aaccb79f28ab614449c4dea8cf13dcfb11e37ed7623852b9177b8899e6f6abb5); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=2.123; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/031-api-versioner-v1-versions-title-37.json_section-2.125.json` as served (sha256 6d928b5fd8ec7a6a2403373732bf3f87fe58420e117d140526ba13915fcec701); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=2.125; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/032-api-versioner-v1-versions-title-37.json_section-2.128.json` as served (sha256 84c1c21d2f4b29b9a74f282c7cbce3a59d43d1300977eaa25185e8bd692fe267); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=2.128; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/033-api-versioner-v1-versions-title-37.json_section-2.129.json` as served (sha256 46be73aa4f50e11629cfe8229278a767a4f587dc6aaad5c867e6928e4ab5f45d); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=2.129; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/034-api-versioner-v1-versions-title-37.json_section-2.133.json` as served (sha256 83b7d43f1b32283d04c6a66c8c12ef47df669234152394d60a533b8df55c7808); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=2.133; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/035-api-versioner-v1-versions-title-37.json_section-2.135.json` as served (sha256 57ea6363fff223478410917f2fbdea6cd1fa2362fb10f035ef894c44c79e1575); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=2.135; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/036-api-versioner-v1-versions-title-37.json_section-2.136.json` as served (sha256 3883d7604e57461bca0e5e369061981d122190f76c99826ec4879e44161d9143); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=2.136; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/037-api-versioner-v1-versions-title-37.json_section-2.145.json` as served (sha256 b8b24d0d7ec1cb15937362ee8757a8856587e90baf74d7b2a6fea0d8d34a007b); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=2.145; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/038-api-versioner-v1-versions-title-37.json_section-2.196.json` as served (sha256 155ecb2188b4f43e202e074a05847754add55d6f03cc4acb73153c254be56ccb); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=2.196; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/039-api-versioner-v1-versions-title-37.json_section-42.5.json` as served (sha256 ad8584b6f44f8258a42f04904bc65ad3e9b797c63f092a73bc25854f990b28c2); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=42.5; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/040-api-versioner-v1-versions-title-37.json_section-42.23.json` as served (sha256 cd4d5610fbb61e14540badd51f9bb8e69f98dfe5c6e2b811dbe2b11b6562852a); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=42.23; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/041-api-versioner-v1-versions-title-37.json_section-42.51.json` as served (sha256 96e272b43757047438bebd6040f3be92414702bdcb6a38cf072408e129ae8f59); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=42.51; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/042-api-versioner-v1-versions-title-37.json_section-42.54.json` as served (sha256 d536aac43c230b3829f992d6094b3352ee27b407f1d6a256421887458b16d2b5); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=42.54; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/043-api-versioner-v1-versions-title-37.json_section-42.70.json` as served (sha256 a992cb2349c371cd96924d00ccbae5007b1184c51d5dd88cbb5cbf7d25b5a1c5); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=42.70; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/044-api-versioner-v1-versions-title-37.json_section-42.71.json` as served (sha256 8821cdf86cc686b2cb114d91fa18c47c09c4862768303fdeae5d885bcc89f7f5); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=42.71; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/045-api-versioner-v1-versions-title-37.json_section-42.73.json` as served (sha256 a094844ec3755b2bfaae6307843923e5e71ec492392a173aedd3ed2d619d3918); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=42.73; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/046-api-versioner-v1-versions-title-37.json_section-42.74.json` as served (sha256 c027a7caba2e40c7fd62d67abd987160040fdc71e5b3e72fbad397a8388581ca); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=42.74; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/047-api-versioner-v1-versions-title-37.json_section-42.104.json` as served (sha256 2fb8632176a967fdbbfb54dda888edea6e206f4f8cda7cf8fbc45043d116d020); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=42.104; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/048-api-versioner-v1-versions-title-37.json_section-42.106.json` as served (sha256 ebfcec77e715a797a50c95f84387f892a75e1f9566b56f50df4106f4c5188d78); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=42.106; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/049-api-versioner-v1-versions-title-37.json_section-42.107.json` as served (sha256 d568d69556be8736e4317c4839fe5baf15335d3e12a60520769d0c7ef4ecc295); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=42.107; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/050-api-versioner-v1-versions-title-37.json_section-42.108.json` as served (sha256 7e1d5e33e5c00dafa685a99cd18769674f5c5d62a7f8de1fde20d68decadb5b8); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=42.108; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/051-api-versioner-v1-versions-title-37.json_section-42.120.json` as served (sha256 8a94bc1d501668bcacbb20e1f0328f21baf83bf23605c6a5f70639bd491c4269); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=42.120; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/052-api-versioner-v1-versions-title-37.json_section-42.121.json` as served (sha256 fd40f74f4930ffbdfc57a64fd311b2d540a70692afb44d56885c862bf6e66567); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=42.121; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/053-api-versioner-v1-versions-title-37.json_section-42.122.json` as served (sha256 97b014134fa28a48101dd37780b44c417558f5216b61fdca8cd493dfefcc7f8f); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=42.122; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/054-api-versioner-v1-versions-title-37.json_section-42.204.json` as served (sha256 1e5dac78eda9bb275a1f82817a7a2da8811c8c28c6a0b2e3470d7f0526adb440); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=42.204; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/055-api-versioner-v1-versions-title-37.json_section-42.206.json` as served (sha256 4e5bb6f2549185ec1066db76a6825964ad85208ceecb52d8c989e4242c24b018); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=42.206; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/056-api-versioner-v1-versions-title-37.json_section-42.207.json` as served (sha256 492b8169f500af5691e238c57d13fc70a5bd49fb1bee44f0bdfc55519c354619); extraction sha256 083c3103173ebdc82ab3caa6b704469dd825b22ddfe1419ff8855d4f56d42058. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=42.207; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.

### `scotus-rules.txt` — Rules of the Supreme Court of the United States 13, 29 and 30 (adopted February 17, 2026, effective March 16, 2026)

- The cert-petition period and its extension (Rule 13), service (Rule 29)
  and computation and extension of time (Rule 30) `lit/core`'s `sup-ct-30`
  counting rule and the FRAP and Federal Circuit cert periods cite, one
  `## Sup. Ct. R. N.P` section per numbered paragraph, each restating its
  rule's heading as its first line.
- Fetched 2026-10-24 from `www.supremecourt.gov/filingandrules/2026RulesoftheCourt_WEB.pdf`
  (sha256 `5200e38d590cc40929cb719987c936dc529717b4037259d1b63a2309cb6b0f32`),
  the edition the Court's Rules page lists as current, `pdftotext -layout`,
  running headers and page numbers stripped, lines joined into one line
  per numbered paragraph. The text layer keeps the "fi" and "fl"
  ligatures as U+FB01 and U+FB02; they are spelled out. A line-break
  hyphen was dropped when the joined word appears in the 2023 edition's
  text, and kept when the hyphenated word does (one: "cross-petition" in
  13.4); the five others were dropped by reading ("subsections" and
  "submission", new in 29.2, and "identifiable", "office", "officer",
  split at a ligature). No other wording changed.
- Against the 2023 edition (`2023RulesoftheCourt.pdf`, adopted December 5,
  2022, effective January 1, 2023, sha256
  `05f435c766fd596333828846d097828238ff89a266c24e3afca9ecd2fb9c19db`,
  vendored here before as sha256
  `94c56e4ee8ec59d5cf857f1271ee689de1568421ab67e19b71bb823b5feecbb4`),
  Rules 13 and 30 and Rule 29's paragraphs 1, 3, 4, 5 and 7 are word for
  word the same; 29.2 (an electronic submission is timely, paper copies
  within three days) and 29.6 (stock ticker symbols) are the only changes,
  as the Court's "Revisions to Rules of the Supreme Court" of the same
  date (`www.supremecourt.gov/filingandrules/2026_Rules_Revisions.pdf`,
  sha256 `b05a3a77c71708fb390e8e90ad2720cca2b0dc9fb827294b2d4f4ab00b56292f`,
  revising Rules 14, 15, 24, 29 and 48 only) states. So 30.1's computation
  and 13.5's and 30.2's extension timing read the same from January 1,
  2023.
- sha256 (this file): `881464325decc0ce5cc63ae4bac10facc84766e266dbff30454186f78dc3d5f2`
- Original: `original/scotus-rules.pdf` as served (sha256 5200e38d590cc40929cb719987c936dc529717b4037259d1b63a2309cb6b0f32); extraction sha256 881464325decc0ce5cc63ae4bac10facc84766e266dbff30454186f78dc3d5f2. Served at https://www.supremecourt.gov/filingandrules/2026RulesoftheCourt_WEB.pdf; retrieved 2026-10-24 (fetched as recorded above).

### `scotus-rules-2023.txt` — Rules of the Supreme Court 29.3 and 30.1 (adopted December 5, 2022, effective January 1, 2023)

- The service and computation-of-time paragraphs `lit/core`'s 2023 version
  of `sup-ct-30` cites: `## Sup. Ct. R. (2023) 29.3`, `## Sup. Ct. R.
  (2023) 30.1`, each restating its rule's heading as its first line.
- Fetched 2026-10-27 from `www.supremecourt.gov/filingandrules/2023RulesoftheCourt.pdf`
  (sha256 `05f435c766fd596333828846d097828238ff89a266c24e3afca9ecd2fb9c19db`),
  `pdftotext -layout`, running headers and page numbers stripped, lines
  joined into one line per paragraph, line-break hyphens joined. The
  text layer drops the "fi" ligature; it was restored word by word (filing,
  file, suffices, office, first-class, identifiable). No other wording
  changed. Both paragraphs read word for word as in the 2026 Rules
  (`scotus-rules.txt`).
- sha256 (this file): `c83977038d4c8be8e5d79f492e532c289ff112202db06f2784b994ca524f575b`
- Original: `original/scotus-rules-2023.pdf` as served (sha256 05f435c766fd596333828846d097828238ff89a266c24e3afca9ecd2fb9c19db); extraction sha256 c83977038d4c8be8e5d79f492e532c289ff112202db06f2784b994ca524f575b. Served at https://www.supremecourt.gov/filingandrules/2023RulesoftheCourt.pdf; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.

### `usc-15-lanham.txt` — Lanham Act sections (12 sections of 15 U.S.C.)

- Source: the USPTO compilation "US Trademark Law - Rules of Practice and
  Federal Statutes" dated 2026-07-15, `pdftotext -layout`, then the sections
  15 U.S.C. §§ 1051, 1052, 1057, 1063, 1064, 1068, 1071, 1114, 1116, 1117, 1125
  and 1127 cut at the `§ N (15 U.S.C. § NNNN)` heading lines (form feeds and
  running page headers removed). No wording changed. Used by the TTAB pack.
- sha256 (this file): `9ea6b13b33cf0bb61a2a1ea337097c0981665d4d30a045fece2a77324dae6c4c`
- Original: `original/usc-15-lanham.pdf` as served (sha256 58805be56f2dec79c4220800fb4c6d4207a17fc4dd3277bd44b859ec7e3b2830); extraction sha256 9ea6b13b33cf0bb61a2a1ea337097c0981665d4d30a045fece2a77324dae6c4c. Served at https://www.uspto.gov/sites/default/files/documents/tmlaw.pdf; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.

### `tbmp.txt` — Trademark Trial and Appeal Board Manual of Procedure §§ 112, 113.05 and 509 (June 2026)

- The TBMP sections the TTAB package's chained briefing periods (§ 112:
  a period running from a prior due date counts from "the actual due date
  ..., not the adjusted date under 37 C.F.R. § 2.196") and its service rule
  (§ 113.05: no added time after service by mail) cite, and § 509 (motions
  to extend or reopen time, 509 through 509.02: FRCP 6(b) "made applicable
  to Board proceedings by 37 C.F.R. § 2.116(a)"), which the answer-time
  extension cites as practice guidance, one `## TBMP § N` section each
  (each numbered subsection its own), the section's own heading as its
  first line. § 509 opens with the rule texts the manual reprints (FRCP
  6(b), 37 C.F.R. §§ 2.120(a), 2.121, 2.127(a)), kept as printed.
- Fetched 2026-10-23 from
  `www.uspto.gov/sites/default/files/documents/tbmp-Master-June2026.pdf`
  (sha256 `015cf2b665d74b25080321f67dad7ffebe412118f6f0460ec3530f4801639b8e`,
  the digest `lit/ttab/Lit.toml` records for its `tbmp` source),
  `pdftotext` (no `-layout`); running page headers and page numbers
  removed and each section's NOTES (its footnoted authorities) omitted. No
  wording changed. The TBMP binds no one: it restates the Board's practice
  under 37 C.F.R. Part 2. § 509 was cut the same way on 2026-10-24 from
  the same PDF (digest checked again), with form feeds dropped.
- sha256 (this file): `39df33edc821a318331129cdc6730ac961cd962d7d3f3f1c758309465eb21d86`
- Original: `original/tbmp.pdf` as served (sha256 015cf2b665d74b25080321f67dad7ffebe412118f6f0460ec3530f4801639b8e); extraction sha256 39df33edc821a318331129cdc6730ac961cd962d7d3f3f1c758309465eb21d86. Served at https://www.uspto.gov/sites/default/files/documents/tbmp-Master-June2026.pdf; retrieved 2026-10-23 (fetched as recorded above).

### `fr-uspto-90-3-history.txt` — 37 C.F.R. § 90.3 as added in 2012 (77 FR 48612, 48677-48678)

- The text of § 90.3 in force from September 16, 2012 to March 29, 2024,
  which `usc-cfr-prosecution-fragments.txt` (the current text) cannot
  give: one `## 77 FR <page>` section per printed page, each opening with
  a bracketed line naming the issue, page and FR Doc. number; everything
  after it is the printed text, verbatim except for line joining, `Sec.`
  printed as `§`, the GPO text's doubled backtick and apostrophe quotes
  as straight double quotes, and its `----` as an em dash. Page 48612 carries the rule's summary and effective date;
  48677-48678 the amendatory instruction adding Part 90 and § 90.3 in
  full, including the verbatim "appeal of commencing" in (c)(1)(ii).
- Fetched 2026-10-24 from `www.govinfo.gov`'s Federal Register HTML
  rendering of FR Doc. 2012-17900
  (`https://www.govinfo.gov/content/pkg/FR-2012-08-14/html/2012-17900.htm`,
  sha256 `230a9429a1acfe78b073b0d75d9a2a08e8ecfa8eecfce3a22ab8f163e82aaae9`),
  whose `[[Page N]]` markers give the page.
- How the version list was found: the federalregister.gov API's rules
  with a CFR reference to 37 C.F.R. Part 90 are exactly two, 77 FR 48612
  (2012-17900, effective 2012-09-16) and 89 FR 22084 (2024-06659,
  effective 2024-03-29), matching the current section's history line.
- sha256 (this file): `86152eebd93a8b638b25c9263e235194c7400797d2840e73d59da0977e4f4d74`
- Original: `original/fr-uspto-90-3-history.html` as served (sha256 230a9429a1acfe78b073b0d75d9a2a08e8ecfa8eecfce3a22ab8f163e82aaae9); extraction sha256 86152eebd93a8b638b25c9263e235194c7400797d2840e73d59da0977e4f4d74. Served at https://www.govinfo.gov/content/pkg/FR-2012-08-14/html/2012-17900.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.

### `change-frcv-2007-restyled-excerpt.txt` — FRCP as restyled effective December 1, 2007: the order and Rules 6(b), 59(e)

- The Supreme Court's order of April 30, 2007 and the restyled text of Rule 6(b) and Rule 59(e), the lg-fq2.1 base example's pre-2009 Rule 59(e) (10 days, locked by 6(b)(2)). Lines taken from `sources/change-corpus/rules/scotus/frcv07p.txt` (the supremecourt.gov order PDF `frcv07p.pdf`, see that corpus's MANIFEST.json), blank lines and running page headers dropped; no wording changed. Sections `## FRCP (2007) Order`, `## FRCP (2007) Rule 6`, `## FRCP (2007) Rule 59`.
- sha256 (this file): `a7287e6fd94acc165ac1eb4a5cdd5c9f404acc3abfe01182cbfb490c3d766f6e`
- Original: `original/change-frcv-2007-restyled-excerpt.pdf` as served (sha256 f43e6e62aab27c02e357ece586b499b54f83bae7496073a6f6bfd4a6e8c3220d); extraction sha256 a7287e6fd94acc165ac1eb4a5cdd5c9f404acc3abfe01182cbfb490c3d766f6e. Served at https://www.supremecourt.gov/orders/courtorders/frcv07p.pdf; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.

### `change-frcv-2009-amendments-excerpt.txt` — FRCP amendments effective December 1, 2009: the order and Rules 6(b)(2), 59(e)

- The Supreme Court's order of March 26, 2009 (with its pending-case clause) and the amended text of Rule 6(b)(2) and Rule 59(e) (28 days), the lg-fq2.1 amendment example's instrument. Lines taken from `sources/change-corpus/rules/scotus/frcv09.txt` (supremecourt.gov `frcv09.pdf`), blank lines dropped; no wording changed.
- sha256 (this file): `0eb155fa9a5911540a1b4a5f1784c944c235f6fd5fa9fe8da29ba716cc6b9690`
- Original: `original/change-frcv-2009-amendments-excerpt.pdf` as served (sha256 966fc6224e82c4bee3dbe99b247277d44eebf660e942b0879e15af6476864ab5); extraction sha256 0eb155fa9a5911540a1b4a5f1784c944c235f6fd5fa9fe8da29ba716cc6b9690. Served at https://www.supremecourt.gov/orders/courtorders/frcv09.pdf; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.

### `change-frcv-2007-time-periods-excerpt.txt` — FRCP as restyled effective December 1, 2007: the order and Rules 6(b)(2), 23(f), 50(b), 59(b), 59(e)

- The Supreme Court's order of April 30, 2007 and the restyled text of the four post-judgment periods the 2009 time amendments changed and the timeliness rulings count (10 days each) and Rule 6(b)(2), which forbids extending all but 23(f), the base of the deadline domain's change files (lg-fq2.6, `docs/litigate/change/frcp-base`). Lines taken from `sources/change-corpus/rules/scotus/frcv07p.txt` (supremecourt.gov `frcv07p.pdf`, see that corpus's MANIFEST.json), leading spaces, blank lines and running page headers dropped, runs of spaces collapsed; no wording changed. Each `##` section opens with one descriptive line naming the subdivisions it holds.
- sha256 (this file): `1401feb1cffb5ad4e36a8d11073e200c638b86a4e947771d2cce2e3c144cf05e`
- Original: `original/change-frcv-2007-time-periods-excerpt.pdf` as served (sha256 f43e6e62aab27c02e357ece586b499b54f83bae7496073a6f6bfd4a6e8c3220d); extraction sha256 1401feb1cffb5ad4e36a8d11073e200c638b86a4e947771d2cce2e3c144cf05e. Served at https://www.supremecourt.gov/orders/courtorders/frcv07p.pdf; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.

### `change-frcv-2009-time-periods-excerpt.txt` — FRCP amendments effective December 1, 2009: the order and Rules 6(b)(2), 23(f), 50(b), 59(b), 59(e)

- The Supreme Court's order of March 26, 2009 (with its pending-case clause) and the amended text of the same four periods (14 days for 23(f), 28 for the others) and Rule 6(b)(2), the 2009 amendment layer of the deadline domain's change files (lg-fq2.6, `docs/litigate/change/frcp-2009`). Lines taken from `sources/change-corpus/rules/scotus/frcv09.txt` (supremecourt.gov `frcv09.pdf`), prepared as the 2007 excerpt; no wording changed (the PDF's "entry ofjudgment" is kept as extracted).
- sha256 (this file): `7c07441c623854e91e225b5adad541f96c7375637999e9f998df7387cd63f173`
- Original: `original/change-frcv-2009-time-periods-excerpt.pdf` as served (sha256 966fc6224e82c4bee3dbe99b247277d44eebf660e942b0879e15af6476864ab5); extraction sha256 7c07441c623854e91e225b5adad541f96c7375637999e9f998df7387cd63f173. Served at https://www.supremecourt.gov/orders/courtorders/frcv09.pdf; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.

### `change-frap-2005-2009-rule-4-excerpt.txt` — FRAP 4(a)(6) as amended effective December 1, 2005 and December 1, 2009, with the orders

- The Supreme Court's orders of April 25, 2005 and March 26, 2009 amending the FRAP, and the amended text of Rule 4(a)(6): in 2005 the 7-day window to move to reopen runs from notice "under Federal Rule of Civil Procedure 77(d)"; in 2009 it is 14 days. The FRAP amendment layers of the deadline domain's change files (lg-fq2.6, `docs/litigate/change/frap-2005`, `frap-2009`). Lines taken from `sources/change-corpus/rules/scotus/frap05p.txt` and `frap09.txt` (supremecourt.gov), prepared as the FRCP excerpts; the 2009 section keeps the paragraph's opening words and (B), the only part the 2009 order changed in (a)(6). No wording changed.
- sha256 (this file): `1b8b10906ed7f5b9f02ebf846440365f156877a62abce46aded271b456b57059`
- Original: `original/change-frap-2005-2009-rule-4-excerpt/001-orders-courtorders-frap05p.pdf` as served (sha256 59c5e88e9ee31ae112f365ba28362f6c0e9ab027b030e894f13451a4237239d1); extraction sha256 1b8b10906ed7f5b9f02ebf846440365f156877a62abce46aded271b456b57059. Served at https://www.supremecourt.gov/orders/courtorders/frap05p.pdf; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/change-frap-2005-2009-rule-4-excerpt/002-orders-courtorders-frap09.pdf` as served (sha256 e0165d121f8bad45d3f3e9b97bcbea6f24fcd80146c38739ec9ca9ff20d5f46b); extraction sha256 1b8b10906ed7f5b9f02ebf846440365f156877a62abce46aded271b456b57059. Served at https://www.supremecourt.gov/orders/courtorders/frap09.pdf; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.

### `change-sdny-edny-local-civil-rule-6.txt` — S.D.N.Y./E.D.N.Y. Joint Local Civil Rules 6.1 and 6.4 (effective January 2, 2026)

- Fetched 2026-10-26 from `www.nysd.uscourts.gov/sites/default/files/local_rules/2026-01-02%20-%20EDNY%20and%20SDNY%20Joint%20Local%20Rules%20As%20Amended.pdf` (sha256 `439b9e2c67fd4ca1d1be65a7f28ca53707820bcbabb4b459323ade1a773607af`), `pdftotext -layout`, the text of Local Civil Rule 6.1 and the withdrawn 6.4 with leading spaces and blank lines dropped; no wording changed. The local-rule example's instrument.
- sha256 (this file): `d7ea4a0e259c8609c98efe869d04776cb7e9ca119c701c3760679a01e3110aae`
- Original: `original/change-sdny-edny-local-civil-rule-6.pdf` as served (sha256 439b9e2c67fd4ca1d1be65a7f28ca53707820bcbabb4b459323ade1a773607af); extraction sha256 d7ea4a0e259c8609c98efe869d04776cb7e9ca119c701c3760679a01e3110aae. Served at https://www.nysd.uscourts.gov/sites/default/files/local_rules/2026-01-02%20-%20EDNY%20and%20SDNY%20Joint%20Local%20Rules%20As%20Amended.pdf; retrieved 2026-10-26 (fetched as recorded above).

### `change-cofc-general-order-2025-02.txt` — U.S. Court of Federal Claims General Order No. 2025-02 (Oct. 6, 2025)

- The order's operative text as quoted verbatim in the lg-fq2.1 research register (fetched 2026-10-23 from `www.uscfc.uscourts.gov/sites/cfc/files/general-orders/2025.10.06%20General%20Order%20re%202025%20day%20after%20Thanksgiving.pdf`); a command-line fetch on 2026-10-26 was refused by the court's firewall, so the PDF itself is not hashed here. The general-order example's instrument.
- sha256 (this file): `3928c4ed72588200af3a643584ab3456806bedd3acf116a6d4a463df52ca0a27`
- Original: `original/change-cofc-general-order-2025-02.pdf` as served (sha256 5b8c1c8cfee7ae10f041d0d6155a1663209416db7b7cc4d56c7c9f5d29e64906); extraction sha256 3928c4ed72588200af3a643584ab3456806bedd3acf116a6d4a463df52ca0a27. Served at https://www.uscfc.uscourts.gov/sites/cfc/files/general-orders/2025.10.06%20General%20Order%20re%202025%20day%20after%20Thanksgiving.pdf; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.

### `change-breyer-general-standing-order-2025.txt` — Judge Charles R. Breyer, General Standing Order (N.D. Cal., rev. July 16, 2025), two paragraphs

- The courtesy-copy and scheduling paragraphs as quoted verbatim in the lg-fq2.1 research register (fetched 2026-10-23 from `cand.uscourts.gov/sites/default/files/standing-orders/General-Standing-Order-Judge-Charles-R.-Breyer-rev-7-16-2025.pdf`); a command-line fetch on 2026-10-26 was refused by the court's firewall. The standing-order example's instrument.
- sha256 (this file): `a36186c206ae59a2578f63e45ba8af6c998a4dcdbc27da1da0729aab4b315283`
- Original: `original/change-breyer-general-standing-order-2025.pdf` as served (sha256 c3782c60e8fdf2c582eae5b8cd814d01777acc36ae32b56b0f9aab07f8a89f9b); extraction sha256 a36186c206ae59a2578f63e45ba8af6c998a4dcdbc27da1da0729aab4b315283. Served at https://cand.uscourts.gov/sites/default/files/standing-orders/General-Standing-Order-Judge-Charles-R.-Breyer-rev-7-16-2025.pdf; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.

### `change-cancino-perez-d-colo-ecf-45.txt` — Cancino-Perez v. McCoy Global, Inc., No. 1:25-cv-00090 (D. Colo.), ECF No. 45 (July 10, 2025), page 1

- Fetched 2026-10-26 from `www.govinfo.gov/content/pkg/USCOURTS-cod-1_25-cv-00090/pdf/USCOURTS-cod-1_25-cv-00090-0.pdf` (sha256 `bb04e7bb83ecd5df14b961703ff01dab86f715f45778048bb192391c76b896bc`), `pdftotext -layout`, the caption and opening paragraphs (which recite the scheduling order's June 20, 2025 deadline, ECF No. 42 § 9.a) with blank lines dropped; no wording changed. A public court record; the case-order example's instrument.
- sha256 (this file): `dbb6f366760d5397318eb4284f3d0cb7ced95e87b16651c1a9d165d442b002c9`
- Original: `original/change-cancino-perez-d-colo-ecf-45.pdf` as served (sha256 bb04e7bb83ecd5df14b961703ff01dab86f715f45778048bb192391c76b896bc); extraction sha256 dbb6f366760d5397318eb4284f3d0cb7ced95e87b16651c1a9d165d442b002c9. Served at https://www.govinfo.gov/content/pkg/USCOURTS-cod-1_25-cv-00090/pdf/USCOURTS-cod-1_25-cv-00090-0.pdf; retrieved 2026-10-26 (fetched as recorded above).

### `change-cfr-37-42-5-71-75-2026-10-20.txt` — 37 C.F.R. §§ 42.5, 42.71, 42.75 (eCFR, as of 2026-10-20)

- Fetched 2026-10-26 from the eCFR versioner API (`www.ecfr.gov/api/versioner/v1/full/2026-10-20/title-37.xml?section=42.5`, `...=42.71`, `...=42.75`), XML tags stripped, one line per paragraph; no wording changed. Headings `## 37 C.F.R. (2026) § N` keep them apart from `cfr-37-part42.txt`, which predates § 42.75. The PTAB base example's text.
- sha256 (this file): `9fda502b4d2026074b411da4c2556222780ccda8154a57275373b42faa9628ee`
- Original: `original/change-cfr-37-42-5-71-75-2026-10-20/001-CFR-2025-title37-vol1-part42.xml` as served (sha256 77256147aecfeded3601dc9b424fa11979b08755beafb7e544cde110da7654ec); extraction sha256 9fda502b4d2026074b411da4c2556222780ccda8154a57275373b42faa9628ee. Served at https://www.govinfo.gov/content/pkg/CFR-2025-title37-vol1/xml/CFR-2025-title37-vol1-part42.xml; this is the govinfo annual CFR edition of 2025-07-01, not the eCFR point-in-time bytes the extraction was taken from; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original coverage (lg-f3q8.82, 2026-10-27): ecfr.gov's robots.txt disallows the point-in-time URL (`/api/versioner/v1/full/`). The official original is the govinfo annual edition, title 37 revised as of 2025-07-01 (the 2026 edition is not yet published there); it is not a 2026-10-20 point-in-time copy, so amendments after 2025-07-01 would not show. Sections 42.5, 42.71 and 42.75 are word for word identical to this extraction, so none changed after that date.

### `change-light-wonder-ipr2025-01072-paper-30.txt` — Light & Wonder, Inc. v. Evolution Malta Ltd., IPR2025-01072, Paper 30 (Director June 22, 2026), excerpt

- Fetched 2026-10-26 from `www.uspto.gov/sites/default/files/documents/ipr2025-01072-dr-paper_30.pdf` (sha256 `5cba76995e71eae32656892dde2c1e92265f9bac30be3254f51e48e4f1407c80`), `pdftotext -layout`, the caption and the waiver paragraph (pages 1 and 3-4), footnote markers omitted; no wording changed. Designated precedential June 22, 2026 (PTAB precedential and informative decisions page, per the research register). The precedent example's instrument.
- sha256 (this file): `1e32fb02af2ee2caeb5956f0fb21a93eb5c2ffa6dc3289ec4a73c5902df4edc5`
- Original: `original/change-light-wonder-ipr2025-01072-paper-30.pdf` as served (sha256 5cba76995e71eae32656892dde2c1e92265f9bac30be3254f51e48e4f1407c80); extraction sha256 1e32fb02af2ee2caeb5956f0fb21a93eb5c2ffa6dc3289ec4a73c5902df4edc5. Served at https://www.uspto.gov/sites/default/files/documents/ipr2025-01072-dr-paper_30.pdf; retrieved 2026-10-26 (fetched as recorded above).

### `change-thompson-v-ins-375-us-384.txt` — Thompson v. INS, 375 U.S. 384 (1964) (per curiam)

- Fetched 2026-10-26 from `tile.loc.gov/storage-services/service/ll/usrep/usrep375/usrep375384/usrep375384.pdf` (the U.S. Reports, Library of Congress), `pdftotext -layout`, every non-blank line kept with its leading spaces dropped; no wording changed. The precedent the overruling example (Bowles v. Russell, `sources/change-corpus/rulings/bowles-scotus-2007.txt`) withdraws.
- sha256 (this file): `adb3b1d49b1a12c14f58d973942a3dc62825bb5287c56df17f4ed0eac64f9cc9`
- Original: `original/change-thompson-v-ins-375-us-384.pdf` as served (sha256 a91fe042e81e3bb1609ae0d0e98e9dff546f9dd5747cc0b85ead8a4eb6b52be5); extraction sha256 adb3b1d49b1a12c14f58d973942a3dc62825bb5287c56df17f4ed0eac64f9cc9. Served at https://tile.loc.gov/storage-services/service/ll/usrep/usrep375/usrep375384/usrep375384.pdf; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
### Post-dismissal opinions (`waetzig-v-halliburton.txt`, `crst-v-eeoc.txt`, `cadkin-v-loose.txt`)

- Waetzig is the Supreme Court's official slip opinion (No. 23-971, decided
  2025-02-26), downloaded from supremecourt.gov and extracted with
  `pdftotext -layout`; sha256 `4325f20a34f4be3e51cc2b96eea4ef6ca20b8e50a1834ad8b5d51701853789ff`.
- CRST is U.S. Reports vol. 578, pp. 419-436, downloaded from GovInfo and
  extracted with `pdftotext -layout`; sha256
  `3598727ae67d2b2a67986920aedf82b13435dadf4e1ab45a5b915c69f3a34b40`.
- Cadkin is the Ninth Circuit's official published opinion, No. 08-55311,
  downloaded from ca9.uscourts.gov and extracted with `pdftotext -layout`;
  sha256 `52a8a2fa29edf0fed607bda629e6e8788a22e7b272d16ffd31f1fff4274e106b`.

### Eighth Circuit Bankruptcy Appellate Panel source set (retrieved 2026-10-27)

- `frbp.txt` — Federal Rules of Bankruptcy Procedure, December 1, 2025
  edition. Extracted from the Administrative Office of the U.S. Courts official
  PDF at `https://www.uscourts.gov/sites/default/files/document/federal-rules-of-bankruptcy-procedure.pdf`
  with `pdftotext -layout`; headings were segmented by rule number and the
  extracted rule text retained. This is the current edition listed on the
  official current-rules page, which identifies the 2025 amendments effective
  December 1, 2025. sha256: `0c8e487aa7093bcee260d9037c459344f489b4096d3cdd91177ebb2d76c403d8`.
- `ca8-bap-local-rules.txt` — Eighth Circuit Bankruptcy Appellate Panel Local
  Rules, revised December 1, 2023. Extracted from the official court PDF at
  `https://ecf.ca8.uscourts.gov/newbap/rules/localrules_bapDec2023.pdf` with
  `pdftotext -layout`; each local rule is headed by its canonical rule
  reference. The official Panel Rules and Publications page lists this edition.
  sha256: `ccc1e2198d478e0b31cd6c4d700bd699eb7e4993f073b3b894cb2f0cf48108ca`.
- `ca8-bap-iop.txt` — Eighth Circuit Bankruptcy Appellate Panel Internal
  Operating Procedures Manual, revised January 2024. Extracted from the
  official court PDF at
  `https://www.ca8.uscourts.gov/sites/ca8/files/iops%20jan%202024.pdf` with
  `pdftotext -layout`. sha256:
  `2fb5815bf87b25d8af30298fa0fa21e148d81ddc6992c305bd94716c68fb0485`.
- `ca8-local-rules.txt` — Eighth Circuit Local Rules, June 17, 2024. Extracted
  from the official court PDF at
  `https://www.ca8.uscourts.gov/sites/ca8/files/Eighth%20Circuit%20Local%20Rules%206.17.24.pdf`
  with `pdftotext -layout`; the Rules & Procedures page lists this edition.
  sha256: `74b1a57311b9f597caac5a9778ede3e8328b6076d816e5a19eb62c3bfd14f907`.
- `ca8-iop.txt` — Eighth Circuit Internal Operating Procedures, extracted from
  the official court PDF at `https://ecf.ca8.uscourts.gov/newrules/coa/iops.pdf`
  with `pdftotext -layout`; the Rules & Procedures page links this edition.
  sha256: `a346115379cdf130c46d5d6c33462f03a0984fcebac95a8617b2d0222e6bc8f8`.
- `usc-28-158.txt` — 28 U.S.C. § 158, extracted from the official Office of the
  Law Revision Counsel page at `https://uscode.house.gov/us/usc/119-118/t28/s158/`
  (text current through Public Law 119-120, October 7, 2026). HTML navigation
  removed; statutory text retained. sha256:
  `a0f615748b6a10805e6eeec6a05e94d7a3fc2dcd5fafbe94efae7af8ebf9987b`.
- Leading decisions named in the outline were checked against their published
  United States Reports: *Catlin v. United States*, 324 U.S. 229 (1945),
  official Library of Congress scan at
  `https://tile.loc.gov/storage-services/service/ll/usrep/usrep324/usrep324229/usrep324229.pdf`,
  and *Bowles v. Russell*, 551 U.S. 205 (2007), official scan at
  `https://tile.loc.gov/storage-services/service/ll/usrep/usrep551/usrep551205/usrep551205.pdf`.
  They are omitted from the promoted package: *Catlin* addresses the finality
  rule under the former 28 U.S.C. § 1291, not the bankruptcy appellate routes
  modeled here; *Bowles* holds the statutory civil notice period in 28 U.S.C.
  § 2107 jurisdictional, while the modeled bankruptcy notice period is set by
  FRBP 8002. Neither decision supports a proposition needed by this outline.

## First Circuit Rulebook

### `ca1-first-circuit-rulebook-2026-06-01.txt`

- Official First Circuit Rulebook, including the Federal Rules of Appellate
  Procedure, First Circuit Local Rules, Internal Operating Procedures and
  administrative materials, amendments through June 1, 2026:
  <https://www.ca1.uscourts.gov/sites/ca1/files/rulebook.pdf>
- Retrieved 2026-10-27. Extracted with `pdftotext -layout`; section markers
  (`## FRAP N`, `## 1st Cir. R. N`, `## 1st Cir. IOP N`, and the fee-schedule
  item marker) were added for
  cite resolution. Rule text was not edited.
- Original PDF sha256:
  `e126dc83b8a7ab1044273dc3fdc22cbc735cc6c81ce5793b345eb0e7bc7a6133`.
- Vendored text sha256:
  `9e99b6ae551dc12ddf322afe896c98d7fc4c651c49a209f0cf45f313403b477a`.
- Original: `original/ca1-first-circuit-rulebook-2026-06-01.pdf` as served (sha256 e126dc83b8a7ab1044273dc3fdc22cbc735cc6c81ce5793b345eb0e7bc7a6133); extraction sha256 9e99b6ae551dc12ddf322afe896c98d7fc4c651c49a209f0cf45f313403b477a.
