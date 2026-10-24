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
- **Small.** The whole corpus is under 4 MB uncompressed (MPEP's 462 sections
  are most of that) — still a rounding error next to vendoring full PDFs,
  and small enough to embed in the binary the same way `packs/*.json` is.
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

## Format

Each `.txt` file is one or more sections, each starting at column 0 with
`## <canonical ref>` (matching the ref strings `litgraph::cite::normalize`
produces — see `crates/litgraph/src/cite/normalize.rs`), optionally followed
by a heading line, then the verbatim body text up to the next `## ` line.

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

### `frap.txt` — Federal Rules of Appellate Procedure (55 rules)

- Ported from `civ-pro-the-gathering/content/reference/frap-full.json`,
  whose provenance records each rule "extracted from the Cornell LII...
  pages and then verified line by line against the official Administrative
  Office edition (uscourts.gov, December 1, 2024)" (Rules 6/39 verified
  against the Apr. 23, 2025 Supreme Court amendment order instead),
  retrieved 2026-08-15.
- sha256 (this file): `87d71a1f443c5c084cc9f11932a66436ab300f83146ee2c5ce2654a8a4824479`

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

### `usc-19-1337.txt` — 19 U.S.C. § 1337 (ITC unfair-import-practices statute)

- Fetched directly 2026-09-28 from `uscode.house.gov`
  (`https://uscode.house.gov/view.xhtml?req=(title:19%20section:1337%20edition:prelim)`,
  official U.S. House Office of the Law Revision Counsel text), full text
  including subsections (a)-(n). HTML chrome/nav stripped mechanically
  (kept the span from the `§1337.` heading to the `Editorial Notes`
  marker), tags stripped, entities decoded.
- sha256 (this file): `3865d7c3c1b4d9c0883d3eb29fd160c3c789d8cb8ffed675311e5dccd0892408`

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

### `rcfc.txt` — Rules of the U.S. Court of Federal Claims (112 rules, RCFC 1-87)

- The RCFC main-rules PDF is the same one `packs/cofc-court-of-federal-claims.json`'s
  `rcfc-rules` source already pins (`Rules%207.27.2026.pdf`,
  `www.uscfc.uscourts.gov`, as amended through July 27, 2026); re-fetched
  2026-09-29 and confirmed byte-identical (same sha256 as the pack's own
  record: `ff02c342f055be5e5f51df4de1dcec3c1264bb9907895d379d5baa796ec30a97`).
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
- sha256 (this file): `aefd134e00b71dad05886f59cb11479e6381bbf1d5ad7a2c8e63191629761daf`

### `fedcir-rules.txt` — Federal Circuit Rules of Practice (55 numbered local rules)

