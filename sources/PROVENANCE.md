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

### `frcp.txt` — Federal Rules of Civil Procedure (98 rules), December 1, 2025

- Ported from `civ-pro-the-gathering/content/reference/frcp-full.json`
  (sibling project, same author, also public-domain rule text with its own
  provenance block: "official 2024 committee print... extracted locally
  (pdftotext) from David's library copy", retrieved 2026-07-26, canonical
  `https://www.uscourts.gov/rules-policies/current-rules-practice-procedure`).
  Re-formatted into this file's `## Rule N` section convention; no wording
  changed.
- Re-vendored (lg-f3q8.83, 2026-10-28) to the edition of December 1, 2025,
  the House Judiciary Committee print below, which adds the amendments of
  the order of April 23, 2025, effective December 1, 2025. Only `## Rule
  16` and `## Rule 26` were edited, each to the print's wording (from
  `pdftotext -layout` of the original; compared word for word with the
  print afterwards, running heads and GPO hyphenation ignored, no
  difference): Rule 16(b) is retitled "SCHEDULING AND MANAGEMENT."
  (missed by the lg-f3q8.82 comparison, which ignored title lines);
  16(b)(3)(B)(iv) and 26(f)(3)(D) gain "the timing and method for
  complying with Rule 26(b)(5)(A) and"; both credits gain "Apr. 23, 2025,
  eff. Dec. 1, 2025". `## Rule 16.1` (Multidistrict Litigation, "As
  added Apr. 23, 2025, eff. Dec. 1, 2025") is new, from the same
  extraction with the running heads dropped. Every other section is
  byte-identical to the December 1, 2024 extraction (sha256
  `cb9a19a0ad99acdbd04f46acd3893bb96fbd34c9a8f66de57f5d171dbd268b44`).
  `lit/frcp` cites this file as of 2024-12-01 (none of its cites names an
  amended provision); `lit/frcp-2025`, the amendment layer, cites it as
  `FRCP (2025)`, as of 2025-12-01.
- sha256 (this file): `527fb9fbfe1665bdbdc5565ea5614b299e16b07b07edba803aae6eb5f957b990`
- Original: `original/frcp.pdf` as served (sha256 bd8705fc038d87e4fe222a7ea2e4324222c9430e2373fce56826bd2dfa2f8baf); extraction sha256 527fb9fbfe1665bdbdc5565ea5614b299e16b07b07edba803aae6eb5f957b990. Served at https://www.uscourts.gov/sites/default/files/document/federal-rules-of-civil-procedure.pdf; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Comparison (lg-f3q8.82, 2026-10-27, against the December 1, 2024 extraction): the served file is the House Judiciary Committee print "Federal Rules of Civil Procedure, December 1, 2025" (142 pages). Each of the 97 vendored rules was compared word for word with the print's text (page headers, title lines and GPO hyphenation ignored). The vendored text predated the amendments effective December 1, 2025 (order of April 23, 2025, which "affected Rules 16 and 26, and added new Rule 16.1"). Differences: (1) Rule 16(b)(3)(B)(iv) gains the words "the timing and method for complying with Rule 26(b)(5)(A) and" before "any agreements the parties reach for asserting claims of privilege or of protection as trial-preparation material after information is produced, including agreements reached under Federal Rule of Evidence 502"; (2) Rule 26(f)(3)(D) gains the same words ("...method for complying with Rule 26(b)(5)(A) and—if the parties agree on a procedure..."); (3) Rule 16.1 (Multidistrict Litigation) was absent; (4) the amendment credits of Rules 16 and 26 gain "Apr. 23, 2025, eff. Dec. 1, 2025" and Rule 16.1's reads "As added Apr. 23, 2025, eff. Dec. 1, 2025". No other rule's wording differs. Re-vendored by lg-f3q8.83 (above).

### `fre.txt` — Federal Rules of Evidence (69 rules), as amended to December 1, 2024

- The House Judiciary Committee print "Federal Rules of Evidence, December
  1, 2024" (Committee Print No. 11, 118th Cong., 2d Sess.; 44 pages), as
  served by `www.uscourts.gov`, fetched 2026-10-28 with curl (User-Agent
  lit-lang-provenance/1.0) for `lit/frcp`'s FRE cites (lg-pd9k).
  `pdftotext -layout`; from the first `Rule 101.` heading after the
  contents, one `## FRE N` section per `Rule N. Title` line (the line kept
  as the section's first line), page headers (`N  FEDERAL RULES OF
  EVIDENCE  Rule N`), `(1)` page numbers and the printer's mark dropped,
  leading indentation stripped; lines, end-of-line hyphens and each
  rule's "(As amended ...)" credit kept as printed. No wording changed.
- sha256 (this file): `151363e8d73035fb2ff533fafe366d102d4d2a756f512d868edab1d8d5b03b32`
- Original: `original/fre.pdf` as served (sha256 6d40b9dca62b1acb0d7e762473b6fad7779ef58b6cf19384575210af81a444ad); extraction sha256 151363e8d73035fb2ff533fafe366d102d4d2a756f512d868edab1d8d5b03b32. Served at https://www.uscourts.gov/sites/default/files/2025-02/federal-rules-of-evidence-dec-1-2024_0.pdf; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.

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
- Ported 2026-09-29 (4ce40edb). This repo holds no record of when the
  sibling project retrieved its sources; their official URLs are the
  USPTO Consolidated Patent Laws
  (`https://www.uspto.gov/web/offices/pac/mpep/consolidated_laws.pdf`,
  July 2025 update, still the edition served 2026-10-28), the USPTO
  Consolidated Patent Rules
  (`https://www.uspto.gov/web/offices/pac/mpep/consolidated_rules.pdf`,
  served 2026-10-28 as the July 2026 update, not the July 2025 edition
  the port used) and, for the 28 U.S.C. sections,
  `https://uscode.house.gov/view.xhtml?req=(title:28%20section:<N>%20edition:prelim)`
  (now 404; the same Code is at
  `https://www.govinfo.gov/link/uscode/28/<N>?link-type=html`).
- Checked 2026-10-28 (retrieved that day, with curl, from the URLs above):
  the four 28 U.S.C. sections match the govinfo pages within 10-44 words
  each (hyphenation and spacing). The 35 U.S.C. and 37 C.F.R. excerpts
  could not be lined up word for word against the consolidations'
  `pdftotext -layout` extraction (page furniture, and for the rules a later
  edition), so they are not verified here.
