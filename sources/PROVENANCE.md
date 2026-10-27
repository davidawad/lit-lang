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

### `cadc-handbook-2026-09-17.txt`

- Official D.C. Circuit Handbook of Practice and Internal Procedures:
  <https://www.cadc.uscourts.gov/sites/cadc/files/rules-Handbook20260917.pdf>
- Retrieved 2026-10-27. Extracted with `pdftotext -layout`.
- Original PDF sha256:
  `1fd1eebbba92611c2b99ae08d5078b841ae6b9a76eef99ced920fa9bc9bb4d5e`.
- Vendored text sha256:
  `e541d181253bd50730add3b957ecb760cbb537ae3eb6020aef842fd896fe60d1`.

## Format

Each `.txt` file is one or more sections, each starting at column 0 with
`## <canonical ref>` (matching the ref strings `litigate_engine::cite::normalize`
produces — see `crates/litigate-engine/src/cite/normalize.rs`), optionally followed
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
  `cfr210.N.x` shorthand (see `crates/litigate-engine/src/cite/normalize.rs`'s
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


### `change-frcv-2007-restyled-excerpt.txt` — FRCP as restyled effective December 1, 2007: the order and Rules 6(b), 59(e)

- The Supreme Court's order of April 30, 2007 and the restyled text of Rule 6(b) and Rule 59(e), the lg-fq2.1 base example's pre-2009 Rule 59(e) (10 days, locked by 6(b)(2)). Lines taken from `sources/change-corpus/rules/scotus/frcv07p.txt` (the supremecourt.gov order PDF `frcv07p.pdf`, see that corpus's MANIFEST.json), blank lines and running page headers dropped; no wording changed. Sections `## FRCP (2007) Order`, `## FRCP (2007) Rule 6`, `## FRCP (2007) Rule 59`.
- sha256 (this file): `a7287e6fd94acc165ac1eb4a5cdd5c9f404acc3abfe01182cbfb490c3d766f6e`

### `change-frcv-2009-amendments-excerpt.txt` — FRCP amendments effective December 1, 2009: the order and Rules 6(b)(2), 59(e)

- The Supreme Court's order of March 26, 2009 (with its pending-case clause) and the amended text of Rule 6(b)(2) and Rule 59(e) (28 days), the lg-fq2.1 amendment example's instrument. Lines taken from `sources/change-corpus/rules/scotus/frcv09.txt` (supremecourt.gov `frcv09.pdf`), blank lines dropped; no wording changed.
- sha256 (this file): `0eb155fa9a5911540a1b4a5f1784c944c235f6fd5fa9fe8da29ba716cc6b9690`

### `change-sdny-edny-local-civil-rule-6.txt` — S.D.N.Y./E.D.N.Y. Joint Local Civil Rules 6.1 and 6.4 (effective January 2, 2026)

- Fetched 2026-10-26 from `www.nysd.uscourts.gov/sites/default/files/local_rules/2026-01-02%20-%20EDNY%20and%20SDNY%20Joint%20Local%20Rules%20As%20Amended.pdf` (sha256 `439b9e2c67fd4ca1d1be65a7f28ca53707820bcbabb4b459323ade1a773607af`), `pdftotext -layout`, the text of Local Civil Rule 6.1 and the withdrawn 6.4 with leading spaces and blank lines dropped; no wording changed. The local-rule example's instrument.
- sha256 (this file): `d7ea4a0e259c8609c98efe869d04776cb7e9ca119c701c3760679a01e3110aae`

### `change-cofc-general-order-2025-02.txt` — U.S. Court of Federal Claims General Order No. 2025-02 (Oct. 6, 2025)

- The order's operative text as quoted verbatim in the lg-fq2.1 research register (fetched 2026-10-23 from `www.uscfc.uscourts.gov/sites/cfc/files/general-orders/2025.10.06%20General%20Order%20re%202025%20day%20after%20Thanksgiving.pdf`); a command-line fetch on 2026-10-26 was refused by the court's firewall, so the PDF itself is not hashed here. The general-order example's instrument.
- sha256 (this file): `3928c4ed72588200af3a643584ab3456806bedd3acf116a6d4a463df52ca0a27`

### `change-breyer-general-standing-order-2025.txt` — Judge Charles R. Breyer, General Standing Order (N.D. Cal., rev. July 16, 2025), two paragraphs

- The courtesy-copy and scheduling paragraphs as quoted verbatim in the lg-fq2.1 research register (fetched 2026-10-23 from `cand.uscourts.gov/sites/default/files/standing-orders/General-Standing-Order-Judge-Charles-R.-Breyer-rev-7-16-2025.pdf`); a command-line fetch on 2026-10-26 was refused by the court's firewall. The standing-order example's instrument.
- sha256 (this file): `a36186c206ae59a2578f63e45ba8af6c998a4dcdbc27da1da0729aab4b315283`

### `change-cancino-perez-d-colo-ecf-45.txt` — Cancino-Perez v. McCoy Global, Inc., No. 1:25-cv-00090 (D. Colo.), ECF No. 45 (July 10, 2025), page 1

- Fetched 2026-10-26 from `www.govinfo.gov/content/pkg/USCOURTS-cod-1_25-cv-00090/pdf/USCOURTS-cod-1_25-cv-00090-0.pdf` (sha256 `bb04e7bb83ecd5df14b961703ff01dab86f715f45778048bb192391c76b896bc`), `pdftotext -layout`, the caption and opening paragraphs (which recite the scheduling order's June 20, 2025 deadline, ECF No. 42 § 9.a) with blank lines dropped; no wording changed. A public court record; the case-order example's instrument.
- sha256 (this file): `dbb6f366760d5397318eb4284f3d0cb7ced95e87b16651c1a9d165d442b002c9`

### `change-cfr-37-42-5-71-75-2026-10-20.txt` — 37 C.F.R. §§ 42.5, 42.71, 42.75 (eCFR, as of 2026-10-20)

- Fetched 2026-10-26 from the eCFR versioner API (`www.ecfr.gov/api/versioner/v1/full/2026-10-20/title-37.xml?section=42.5`, `...=42.71`, `...=42.75`), XML tags stripped, one line per paragraph; no wording changed. Headings `## 37 C.F.R. (2026) § N` keep them apart from `cfr-37-part42.txt`, which predates § 42.75. The PTAB base example's text.
- sha256 (this file): `9fda502b4d2026074b411da4c2556222780ccda8154a57275373b42faa9628ee`

### `change-light-wonder-ipr2025-01072-paper-30.txt` — Light & Wonder, Inc. v. Evolution Malta Ltd., IPR2025-01072, Paper 30 (Director June 22, 2026), excerpt

- Fetched 2026-10-26 from `www.uspto.gov/sites/default/files/documents/ipr2025-01072-dr-paper_30.pdf` (sha256 `5cba76995e71eae32656892dde2c1e92265f9bac30be3254f51e48e4f1407c80`), `pdftotext -layout`, the caption and the waiver paragraph (pages 1 and 3-4), footnote markers omitted; no wording changed. Designated precedential June 22, 2026 (PTAB precedential and informative decisions page, per the research register). The precedent example's instrument.
- sha256 (this file): `1e32fb02af2ee2caeb5956f0fb21a93eb5c2ffa6dc3289ec4a73c5902df4edc5`

### `change-thompson-v-ins-375-us-384.txt` — Thompson v. INS, 375 U.S. 384 (1964) (per curiam)

- Fetched 2026-10-26 from `tile.loc.gov/storage-services/service/ll/usrep/usrep375/usrep375384/usrep375384.pdf` (the U.S. Reports, Library of Congress), `pdftotext -layout`, every non-blank line kept with its leading spaces dropped; no wording changed. The precedent the overruling example (Bowles v. Russell, `sources/change-corpus/rulings/bowles-scotus-2007.txt`) withdraws.
- sha256 (this file): `adb3b1d49b1a12c14f58d973942a3dc62825bb5287c56df17f4ed0eac64f9cc9`
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

### `ddc/ddc-local-rules.txt` — Local Rules of the U.S. District Court for the District of Columbia

- Source: https://www.dcd.uscourts.gov/sites/dcd/files/local_rules/Local%20Rules%20SEP_2026.pdf (official court site; 212 pages, sha256 e836e8706dc824294b73933ece7429cdbfc321104305501629b994631ebf5a86).
- Edition: cover: "Effective as of September 2015, Updated: September 2026"; listed as the current edition on https://www.dcd.uscourts.gov/court-info/local-rules-and-orders/local-rules (newest listed change: Local Rule 6.2, 9/14/2026).
- Retrieved 2026-10-27 on carbon (the court site refuses agent pods: lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 e6084a6dc48c050f96035345573a3de47f57eadd0ff4e3d04039f0fb8e537928. Raw extract, not yet sectioned into `## <ref>` form; it sits in a subdirectory, so the binary does not embed it until a pack wires a sectioned file.

### `dc-state-appellate/dcca-rules.txt` — Rules of the District of Columbia Court of Appeals

- Source: https://www.dccourts.gov/sites/default/files/coa-rules/DCCA-Rules.pdf (official court site; 135 pages, sha256 5b71b874d774c74dc7f98b2873d3393363d259dd7edbaf315a18b7962d0197bf).
- Edition: title page: "Reflecting amendments in effect as of May 11, 2026"; linked from https://www.dccourts.gov/court-of-appeals/court-of-appeals-resources/rules-of-the-dc-court-of-appeals (page last updated 02/04/2026).
- Retrieved 2026-10-27 on carbon (the court site refuses agent pods: lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 6ebf6a2ad4b75f15c0acd71ef1a3ce7adf02913d5e41183086c04449f5caeea4. Raw extract, not yet sectioned into `## <ref>` form; it sits in a subdirectory, so the binary does not embed it until a pack wires a sectioned file.

### `dc-state-civil/scr-civil-rules.txt` — Superior Court of the District of Columbia Rules of Civil Procedure

- Source: https://www.dccourts.gov/sites/default/files/superior-rules/Superior-Court-Rules-of-Civil-Procedure-Download-All-Rules-Version.pdf (official court site; 296 pages, sha256 b46082a8edee555671302aab2203905abb57f965e98d3e13c6ccbf89976e9c43).
- Edition: the document states no edition date; it carries comments to 2026 amendments, its PDF is dated 2026-05-13, and it is the all-rules version linked from https://www.dccourts.gov/superior-court/superior-court-resources/rules-of-the-superior-court (page last updated 02/13/2026, before the PDF).
- Retrieved 2026-10-27 on carbon (the court site refuses agent pods: lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 bbf2960ed59c98a8d7a32ea831919317ad59e41f63e9786fb1f27ebf13736f23. Raw extract, not yet sectioned into `## <ref>` form; it sits in a subdirectory, so the binary does not embed it until a pack wires a sectioned file.

### `ared/ared-arwd-local-rules.txt` — Local Rules of the U.S. District Courts for the Eastern and Western Districts of Arkansas

- Source: https://www.are.uscourts.gov/sites/are/files/local_rules/All_LR.pdf (official court site; 84 pages, sha256 9fc211b94a6b06e77953f28f1b88c5e61f8d567dc68d71a78a91eee7715fa459).
- Edition: title page: "Local Rules of the United States District Court for the Eastern and Western Districts of Arkansas", last amendment listed September 21, 2017; the "Complete Copy" on https://www.are.uscourts.gov/court-info/local-rules-and-orders/local-rules, last modified 2020-11-17, newer than every individual rule PDF the page lists. One joint text: it also governs forum arwd (the W.D. Ark. site copy is a 2016 file, not vendored).
- Retrieved 2026-10-27 on carbon (the court site refuses agent pods: lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 0bc073ca4ceb05726d03a65d05a6294c20199591be8b23dc465ff14d724c8950. Raw extract, not yet sectioned into `## <ref>` form; it sits in a subdirectory, so the binary does not embed it until a pack wires a sectioned file.

### `ar-circuit/arcp.txt` — Arkansas Rules of Civil Procedure

- Source: https://opinions.arcourts.gov/ark/cr/en/16712/1/document.do (official court site; 196 pages, sha256 b72f17dcaf0261c1b94a76509f2a35ef4d6eceeb21c613a3aea13d682e19c0d6).
- Edition: title: "Rules of Civil Procedure (current through October 1, 2026)"; Arkansas Judiciary court rules portal https://opinions.arcourts.gov/ark/cr/en/nav_date.do lists it current to 10/01/2026.
- Retrieved 2026-10-27 on carbon (the court site refuses agent pods: lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 9e7950b1cca7fc771ef258b4ac2190375118c517646958b83e9c5bb3e2c01430. Raw extract, not yet sectioned into `## <ref>` form; it sits in a subdirectory, so the binary does not embed it until a pack wires a sectioned file.

### `arkctapp/arap-civil.txt` — Arkansas Rules of Appellate Procedure—Civil

- Source: https://opinions.arcourts.gov/ark/cr/en/1885/1/document.do (official court site; 22 pages, sha256 c854fb5600e8cb1595791154a4befca952fd984ae346f1e4aacf45672923d36e).
- Edition: title: "Rules of Appellate Procedure—Civil (current through October 1, 2026)"; listed current to 10/01/2026 on https://opinions.arcourts.gov/ark/cr/en/nav_date.do.
- Retrieved 2026-10-27 on carbon (the court site refuses agent pods: lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 baa4d8fdce843d9f59ad162212be7de918cd3ab5566e6ae9c6a2bbdf530dfdda. Raw extract, not yet sectioned into `## <ref>` form; it sits in a subdirectory, so the binary does not embed it until a pack wires a sectioned file.

### `arkctapp/ar-supreme-court-rules.txt` — Rules of the Supreme Court and Court of Appeals of the State of Arkansas

- Source: https://opinions.arcourts.gov/ark/cr/en/1871/1/document.do (official court site; 44 pages, sha256 7f26989c58f09c36964c5add5424f6709db177bfc367ea648aab2905b88c4484).
- Edition: title: "Rules of the Supreme Court and Court of Appeals of the State of Arkansas (current through December 15, 2025)"; listed current to 12/15/2025 on https://opinions.arcourts.gov/ark/cr/en/nav_date.do.
- Retrieved 2026-10-27 on carbon (the court site refuses agent pods: lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 039cb6ae5df1eeb38a0be219ebf093b41622768649397394a82652bbaf3c3052. Raw extract, not yet sectioned into `## <ref>` form; it sits in a subdirectory, so the binary does not embed it until a pack wires a sectioned file.

### `ca8/ca8-local-rules.txt` — Local Rules of the United States Court of Appeals for the Eighth Circuit

- Source: https://www.ca8.uscourts.gov/sites/ca8/files/Eighth%20Circuit%20Local%20Rules%206.17.24.pdf (official site; 39 pages, sha256 26cfe150aa5813ffdc7f5c55ef5a2604c02ecc76526d5ee21669806b937a41ac).
- Edition: dated June 17, 2024 on its first page; listed as "Local Rules of the Eighth Circuit, June 2024" on https://www.ca8.uscourts.gov/rules-procedures.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 462e6deddebce2f420f7628c80ce8c617971bdecbfebb4be817ca99772d375b8. Raw extract, not yet sectioned into `## <ref>` form; it sits in a subdirectory, so the binary does not embed it until a pack wires a sectioned file.