- Same PDF `packs/cafc-federal-circuit.json`'s `fedcir-rules` source already
  pins (`FederalCircuitRulesofPractice.pdf`, `www.cafc.uscourts.gov`,
  December 1, 2025 edition); re-fetched 2026-09-29, confirmed byte-identical
  (sha256 `b6346ed56a0ef66e8a681765873caf69239cda273f8946f1625571e013712c05`
  matches the pack's own record). `pdftotext` (no `-layout`). This PDF
  combines FRAP-numbered local rules, Attorney Discipline Rules, and
  practice notes in one document; only the `FEDERAL CIRCUIT RULE N` running
  page headers are extracted (Attorney Discipline Rules and IOPs are a
  separate numbering scheme no pack currently cites). A rule spanning
  several pages repeats its own running header each page; those repeats are
  merged into one section rather than treated as a new one.
- sha256 (this file): `f38803d4f0370b737c9fcc85f2013b757a6a7648609637c88e652322de75b1a9`

### `cfr-19-part210.txt` — 19 C.F.R. Part 210 (USITC Rules of Adjudication and Enforcement, 79 sections)

- Fetched 2026-09-29 from `www.govinfo.gov`'s official annual CFR XML
  (`CFR-2024-title19-vol3-part210.xml`, title 19 "Customs Duties", chapter
  II "United States International Trade Commission", 2024-04-01 edition),
  same method and same `<SECTION>`/`<SECTNO>`/`<SUBJECT>`/`<P>` parse as
  `cfr-37-part42.txt` below. Resolves `itc-337.json`'s internal
  `cfr210.N.x` shorthand (see `crates/litgraph/src/cite/normalize.rs`'s
  `try_itc_cfr_slug`), which needs the pack's `forum: "itc"` field to
  disambiguate — set as part of this same pass.
- sha256 (this file): `92d29ab6a95dc52eaaa5b4fa5b1d0991131546e01612bc83f8e365f40420c99e`

### `cfr-37-part1.txt` — 37 C.F.R. Part 1 (Rules of Practice in Patent Cases, all 363 sections)

- Fetched 2026-09-29 from `www.govinfo.gov` (`CFR-2024-title37-vol1-part1.xml`,
  2024-07-01 edition), same method as `cfr-37-part42.txt`. Supersedes the
  17-section Part 1 subset already embedded piecemeal in
  `usc-cfr-prosecution-fragments.txt` (both are kept; a citation resolves
  against whichever file a given pack's `sources[].path` lists, so there is
  no conflict — see that test in `corpus.rs`, `duplicate_headings_across_files_both_appear`).
- sha256 (this file): `e5765fec0e8c5f295d4f230ed9743b7972dbd41340693833abf990fe1918d1ec`

### `cfr-37-part41.txt` — 37 C.F.R. Part 41 (Board of Patent Appeals and Interferences / PTAB ex parte appeal practice, 75 sections)

- Fetched 2026-09-29 from `www.govinfo.gov` (`CFR-2024-title37-vol1-part41.xml`,
  2024-07-01 edition), same method as `cfr-37-part42.txt`/`cfr-19-part210.txt`.
  Resolves `mpep-prosecution.json`'s citations into the examiner's-answer/
  appeal-to-PTAB region (§§ 41.20, 41.45).
- sha256 (this file): `912ba8c173b834a4030c979c0ff3bc6f96d393dcff717c6d14812daba23934ee`

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

### `cfr-19-part201.txt` — 19 C.F.R. §§ 201.14, 201.16 (USITC computation of time, service)

- Fetched 2026-10-22 from `www.govinfo.gov`'s official annual CFR XML
  (`CFR-2024-title19-vol3-part201.xml`, 2024-04-01 edition; sha256 of the
  fetched XML:
  `68669957569c7e7071252a1202cddde538173f019665e38e9965ae9a9741f4e2`), the
  same edition and `<SECTION>`/`<SECTNO>`/`<SUBJECT>`/`<P>` parse as
  `cfr-19-part210.txt` above, for the two sections § 210.6(a) incorporates.
- sha256 (this file): `436ec5247f1328fe6d6e2c15bcefd8f627cc3fcd44f0a0c034c7884609015879`

### `usc-28-app-time-history.txt` — FRCP 6 and FRAP 26 as printed in the 2008 and 2011 editions of 28 U.S.C. App.

- The superseded (pre-December 1, 2009) time-computation texts and the
  amendment history `lit/core`'s earlier counting-rule versions cite:
  `## FRCP (2008 ed.) Rule 6`, `## FRCP (2011 ed.) Rule 6`, `## FRAP (2008
  ed.) Rule 26`, `## FRAP (2011 ed.) Rule 26` (each rule's text, its
  amendment credits, and its Advisory Committee Notes). The headings are
  deliberately not canonical `litgraph::cite::normalize` refs, so pack cites
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

### `cfr-37-part2-answer-time-history.txt` — 37 C.F.R. §§ 2.105, 2.106 and 2.113 (TTAB answer time) as of January 13 and 14, 2017

- The dates `lit/ttab`'s answer-time deadlines are in force from. For each
  section, `## 37 C.F.R. (eCFR versions) § N` opens with a bracketed line
  listing every version the eCFR records for it (2017-01-01, its first
  recorded version, and 2017-01-14, the last amendment), then the section's
  text as of 2017-01-14; `## 37 C.F.R. (eCFR 2017-01-13) § N` is the text
  the day before. The headings are deliberately not canonical
  `litgraph::cite::normalize` refs, so cites of the current sections resolve
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
- sha256 (this file): `93e7715ba5a605f480efdb1b9d82191fc1cfff00a242e066710adcfd298260f8`

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

### `scotus-rules.txt` — Rules of the Supreme Court of the United States 13, 29 and 30 (adopted December 5, 2022, effective January 1, 2023)

- The cert-petition period (Rule 13), service (Rule 29) and computation of
  time (Rule 30) `lit/core`'s `sup-ct-30` counting rule and the FRAP and
  Federal Circuit cert periods cite, one `## Sup. Ct. R. N.P` section per
  numbered paragraph, each restating its rule's heading as its first line.
- Fetched 2026-10-23 from `www.supremecourt.gov/filingandrules/2023RulesoftheCourt.pdf`
  (sha256 `05f435c766fd596333828846d097828238ff89a266c24e3afca9ecd2fb9c19db`),
  `pdftotext` (no `-layout`), running headers and page numbers stripped,
  lines joined into one line per numbered paragraph. The PDF's text layer
  drops the "fi" ligature (so "file" reads "f-l-e"); it was restored word
  by word (file, filed, filing, filers, first, final, specified, specific,
  justified, office, officer, suffices, identifiable, confined, defined,
  certified, certificate, affidavit, Notifications), and two line-break
  hyphens inside words ("inter- nal", "ear- lier") were joined. No other
  wording changed.
- sha256 (this file): `94c56e4ee8ec59d5cf857f1271ee689de1568421ab67e19b71bb823b5feecbb4`

### `usc-15-lanham.txt` — Lanham Act sections (12 sections of 15 U.S.C.)

- Source: the USPTO compilation "US Trademark Law - Rules of Practice and
  Federal Statutes" dated 2026-07-15, `pdftotext -layout`, then the sections
  15 U.S.C. §§ 1051, 1052, 1057, 1063, 1064, 1068, 1071, 1114, 1116, 1117, 1125
  and 1127 cut at the `§ N (15 U.S.C. § NNNN)` heading lines (form feeds and
  running page headers removed). No wording changed. Used by the TTAB pack.
- sha256 (this file): `9ea6b13b33cf0bb61a2a1ea337097c0981665d4d30a045fece2a77324dae6c4c`

### `tbmp.txt` — Trademark Trial and Appeal Board Manual of Procedure §§ 112 and 113.05 (June 2026)

- The TBMP sections the TTAB package's chained briefing periods (§ 112:
  a period running from a prior due date counts from "the actual due date
  ..., not the adjusted date under 37 C.F.R. § 2.196") and its service rule
  (§ 113.05: no added time after service by mail) cite, one `## TBMP § N`
  section each, the section's own heading as its first line.
- Fetched 2026-10-23 from
  `www.uspto.gov/sites/default/files/documents/tbmp-Master-June2026.pdf`
  (sha256 `015cf2b665d74b25080321f67dad7ffebe412118f6f0460ec3530f4801639b8e`,
  the digest `lit/ttab/Lit.toml` records for its `tbmp` source),
  `pdftotext` (no `-layout`); running page headers and page numbers
  removed and each section's NOTES (its footnoted authorities) omitted. No
  wording changed. The TBMP binds no one: it restates the Board's practice
  under 37 C.F.R. Part 2.
- sha256 (this file): `84d70300d5b36ac966e3b65513b86726ebc5fc04c0b97218704a3c55f02a3e86`