- sha256 (this file): `3d8bc2f973ecf54ebe7efc00ccb9756f0ed1e438dee88663f4b6ea65ee32be2a`
- Repair (lg-2vwt, 2026-10-28): the 35 U.S.C. § 41 excerpt stopped at (a)(7) and omitted § 41(b) (maintenance fees), which `lit/mpep` cites. § 41(b)(1)–(3) added after (a)(7), copied verbatim from GPO's govinfo (United States Code, 2024 Edition, current through January 6, 2025; headings kept in that print's case). No other section was changed.
- Original: none as served (assembled from a port of the sibling project's uscfr-full.json (USPTO Consolidated Patent Laws/Rules, July 2025, and uscode.house.gov pages for the 28 U.S.C. sections)); see the sources listed above.
- Original: `original/usc-35-41.html` as served, the source of § 41(b) only (sha256 3f9eff84f88989cb8f4c92f0d81902910a3c78bf671d3c7405e8ca19d6e9db65). Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title35/html/USCODE-2024-title35-partI-chap4-sec41.htm (via https://www.govinfo.gov/link/uscode/35/41?link-type=html); fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.

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
  Hand re-ordered 2026-10-28 (lg-nys9): on printed pp. 80-82 the content-stream
  order scattered RCFC 56.1-59 across the wrong `##` headings (RCFC 58(a)
  tail-(e) under RCFC 56.1, RCFC 59(b)(2)-(f) under RCFC 58.1, the RCFC 59
  2002 note under RCFC 60); those lines were moved back under their own rule
  and the page numbers/running heads dropped, no wording changed.
  Hand re-ordered 2026-10-29 (lg-2px4), same method on printed pp. 83-86:
  RCFC 60(b)(1)-(e) had landed under RCFC 61, RCFC 60's notes and RCFC 62(d)
  tail-(h) under RCFC 62's 2019 note, RCFC 62's 2002 note under RCFC 62.1,
  and RCFC 62's 2008/2010 notes under RCFC 63; each moved back under its own rule.
  Hand re-ordered 2026-10-29 (lg-lixc), same method on printed pp. 86-87:
  RCFC 65(b)(3)-(e)(1) had landed above the TITLE VIII / RCFC 64 heading,
  RCFC 65.1(b)'s tail before RCFC 65(e)(1)'s tail, and the RCFC 65 and 65.1
  credits and notes interleaved; each moved back under its own rule.
- sha256 (this file): `56246e500cf798bfcdeaaec8bf42a23012c4cb8c872639dd424ccd8877b57880`
- Original: `original/rcfc.pdf` as served (sha256 943733526f5391cad22562adf31411148fecfb03f01ae7bcf8bd6ec051a0c971); extraction sha256 2cd0ad815721d3234273142db6f8b32e5235d395c98273f94a51a2e8d2e81be9 (before the lg-nys9, lg-2px4 and lg-lixc re-orderings). Served at https://www.uscfc.uscourts.gov/sites/cfc/files/Rules%2010.08.26.pdf; same bytes as original/rcfc-appendices.pdf (fetched 2026-10-24), copied not refetched; retrieved 2026-10-27.

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

### `cfr-37-fees-2025-07-01.txt` — 37 C.F.R. §§ 1.16, 1.17, 1.18, 1.20, 41.20, 42.15 (fees; annual edition revised as of 2025-07-01)

- Fetched 2026-10-29 from `www.govinfo.gov` (`CFR-2025-title37-vol1-part1.xml`, `...-part41.xml`, `...-part42.xml`, the 2025-07-01 edition), same method as `cfr-37-part1.txt`: XML tags stripped, one paragraph per line, each fee table flattened to one line; no wording changed. The extraction reproduces `cfr-37-part1.txt` from the 2024 edition word for word (only the `[Reserved]` range headings differ). Headings `## 37 C.F.R. (2025) § N` keep it apart from the 2024 base files `cfr-37-part1.txt`, `cfr-37-part41.txt` and `cfr-37-part42.txt`. A dated layer over those files: the fees the January 19, 2025 fee rule (89 FR 91662, Nov. 20, 2024) set, which `lit/mpep`'s 2025-01-19 node versions cite.
- The eCFR versioner (`ecfr-versions-title-37.txt`) records later amendments to § 1.17 (2025-08-14, 2026-02-17, 2026-06-24, 2026-08-13) and § 1.20 (2026-03-30) that this edition predates; the 2026 annual edition was not yet on govinfo when fetched. Of those, only the 2026-08-13 revision of § 1.17(m)(1) changes a fee term the pack states; it is the dated layer `cfr-37-1-17-m-2026-08-13.txt`, below.
- sha256 (this file): `ac39bb4a121f72298602669dd48e02050438a3b3c3394abb9180e5e9c677072d`
- Original: `original/cfr-37-fees-2025-07-01/001-CFR-2025-title37-vol1-part1.xml` as served (sha256 d1437bebcfc70bc9fbb56b303fb3c79c7194943770ee8b62fcb770529262d186) and `original/cfr-37-fees-2025-07-01/002-CFR-2025-title37-vol1-part41.xml` as served (sha256 8662907257a9d0062f4f1c0084306381829481858a1bdb47615d0709c8171e73); § 42.15 is from the Part 42 edition already kept as `original/change-cfr-37-42-5-71-75-2026-10-20/001-CFR-2025-title37-vol1-part42.xml` (sha256 77256147aecfeded3601dc9b424fa11979b08755beafb7e544cde110da7654ec, identical bytes). Served at https://www.govinfo.gov/content/pkg/CFR-2025-title37-vol1/xml/CFR-2025-title37-vol1-part1.xml and `...-part41.xml`; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-29.

### `cfr-37-1-17-m-2026-08-13.txt` — 37 C.F.R. § 1.17(m) as revised by 91 FR 37826 (June 24, 2026), effective 2026-08-13

- Fetched 2026-10-29 from the Federal Register (FR Doc. 2026-12717, "Conditions for Additional Information and Fee in Petitions Filed in Patent Applications and Patents Based on Unintentional Delay", final rule, RIN 0651-AD92). The rule's only amendment ("Section 1.17 is amended by revising paragraph (m)(1)") is copied from its regulatory text with XML tags stripped, one paragraph per line, the fee table flattened to one line as in `cfr-37-fees-2025-07-01.txt`; no wording changed (the rule prints the first note entry as "1.55(e)", without "§", and ends the note heading with a period; both are kept). Paragraph (m)(2), which the rule leaves alone and which keys its tier to "the time period specified in paragraph (m)(1)", is carried over unchanged from `cfr-37-fees-2025-07-01.txt` (2025-07-01 edition). The other paragraphs of § 1.17 are not reproduced. Heading `## 37 C.F.R. (2026) § 1.17` keeps it apart from the 2025 layer and the 2024 base. A dated layer over `cfr-37-fees-2025-07-01.txt`: the text `lit/mpep`'s 2026-08-13 versions of `abandoned-no-reply` and `abandoned-issue-fee` cite. The change: the higher (m)(1) tier ($3,000.00 undiscounted) now applies to a petition filed more than one year (was two) after the due date; the amounts are unchanged. The rule "will be applicable to any new petition filed after the effective date".
- Currency check of the other post-2025-07-01 eCFR versions (lg-qif8, 2026-10-29): the Federal Register API lists four other final rules amending 37 CFR part 1 in force after 2025-07-01, none changing a fee the pack states: 90 FR 39124 (Aug. 14, 2025; removes and reserves § 1.17(k), design expedited examination, which the pack does not cite), 90 FR 29990 and 90 FR 24324 (§ 1.102 only), and 91 FR 13510 (§§ 1.9, 1.31-1.33, no fees). It lists no rule for the eCFR's 2026-02-17 versions (§§ 1.17, 1.482, 1.492, 1.1031) or 2026-03-30 versions (§§ 1.19, 1.20, 1.21), and the eCFR corrections API records no part 1 correction since 2025-06; every one of those sections is a fee table, which points to an eCFR table re-rendering, not an amendment. The USPTO fee schedule (https://www.uspto.gov/learning-and-resources/fees-and-payment/uspto-fee-schedule, "Effective January 19, 2025 (Last revised August 14, 2026)", page sha256 2b9f86e8b9dd6fab93d3b50f84b8f22c1a436b3954bd210680a48e37454d948b, not kept: a web page, not the rule) lists every other fee the pack states at its 2025-01-19 amount (§ 1.16(a), (e), (k), (n), (o), (r); § 1.17(a)(1)-(5), (e)(1)-(2); § 1.18(a); § 1.20(c)(1)-(2), (e)-(g); § 41.20(b)(1), (b)(4)), and gives § 1.17(m) with the one-year tier. govinfo had no 2026 title 37 edition when fetched (the part 1 XML URL redirects to its error page), and ecfr.gov serves its current-text pages only to browsers.
- sha256 (this file): `81ab38da9775e6572daff39c3692472948873dd2359129581e06e905000f9eb0`
- Original: `original/cfr-37-1-17-m-2026-08-13/001-FR-2026-06-24-2026-12717.pdf` as served (sha256 d05d9352276457a6d5a5a460969a0f4ee8ff0d7d0d240866f6cf4f913c87d01c), served at https://www.govinfo.gov/content/pkg/FR-2026-06-24/pdf/2026-12717.pdf; and `original/cfr-37-1-17-m-2026-08-13/002-FR-2026-06-24-2026-12717.xml` as served (sha256 6439a727b9b77e452fcc89f0cb5e4582c7fdf212b803f876ccb2911b9bdbf553), served at https://www.federalregister.gov/documents/full_text/xml/2026/06/24/2026-12717.xml, the extraction's source; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-29.

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
- Currency checked 2026-10-28 (lg-gddt): the current uscourts.gov print, "Federal Rules of Criminal Procedure, December 1, 2025" (House Judiciary Committee Print No. 5, https://www.uscourts.gov/sites/default/files/document/federal-rules-of-criminal-procedure.pdf, sha256 f1f7f98b64160e9174bca974d2bef2b06aaf949b9bfe9dba475d9d03710c689c), has rule text identical to the vendored Dec. 1, 2024 print (pdftotext of both from TITLE I on, running heads and page numbers dropped: 34,904 words each, 0 differences; only the committee roster differs). No criminal-rule amendments took effect Dec. 1, 2025, and none is pending for Dec. 1, 2026 (the pending-amendments page lists Criminal Rule 17 for Dec. 1, 2027). The 2025 print is kept as an original, below.
- Original: `original/frcrimp-2025-12-01.pdf` as served (sha256 f1f7f98b64160e9174bca974d2bef2b06aaf949b9bfe9dba475d9d03710c689c): the Dec. 1, 2025 print, rule text identical to this extraction's source.

### `mpep.txt` — MPEP chapters 200 (partial), 600, 700, 800 (partial), 1200, 1300, 1400 (partial), 2200 (partial) — 473 sections

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
  section — MPEP § 803 — could not be cleanly isolated this way at all); the
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
- §§ 803 and 818 (chapter 800, Rev. 07.2022) and §§ 2210–2214 (chapter
  2200, Rev. 01.2024) — 7 sections `lit/mpep` cites (`MPEP 803`, `MPEP 818`,
  the range `MPEP 2209–2214`) — fetched 2026-10-27 with curl (User-Agent
  lit-lang-provenance/1.0) from
  https://www.uspto.gov/web/offices/pac/mpep/mpep-0800.pdf and
  https://www.uspto.gov/web/offices/pac/mpep/mpep-2200.pdf (lg-3djd). Each
  page was extracted one column at a time (`pdftotext -x 0|288 -y 60 -W 288
  -H 680`, page running heads and footers cropped out), so these sections
  read in column order with none of the sidebar bleed above; each runs from
  its own heading line to the next section's, runs of blank lines collapsed
  to one. No wording changed.
- sha256 (this file): `cc1a2ff8b62594b72c5cc085be90e8f67cb6d86ec9ade9ba0d06bd52fa9c2ead`
- Original: `original/mpep-0800.pdf` as served (sha256 49ae8c8e65040a79d14c0010b6648fe8c61bf143586e9fe08c4677d69195d661), the chapter §§ 803, 818, 818.01 and 821.04 are cut from. Served at https://www.uspto.gov/web/offices/pac/mpep/mpep-0800.pdf; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/mpep-2200.pdf` as served (sha256 3e6dc8d4d31c9de5c6651bd8cd5ba2a920378d4f4a4f677a9d74661d241fc2d8), the chapter §§ 2209–2214, 2249 and 2254 are cut from. Served at https://www.uspto.gov/web/offices/pac/mpep/mpep-2200.pdf; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- The rest of the file has no original as served (a port of the sibling project's mpep-sections.json plus windows cut with pdftotext -layout from mpep-0200 and mpep-1400); see the sources listed above.

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

### `usc-18-app-frcrimp45-history.txt` — FRCrimP 45 as printed in the 1996, 2002, 2008, 2011, 2017 and 2019 editions of 18 U.S.C. App.

- The texts `lit/core`'s earlier versions of `frcrimp45` and
  `frcrimp45-delaware` took effect with: `## FRCrimP (1996 ed.) Rule 45`
  (as amended August 1, 1987), `## FRCrimP (2002 ed.) Rule 45` (as amended
  December 1, 2002), `## FRCrimP (2008 ed.) Rule 45` (as amended December
  1, 2008), `## FRCrimP (2011 ed.) Rule 45`
  (as amended December 1, 2009), `## FRCrimP (2017 ed.) Rule 45` (as
  amended December 1, 2016) and `## FRCrimP (2019 ed.) Rule 45` (as amended
  December 1, 2018), each the last amendment its credits list; each rule's
  text, amendment credits and Advisory Committee Notes. Not canonical refs,
  as in `usc-28-app-time-history.txt`.
- Fetched 2026-10-29 (lg-3v3h) from `www.govinfo.gov`'s official United
  States Code (GPO), granules
  `USCODE-2011-title18-app-federalru-dup1-rule45.htm` (sha256
  `e285f79511fc5b1064a9c3be9fe90b9b45f487e82a04153c0c31c8d6794e76a0`),
  `USCODE-2017-title18-app-federalru-rule45.htm`
  (`99fe62cff33a5fd5d13aa15c7d223257cdf5cbce745d0177646bc3c79623152f`) and
  `USCODE-2019-title18-app-federalru-rule45.htm`
  (`8b85a04e52e9f71d437d375dce67b8717054f7d782a0c23e24fe266f27847284`).
  Everything from the page's section head to its end, one line per
  paragraph, comments and tags stripped, entities decoded, runs of spaces
  collapsed.
- The 1996, 2002 and 2008 sections added 2026-10-29 (lg-h549), the same
  extraction, from granules
  `USCODE-1996-title18-app-federalru-dup1-rule45.htm` (sha256
  `df6a087941a39c44c31d8e27166445091bbace6cd8b4308ba325ab0460d5243a`),
  `USCODE-2002-title18-app-federalru-dup1-rule45.htm`
  (`326f10f28c3acd97a2cb14a66cbafa957671b519789019ceaf574cbcaddadb48`) and
  `USCODE-2008-title18-app-federalru-dup1-rule45.htm`
  (`d515da2e113066c4de50f6d76b07d1f0cecc6fc1a806db34a73d0c85ed398e07`);
  the extraction reproduces the 2011 section byte for byte.
- sha256 (this file): `808ebd72338065d7612f15f2a5945d0713572112f6dab6264e0f9fafb72e298e`
- Original: `original/usc-18-app-frcrimp45-history/1996-USCODE-1996-title18-app-federalru-dup1-rule45.html` as served (sha256 df6a087941a39c44c31d8e27166445091bbace6cd8b4308ba325ab0460d5243a); extraction sha256 808ebd72338065d7612f15f2a5945d0713572112f6dab6264e0f9fafb72e298e. Served at https://www.govinfo.gov/content/pkg/USCODE-1996-title18/html/USCODE-1996-title18-app-federalru-dup1-rule45.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-29.
- Original: `original/usc-18-app-frcrimp45-history/2002-USCODE-2002-title18-app-federalru-dup1-rule45.html` as served (sha256 326f10f28c3acd97a2cb14a66cbafa957671b519789019ceaf574cbcaddadb48); extraction sha256 808ebd72338065d7612f15f2a5945d0713572112f6dab6264e0f9fafb72e298e. Served at https://www.govinfo.gov/content/pkg/USCODE-2002-title18/html/USCODE-2002-title18-app-federalru-dup1-rule45.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-29.
- Original: `original/usc-18-app-frcrimp45-history/2008-USCODE-2008-title18-app-federalru-dup1-rule45.html` as served (sha256 d515da2e113066c4de50f6d76b07d1f0cecc6fc1a806db34a73d0c85ed398e07); extraction sha256 808ebd72338065d7612f15f2a5945d0713572112f6dab6264e0f9fafb72e298e. Served at https://www.govinfo.gov/content/pkg/USCODE-2008-title18/html/USCODE-2008-title18-app-federalru-dup1-rule45.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-29.
- Original: `original/usc-18-app-frcrimp45-history/2011-USCODE-2011-title18-app-federalru-dup1-rule45.html` as served (sha256 e285f79511fc5b1064a9c3be9fe90b9b45f487e82a04153c0c31c8d6794e76a0); extraction sha256 808ebd72338065d7612f15f2a5945d0713572112f6dab6264e0f9fafb72e298e. Served at https://www.govinfo.gov/content/pkg/USCODE-2011-title18/html/USCODE-2011-title18-app-federalru-dup1-rule45.htm; fetched with curl, retrieved 2026-10-29.
- Original: `original/usc-18-app-frcrimp45-history/2017-USCODE-2017-title18-app-federalru-rule45.html` as served (sha256 99fe62cff33a5fd5d13aa15c7d223257cdf5cbce745d0177646bc3c79623152f); extraction sha256 808ebd72338065d7612f15f2a5945d0713572112f6dab6264e0f9fafb72e298e. Served at https://www.govinfo.gov/content/pkg/USCODE-2017-title18/html/USCODE-2017-title18-app-federalru-rule45.htm; fetched with curl, retrieved 2026-10-29.
- Original: `original/usc-18-app-frcrimp45-history/2019-USCODE-2019-title18-app-federalru-rule45.html` as served (sha256 8b85a04e52e9f71d437d375dce67b8717054f7d782a0c23e24fe266f27847284); extraction sha256 808ebd72338065d7612f15f2a5945d0713572112f6dab6264e0f9fafb72e298e. Served at https://www.govinfo.gov/content/pkg/USCODE-2019-title18/html/USCODE-2019-title18-app-federalru-rule45.htm; fetched with curl, retrieved 2026-10-29.

### `usc-28-app-1997-time.txt` — FRAP 26 as printed in the 1997 edition of 28 U.S.C. App.

- The text `lit/core`'s 1996 version of `frap26` took effect with (as
  amended December 1, 1996, the last amendment its credits list; next
  amended December 1, 1998): `## FRAP (1997 ed.) Rule 26`, the rule's text,
  amendment credits and Advisory Committee Notes. Not a canonical ref, as in
  `usc-28-app-time-history.txt`.
- Fetched 2026-10-29 from `www.govinfo.gov`'s official United States Code
  (GPO), 1997 Edition, granule
  `USCODE-1997-title28-app-federalru-rule26.htm` (sha256
  `a0cd0764bc78a848b77ff9067604fd7fe4e70795978a6fa2a0dcd9f24329ca67`).
  Everything from the page's section head to its end, one line per
  paragraph, tags and comments stripped, entities decoded, runs of spaces
  collapsed (the same extraction reproduces `usc-28-app-2002-time.txt`
  byte for byte).
- sha256 (this file): `4915165f4a157fdd8c4c5bae85519368f9d6ed00591f11815b8b1fcf320c5553`
- Original: `original/usc-28-app-1997-time/001-ontent-pkg-USCODE-1997-title28-html-USCODE-1997-title28-app-federalru-rule26.html` as served (sha256 a0cd0764bc78a848b77ff9067604fd7fe4e70795978a6fa2a0dcd9f24329ca67); extraction sha256 4915165f4a157fdd8c4c5bae85519368f9d6ed00591f11815b8b1fcf320c5553. Served at https://www.govinfo.gov/content/pkg/USCODE-1997-title28/html/USCODE-1997-title28-app-federalru-rule26.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-29.

### `usc-28-app-2002-time.txt` — FRAP 26 as printed in the 2002 edition of 28 U.S.C. App.

- The text `lit/core`'s 2002 version of `frap26` took effect with (as
  amended December 1, 2002, the last amendment its credits list; next
  amended December 1, 2005): `## FRAP (2002 ed.) Rule 26`, the rule's text,
  amendment credits and Advisory Committee Notes. Not a canonical ref, as in
  `usc-28-app-time-history.txt`.
- Fetched 2026-10-28 from `www.govinfo.gov`'s official United States Code
  (GPO), 2002 Edition (current through January 6, 2003), granule
  `USCODE-2002-title28-app-federalru-rule26.htm` (sha256
  `376b24ff236b967e9d4b3d3fad881f3f479218e9d3945487b727f226a4b20d0d`).
  Everything from the page's section head to its end, one line per
  paragraph, tags stripped, entities decoded, runs of spaces collapsed.
- sha256 (this file): `77560dcd00af4016c24f1ed1817228500e35bc32797ec4d94a980b14556bb827`
- Original: `original/usc-28-app-2002-time/001-ontent-pkg-USCODE-2002-title28-html-USCODE-2002-title28-app-federalru-rule26.html` as served (sha256 376b24ff236b967e9d4b3d3fad881f3f479218e9d3945487b727f226a4b20d0d); extraction sha256 77560dcd00af4016c24f1ed1817228500e35bc32797ec4d94a980b14556bb827. Served at https://www.govinfo.gov/content/pkg/USCODE-2002-title28/html/USCODE-2002-title28-app-federalru-rule26.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.

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
    fetched 2026-10-27; `frap.txt` prints none. Re-read (lg-eapo,
    2026-10-28) against the credits closing FRAP 26 in the House Judiciary
    print of December 1, 2025 (`frap/frap.txt`, printed p. 29): the same
    thirteen amendments, the last "Apr. 24, 2023, eff. Dec. 1, 2023" (the
    OCR reads "2028" for the order year; the 2024 granule gives 2023), so
    the list is unchanged and its `as_of` moves to 2025-12-01.
  - `RCFC 6`: "As revised and reissued May 1, 2002; as amended June 20,
    2006, Nov. 3, 2008, Jan. 11, 2010, Aug. 1, 2017, July 29, 2024" in
    `rcfc.txt` (no issue dates printed).
  - `19 C.F.R. § 201.14`: the source note in `cfr-19-part201.txt` (41 FR
    17710; 56 FR 11923; 68 FR 32975; 76 FR 61941), each dated by the
    effective date its notice gives in `fr-usitc-time-history.txt` (May
    27, 1976; August 4, 2003; November 7, 2011), except 56 FR 11923, whose
    effective date is not vendored and which is dated by publication
    (March 21, 1991).
- Added 2026-10-28 (lg-pd9k), for the cites of `lit/frap`, `lit/frcp`,
  `lit/frcrimp`, `lit/cafc` and `lit/ttab` whose vendored text postdates
  the citing version:
  - `FRAP 3`-`FRAP 42` (18 rules), `Rule 3`-`Rule 60` (FRCP, 11 rules) and
    `Fed. R. Crim. P. 3`-`48` (20 rules): the credits closing each rule in
    the govinfo 2024 edition of 28 U.S.C. App. and 18 U.S.C. App. (current
    through January 6, 2025; FRAP 35, abrogated in 2024, from the 2023
    edition), one granule per rule, the originals below; `date` and
    `amendment_date` the "eff." date, `issue_date` the order date. The FRAP
    lists are read to the House Judiciary print of December 1, 2025
    (`frap.txt`), whose 2025 amendments touch only FRAP 6 and 39, so their
    `as_of` is 2025-12-01; the FRCP and Criminal Rules lists are as of the
    granules' currency date, 2025-01-06.
  - `15 U.S.C. § 1051`-`§ 1125` (11 sections): the "(Amended ...)"
    credits printed under each section in `usc-15-lanham.txt` (the USPTO
    compilation of 2026-07-15), each amendment dated by the approval date
    the credit prints (an act effective later is dated earlier, so a list
    can only break a proof sooner).
  - `Fed. Cir. R. 8`-`47.6` (14 rules): the December 1, 2025 edition of the
    Federal Circuit Rules (sha256 `b6346ed5...`, recorded under
    `fedcir-rules.txt`) compared with the October 6, 2026 edition
    vendored as `fedcir-rules.txt`; each rule's text is unchanged between
    them, so its list records the December 1, 2025 version only and
    proves nothing about a version in force before then.
- Added 2026-10-29 (lg-k1tq) for `lit/frcrimp`'s mistrial cite:
  `Fed. R. Crim. P. 26.3`, transcribed from the credit closing the rule in
  `frcrimp.txt` ("As added Apr. 22, 1993, eff. Dec. 1, 1993; amended Apr.
  29, 2002, eff. Dec. 1, 2002"), as of that text's December 1, 2024 date.
- Added 2026-10-28 (lg-pd9k) for `lit/cofc`: `RCFC 12`, `RCFC 54`,
  `RCFC 56` and `RCFC 59`, from the "(As revised and reissued May 1,
  2002; as amended ...)" credit closing each rule in the court's compiled
  rules `Rules 10.08.26.pdf` (as amended through July 27, 2026; recorded
  under `rcfc-appendices.txt`, original `original/rcfc-appendices.pdf`),
  read column by column (`pdftotext -layout`, each column cropped at the
  page's midline): `rcfc.txt` places each credit under the following
  rule's heading, so it is not read here. `RCFC App. C 2`: Appendix C's
  Rules Committee Notes in the same PDF, which name paragraph 2 in the
  2016, 2019 and 2021 Amendments (and minor changes "throughout the
  Appendix" in 2007); each year is dated by the court's amendment date
  that year as the rules' credits print it (Nov. 15, 2007; Aug. 1, 2016;
  July 1, 2019; Aug. 2, 2021), the first version by the May 1, 2002
  reissue. No issue dates are printed; `as_of` is 2026-07-27, the
  compilation's date.
- Added 2026-10-28 (lg-pd9k): `Sup. Ct. R. 10`, for `lit/cafc` (from
  2025-12-01) citing the 2026 Rules (`scotus-rules.txt`, effective
  2026-03-16): one version, the 2023 edition's (adopted December 5, 2022,
  effective January 1, 2023), whose Rule 10 reads word for word as the
  2026 edition's (both editions recorded under `scotus-rules.txt`; the
  Court's 2026 revisions list names Rules 14, 15, 24, 29 and 48 only).
- Transcribed 2026-10-27 (f981db86). The vendored files read here record
  their own source URLs in their entries in this file; the one source not
  vendored as text, the FRAP 26 granule, was retrieved 2026-10-27 from
  `https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-app-federalru-rule26.htm`
  and re-fetched 2026-10-28 with the same sha256.
- sha256 (this file): `ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02`
- Original (FRAP 26, RCFC 6, FRCP 6/14/81/86, Fed. R. Crim. P. 26.3 and 45, 19 C.F.R. § 201.14 and the Lanham Act and Federal Circuit lists): none as served for those blocks (transcribed from frcp.txt, frcrimp.txt, rcfc.txt, cfr-19-part201.txt, usc-15-lanham.txt, fedcir-rules.txt and the govinfo 2024 FRAP 26 granule); see the sources listed above.
- Original: `original/rule-amendment-history/USCODE-2024-title28-app-federalru-rule26.htm` as served (sha256 1b1e1f0b76345262b3e275d5c2d5cfff2d47b813bdc1b0921ce96ed8a58fb878), the FRAP 26 credits only; extraction sha256 33425acd0e8c0ff8f9ab6a8701956acf866c6f77a2b012dde260104f689d050d. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-app-federalru-rule26.htm; fetched with curl, retrieved 2026-10-28.

- Original: `original/rule-amendment-history/001-USCODE-2024-title28-app-federalru-rule3.htm` as served (sha256 db8831a9a2f1a0a9a3462f11aae439c1a56f887f97c53e28c235b61818c8562b); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-app-federalru-rule3.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/002-USCODE-2024-title28-app-federalru-rule4.htm` as served (sha256 6ca5279c447208cc31d03844ed2ba79bca4d79ba843c33b27bf54135f0230a63); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-app-federalru-rule4.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/003-USCODE-2024-title28-app-federalru-rule5.htm` as served (sha256 cca0a461967d8b1e9b81177b5510a6b87c26553b74c05ee90716c31cff4e01d3); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-app-federalru-rule5.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/004-USCODE-2024-title28-app-federalru-rule8.htm` as served (sha256 18860274bad2539ccab28943ec30d6f4f65a6c1649b178e1f922cafb4cf485c8); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-app-federalru-rule8.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/005-USCODE-2024-title28-app-federalru-rule10.htm` as served (sha256 3fbfd57d1261370263eaab1741ee9b710c374fba5a11aa2994dc8cd6751bc954); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-app-federalru-rule10.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/006-USCODE-2024-title28-app-federalru-rule11.htm` as served (sha256 a18a1c94ee7de1acb7825f09460980073418a072b200210d3b6cfb6cd2ea1046); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-app-federalru-rule11.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/007-USCODE-2024-title28-app-federalru-rule12.htm` as served (sha256 7fdf33435189178e939e00940a202305fd5fa61dec29d610c6f363839fe302be); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-app-federalru-rule12.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/008-USCODE-2024-title28-app-federalru-rule27.htm` as served (sha256 a8ad9cc4041a59e6cbf0b2280d8e95ad0bdeebc9b4f2709e931236d66a9e41a4); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-app-federalru-rule27.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/009-USCODE-2024-title28-app-federalru-rule28.htm` as served (sha256 e933959e9b2fda8be39267bd6a66a3c7fd0b3338f8f3afafcd675f431d1b2ed6); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-app-federalru-rule28.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/010-USCODE-2024-title28-app-federalru-rule28.1.htm` as served (sha256 f89c1904bf8959e38fc92528cdaf78cb9cc713855cf13e462a0729c7217a0d52); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-app-federalru-rule28.1.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/011-USCODE-2024-title28-app-federalru-rule31.htm` as served (sha256 a35c2f27d9fa12991e80b54004ae833e2b5de293490e944d4073d5bf70c04198); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-app-federalru-rule31.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/012-USCODE-2024-title28-app-federalru-rule32.htm` as served (sha256 f99f73d96253009913aa39c5b6ae46d1e22dbfd99a9f7513667e7e131299dac1); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-app-federalru-rule32.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/013-USCODE-2024-title28-app-federalru-rule34.htm` as served (sha256 b6a7da26ded82b6b9b537fc397e78a3bf261b5e3b9486331b0dfa5668616d09d); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-app-federalru-rule34.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/014-USCODE-2023-title28-app-federalru-rule35.htm` as served (sha256 c2fddb46e2e069fb80031dd6717fc6f968fc5b7c2bd6b9faa200be3db6c20dec); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2023-title28/html/USCODE-2023-title28-app-federalru-rule35.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/015-USCODE-2024-title28-app-federalru-rule36.htm` as served (sha256 dca70ec328b5f497fca33ca797350d3375db98427940925aa47936b02f476e50); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-app-federalru-rule36.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/016-USCODE-2024-title28-app-federalru-rule40.htm` as served (sha256 7690296b4199fc2a26e78b78ba1c1e97163fe369c2324ec4f38464d773bd94b6); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-app-federalru-rule40.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/017-USCODE-2024-title28-app-federalru-rule41.htm` as served (sha256 08ff4ffa3f1cc947239740762ff5d9d66c3f9c3a0e37a01dc88c923538c7fd03); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-app-federalru-rule41.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/018-USCODE-2024-title28-app-federalru-rule42.htm` as served (sha256 effb4a2707a36ab9ae58d4402e7c27036ae4c0f1732631904ffb3ca330774b1e); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-app-federalru-rule42.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/019-USCODE-2024-title28-app-federalru-dup1-rule3.htm` as served (sha256 8010a3d248a5cd2646c59eba4e0045f776e3ac8051382b58aec2a2139e99a1cf); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-app-federalru-dup1-rule3.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/020-USCODE-2024-title28-app-federalru-dup1-rule4.htm` as served (sha256 e14aa6f36ad2578548ec2d104f3e5f4dfbeaecc2f01f69bb65485706b6c3e2e1); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-app-federalru-dup1-rule4.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/021-USCODE-2024-title28-app-federalru-dup1-rule12.htm` as served (sha256 fb845e23910f7660e02030ba5f5439f603f06b62f6a1dea8a70d380cf9c62753); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-app-federalru-dup1-rule12.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/022-USCODE-2024-title28-app-federalru-dup1-rule15.htm` as served (sha256 ca13232723623319e8ffc7b479c5a376734bf4e78bc79a8cf132e574e94056cd); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-app-federalru-dup1-rule15.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/023-USCODE-2024-title28-app-federalru-dup1-rule26.htm` as served (sha256 f149ef24358636b52d70c740506386bb4c46903e7d646671eab0af8474446e04); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-app-federalru-dup1-rule26.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/024-USCODE-2024-title28-app-federalru-dup1-rule38.htm` as served (sha256 0a13121da5a48d90ba5876d697be869ca7c558af4f9d2fed0d11f07041f329be); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-app-federalru-dup1-rule38.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/025-USCODE-2024-title28-app-federalru-dup1-rule50.htm` as served (sha256 f4e26edaf63844a4f716be6998e68a85206256cf84bc37c0d4299d35c93df4ba); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-app-federalru-dup1-rule50.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/026-USCODE-2024-title28-app-federalru-dup1-rule52.htm` as served (sha256 964864475dbe771e08acdac2692c8ab6ab5088d2d2b8de738aabd04252276529); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-app-federalru-dup1-rule52.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/027-USCODE-2024-title28-app-federalru-dup1-rule54.htm` as served (sha256 22e63d965fcf0a5ac094aaca91b657adcc0cdd16c30c3ae83bae7b0812b0b844); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-app-federalru-dup1-rule54.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/028-USCODE-2024-title28-app-federalru-dup1-rule59.htm` as served (sha256 73b10a20976218fba9594cae77877b7d4abca63016e510f0a29f73c80de98c6c); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-app-federalru-dup1-rule59.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/029-USCODE-2024-title28-app-federalru-dup1-rule60.htm` as served (sha256 8ed03ba9f211489442521559eff4b142c71cdb60c87e3901c66e50650a306212); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-app-federalru-dup1-rule60.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/030-USCODE-2024-title18-app-federalru-rule3.htm` as served (sha256 520a4fb7141d7f36b943d6a93add7607934dd35b0ba54c83af781aba43325d57); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title18/html/USCODE-2024-title18-app-federalru-rule3.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/031-USCODE-2024-title18-app-federalru-rule4.htm` as served (sha256 e8a84a7760f24dc27e23125cc1f23b8cfcb821ec2a53e84fb4bdd6dfdf99e172); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title18/html/USCODE-2024-title18-app-federalru-rule4.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/032-USCODE-2024-title18-app-federalru-rule5.htm` as served (sha256 7eb21b6b42c44e9906d74e152f55e9edb36816d62c56dfc528c13cf0a2e7e483); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title18/html/USCODE-2024-title18-app-federalru-rule5.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/033-USCODE-2024-title18-app-federalru-rule5.1.htm` as served (sha256 53eff8f331d25add9667a2b5d7705ae157fd91a0c4895c381d8fb40080774729); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title18/html/USCODE-2024-title18-app-federalru-rule5.1.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/034-USCODE-2024-title18-app-federalru-rule6.htm` as served (sha256 ae33c1c57fb64d2fb5db5c42ab11094638cf2995cdfce3783b7c0113f9e7f3fd); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title18/html/USCODE-2024-title18-app-federalru-rule6.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/035-USCODE-2024-title18-app-federalru-rule7.htm` as served (sha256 b0bad685a52faa9aeb964229d862732651fb79fe5d4b0df5e3cf8390278fd551); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title18/html/USCODE-2024-title18-app-federalru-rule7.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/036-USCODE-2024-title18-app-federalru-rule9.htm` as served (sha256 4a4bb2f20f8c948d40fbc6d910b04da472f7ff35a29ac928e4f3f61e7eb43ab2); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title18/html/USCODE-2024-title18-app-federalru-rule9.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/037-USCODE-2024-title18-app-federalru-rule10.htm` as served (sha256 c06a625aea538153828f34603027e8880fabe106c8332b3dd41b045b7361f456); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title18/html/USCODE-2024-title18-app-federalru-rule10.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/038-USCODE-2024-title18-app-federalru-rule11.htm` as served (sha256 83f6d5cef060fb3f1faf3d9204153e6473b7c3038b531cc3e32044d400f0b8b7); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title18/html/USCODE-2024-title18-app-federalru-rule11.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/039-USCODE-2024-title18-app-federalru-rule12.htm` as served (sha256 75852a58a9671842b77323c25207b602049a9d5a21612ec94cc05e010f348451); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title18/html/USCODE-2024-title18-app-federalru-rule12.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/040-USCODE-2024-title18-app-federalru-rule16.htm` as served (sha256 9140012b582e0eb4fdf5554e28bd3eb05ed43a1bcf19c9f88c92c35ffca8e0bb); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title18/html/USCODE-2024-title18-app-federalru-rule16.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/041-USCODE-2024-title18-app-federalru-rule24.htm` as served (sha256 3f670fab2e62c25b3a5634bc83dc8af5dbfc9d1d4adf2b7821d2bff56c502746); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title18/html/USCODE-2024-title18-app-federalru-rule24.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/042-USCODE-2024-title18-app-federalru-rule26.htm` as served (sha256 d2a2228691777e27d835a86a698318b0e2bf77c98cb159eca33697f9e1a4cada); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title18/html/USCODE-2024-title18-app-federalru-rule26.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/043-USCODE-2024-title18-app-federalru-rule29.htm` as served (sha256 495b8013c2a643e29a99577d150e53ea8e17d4e128c1d1b1d0e28a4a31825d81); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title18/html/USCODE-2024-title18-app-federalru-rule29.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/044-USCODE-2024-title18-app-federalru-rule30.htm` as served (sha256 9c00d380903e3a2e6c246a801b3417003219b701ce5ecf6449b9e133739ceefb); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title18/html/USCODE-2024-title18-app-federalru-rule30.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/045-USCODE-2024-title18-app-federalru-rule31.htm` as served (sha256 dd012362f2fa0d6380fc079b6af70645a7adeeb736ea81bf86283c7dfe6f216b); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title18/html/USCODE-2024-title18-app-federalru-rule31.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/046-USCODE-2024-title18-app-federalru-rule32.htm` as served (sha256 e66559a98b4ab727db58e234abb3738aef764025b8d7af408adff60d536a07a0); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title18/html/USCODE-2024-title18-app-federalru-rule32.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/047-USCODE-2024-title18-app-federalru-rule33.htm` as served (sha256 ef93ceaad6bd82089cc1857bc7cf538c71d3769936bdda870894a3e630351de8); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title18/html/USCODE-2024-title18-app-federalru-rule33.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/048-USCODE-2024-title18-app-federalru-rule35.htm` as served (sha256 511e157ace011aad9091fec0bb5af7117a4d64d2debfbdd4cde6829adb7307cc); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title18/html/USCODE-2024-title18-app-federalru-rule35.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/rule-amendment-history/049-USCODE-2024-title18-app-federalru-rule48.htm` as served (sha256 427f2194bd014e4409de8beb95e54ff069d7c220a7f20737bbc58e58aef361b8); extraction sha256 ed7828881a7c2fc6ffbc9f3241e298e4e08271a7ec170aa479dbc9a822682c02. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title18/html/USCODE-2024-title18-app-federalru-rule48.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.

### `ecfr-versions-title-19.txt` — eCFR versioner version lists of the 19 C.F.R. part 210 sections `lit/itc` cites

- The amendment histories `cite-resolves` reads (SPEC §4.3) for cites of
  `cfr-19-part210.txt` (bound in `lit/itc` as `ecfr-versions-19`). One
  `## versions 19 C.F.R. § N` block per cited section (17): an `as_of
  2026-10-28` line (the fetch date), then the response to
  `https://www.ecfr.gov/api/versioner/v1/versions/title-19.json?section=N`
  (Office of the Federal Register), byte for byte on one line, fetched with
  curl (lg-pd9k). The headings are deliberately not canonical refs, so no
  cite resolves to them. The lists start at 2017-01-03, the start of the
  eCFR's point-in-time coverage of title 19, so they prove nothing about a
  version in force before then.
- sha256 (this file): `f5772fc7e9852994796aa81a644ffa7be4c3f664a3543d5c6297c186c79e7b32`
- Original: `original/ecfr-versions-title-19/001-api-versioner-v1-versions-title-19.json_section-210.5.json` as served (sha256 214d44dc528a8e3272be7ef2f9522bea45d1b682b6b7ae48c079ecbb2c965702); extraction sha256 f5772fc7e9852994796aa81a644ffa7be4c3f664a3543d5c6297c186c79e7b32. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-19.json?section=210.5; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/ecfr-versions-title-19/002-api-versioner-v1-versions-title-19.json_section-210.10.json` as served (sha256 461f5eb24b3fa913f53e9a4547b584234f8c99b626490d8e2267bc6a85196ee8); extraction sha256 f5772fc7e9852994796aa81a644ffa7be4c3f664a3543d5c6297c186c79e7b32. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-19.json?section=210.10; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/ecfr-versions-title-19/003-api-versioner-v1-versions-title-19.json_section-210.13.json` as served (sha256 f5eb25a7788c4fc0e74e34a8aa8f2cc62d44717e6c558d3a82b0e07fa521b1bd); extraction sha256 f5772fc7e9852994796aa81a644ffa7be4c3f664a3543d5c6297c186c79e7b32. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-19.json?section=210.13; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/ecfr-versions-title-19/004-api-versioner-v1-versions-title-19.json_section-210.16.json` as served (sha256 394d4af4940ace4452bada37ad4d487ea981380e3cac261b9b57717e6243fbae); extraction sha256 f5772fc7e9852994796aa81a644ffa7be4c3f664a3543d5c6297c186c79e7b32. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-19.json?section=210.16; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/ecfr-versions-title-19/005-api-versioner-v1-versions-title-19.json_section-210.18.json` as served (sha256 36764c0b64eae8e55c58d64694ca2ec6b6804c292d4ac1d98a5f5ca4751c309a); extraction sha256 f5772fc7e9852994796aa81a644ffa7be4c3f664a3543d5c6297c186c79e7b32. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-19.json?section=210.18; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/ecfr-versions-title-19/006-api-versioner-v1-versions-title-19.json_section-210.27.json` as served (sha256 7dbb13adf21d819de3fe2c848fa693fe89c05927ce0ca6548be4f6a3166e344a); extraction sha256 f5772fc7e9852994796aa81a644ffa7be4c3f664a3543d5c6297c186c79e7b32. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-19.json?section=210.27; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/ecfr-versions-title-19/007-api-versioner-v1-versions-title-19.json_section-210.28.json` as served (sha256 d577edca298cae058ef599a30e9622bde99faf57478c3921736cde772d0cbe3c); extraction sha256 f5772fc7e9852994796aa81a644ffa7be4c3f664a3543d5c6297c186c79e7b32. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-19.json?section=210.28; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/ecfr-versions-title-19/008-api-versioner-v1-versions-title-19.json_section-210.29.json` as served (sha256 a6216f27ce137f66ebbc68a4c5a266371817cd75d47f2caf184a061a984a8fa2); extraction sha256 f5772fc7e9852994796aa81a644ffa7be4c3f664a3543d5c6297c186c79e7b32. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-19.json?section=210.29; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/ecfr-versions-title-19/009-api-versioner-v1-versions-title-19.json_section-210.30.json` as served (sha256 2b3eca473099d3de5fe1aa9b9669e96603b0f57429475cefb0d091afce9770c6); extraction sha256 f5772fc7e9852994796aa81a644ffa7be4c3f664a3543d5c6297c186c79e7b32. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-19.json?section=210.30; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/ecfr-versions-title-19/010-api-versioner-v1-versions-title-19.json_section-210.31.json` as served (sha256 d965ac7a1964c198227f48d9c6934bae1def648d0a2e79678be450c8301ec0a3); extraction sha256 f5772fc7e9852994796aa81a644ffa7be4c3f664a3543d5c6297c186c79e7b32. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-19.json?section=210.31; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/ecfr-versions-title-19/011-api-versioner-v1-versions-title-19.json_section-210.32.json` as served (sha256 9cced068656e4add280786a7cb226728f6984d410057415d329a856a6b374762); extraction sha256 f5772fc7e9852994796aa81a644ffa7be4c3f664a3543d5c6297c186c79e7b32. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-19.json?section=210.32; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/ecfr-versions-title-19/012-api-versioner-v1-versions-title-19.json_section-210.33.json` as served (sha256 c57f6e18a926c937c8f0245d90bdbb38188206bae0886a61406e9a6b2ddc91f4); extraction sha256 f5772fc7e9852994796aa81a644ffa7be4c3f664a3543d5c6297c186c79e7b32. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-19.json?section=210.33; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/ecfr-versions-title-19/013-api-versioner-v1-versions-title-19.json_section-210.34.json` as served (sha256 f7ffa6faab4a436badc4fd436c535e050cce6f73937dd3613dcdb157f18c622b); extraction sha256 f5772fc7e9852994796aa81a644ffa7be4c3f664a3543d5c6297c186c79e7b32. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-19.json?section=210.34; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/ecfr-versions-title-19/014-api-versioner-v1-versions-title-19.json_section-210.37.json` as served (sha256 b435ac16a2b0aa4cee1e8882e86a1bfaca487c8330dc8c8b2d6e8198677ccf71); extraction sha256 f5772fc7e9852994796aa81a644ffa7be4c3f664a3543d5c6297c186c79e7b32. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-19.json?section=210.37; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/ecfr-versions-title-19/015-api-versioner-v1-versions-title-19.json_section-210.42.json` as served (sha256 ad6b467a696b7aecc82bf4d2cbbcc7d28bb017bd700875fb491509dd66c57975); extraction sha256 f5772fc7e9852994796aa81a644ffa7be4c3f664a3543d5c6297c186c79e7b32. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-19.json?section=210.42; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/ecfr-versions-title-19/016-api-versioner-v1-versions-title-19.json_section-210.43.json` as served (sha256 4048f37873e05866e148dc1f54f88d358cbe4e005034dd5ac038c0d58c0ae78f); extraction sha256 f5772fc7e9852994796aa81a644ffa7be4c3f664a3543d5c6297c186c79e7b32. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-19.json?section=210.43; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/ecfr-versions-title-19/017-api-versioner-v1-versions-title-19.json_section-210.51.json` as served (sha256 94c857f3808eb454a448d7c646dc76da77904191d3f5d9ae4401e7cdb31c1ffc); extraction sha256 f5772fc7e9852994796aa81a644ffa7be4c3f664a3543d5c6297c186c79e7b32. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-19.json?section=210.51; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.

### `frap22-amendment-history.txt` — amendment history of FRAP 22

- One `## versions FRAP 22` block (SPEC §4.3), in the form of
  `rule-amendment-history.txt`, for `lit/habeas` (from 2024-12-01) citing
  FRAP 22 in `frap.txt` (as of 2025-12-01) (lg-cnge, 2026-10-29). Transcribed
  from the credit closing FRAP 22 in the govinfo 2024 edition of 28 U.S.C.
  App. (current through January 6, 2025): "(As amended Pub. L. 104–132,
  title I, §103, Apr. 24, 1996, 110 Stat. 1218; Apr. 24, 1998, eff. Dec. 1,
  1998; Mar. 26, 2009, eff. Dec. 1, 2009.)"; `date` and `amendment_date`
  the "eff." date (the 1996 act's approval date, as `FRAP 4`'s 1988 act is
  dated in `rule-amendment-history.txt`), `issue_date` the order date. Read
  to the House Judiciary print of December 1, 2025 (`frap/frap.txt`,
  printed p. 23), whose credit for Rule 22 lists the same three amendments
  (its 2025 amendments touch only FRAP 6 and 39), so `as_of` is 2025-12-01.
- sha256 (this file): `45dc79a4517f57b2bddd5c5a2d1829e2da57a4f3d4232847e2623b83fdc82bd2`
- Original: `original/frap22-amendment-history/USCODE-2024-title28-app-federalru-rule22.htm` as served (sha256 98c369e6e1da8640dd961e7e724cce736d4866b1417b46da3bd38151899b6df5); extraction sha256 45dc79a4517f57b2bddd5c5a2d1829e2da57a4f3d4232847e2623b83fdc82bd2. Served at https://www.govinfo.gov/content/pkg/USCODE-2024-title28/html/USCODE-2024-title28-app-federalru-rule22.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-29.

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

### `usc-28-app-frap-editions.txt` — FRAP Rules 3-5, 8, 11, 27, 28, 28.1, 31, 32, 35 and 39-42 as printed in the 2009 edition of 28 U.S.C. App., FRAP 4 in the 2002 edition and FRAP 32 in the 2017 edition

- The texts `lit/frap`'s dated versions took effect with, so that each
  version cites the text in force on its first day (SPEC §4.3) rather than
  the Dec. 1, 2025 print: `## FRAP (2009 ed.) Rule N` for the rules the
  package's December 1, 2009 versions cite that were amended after that
  date (each as amended through December 1, 2009; the edition is current
  through February 1, 2010, and no rule's credits list an amendment after
  2009), `## FRAP (2002 ed.) Rule 4` (as amended December 1, 2002, the
  earlier appeal periods' start; current through January 6, 2003) and
  `## FRAP (2017 ed.) Rule 32` (as amended December 1, 2016, the
  type-volume limits' start; current through January 12, 2018). Each is
  the rule's text, amendment credits and Committee Notes. The headings are
  deliberately not canonical refs, as in `usc-28-app-time-history.txt`.
- Fetched 2026-10-29 from `www.govinfo.gov`'s official United States Code
  (GPO), 2002, 2009 and 2017 Editions, one granule per rule
  (`USCODE-<year>-title28-app-federalru-rule<N>.htm`, listed below).
  Everything from the page's section head to its end, one line per
  paragraph, tags and comments stripped, entities decoded, runs of spaces
  collapsed (the extraction of `usc-28-app-2002-time.txt`, which it
  reproduces byte for byte from that file's original).
- sha256 (this file): `007a4635a31187bca31cfbaf45a83cb94770c4777536bfb875bd71312f769244`
- Original: `original/usc-28-app-frap-editions/001-USCODE-2002-title28-app-federalru-rule4.htm` as served (sha256 287560f61c1ba978c0d1e82aff31ad1a879786164e1fd89e6288034dd66364b4); extraction sha256 007a4635a31187bca31cfbaf45a83cb94770c4777536bfb875bd71312f769244. Served at https://www.govinfo.gov/content/pkg/USCODE-2002-title28/html/USCODE-2002-title28-app-federalru-rule4.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-29.
- Original: `original/usc-28-app-frap-editions/002-USCODE-2009-title28-app-federalru-rule3.htm` as served (sha256 5bb85ad4439fd5a860913ddbdf0935c9913f2e45d63af0a7bfe7c3e0784f0d57); extraction sha256 007a4635a31187bca31cfbaf45a83cb94770c4777536bfb875bd71312f769244. Served at https://www.govinfo.gov/content/pkg/USCODE-2009-title28/html/USCODE-2009-title28-app-federalru-rule3.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-29.
- Original: `original/usc-28-app-frap-editions/003-USCODE-2009-title28-app-federalru-rule4.htm` as served (sha256 2ae3f9788f5f16be860e01a2bd1d256212b7f22731762554c3547290e9daa296); extraction sha256 007a4635a31187bca31cfbaf45a83cb94770c4777536bfb875bd71312f769244. Served at https://www.govinfo.gov/content/pkg/USCODE-2009-title28/html/USCODE-2009-title28-app-federalru-rule4.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-29.
- Original: `original/usc-28-app-frap-editions/004-USCODE-2009-title28-app-federalru-rule5.htm` as served (sha256 0943d8a12177d8634fe0048c0c39b972eb33c77a38524c4ca2e82e5932f7f2d7); extraction sha256 007a4635a31187bca31cfbaf45a83cb94770c4777536bfb875bd71312f769244. Served at https://www.govinfo.gov/content/pkg/USCODE-2009-title28/html/USCODE-2009-title28-app-federalru-rule5.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-29.
- Original: `original/usc-28-app-frap-editions/005-USCODE-2009-title28-app-federalru-rule8.htm` as served (sha256 3cb8c619b15486e2248f1775da9e9f8f52131ca4c5fed106188ce7483b220f83); extraction sha256 007a4635a31187bca31cfbaf45a83cb94770c4777536bfb875bd71312f769244. Served at https://www.govinfo.gov/content/pkg/USCODE-2009-title28/html/USCODE-2009-title28-app-federalru-rule8.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-29.
- Original: `original/usc-28-app-frap-editions/006-USCODE-2009-title28-app-federalru-rule11.htm` as served (sha256 412a2e0a2546d848bf9b5dea347d3d16862a959ba03826e64e893d18c4243a41); extraction sha256 007a4635a31187bca31cfbaf45a83cb94770c4777536bfb875bd71312f769244. Served at https://www.govinfo.gov/content/pkg/USCODE-2009-title28/html/USCODE-2009-title28-app-federalru-rule11.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-29.
- Original: `original/usc-28-app-frap-editions/007-USCODE-2009-title28-app-federalru-rule27.htm` as served (sha256 1469f963b72caaddd9e8fe4df9c7e035cd9ea440423980c3769f7766034ed6fe); extraction sha256 007a4635a31187bca31cfbaf45a83cb94770c4777536bfb875bd71312f769244. Served at https://www.govinfo.gov/content/pkg/USCODE-2009-title28/html/USCODE-2009-title28-app-federalru-rule27.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-29.
- Original: `original/usc-28-app-frap-editions/008-USCODE-2009-title28-app-federalru-rule28.htm` as served (sha256 90a324906931db0f4d29fe2889beb53e4b3e9082ae0f7b68097f3b468ed2fa2b); extraction sha256 007a4635a31187bca31cfbaf45a83cb94770c4777536bfb875bd71312f769244. Served at https://www.govinfo.gov/content/pkg/USCODE-2009-title28/html/USCODE-2009-title28-app-federalru-rule28.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-29.
- Original: `original/usc-28-app-frap-editions/009-USCODE-2009-title28-app-federalru-rule28.1.htm` as served (sha256 c0d767a51f019d0b39e2b38102b05484e7152f127453adfdf59c8c1f8daad75d); extraction sha256 007a4635a31187bca31cfbaf45a83cb94770c4777536bfb875bd71312f769244. Served at https://www.govinfo.gov/content/pkg/USCODE-2009-title28/html/USCODE-2009-title28-app-federalru-rule28.1.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-29.
- Original: `original/usc-28-app-frap-editions/010-USCODE-2009-title28-app-federalru-rule31.htm` as served (sha256 ee0e6c84cb4b4db04bb5d22dc7602a19a169b42f2ca39f673deaf302820a12e5); extraction sha256 007a4635a31187bca31cfbaf45a83cb94770c4777536bfb875bd71312f769244. Served at https://www.govinfo.gov/content/pkg/USCODE-2009-title28/html/USCODE-2009-title28-app-federalru-rule31.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-29.
- Original: `original/usc-28-app-frap-editions/011-USCODE-2009-title28-app-federalru-rule32.htm` as served (sha256 90199aabf6bc8de52662cb22591a1ae529aa32ee4dd39f62db95793ad0bad185); extraction sha256 007a4635a31187bca31cfbaf45a83cb94770c4777536bfb875bd71312f769244. Served at https://www.govinfo.gov/content/pkg/USCODE-2009-title28/html/USCODE-2009-title28-app-federalru-rule32.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-29.
- Original: `original/usc-28-app-frap-editions/012-USCODE-2009-title28-app-federalru-rule35.htm` as served (sha256 62220ded24adcf6787c40cc52f611504645b80a2bf18b1f82ddc98fbc5977e90); extraction sha256 007a4635a31187bca31cfbaf45a83cb94770c4777536bfb875bd71312f769244. Served at https://www.govinfo.gov/content/pkg/USCODE-2009-title28/html/USCODE-2009-title28-app-federalru-rule35.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-29.
- Original: `original/usc-28-app-frap-editions/013-USCODE-2009-title28-app-federalru-rule39.htm` as served (sha256 6146f1ba3708dad3e5c018d8ca74067bc9742e135c8a45872736cdd891bb4794); extraction sha256 007a4635a31187bca31cfbaf45a83cb94770c4777536bfb875bd71312f769244. Served at https://www.govinfo.gov/content/pkg/USCODE-2009-title28/html/USCODE-2009-title28-app-federalru-rule39.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-29.
- Original: `original/usc-28-app-frap-editions/014-USCODE-2009-title28-app-federalru-rule40.htm` as served (sha256 0eaf4305b86641aca7fa3a67049e00447e7a53313885ad31289b42c9a919d68e); extraction sha256 007a4635a31187bca31cfbaf45a83cb94770c4777536bfb875bd71312f769244. Served at https://www.govinfo.gov/content/pkg/USCODE-2009-title28/html/USCODE-2009-title28-app-federalru-rule40.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-29.
- Original: `original/usc-28-app-frap-editions/015-USCODE-2009-title28-app-federalru-rule41.htm` as served (sha256 625453dd3ada87085106af2c7510ababbb12ce961ef34d69834aafddcbeed730); extraction sha256 007a4635a31187bca31cfbaf45a83cb94770c4777536bfb875bd71312f769244. Served at https://www.govinfo.gov/content/pkg/USCODE-2009-title28/html/USCODE-2009-title28-app-federalru-rule41.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-29.
- Original: `original/usc-28-app-frap-editions/016-USCODE-2009-title28-app-federalru-rule42.htm` as served (sha256 5fea139172277bc2541782baa71187cb48fb182686d9a3128396429c92c23f20); extraction sha256 007a4635a31187bca31cfbaf45a83cb94770c4777536bfb875bd71312f769244. Served at https://www.govinfo.gov/content/pkg/USCODE-2009-title28/html/USCODE-2009-title28-app-federalru-rule42.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-29.
- Original: `original/usc-28-app-frap-editions/017-USCODE-2017-title28-app-federalru-rule32.htm` as served (sha256 5800c13420f58e6041300e285ba7e090603a6719d2bf9ddd110ccc16c4c132ee); extraction sha256 007a4635a31187bca31cfbaf45a83cb94770c4777536bfb875bd71312f769244. Served at https://www.govinfo.gov/content/pkg/USCODE-2017-title28/html/USCODE-2017-title28-app-federalru-rule32.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-29.

### `rcfc-appendices.txt` — RCFC Appendices A and C (every paragraph), H (¶¶ 1-3) and J (Patent Rules 1-24)

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
- Added 2026-10-28 (lg-pd9k), from the same compiled `Rules 10.08.26.pdf`
  (re-fetched 2026-10-28 with curl, same sha256 `943733526f...`), so that
  `lit/cofc`'s appendix cites resolve: `## RCFC App. A 1`-`19` (Appendix
  A, Case Management Procedure), `## RCFC App. C 1`-`29` (Appendix C,
  Procedure in Procurement Protest Cases), `## RCFC App. H 1`-`2` and
  `## PRCFC 1`-`15`, `18`-`24` (the Patent Rules but 16-17 above). Each
  page's two columns are read left then right (`pdftotext -layout`, each
  column cropped at the page's midline), page numbers and running
  `Appendix X` footers dropped, as are the all-capitals title and
  division headings (`II. REQUIREMENT FOR PRE-FILING NOTIFICATION`) and
  each appendix's closing Rules Committee Notes; a paragraph runs from its
  `N.` (a Patent Rule from its `Rule N.` title line, kept as its own line)
  to the next, lines joined with single spaces, a hyphen ending a line
  kept and joined without a space. No wording changed. Each section opens
  with its appendix's title line.
- sha256 (this file): `0d9824cde486bfa401e7b5df88252e08121c19d630928090e52b3c7484266739`
- Original: `original/rcfc-appendices.pdf` as served (sha256 943733526f5391cad22562adf31411148fecfb03f01ae7bcf8bd6ec051a0c971); extraction sha256 0d9824cde486bfa401e7b5df88252e08121c19d630928090e52b3c7484266739. Served at https://www.uscfc.uscourts.gov/sites/cfc/files/Rules%2010.08.26.pdf; retrieved 2026-10-24 (fetched as recorded above).

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
- Added 2026-10-21 (641a650c); the library copy's own retrieval date and
  method are not recorded. Verified 2026-10-28 instead against the eCFR's
  point-in-time text for the same date, retrieved with curl from the eCFR
  versioner API (below): all 155 sections match it word for word, apart
  from the restated `37 C.F.R. § 2.N —` heading line, four `### §§ ...
  [Reserved]` lines, and the XML's spacing inside parentheses and before
  em dashes.
- sha256 (this file): `66402ce5e48bf2a9aa039058b3bfc181d3511b9e94137fac8b6a3d84a26b33a7`
- Original: `original/cfr-37-part2.xml` as served (sha256 3925c9b1ea8e2f426b491e8efc7d0ba57d0393d11f11aa715e59ebe678e050fd), the eCFR point-in-time XML of 37 C.F.R. Part 2 as of 2026-10-19, not the library copy the extraction was split from; extraction sha256 66402ce5e48bf2a9aa039058b3bfc181d3511b9e94137fac8b6a3d84a26b33a7. Served at https://www.ecfr.gov/api/versioner/v1/full/2026-10-19/title-37.xml?part=2; fetched with curl, retrieved 2026-10-28.

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
  `cfr-37-part1.txt` or `cfr-37-part2.txt` (dated by
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
  before then. The Part 42 lists moved to `cfr-37-part42-versions.txt`
  (lg-pd9k), which carries each section's history back to 2012.
- Fetched 2026-10-24. SHA-256 of each response (the block's last line):
  - § 1.7: `b05d1efdaec5cec72f84229707f964daf359e135632c8b4991b392f852559924`
  - § 1.16: `900c41f00037c23fa8b52824d40130736add9505ae93d6281a6ab1278dd877d2`
  - § 1.17: `b513e1f80b76adf6307496fdf47f9cd72ab1a9ab19ce32937bf6399fd256ac67`
  - § 1.18: `a4146e56fed715df85c88743dc5e4aa6082a443e862008370f20c5a77e6dd8d6`
  - § 1.20: `bdd9ed0d99fd816ba08cd73171f91f889ebdc7e5c7ff4b2e19808e4ce2d8ccf7`
  - § 1.53 (fetched 2026-10-28, `as_of 2026-10-28`, lg-pd9k): `199cd6b286b0913c159eaa6dcfff2643f4e2bb5e2ad235ae812a0919001e41f3`
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
  - § 2.117 (fetched 2026-10-28, `as_of 2026-10-28`, lg-pd9k): `852e5422c81f06234faeaeeefb99a8dd767ca0b6017ba733b4d3b6cb0961d61b`
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
- sha256 (this file): `b8228facdc76d2494eb95b7008538a73f1c6b521ce64e09e7178ff6c0cb1820b`
- Original: `original/ecfr-versions-title-37/001-api-versioner-v1-versions-title-37.json_section-1.7.json` as served (sha256 b05d1efdaec5cec72f84229707f964daf359e135632c8b4991b392f852559924); extraction sha256 b8228facdc76d2494eb95b7008538a73f1c6b521ce64e09e7178ff6c0cb1820b. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=1.7; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/002-api-versioner-v1-versions-title-37.json_section-1.16.json` as served (sha256 900c41f00037c23fa8b52824d40130736add9505ae93d6281a6ab1278dd877d2); extraction sha256 b8228facdc76d2494eb95b7008538a73f1c6b521ce64e09e7178ff6c0cb1820b. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=1.16; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/003-api-versioner-v1-versions-title-37.json_section-1.17.json` as served (sha256 b513e1f80b76adf6307496fdf47f9cd72ab1a9ab19ce32937bf6399fd256ac67); extraction sha256 b8228facdc76d2494eb95b7008538a73f1c6b521ce64e09e7178ff6c0cb1820b. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=1.17; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/004-api-versioner-v1-versions-title-37.json_section-1.18.json` as served (sha256 a4146e56fed715df85c88743dc5e4aa6082a443e862008370f20c5a77e6dd8d6); extraction sha256 b8228facdc76d2494eb95b7008538a73f1c6b521ce64e09e7178ff6c0cb1820b. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=1.18; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/005-api-versioner-v1-versions-title-37.json_section-1.20.json` as served (sha256 bdd9ed0d99fd816ba08cd73171f91f889ebdc7e5c7ff4b2e19808e4ce2d8ccf7); extraction sha256 b8228facdc76d2494eb95b7008538a73f1c6b521ce64e09e7178ff6c0cb1820b. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=1.20; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/006-api-versioner-v1-versions-title-37.json_section-1.104.json` as served (sha256 5fddd6eedd918f2df6f119fb47e121b5b1e7cebf3486640b3d9e9ee0a0160607); extraction sha256 b8228facdc76d2494eb95b7008538a73f1c6b521ce64e09e7178ff6c0cb1820b. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=1.104; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/007-api-versioner-v1-versions-title-37.json_section-1.133.json` as served (sha256 38b8935cde1f81a0cfcc14bcff451321c1123467502d77bfc551619479cfd85c); extraction sha256 b8228facdc76d2494eb95b7008538a73f1c6b521ce64e09e7178ff6c0cb1820b. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=1.133; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/008-api-versioner-v1-versions-title-37.json_section-1.134.json` as served (sha256 55620caa0644f3718d1bef52df4695e19615d327c22308653c59d4c427868dbb); extraction sha256 b8228facdc76d2494eb95b7008538a73f1c6b521ce64e09e7178ff6c0cb1820b. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=1.134; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/009-api-versioner-v1-versions-title-37.json_section-1.137.json` as served (sha256 f26dd480c778a76b648ba4c75a64f9f9f0fd39ef82c79557136eee5a5285f45a); extraction sha256 b8228facdc76d2494eb95b7008538a73f1c6b521ce64e09e7178ff6c0cb1820b. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=1.137; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/010-api-versioner-v1-versions-title-37.json_section-1.142.json` as served (sha256 0129bd8c6dfe95d6a4b9f368fa4fd8ed2dbfa69a3cf11bc7cbcfbe0f77754876); extraction sha256 b8228facdc76d2494eb95b7008538a73f1c6b521ce64e09e7178ff6c0cb1820b. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=1.142; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/011-api-versioner-v1-versions-title-37.json_section-1.143.json` as served (sha256 1d8c24c98826b1c45afa62e3d3beb0e9058f7f69dbc39f6f52894e265f3f43eb); extraction sha256 b8228facdc76d2494eb95b7008538a73f1c6b521ce64e09e7178ff6c0cb1820b. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=1.143; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/012-api-versioner-v1-versions-title-37.json_section-1.144.json` as served (sha256 ee0c94275014897f8f7cf1d79159279282f42ebb0409834b725d014698603e1f); extraction sha256 b8228facdc76d2494eb95b7008538a73f1c6b521ce64e09e7178ff6c0cb1820b. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=1.144; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/013-api-versioner-v1-versions-title-37.json_section-1.171.json` as served (sha256 38ca49f03e43511563a695250d4606dc97823f070c9ddd5360846a41f99fe9ce); extraction sha256 b8228facdc76d2494eb95b7008538a73f1c6b521ce64e09e7178ff6c0cb1820b. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=1.171; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/014-api-versioner-v1-versions-title-37.json_section-1.510.json` as served (sha256 e0bf431ace62f84139482f725a0744f75ddee66bf1e5a6ee4ed5045925f184db); extraction sha256 b8228facdc76d2494eb95b7008538a73f1c6b521ce64e09e7178ff6c0cb1820b. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=1.510; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/015-api-versioner-v1-versions-title-37.json_section-1.515.json` as served (sha256 08132fb4845e07a2700b1cd59d51c3fe8ef8d9459bf41e260c6ce648220b8652); extraction sha256 b8228facdc76d2494eb95b7008538a73f1c6b521ce64e09e7178ff6c0cb1820b. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=1.515; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/016-api-versioner-v1-versions-title-37.json_section-1.530.json` as served (sha256 bcf8895a4e81671153c042587d5998287b2861a5d28856aec6a7f97cff3ed19d); extraction sha256 b8228facdc76d2494eb95b7008538a73f1c6b521ce64e09e7178ff6c0cb1820b. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=1.530; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/017-api-versioner-v1-versions-title-37.json_section-1.550.json` as served (sha256 227ae4a84aafa256f300f8d0038cb00c11fe3a707811653aa4e121d4469cd599); extraction sha256 b8228facdc76d2494eb95b7008538a73f1c6b521ce64e09e7178ff6c0cb1820b. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=1.550; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/018-api-versioner-v1-versions-title-37.json_section-1.570.json` as served (sha256 2d8de876d87fc0959d7fc3b75a5474f29f656c2aaee5d94f6e170860a82faa40); extraction sha256 b8228facdc76d2494eb95b7008538a73f1c6b521ce64e09e7178ff6c0cb1820b. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=1.570; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/019-api-versioner-v1-versions-title-37.json_section-2.6.json` as served (sha256 1feac2b52422420078093d563df7306811bd966b277904d4ab0522222c2f55b7); extraction sha256 b8228facdc76d2494eb95b7008538a73f1c6b521ce64e09e7178ff6c0cb1820b. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=2.6; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/020-api-versioner-v1-versions-title-37.json_section-2.68.json` as served (sha256 d424543583ec96f886e0882c5a4ef0b009024808e7793623d658e771365ccae5); extraction sha256 b8228facdc76d2494eb95b7008538a73f1c6b521ce64e09e7178ff6c0cb1820b. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=2.68; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/021-api-versioner-v1-versions-title-37.json_section-2.88.json` as served (sha256 f398efe4ae636f678ab9f3ff310179ea17dff38893d7a7d329cc3d98711376cb); extraction sha256 b8228facdc76d2494eb95b7008538a73f1c6b521ce64e09e7178ff6c0cb1820b. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=2.88; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/022-api-versioner-v1-versions-title-37.json_section-2.101.json` as served (sha256 55690d36a068e612d3be376204f62a6f09348df1def93b51dcb7a70c51e4f856); extraction sha256 b8228facdc76d2494eb95b7008538a73f1c6b521ce64e09e7178ff6c0cb1820b. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=2.101; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/023-api-versioner-v1-versions-title-37.json_section-2.102.json` as served (sha256 cebb4ba1e7e3b5726dea84d9f63e2d87e473a228ec3fad3a8d9a4bfbcd2bc364); extraction sha256 b8228facdc76d2494eb95b7008538a73f1c6b521ce64e09e7178ff6c0cb1820b. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=2.102; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/024-api-versioner-v1-versions-title-37.json_section-2.105.json` as served (sha256 8196196e26e9e069cc98745fcf25745f8c1d89f0b60b2694c8f4c9e033d6e923); extraction sha256 b8228facdc76d2494eb95b7008538a73f1c6b521ce64e09e7178ff6c0cb1820b. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=2.105; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/025-api-versioner-v1-versions-title-37.json_section-2.106.json` as served (sha256 bafe6722400078cf4e3850cceb4b1d29103bbef827ca65187cf83f044339290e); extraction sha256 b8228facdc76d2494eb95b7008538a73f1c6b521ce64e09e7178ff6c0cb1820b. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=2.106; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/026-api-versioner-v1-versions-title-37.json_section-2.111.json` as served (sha256 fb478d7488ffb7792802e803e75cee79a136c7701fce99de636281dce5e7f45b); extraction sha256 b8228facdc76d2494eb95b7008538a73f1c6b521ce64e09e7178ff6c0cb1820b. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=2.111; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/027-api-versioner-v1-versions-title-37.json_section-2.113.json` as served (sha256 e37846c21e075a78123b969a47ebd61d86329f57978b4850b06a07aa5aaa07c0); extraction sha256 b8228facdc76d2494eb95b7008538a73f1c6b521ce64e09e7178ff6c0cb1820b. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=2.113; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/028-api-versioner-v1-versions-title-37.json_section-2.120.json` as served (sha256 7abdbb0667ea2eaaa70359c0aa3cb31322564c946465be1a43ad99fcafe11fb3); extraction sha256 b8228facdc76d2494eb95b7008538a73f1c6b521ce64e09e7178ff6c0cb1820b. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=2.120; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/029-api-versioner-v1-versions-title-37.json_section-2.121.json` as served (sha256 bd59901972a1e2b677dcba6440a8a1a0c42c64ed2e09c2918500fce51810f9c1); extraction sha256 b8228facdc76d2494eb95b7008538a73f1c6b521ce64e09e7178ff6c0cb1820b. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=2.121; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/030-api-versioner-v1-versions-title-37.json_section-2.123.json` as served (sha256 aaccb79f28ab614449c4dea8cf13dcfb11e37ed7623852b9177b8899e6f6abb5); extraction sha256 b8228facdc76d2494eb95b7008538a73f1c6b521ce64e09e7178ff6c0cb1820b. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=2.123; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/031-api-versioner-v1-versions-title-37.json_section-2.125.json` as served (sha256 6d928b5fd8ec7a6a2403373732bf3f87fe58420e117d140526ba13915fcec701); extraction sha256 b8228facdc76d2494eb95b7008538a73f1c6b521ce64e09e7178ff6c0cb1820b. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=2.125; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/032-api-versioner-v1-versions-title-37.json_section-2.128.json` as served (sha256 84c1c21d2f4b29b9a74f282c7cbce3a59d43d1300977eaa25185e8bd692fe267); extraction sha256 b8228facdc76d2494eb95b7008538a73f1c6b521ce64e09e7178ff6c0cb1820b. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=2.128; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/033-api-versioner-v1-versions-title-37.json_section-2.129.json` as served (sha256 46be73aa4f50e11629cfe8229278a767a4f587dc6aaad5c867e6928e4ab5f45d); extraction sha256 b8228facdc76d2494eb95b7008538a73f1c6b521ce64e09e7178ff6c0cb1820b. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=2.129; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/034-api-versioner-v1-versions-title-37.json_section-2.133.json` as served (sha256 83b7d43f1b32283d04c6a66c8c12ef47df669234152394d60a533b8df55c7808); extraction sha256 b8228facdc76d2494eb95b7008538a73f1c6b521ce64e09e7178ff6c0cb1820b. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=2.133; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/035-api-versioner-v1-versions-title-37.json_section-2.135.json` as served (sha256 57ea6363fff223478410917f2fbdea6cd1fa2362fb10f035ef894c44c79e1575); extraction sha256 b8228facdc76d2494eb95b7008538a73f1c6b521ce64e09e7178ff6c0cb1820b. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=2.135; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/036-api-versioner-v1-versions-title-37.json_section-2.136.json` as served (sha256 3883d7604e57461bca0e5e369061981d122190f76c99826ec4879e44161d9143); extraction sha256 b8228facdc76d2494eb95b7008538a73f1c6b521ce64e09e7178ff6c0cb1820b. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=2.136; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/037-api-versioner-v1-versions-title-37.json_section-2.145.json` as served (sha256 b8b24d0d7ec1cb15937362ee8757a8856587e90baf74d7b2a6fea0d8d34a007b); extraction sha256 b8228facdc76d2494eb95b7008538a73f1c6b521ce64e09e7178ff6c0cb1820b. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=2.145; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/038-api-versioner-v1-versions-title-37.json_section-2.196.json` as served (sha256 155ecb2188b4f43e202e074a05847754add55d6f03cc4acb73153c254be56ccb); extraction sha256 b8228facdc76d2494eb95b7008538a73f1c6b521ce64e09e7178ff6c0cb1820b. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=2.196; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `original/ecfr-versions-title-37/057-api-versioner-v1-versions-title-37.json_section-1.53.json` as served (sha256 199cd6b286b0913c159eaa6dcfff2643f4e2bb5e2ad235ae812a0919001e41f3); extraction sha256 b8228facdc76d2494eb95b7008538a73f1c6b521ce64e09e7178ff6c0cb1820b. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=1.53; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/ecfr-versions-title-37/058-api-versioner-v1-versions-title-37.json_section-2.117.json` as served (sha256 852e5422c81f06234faeaeeefb99a8dd767ca0b6017ba733b4d3b6cb0961d61b); extraction sha256 b8228facdc76d2494eb95b7008538a73f1c6b521ce64e09e7178ff6c0cb1820b. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?section=2.117; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.

### `tbmp-versions.txt` — the TBMP editions in which each vendored TBMP section changed (June 2024, 2025, 2026)

- The amendment histories `cite-resolves` reads (SPEC §4.3) for cites of
  `tbmp.txt` (bound in `lit/ttab`). One `## versions TBMP § N` block per
  `## TBMP § N` section of `tbmp.txt` but § 528.05(a), whose bare-number
  heading the comparison cannot find: an `as_of 2026-06-30` line (the
  June 2026 edition `tbmp.txt` is cut from), then a `content_versions`
  list of the editions in which the section's text changed.
- Method (lg-pd9k): the June 2024 (001), June 2025 (002) and June 2026
  (`original/tbmp.pdf`) editions, each `pdftotext` (no `-layout`); in
  each, the section is cut from its body heading (`N Title`, N listed in
  the chapter's table of contents) to the next heading that is not one of
  its subsections, so a section's text includes all its subsections (a
  cite of the section is a cite of all of it); NOTES (the footnoted
  authorities), form feeds, page numbers, `June YYYY` lines, running
  `§ N` headers and repeated all-capitals running titles are dropped, as
  is all whitespace and any hyphen ending a line. The texts so normalized
  are compared exactly, by a Python script. The first version is the June
  2024 edition, dated 2024-06-30 (its latest possible date; earlier
  editions were not compared, so a list proves nothing about a version in
  force before then); a section whose text differs in the June 2025 or
  June 2026 edition gets a version dated 2025-06-01 or 2026-06-01 (the
  earliest possible date). Any difference counts, including extraction
  artifacts (a NOTES label the PDF text omits), so a list can only break
  a proof sooner.
- sha256 (this file): `ed91a2fc8e1bc24b8ef5f6feec2f0f132de28c3929aaaf8ff2fea2b9d9d577a6`
- Original: `original/tbmp-versions/001-tbmp-Master-June2024.pdf` as served (sha256 5b1f54e3495ebc04c24b5ebbdb44d4a63d12de4ee53038642c499356696ee502); extraction sha256 ed91a2fc8e1bc24b8ef5f6feec2f0f132de28c3929aaaf8ff2fea2b9d9d577a6. Served at https://www.uspto.gov/sites/default/files/documents/tbmp-Master-June2024.pdf; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/tbmp-versions/002-tbmp-Master-June2025.pdf` as served (sha256 e60e01f32683c9287fdbe64a99351247b3279ee4f71b6046ba94eb90fd24dd61); extraction sha256 ed91a2fc8e1bc24b8ef5f6feec2f0f132de28c3929aaaf8ff2fea2b9d9d577a6. Served at https://www.uspto.gov/sites/default/files/documents/tbmp-Master-June2025.pdf; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- The June 2026 edition is `original/tbmp.pdf` (sha256 015cf2b665d74b25080321f67dad7ffebe412118f6f0460ec3530f4801639b8e, re-fetched 2026-10-28 from https://www.uspto.gov/sites/default/files/documents/tbmp-Master-June2026.pdf with the same digest), recorded under `tbmp.txt`.

### `cfr-37-part42-versions.txt` — amendment histories of the 37 C.F.R. Part 42 sections, from 2012

- The amendment histories `cite-resolves` reads (SPEC §4.3) for cites of
  `cfr-37-part42.txt` (bound in `lit/ptab` and `lit/mpep`). One `## versions
  37 C.F.R. § N` block per section of Part 42: an `as_of 2026-10-28` line
  (the date the history was compiled), then one JSON line
  `{"source_note", "content_versions"}`.
- Method (lg-pd9k): `source_note` is the section's source note in the
  govinfo annual edition of 37 C.F.R. (revised as of July 1, 2025, 001):
  its `[CITA]` line, else the subpart's or part's `Source:` line ("unless
  otherwise noted"), which records every amendment to the section. Each
  Federal Register citation in it is matched by volume and page to the
  rule document holding it in the Federal Register API's list of final
  rules amending 37 C.F.R. part 42 (002); that document's `effective_on`
  (else its publication date) is the version's `date`/`amendment_date`
  and its publication date the `issue_date` (entries marked `fr`). Every
  version after 2017-01-01 in the eCFR versioner's list for part 42
  (003) is added too (entries marked `ecfr`; its 2017-01-01 entries are
  the start of the eCFR's coverage, not amendments, and are left out).
  The eCFR list shows one amendment after July 1, 2025: § 42.15 on
  2026-02-17. Publication-date entries the eCFR records ahead of a later
  effective date are kept, so a history can only break a proof sooner.
  A section revised in full lists only the documents from its revision
  on, so a cite of an earlier version stays a hole. Compiled with a
  Python script from the three originals; no wording changed.
- sha256 (this file): `86e68377fd571c93cfca6d320901fd82c7551a60d5249db7765dde071d37521b`
- Original: `original/cfr-37-part42-versions/001-CFR-2025-title37-vol1-part42.xml` as served (sha256 77256147aecfeded3601dc9b424fa11979b08755beafb7e544cde110da7654ec); extraction sha256 86e68377fd571c93cfca6d320901fd82c7551a60d5249db7765dde071d37521b. Served at https://www.govinfo.gov/content/pkg/CFR-2025-title37-vol1/xml/CFR-2025-title37-vol1-part42.xml; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/cfr-37-part42-versions/002-federalregister-api-v1-documents-cfr-37-42-rules.json` as served (sha256 ae43afb11a4af648b6e70d05ebae1dca509ea972049b967e28775f58aa203096); extraction sha256 86e68377fd571c93cfca6d320901fd82c7551a60d5249db7765dde071d37521b. Served at https://www.federalregister.gov/api/v1/documents.json?conditions%5Bcfr%5D%5Btitle%5D=37&conditions%5Bcfr%5D%5Bpart%5D=42&conditions%5Btype%5D%5B%5D=RULE&fields%5B%5D=citation&fields%5B%5D=start_page&fields%5B%5D=end_page&fields%5B%5D=effective_on&fields%5B%5D=publication_date&fields%5B%5D=title&fields%5B%5D=document_number&per_page=200&order=oldest; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.
- Original: `original/cfr-37-part42-versions/003-api-versioner-v1-versions-title-37.json_part-42.json` as served (sha256 88cf068667e88c4ba285c1c169fe1a744c170d6425f2d68a48951e9f94210dac); extraction sha256 86e68377fd571c93cfca6d320901fd82c7551a60d5249db7765dde071d37521b. Served at https://www.ecfr.gov/api/versioner/v1/versions/title-37.json?part=42; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-28.

### `cfr-37-part42-2025.txt` — 37 C.F.R. Part 42, revised as of July 1, 2025

- The govinfo annual CFR edition of Title 37, vol. 1, Part 42 (2025-07-01
  edition), the text `lit/ptab` binds as `cfr-42` (`as_of 2025-07-01`):
  it holds the 2024 motion-to-amend rule (§ 42.121(e)-(f), 89 FR 76421)
  and Director Review (§ 42.75, 89 FR 79744), which the 2024-07-01
  edition in `cfr-37-part42.txt` predates. Extracted exactly as
  `cfr-37-part42.txt` was (every `<SECTION>` one `## 37 C.F.R. § N` block
  from its `<SECTNO>`/`<SUBJECT>`/`<P>`/`<FP>`/`<CITA>` children,
  whitespace collapsed, tags stripped); the same script reproduces
  `cfr-37-part42.txt` byte for byte from the 2024 edition.
- sha256 (this file): `802de976aed8ea3f59996284cca73e41947e0c26da2c217e4318fc422c2b2395`
- Original: `original/cfr-37-part42-versions/001-CFR-2025-title37-vol1-part42.xml` as served (sha256 77256147aecfeded3601dc9b424fa11979b08755beafb7e544cde110da7654ec, the same bytes, re-fetched 2026-10-29); extraction sha256 802de976aed8ea3f59996284cca73e41947e0c26da2c217e4318fc422c2b2395. Served at https://www.govinfo.gov/content/pkg/CFR-2025-title37-vol1/xml/CFR-2025-title37-vol1-part42.xml; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-29.

### `cfr-37-part42-2012.txt` — 37 C.F.R. Part 42, subparts A-C, as promulgated (77 FR 48612 and 77 FR 48680, Aug. 14, 2012)

- The text in force from September 16, 2012 of §§ 42.1-42.80 (subpart A,
  77 FR 48669-48677, FR Doc. 2012-17900) and §§ 42.100-42.224 (subparts
  B and C, 77 FR 48727-48732, FR Doc. 2012-17906), the regulatory text of
  the two final rules, so a version in force before a section's first
  amendment cites its own text (`37 C.F.R. (2012) § N`). One
  `## 37 C.F.R. (2012) § N` block per section, then its heading line and
  one line per paragraph: verbatim except for line joining, page markers
  (`[[Page N]]`) and the rules' center headings dropped, `Sec.` printed as
  `§`, the GPO text's doubled backtick and apostrophe quotes as curly
  quotes, and its `--` as an em dash. Subparts D and E (covered business
  method review, derivation) are left out.
- sha256 (this file): `35a392617c2ff11a2f8417f5f5adad2242f02d4f5b32d3bb29b78f4af75cab6b`
- Original: `original/cfr-37-part42-2012/001-FR-2012-17900.htm` as served (sha256 230a9429a1acfe78b073b0d75d9a2a08e8ecfa8eecfce3a22ab8f163e82aaae9); extraction sha256 35a392617c2ff11a2f8417f5f5adad2242f02d4f5b32d3bb29b78f4af75cab6b. Served at https://www.govinfo.gov/content/pkg/FR-2012-08-14/html/2012-17900.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-29.
- Original: `original/cfr-37-part42-2012/002-FR-2012-17906.htm` as served (sha256 6457ba68b18a9e1198e5129a3a630d1ee28754682e2a4b35838f22a171064a94); extraction sha256 35a392617c2ff11a2f8417f5f5adad2242f02d4f5b32d3bb29b78f4af75cab6b. Served at https://www.govinfo.gov/content/pkg/FR-2012-08-14/html/2012-17906.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-29.

### `fr-uspto-part42-2015.txt` — Amendments to the PTAB trial rules (80 FR 28561, May 19, 2015)

- Page 28561 (summary and effective date, May 19, 2015) and page 28565,
  the amendatory text through § 42.71 (whose first sentence of paragraph
  (d) it revises), as `## 80 FR <page>` sections each opening with a
  bracketed line naming the issue, page and FR Doc. number. Verbatim
  except for line joining, the `0` paragraph markers dropped, and the
  same `§`, quote and dash substitutions as `cfr-37-part42-2012.txt`.
- sha256 (this file): `d3855b455e8e96a1b7f8921090fcc9dbfb7830d2ca7cfec2f12e6cdae03d44cf`
- Original: `original/fr-uspto-part42-2015/001-FR-2015-12117.htm` as served (sha256 66f2896687db94c75ce29d22f829f5b1174025d8d47ca1681fb1c98c87d8eae2); extraction sha256 d3855b455e8e96a1b7f8921090fcc9dbfb7830d2ca7cfec2f12e6cdae03d44cf. Served at https://www.govinfo.gov/content/pkg/FR-2015-05-19/html/2015-12117.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-29.

### `fr-uspto-part42-2016.txt` — Amendments to the PTAB trial rules (81 FR 18750, Apr. 1, 2016) and correction (81 FR 24702, Apr. 27, 2016)

- `## 81 FR 18750` (effective date, May 2, 2016), `## 81 FR 18765` (the
  amendatory text on that page, including § 42.24 revised in full to word
  counts) and `## 81 FR 24703` (the correction striking "grounds for
  standing under § 42.104, § 42.204, or § 42.304," from § 42.24(a)(1)),
  each opening with a bracketed line naming the issue, page and FR Doc.
  number; the 18765 section opens mid-sentence where the page does.
  Verbatim except for line joining, the `0` paragraph markers dropped, and
  the same `§`, quote and dash substitutions as `cfr-37-part42-2012.txt`.
- sha256 (this file): `92ac19c44ca15f2175f3a5145eec571075ce0977b8829e2cffb98b4cac6cb76a`
- Original: `original/fr-uspto-part42-2016/001-FR-2016-07381.htm` as served (sha256 68424d8d6df786be06367f6009ecb9c3028967f6c8082990299f3d43e23460cc); extraction sha256 92ac19c44ca15f2175f3a5145eec571075ce0977b8829e2cffb98b4cac6cb76a. Served at https://www.govinfo.gov/content/pkg/FR-2016-04-01/html/2016-07381.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-29.
- Original: `original/fr-uspto-part42-2016/002-FR-2016-09814.htm` as served (sha256 9dd6ff8f85b99b6602e6409c6b8ca83c6c813edfd8c88c64cffd58d4771e09b5); extraction sha256 92ac19c44ca15f2175f3a5145eec571075ce0977b8829e2cffb98b4cac6cb76a. Served at https://www.govinfo.gov/content/pkg/FR-2016-04-27/html/2016-09814.htm; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-29.

### `ptab-tpg.txt` — PTAB Consolidated Trial Practice Guide (Nov. 2019, updated Oct. 2025), Appendix A-1

- A `## PTAB Trial Practice Guide` section, the appendix list of the
  guide's table of contents, then one `## PTAB Trial Practice Guide App.
  A-1` section: the sample scheduling order's § A.7 (Motion to Amend), § B (Due Dates) and its Due
  Date Appendix (pp. 53-60 of the PDF), the default times `lit/ptab`'s
  reply and sur-reply periods cite. Extracted with `pdftotext -layout`;
  lines joined into paragraphs, page numbers and the LEAP footnotes
  dropped, and the Due Date Appendix table set one due date per line
  (`DUE DATE N — papers — interval`) with its footnote 5 as `[note 5]`.
  The guide carries no edition date of its own; `as_of 2025-10-01` is the
  "oct2025" of its file name.
- sha256 (this file): `026541d23fd0960fa0f2dd27d9f693de71aeeefb28b041076041b098c3c1ae19`
- Original: `original/ptab-tpg/001-trial_practice_guide_nov2019_oct2025_web.pdf` as served (sha256 260c05b6def096819a9bf724438247b34fe839ba4d8351bb2339d6fa20c512b3); extraction sha256 026541d23fd0960fa0f2dd27d9f693de71aeeefb28b041076041b098c3c1ae19. Served at https://www.uspto.gov/sites/default/files/documents/trial_practice_guide_nov2019_oct2025_web.pdf; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-29.

### `scotus-rules.txt` — Rules of the Supreme Court of the United States 10, 13, 29 and 30 (adopted February 17, 2026, effective March 16, 2026)

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
- Added 2026-10-28 (lg-pd9k), for `lit/cafc`'s certiorari cites: `## Sup.
  Ct. R. 10` (the whole rule, which has no numbered paragraphs: its
  heading, the opening paragraph, (a)-(c) and the closing sentence, one
  line each), from the same PDF (re-fetched 2026-10-28, same sha256) by
  the same method; the four line-break hyphens (although, decision,
  departure, important) were dropped by reading. Rule 10 reads word for
  word as in the 2023 edition (re-fetched 2026-10-28, same sha256 as
  above), which renders the ligatures without glyphs.
- sha256 (this file): `899c67fce982d796557bb85b0456a59bab736cab0274f7e4e75c84105067d2fa`
- Original: `original/scotus-rules.pdf` as served (sha256 5200e38d590cc40929cb719987c936dc529717b4037259d1b63a2309cb6b0f32); extraction sha256 899c67fce982d796557bb85b0456a59bab736cab0274f7e4e75c84105067d2fa. Served at https://www.supremecourt.gov/filingandrules/2026RulesoftheCourt_WEB.pdf; retrieved 2026-10-24 (fetched as recorded above).

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

### `scotus-rules-1980-2019.txt` — Rules of the Supreme Court on service and computation of time, editions effective June 30, 1980, January 1, 1990, October 2, 1995, May 1, 1997, May 3, 1999, November 13, 2017 and July 1, 2019

- The service, computation-of-time and effective-date paragraphs `lit/core`'s
  earlier versions of `sup-ct-30` cite (lg-6n31): for 1980, Rules 28.3,
  29.1 and 55; for 1990, Rules 29.3, 30.1 and 48; for 1995, 1997, 1999,
  2017 and 2019, Rules 29.3, 30.1 and 48.1. Sections `## Sup. Ct. R.
  (YYYY) N.P`, `YYYY` the edition's effective year (the 1990 edition was
  adopted December 5, 1989), each restating its rule's heading as its first
  line (the 1980 and 1990 editions print the heading in capitals on its own
  line under `Rule N`; it is given as `Rule N. Title`), then the paragraph
  on one line. The 2003-2013 editions are `change-scotus-rules-2003-cert-excerpt.txt`
  and `change-scotus-rules-2005-2013-cert-excerpt.txt`.
- Fetched 2026-10-29 from the Court's prior-rules page
  (`www.supremecourt.gov/ctrules/scannedrules.aspx`). `pdftotext -layout`,
  running headers, page numbers and print-job lines dropped, lines joined
  into one line per paragraph, line-break and soft hyphens joined, the
  "fi" ligature restored. The 1980-1999 editions are scans whose text layer
  is OCR; its misreadings were corrected, and only these: 1980 "shaH"
  (shall), "cowlsel" (counsel), "delivery." and "mailbox." (delivery, and
  mailbox,), "party." (party,), "mail ," (mail,); 1990 ". 3." and the
  space in "holidays ." dropped; 1995-1999 "AIty" (Any),
  ".If" (If), "counselor to" (counsel or to), "e>.'tend" (extend),
  "Saturd·ay" (Saturday), "May 3,1999" (May 3, 1999). The 1980 edition as
  revised November 21, 1980 (`rules_1980r.pdf`, not vendored) reads Rules
  28.3 and 29.1 alike. No other wording changed.
- Rule 29.1 of 1980 rolls a last day off "a Sunday or a federal legal
  holiday" only: a Saturday last day stands. Saturdays join from 1990.
  Service by commercial carrier is not in Rule 29.3 until the 2003 edition;
  the 2017 and 2019 paragraphs read word for word as the 2023 edition's
  (`scotus-rules-2023.txt`).
- sha256 (this file): `6a87d3305cc1eaf5944787e4d490bbe82608d1fe224ccc8baf595ca0d1dd6250`
- Original: `original/scotus-rules-1980-2019/001-pdfs-rules-rules_1980.pdf` as served (sha256 8141ea65216989b123712e8dd2c6a3a5e0cf639cb98bfff124064d093d579c85); extraction sha256 6a87d3305cc1eaf5944787e4d490bbe82608d1fe224ccc8baf595ca0d1dd6250. Served at https://www.supremecourt.gov/pdfs/rules/rules_1980.pdf; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-29.
- Original: `original/scotus-rules-1980-2019/002-pdfs-rules-rules_1989.pdf` as served (sha256 c06c15a4d3256933932075adb30302f4395a015dac2263af98e77f6e5bc57ddb); extraction sha256 6a87d3305cc1eaf5944787e4d490bbe82608d1fe224ccc8baf595ca0d1dd6250. Served at https://www.supremecourt.gov/pdfs/rules/rules_1989.pdf; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-29.
- Original: `original/scotus-rules-1980-2019/003-pdfs-rules-rules_1995.pdf` as served (sha256 1030c2a292e54f0c3cc777c59d8914b48c3864c2ddc44db960f600812a871b33); extraction sha256 6a87d3305cc1eaf5944787e4d490bbe82608d1fe224ccc8baf595ca0d1dd6250. Served at https://www.supremecourt.gov/pdfs/rules/rules_1995.pdf; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-29.
- Original: `original/scotus-rules-1980-2019/004-pdfs-rules-rules_1997.pdf` as served (sha256 74f8a57ba0f50b18f853ed518a0e8b49afdb980c967abd08617a0cfa41cbb6e8); extraction sha256 6a87d3305cc1eaf5944787e4d490bbe82608d1fe224ccc8baf595ca0d1dd6250. Served at https://www.supremecourt.gov/pdfs/rules/rules_1997.pdf; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-29.
- Original: `original/scotus-rules-1980-2019/005-pdfs-rules-rules_1999.pdf` as served (sha256 c102ab44a2cefd7fda31c7e121f5ddcfb235f808233cec3741682ac316282066); extraction sha256 6a87d3305cc1eaf5944787e4d490bbe82608d1fe224ccc8baf595ca0d1dd6250. Served at https://www.supremecourt.gov/pdfs/rules/rules_1999.pdf; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-29.
- Original: `original/scotus-rules-1980-2019/006-filingandrules-2017RulesoftheCourt.pdf` as served (sha256 59ef6dc90b201c04ac5d5691dbedc459bc0689007e857ad8d01df7b49f70837e); extraction sha256 6a87d3305cc1eaf5944787e4d490bbe82608d1fe224ccc8baf595ca0d1dd6250. Served at https://www.supremecourt.gov/filingandrules/2017RulesoftheCourt.pdf; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-29.
- Original: `original/scotus-rules-1980-2019/007-filingandrules-2019RulesoftheCourt.pdf` as served (sha256 6ab10a2eb3dbca0e4c592dff41a23d27d58f61efafa048c6de7540c6702f337f); extraction sha256 6a87d3305cc1eaf5944787e4d490bbe82608d1fe224ccc8baf595ca0d1dd6250. Served at https://www.supremecourt.gov/filingandrules/2019RulesoftheCourt.pdf; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-29.

### `usc-15-lanham.txt` — Lanham Act sections (12 sections of 15 U.S.C.)

- Source: the USPTO compilation "US Trademark Law - Rules of Practice and
  Federal Statutes" dated 2026-07-15, `pdftotext -layout`, then the sections
  15 U.S.C. §§ 1051, 1052, 1057, 1063, 1064, 1068, 1071, 1114, 1116, 1117, 1125
  and 1127 cut at the `§ N (15 U.S.C. § NNNN)` heading lines (form feeds and
  running page headers removed). No wording changed. Used by the TTAB pack.
- sha256 (this file): `9ea6b13b33cf0bb61a2a1ea337097c0981665d4d30a045fece2a77324dae6c4c`
- Original: `original/usc-15-lanham.pdf` as served (sha256 58805be56f2dec79c4220800fb4c6d4207a17fc4dd3277bd44b859ec7e3b2830); extraction sha256 9ea6b13b33cf0bb61a2a1ea337097c0981665d4d30a045fece2a77324dae6c4c. Served at https://www.uspto.gov/sites/default/files/documents/tmlaw.pdf; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.

### `tbmp.txt` — Trademark Trial and Appeal Board Manual of Procedure, the sections the TTAB package cites (June 2026)

- The TBMP sections the TTAB package cites, `TBMP § N` or `TBMP N`: § 112
  (a period running from a prior due date counts from "the actual due date
  ..., not the adjusted date under 37 C.F.R. § 2.196"), § 113.05 (no added
  time after service by mail), § 509 (motions to extend or reopen time:
  FRCP 6(b) "made applicable to Board proceedings by 37 C.F.R. § 2.116(a)"),
  and the practice the opposition graph cites: §§ 207.03, 212, 309.03,
  309.03(d), 310.03, 310.03(a), 311, 312, 313.02, 401.01, 401.02, 528.05,
  528.05(a), 528.05(a)(2), 602.01, 605, 702.01, 702.04, 801, 901.01 and
  902.02. Each cited section is cut with its numbered subsections, except
  § 309.03, whose subsections (a)-(c) (the pleading grounds, which the
  graph does not cite) are left out, and §§ 310.03 and 528.05, kept for
  their headings only. One `## TBMP § N` section each (each numbered
  subsection its own), the section's own heading as its first line;
  § 528.05(a) bears no title in the manual, so its heading line is the bare
  number. Sections that open with the rule texts the manual reprints keep
  them as printed.
- Fetched 2026-10-23 from
  `www.uspto.gov/sites/default/files/documents/tbmp-Master-June2026.pdf`
  (sha256 `015cf2b665d74b25080321f67dad7ffebe412118f6f0460ec3530f4801639b8e`,
  the digest `lit/ttab/Lit.toml` records for its `tbmp` source),
  `pdftotext` (no `-layout`); running page headers and page numbers
  removed, form feeds dropped and each section's NOTES (its footnoted
  authorities) omitted. No wording changed. The TBMP binds no one: it
  restates the Board's practice under 37 C.F.R. Part 2. § 509 was cut the
  same way on 2026-10-24, and the remaining sections on 2026-10-28, from the
  same PDF (digest checked again).
- sha256 (this file): `322faf8011de80f6cfde7139ba5366d9c24d5fed71fe1867a0c6d249b8263ec2`
- Original: `original/tbmp.pdf` as served (sha256 015cf2b665d74b25080321f67dad7ffebe412118f6f0460ec3530f4801639b8e); extraction sha256 322faf8011de80f6cfde7139ba5366d9c24d5fed71fe1867a0c6d249b8263ec2. Served at https://www.uspto.gov/sites/default/files/documents/tbmp-Master-June2026.pdf; retrieved 2026-10-23 (fetched as recorded above).

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

### `change-frcp-1938-rule6-excerpt.txt` — FRCP 6(a), 6(e) and 86 as adopted December 20, 1937 (308 U.S. 645)

- The computation rule of the original Federal Rules of Civil Procedure, in force September 16, 1938 until the amendments of January 21, 1963 took effect July 1, 1963 (a Sunday or legal holiday rolls the last day; a Saturday does not): the counting rule `frcp6-1938` of the base layer `change-frcp-1948-appeal` (lg-2j7e, `docs/litigate/change/frcp-1948`). Rule 6(a), 6(e) and Rule 86, in sections `## FRCP (1938) Rule N`, each restating its rule's heading, then one line per subdivision.
- From the U.S. Reports, vol. 308, back matter (GPO): <https://www.govinfo.gov/content/pkg/USREPORTS-308/pdf/USREPORTS-308.pdf>, the volume retrieved 2026-10-23 for lg-fq2.2 (sha256 `180cb7e5e277f8acaecaa2feded8a6d0a94b7c69f4c0a30747a655d944d1456a`, 135 MB). Text taken from that pass's `pdftotext -layout` extractions `change-corpus/rules/us-reports/ur308-civ-1938-text.txt` (PDF pages 740-742, printed 669-670) and `ur308-civ-1938-order.txt` (Rule 86, PDF p. 838); running heads and page numbers dropped, lines joined into one per paragraph, line-break hyphens joined, OCR letter-spacing in the headings closed ("Comput ation" → "Computation"). No wording changed.
- sha256 (this file): `d215d6a8fea0fc3c2184ddc586fee237acc658808298bbff33fcd406b53b230c`
- Original: not stored. The volume PDF is 135 MB, over the corpus's 20 MB cap, as for every U.S. Reports volume of `change-corpus/rules` (its README: re-derive with `curl -O` of the URL, check the sha256 above, then `pdftotext -layout -f 740 -l 742`).

### `change-frcp-1946-appeal-time-excerpt.txt` — FRCP 6(b), 59(b), 59(e), 73(a) and 86(b) as amended December 27, 1946 (329 U.S. 839)

- The appeal time of the district courts as the 1946 amendments state it, in force March 19, 1948 (Rule 86(b): three months after the first regular session of the 80th Congress adjourned, which the rule does not date): Rule 73(a)'s 30 days from entry, terminated by a timely Rule 50(b), 52(b) or 59 motion and run in full again from the order on it; Rule 59(b) and (e)'s 10 days; Rule 6(b), which may not extend them "except to the extent and under the conditions stated in them". The base layer `change-frcp-1948-appeal` (lg-2j7e), for Foman v. Davis, 371 U.S. 178 (1962) (its footnote 2 quotes this Rule 73(a)). Sections `## FRCP (1946) Rule N`.
- From the U.S. Reports, vol. 329, back matter (GPO): <https://www.govinfo.gov/content/pkg/USREPORTS-329/pdf/USREPORTS-329.pdf>, fetched with curl, retrieved 2026-10-29 (sha256 `d36cb90b0b8aec1919e02c7f3cbc3a97cab9cdd21c97ca445f07294febc0c276`, 70,787,873 bytes, the same bytes `change-corpus/rules/MANIFEST.json` records for lg-fq2.2). `pdftotext -layout` of PDF pages 941 (Rule 6(b), printed 847), 957 (Rule 59, printed 863), 960-961 (Rule 73(a), printed 866-867) and 969 (Rule 86(b), printed 875); Rule 6(c) and the Rule 73 subdivisions after (a) left out; running heads, page numbers and print codes dropped, lines joined into one per paragraph, line-break hyphens joined, OCR letter-spacing in the headings closed, "September 1,1947" given as "September 1, 1947". No wording changed.
- sha256 (this file): `3a8accde6ea0b2d7c0f2260f43eacf74bed06fc881de098aa5548c9ccdf4032d`
- Original: not stored. The volume PDF is 68 MB, over the corpus's 20 MB cap (re-derive as above with the page ranges given).

### `change-frcp-1948-rule73-excerpt.txt` — FRCP 73(a) and 86(c) as amended December 29, 1948 (335 U.S. 919)

- Rule 73(a) as the 1948 amendments restate it, in force the day after the first regular session of the 81st Congress adjourned (Rule 86(c); October 20, 1949, a date the rule does not state), the text in force for appeals from 1949 until the Federal Rules of Appellate Procedure: the 1946 wording with "circuit court of appeals" renamed "court of appeals", the same periods. With the volume's headnote (the order's adoption, transmittal and report dates) and Rule 86(c). Sections `## FRCP (1948) Order` and `## FRCP (1948) Rule N`. The base layer `change-frcp-1948-appeal` (lg-2j7e).
- From the U.S. Reports, vol. 335, back matter (GPO): <https://www.govinfo.gov/content/pkg/USREPORTS-335/pdf/USREPORTS-335.pdf>, fetched with curl, retrieved 2026-10-29 (sha256 `ad2cb4191cb508c0eb0d89e20f27e47623e4be44a875c2cf8bab1b68f4e4e6e8`, 66,299,759 bytes). `pdftotext -layout` of PDF pages 947 (headnote, printed 919), 961-962 (Rule 73(a), printed 933-934) and 970 (Rule 86(c), printed 942); the Rule 73 subdivisions after (a) left out; normalized as above. No wording changed.
- sha256 (this file): `a8d4feaec7d1803613ef0cdb367ad3da0b8bc118bee7862e5cedab468d05575e`
- Original: not stored. The volume PDF is 63 MB, over the corpus's 20 MB cap (re-derive as above with the page ranges given).

### `change-scotus-rules-2003-cert-excerpt.txt` — Rules of the Supreme Court 13, 29.3, 30.1-30.3 and 48.1, edition effective May 1, 2003

- The certiorari-timing paragraphs of the Rules adopted January 27, 2003, effective May 1, 2003, the edition in force until the May 2, 2005 one: the base layer of the criminal certiorari period (lg-94rl, `docs/litigate/change/scotus-2003-criminal`), for Booker's July 21, 2004 petition (`tests/expect/united-states-v-booker`). Rule 13.1, 13.2, 13.3 and 13.5, 29.3, 30.1-30.3 and 48.1, in the sections and format of `change-scotus-rules-2005-2013-cert-excerpt.txt` (`## Sup. Ct. R. (2003) N.P`).
- Fetched 2026-10-29 from the Court's prior-rules page: `www.supremecourt.gov/pdfs/rules/rules_2003.pdf`, a scan whose text layer is OCR (pdf pages 13-14, 40, 42 and 60; printed pages 9-10, 36, 38-39 and 56). `pdftotext -layout`, running headers and page numbers dropped, lines joined into one line per paragraph, line-break hyphens joined. The OCR misreadings (split words such as "t he", "fil ed", "certi o-rari"; misread characters such as "01'" for "or", "ordel'", "'ilate" for "date", "writ at certi oj'ari" for "writ of certiorari", "p~riod 3hall bo?" for "period shall be", "counselor to" for "counsel or to", "tc file"; stray marks such as "·", "'" and "." runs; and the colon OCR reads after "jurisdictionally out of time" in 13.2) were corrected to the 2005 edition's wording where the two read alike, and in 13.3's differing clause to the OCR's own legible words. Rule 48.1 reads "May 1,2003" in the text layer, given as "May 1, 2003". No other wording changed.
- Rules 13.1, 13.2, 13.5, 29.3 and 30.1-30.3 read word for word as in the 2005 edition. Rule 13.3 differs: the period runs from the denial of a timely rehearing petition below ("the denial of the petition for rehearing or, if the petition for rehearing is granted, the subsequent entry of judgment"); the 2005 edition adds a lower court's entertaining an untimely petition or considering rehearing sua sponte.
- sha256 (this file): `bcae90de46622b558e7950dcc3a21deff62093b980f635620d86f3f9d171ce50`
- Original: `original/change-scotus-rules-2003-cert-excerpt/001-pdfs-rules-rules_2003.pdf` as served (sha256 2a67774e0cedb132ca80a161714519fbe9c0da6e2b837ae4047c024319e51ffe); extraction sha256 bcae90de46622b558e7950dcc3a21deff62093b980f635620d86f3f9d171ce50. Served at https://www.supremecourt.gov/pdfs/rules/rules_2003.pdf; fetched with curl, retrieved 2026-10-29.

### `change-scotus-rules-2005-2013-cert-excerpt.txt` — Rules of the Supreme Court 13, 29.3, 30.1-30.3 and 48.1, editions effective May 2, 2005, October 1, 2007, February 16, 2010 and July 1, 2013

- The certiorari-timing paragraphs of four editions of the Rules of the Supreme Court, the base layer of the certiorari periods (lg-wvlf.5, `docs/litigate/change/scotus-base`): Rule 13.1, 13.2, 13.3 and 13.5 (the 90 days, jurisdictionally out of time, rehearing below, a Justice's extension), 29.3 (service), 30.1-30.3 (computation; the extension application's timing) and 48.1 (the edition's adoption and effective dates). Sections `## Sup. Ct. R. (YYYY) N.P`, each restating its rule's heading as its first line, then the paragraph on one line.
- Fetched 2026-10-28 from the Court's prior-rules page (`www.supremecourt.gov/ctrules/scannedrules.aspx`): `www.supremecourt.gov/pdfs/rules/rules_2005.pdf`, `rules_2007.pdf`, `rules_2010.pdf`, `rules_2013.pdf`. `pdftotext -layout`, running headers, page numbers and print-job lines dropped, lines joined into one line per paragraph, line-break and soft hyphens joined, the "fi"/"fl" ligatures restored. The 2005 and 2007 editions are scans whose text layer is OCR; its misreadings were corrected against the page images, and only these: 2005 "cow't" (court), "e. g. ," (e. g.,), "counselor to" (counsel or to); 2007 "cow·t" (court), "t he", "par ties", "entl'y" (entry), "t his", "fi ling", "extraordinar y", "these Rule;" (these Rules;, read on the page image of Rule 30.2), "counselor to", "coun - sel", "shaH" (shall). The joined "cross-" line break of Rule 13.4 is not excerpted. No other wording changed.
- The four editions state Rules 13.1-13.3 and 30.1 word for word alike; 13.5 adds, from 2013, that the application must identify each party it is sought for; 29.3 adds, from 2013, the electronic copy, which is not service. The 2013 edition was followed by the Rules adopted September 27, 2017, effective November 13, 2017 (`www.supremecourt.gov/filingandrules/2017RulesoftheCourt.pdf`, not vendored here).
- sha256 (this file): `33505bc8ebd05499cfc4bbe8e501edff0bf6cfa8cebdef3f20d756a91642baad`
- Original: `original/change-scotus-rules-2005-2013-cert-excerpt/001-pdfs-rules-rules_2005.pdf` as served (sha256 ccd71fc9b01b2e70687e13d185f922aa1ca13316882fc901ea47c0645c71f1b2); extraction sha256 33505bc8ebd05499cfc4bbe8e501edff0bf6cfa8cebdef3f20d756a91642baad. Served at https://www.supremecourt.gov/pdfs/rules/rules_2005.pdf; fetched with curl, retrieved 2026-10-28.
- Original: `original/change-scotus-rules-2005-2013-cert-excerpt/002-pdfs-rules-rules_2007.pdf` as served (sha256 b7bad6e7d39c5aa0cc9e282aeb5005ee304ce51fc0aaa25cf1730d891fbbe552); extraction sha256 33505bc8ebd05499cfc4bbe8e501edff0bf6cfa8cebdef3f20d756a91642baad. Served at https://www.supremecourt.gov/pdfs/rules/rules_2007.pdf; fetched with curl, retrieved 2026-10-28.
- Original: `original/change-scotus-rules-2005-2013-cert-excerpt/003-pdfs-rules-rules_2010.pdf` as served (sha256 161f3acd66cc158c63c57abd790fc0aafcc897a1469baf8eb144f350cb75d696); extraction sha256 33505bc8ebd05499cfc4bbe8e501edff0bf6cfa8cebdef3f20d756a91642baad. Served at https://www.supremecourt.gov/pdfs/rules/rules_2010.pdf; fetched with curl, retrieved 2026-10-28.
- Original: `original/change-scotus-rules-2005-2013-cert-excerpt/004-pdfs-rules-rules_2013.pdf` as served (sha256 d21dcd33c469d211f8aa7765e8ade99b5ce78186b736b5e46d27ec4f567e8db2); extraction sha256 33505bc8ebd05499cfc4bbe8e501edff0bf6cfa8cebdef3f20d756a91642baad. Served at https://www.supremecourt.gov/pdfs/rules/rules_2013.pdf; fetched with curl, retrieved 2026-10-28.

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
### `waetzig-v-halliburton.txt` — Waetzig v. Halliburton Energy Services, Inc., No. 23-971 (U.S. Feb. 26, 2025), slip opinion

- The Supreme Court's slip opinion, extracted with `pdftotext -layout`; no
  wording changed. Added 2026-10-26 (714d7bac). supremecourt.gov now serves
  the case only as the preliminary print of 604 U.S. 305
  (`https://www.supremecourt.gov/opinions/24pdf/604us1r14_6kg7.pdf`), so
  the slip PDF was re-fetched from Cornell LII's copy of it; that PDF's
  `pdftotext -layout` extraction is byte-identical to this file.
- sha256 (this file): `4325f20a34f4be3e51cc2b96eea4ef6ca20b8e50a1834ad8b5d51701853789ff`
- Original: `original/waetzig-v-halliburton.pdf` as served (sha256 6618d7c6939907c4de5c23b0673bb9c8fce219fb84083d0e42558ed9a9235c91); extraction sha256 4325f20a34f4be3e51cc2b96eea4ef6ca20b8e50a1834ad8b5d51701853789ff. Served at https://www.law.cornell.edu/supct/pdf/23-971.pdf (the slip opinion as issued by the Court); fetched with curl, retrieved 2026-10-28.

### `crst-v-eeoc.txt` — CRST Van Expedited, Inc. v. EEOC, 578 U.S. 419 (2016)

- United States Reports vol. 578, pp. 419-436, from GovInfo, extracted with
  `pdftotext -layout`; no wording changed. Added 2026-10-26 (714d7bac);
  re-fetched 2026-10-28 and the extraction reproduced byte for byte.
- sha256 (this file): `3598727ae67d2b2a67986920aedf82b13435dadf4e1ab45a5b915c69f3a34b40`
- Original: `original/crst-v-eeoc.pdf` as served (sha256 5ee0b16242e5c8656bed03ede66575dbbdb53585fbe08e87c5f9e6aa0d884c27); extraction sha256 3598727ae67d2b2a67986920aedf82b13435dadf4e1ab45a5b915c69f3a34b40. Served at https://www.govinfo.gov/content/pkg/USREPORTS-578/pdf/USREPORTS-578-419.pdf; fetched with curl, retrieved 2026-10-28.

### `cadkin-v-loose.txt` — Cadkin v. Loose, No. 08-55311 (9th Cir. June 26, 2009)

- The Ninth Circuit's official published opinion, extracted with
  `pdftotext -layout`; no wording changed. Added 2026-10-26 (714d7bac);
  re-fetched 2026-10-28 and the extraction reproduced byte for byte.
- sha256 (this file): `52a8a2fa29edf0fed607bda629e6e8788a22e7b272d16ffd31f1fff4274e106b`
- Original: `original/cadkin-v-loose.pdf` as served (sha256 481fb5c8dc2ad53681de70a8360d220b477230a40cb88d01e25efbb013a8c557); extraction sha256 52a8a2fa29edf0fed607bda629e6e8788a22e7b272d16ffd31f1fff4274e106b. Served at https://cdn.ca9.uscourts.gov/datastore/opinions/2009/06/26/08-55311.pdf; fetched with curl, retrieved 2026-10-28.

### Eighth Circuit Bankruptcy Appellate Panel source set

The files below were added for the Eighth Circuit BAP outline (8ac2b842,
retrieved 2026-10-27). Each original was re-fetched 2026-10-28 from the URL
its entry gives, and the vendored text checked against it word for word.
The Eighth Circuit Local Rules (June 17, 2024) this package also cites are
vendored once, as `ca8/ca8-local-rules.txt`; see `ca8/PROVENANCE.md`.

Leading decisions named in the outline were checked against their published
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

### `frbp.txt` — Federal Rules of Bankruptcy Procedure, December 1, 2025 edition

- The Administrative Office of the U.S. Courts official PDF (the current
  edition on its current-rules page, with the 2025 amendments effective
  December 1, 2025), retrieved 2026-10-27. Extracted with `pdftotext
  -layout`; the front matter and table of contents were dropped and each
  rule headed `## FRBP N` for cite resolution. Rule text was not edited:
  apart from those markers, every word of this file is in the extraction
  of the PDF as served 2026-10-28, in order.
- sha256 (this file): `0c8e487aa7093bcee260d9037c459344f489b4096d3cdd91177ebb2d76c403d8`
- Original: `original/frbp.pdf` as served (sha256 664156bed2742130b7c26b6f632ee99544b4b784801448cfca6c5ee16d681ad8); extraction sha256 0c8e487aa7093bcee260d9037c459344f489b4096d3cdd91177ebb2d76c403d8. Served at https://www.uscourts.gov/sites/default/files/document/federal-rules-of-bankruptcy-procedure.pdf; fetched with curl, retrieved 2026-10-28.

### `ca8-bap-local-rules.txt` — Eighth Circuit Bankruptcy Appellate Panel Local Rules, revised December 1, 2023

- The court's official PDF (the edition listed on the Panel Rules and
  Publications page), retrieved 2026-10-27. Extracted with `pdftotext
  -layout`; the table of contents was dropped and each local rule headed
  `## L.R. BAP 8th Cir. N` for cite resolution. Rule text was not edited:
  apart from those markers, every word of this file is in the extraction
  of the PDF as served 2026-10-28, in order.
- sha256 (this file): `ccc1e2198d478e0b31cd6c4d700bd699eb7e4993f073b3b894cb2f0cf48108ca`
- Original: `original/ca8-bap-local-rules.pdf` as served (sha256 791132e2c51ba53300e9d4af7193733c3c0dd1199aabc384284fdf3c4728b3d3); extraction sha256 ccc1e2198d478e0b31cd6c4d700bd699eb7e4993f073b3b894cb2f0cf48108ca. Served at https://ecf.ca8.uscourts.gov/newbap/rules/localrules_bapDec2023.pdf; fetched with curl, retrieved 2026-10-28.

### `ca8-bap-iop.txt` — Eighth Circuit Bankruptcy Appellate Panel Internal Operating Procedures Manual, revised January 2024

- The court's official PDF, retrieved 2026-10-27. Extracted with
  `pdftotext -layout`; no wording changed (the extraction of the PDF as
  served 2026-10-28 matches this file word for word; only whitespace and
  page breaks differ).
- sha256 (this file): `2fb5815bf87b25d8af30298fa0fa21e148d81ddc6992c305bd94716c68fb0485`
- Original: `original/ca8-bap-iop.pdf` as served (sha256 bbc4c6ea689d554ebc20ae3d3ae24cb0202c95df6d36cbd71d3e3f37411260c0); extraction sha256 2fb5815bf87b25d8af30298fa0fa21e148d81ddc6992c305bd94716c68fb0485. Served at https://www.ca8.uscourts.gov/sites/ca8/files/iops%20jan%202024.pdf; fetched with curl, retrieved 2026-10-28.

### `usc-28-158.txt` — 28 U.S.C. § 158 (bankruptcy appeals)

- The Office of the Law Revision Counsel page (text current through Public
  Law 119-120, October 7, 2026), retrieved 2026-10-27. HTML navigation
  removed, tags stripped and entities decoded; the statutory text is
  word for word the page as served 2026-10-28. The closing `## versions`
  block (`as_of 2026-10-07`) was transcribed from the section's source
  credit on that page: Pub. L. 98-353 (July 10, 1984), 101-650 (Dec. 1,
  1990), 103-394 (Oct. 22, 1994), 109-8 (Apr. 20, 2005) and 111-327
  (Dec. 22, 2010).
- sha256 (this file): `a0f615748b6a10805e6eeec6a05e94d7a3fc2dcd5fafbe94efae7af8ebf9987b`
- Original: `original/usc-28-158.html` as served (sha256 37906378d6069c6b5ca07640ef70d1d9ae9fa84cb074c2d90d95c798c320ce76); extraction sha256 a0f615748b6a10805e6eeec6a05e94d7a3fc2dcd5fafbe94efae7af8ebf9987b. Served at https://uscode.house.gov/us/usc/119-118/t28/s158/; fetched with curl, retrieved 2026-10-28.

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


