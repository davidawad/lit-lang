# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Reserved typed guard operations in patches** (lg-flu.35; SPEC §3.6,
  PM ruling 2026-10-23 on L2007). A later decision or order that narrows,
  widens or replaces an edge's guard will be written explicitly, with its
  authority (`cite` and `effective`), inside a `patch`: `when and pred`
  (narrow), `when or pred` (widen), `supersedes when pred` (replace). The
  forms parse into a new `GuardOp` CST node and are rejected with the new
  code L0010, whose message begins `reserved: typed guard operations`,
  until lg-flu.36 implements them. `and` and `or` are reserved words
  (L0010 anywhere, `GUARD_OP_WORDS`). A plain `when` in a patch stays
  L2007 (an implicit overwrite) and now points at the explicit forms. The
  tree-sitter grammar (`patch_fields`, `guard_op`) and the Emacs, Helix
  and Neovim highlights, indents and folds follow.

- **Generated files are build outputs, not committed** (lg-flu.31). Two
  beads changing different `lit/` packages used to conflict on
  `packs/*.json`, `packs/links.json`, `packs/clock/mpep-prosecution.json`,
  `scenarios/*.json` and the TypeScript bindings, which cost an
  integration job per parallel change. The new `litgraph-packs` crate runs
  `litigate build` (the arguments in `crates/litgraph-packs/build.args`,
  formerly the justfile's `lit_sync_inputs`/`lit_sync_names`) over an
  in-memory `lit/` and `cases/` in its build script and embeds the result
  (`PACKS`, `SCENARIOS`, `PACKS_DIR`, `SCENARIOS_DIR`, `catalog()`). The
  files are gitignored. `just lit-sync` regenerates the `Lit.lock` files and
  writes the build to `target/lit-build/`. `just lit-sync-check` fails on a
  stale `Lit.lock` or a committed generated file. `just ts-gen` output is
  gitignored, `ts-build` regenerates it, and `ts-gen-check` is gone.
  Docker, nix and release archives copy the built packs. A test rebases
  two branches that edit different packages, with no merge driver, and
  expects no conflicts; its control commits the generated files and
  expects the old conflict. API: `litgraph` no longer embeds packs or
  scenarios, because the compiler depends on it.
  `Catalog::embedded`, `with_embedded_scenarios` and `default_source` now
  take the embedded files (`EmbeddedFiles`), and `litgraph_packs::catalog()`
  replaces `Catalog::embedded()`. `ClockLibrary::embedded()` is now the
  hand-authored library only: a catalog built from `litgraph-packs` carries
  the generated fragments too.
- **Reserved refinement syntax** (lg-962.7; SPEC §3.5, design review
  2026-10-23 §2.2). The SMT tier's surface forms are reserved so adding
  them later cannot break a `.lit` file: a refinement type
  `{x: T | pred}` wherever a type stands, and a `where pred` clause after
  a parameter's or a type declaration's type. Both parse into their own
  CST nodes (`RefinedType`, `WhereClause`) and are rejected with the new
  code L0009, whose message begins `reserved: refinements (SMT tier)`.
  `where` is a reserved word (L0009 anywhere, `REFINEMENT_WORDS`) and `|`
  is a new punctuation token. The tree-sitter grammar (`refined_type`,
  `where_clause`) and the Emacs, Helix and Neovim highlights follow.

- **Node kinds** (design review §6 item 11, §5.2 L13; SPEC §5.1, §5.6,
  §13.7). A node is a choice or a draw, never both: `p` on a decision's or
  a state's edge, or on a link leaving one, is L2008 (a link out of a
  chance node or a terminal may carry it). A `fact` chance node's named
  edges take `when` guards over the matter's history (`has_event`, the new
  builtin `date_of(Pattern) -> Date`, and `deadline(d)` compared as
  dates), pairwise disjoint (L2005); replay derives the fact from the
  guard that holds, a contradicting `facts` entry is L4003, and the fact
  at the replayed posture lowers to `scenario.facts` as if stated. The
  PTAB § 315(b) time bar is derived from `ComplaintServed` and the
  petition date, so `cases/ptab-ipr-defense.case` no longer states it.
  `info s` puts decisions in an information set (same choices, same
  actors, L2001); litgraph has no information sets yet, so such a package
  does not lower (L6001). New contextual keyword `info`; `when` is now
  also an edge field. Data: the Federal Circuit remand router, the Court
  of Federal Claims ADR and settlement outcomes and plea negotiation are
  `chance` nodes; the PTAB Federal Circuit appeal window (and its link)
  is a pure decision, dropping its 0.92/0.08 teaching-estimate draw.

- **Structured citations; pinned source text** (design review §6 item 8,
  S1, K1; SPEC §2, §4.1, §4.3, §7, §8.6; lg-flu.6). **Breaking:** the lexer
  has no citation mode. Citation text is a string, `cite "28 U.S.C. §§
  1491, 1498"`, so commas, braces, `//`, section signs, case names and URLs
  are plain text; an unquoted legacy cite is L0001 with a machine-applicable
  fix that quotes it exactly as citation mode read it, so `litigate fix`
  ports old sources (every `.lit`/`.case` here is migrated and lowers to
  byte-identical packs). A resolved cite is a structured value: work (the
  source binding), expression (the binding's `as_of`), pinpoint, and the
  SHA-256 of the cited section's text, all in the `*.map.json` sidecar.
  A new `[sources.*]` key `path_sha256` pins the vendored file at `path`;
  `cite-resolves` (L1010) rejects an unpinned binding and a corpus file
  that is not the pinned text, instead of resolving against whatever the
  binary embeds. `sha256` now means the document at `url`; the `lit/core`,
  `cafc` and `ttab` bindings that recorded the vendored file's hash there
  move it to `path_sha256`, so their packs no longer emit it. A version's
  cite whose expression is dated on or after the version's end is L1010
  unless marked `historical` (new contextual keyword) or a hole: the pre-2009
  FRCP 6 / FRAP 26 / RCFC versions citing later editions' notes are marked
  `historical`, and the FRCP appeal-window and TTAB answer-period versions
  that cited post-dating text are now holes. The tree-sitter grammar drops
  its external scanner.

- **Order independence** (design review §6 item 4, P7, P13; SPEC §4.1,
  §5.4, §5.6). The engine visits nodes in canonical (qualified id) order
  and a node's links by id, so renaming a module file, reordering a pack's
  nodes or a links file, or listing packs in another order leaves every
  analysis bit-identical. `.case` events are ordered by date, then an
  optional docket sequence number (`2025-03-05 docket 12 Served(...)`, new
  contextual keyword `docket`), never by source order: same-day events
  need distinct docket numbers unless both elect or remove options
  (L4001), and docket numbers must grow with the date. A transition's
  `has_event` reads only the evaluated event and those before it in
  docket order, so a later same-day order no longer changes the version
  governing an earlier event. Transition arms selecting different
  versions must be disjoint (L3008) instead of first-match, and the
  transitions of a world are a set: two transitions amending one version
  are each L3008, and replay refuses to pick one by load order.
- **Ground refinements** (lg-962.3; SPEC §6, §13.6). A `Probability`
  term that does not fold (a reference to a `Probability` default, a
  `prob` or `match` over unknowns) is decided exactly: `+ - *` and `/` by
  a constant keep it an exact linear form over atoms in `[0, 1]`, whose
  range is read off its coefficients, so `a + b` over two such defaults
  is L2006 ("fails when a = 1, b = 1: `a + b` = 2 > 1") and `(1 - a) + a`
  is accepted. A product or quotient of two unknowns is boxed by interval
  arithmetic; a divisor that may be 0 is L2006. No solver is involved.
- **Counting rules reach an open day from every date** (SPEC §5.3). A
  counting rule whose closed days never end is L3009, decided by
  enumerating one 400-year Gregorian cycle (146 097 days, a whole number
  of weeks) on either side of every holiday's `effective` boundary
  (`litgraph::clock::cycle`). Tests and case replay refuse such a rule
  with that reason instead of counting forever.
- **Lowered artifacts carry their world and their holes** (SPEC §8.1,
  §8.3, §8.6; design review 2026-10-23 LW1, LW2). Every generated document
  has a `lowered` header: `pins` (name, version, source digest of every
  package the build saw), `asOf`/`knownAt`, a `worldDigest` (SHA-256 over
  the RFC 8785 text of those), and `holes`, one JSON pointer per explicit
  `?` in an emitted field, so a `dev` build no longer passes its gaps off as
  absent fields. Schema bump: packs are `schemaVersion` 4, `links.json` and
  named scenarios schema 2, generated clock fragments 4; older versions are
  still read. A v2 scenario also records the matter's `parties` and docket
  `events` (in docket order, with their `docket` numbers). The engine checks the header on read (sorted pins, recomputed
  digest, holes inside the document) and warns `authored-hole` for a pack
  whose holes leave a number open.
- **Canonical JSON output.** Generated packs, links, clock fragments,
  scenarios, source maps and litigatables are written as RFC 8785 (JCS)
  text indented by two spaces (members in UTF-16 order, ECMAScript number
  spelling: `1500`, not `1500.0`); litigatable content hashes are over the
  exact JCS text (`litigate_lower::canonical`). `litgraph::canonical`
  provides the serializer. JSON is now read with correctly rounded number
  parsing (`serde_json` `float_roundtrip`), so emitted numbers read back
  bit for bit.

### Changed

- **Typed boundaries** (design review §6 item 12; SPEC §5.2, §5.6, §6).
  Breaking for `.lit` and `.case` source. A `removes p` or `elects p`
  event parameter has the new type `EdgeRef`, and a case writes the edge
  unquoted: `Elected(e-filingfork-standard)`, `Forfeited(frcp::move-dismiss)`.
  The edge is resolved when the event is typed (L1002 with the nearest id
  as a fix); a quoted string is L2001 with a machine-applicable fix
  unquoting it.
  An instance `patch` writes typed edge fields instead of a JSON merge
  patch: `patch rehearing-file { within rehearing-period-united-states-party
  { extendable false, note "..." } }` names the deadline declaration
  instead of repeating its length, and each field is checked as on an
  edge. A written field replaces the base edge's whole field; `event`,
  `replaces` and a second patch of one edge are L2007, and a patch can no
  longer unset a field. `litigate import` spells a legacy merge patch as
  typed fields and reports what has no spelling.
  An instance's `payoff_transform` is a typed `Money<usd>` expression over
  `payoff` and a declared `stake`, not a string:
  `payoff_transform (stake - payoff)`; a number is not money
  (`1_000_000.usd - payoff`). Sema checks it (L1002, L2001, L2002, and
  L2007 for an unread `stake`), and lowering writes the engine's
  expression text.
  A `.case` `analysis` block is a record of expressions typed field by
  field instead of a JSON passthrough (SPEC §8.3): a closed set of fields
  (an unknown one is L4007 with the nearest as a fix), `params` exact
  numbers, engine expressions as strings, `remove_edges` unquoted
  `EdgeRef`s, and the engine options checked alone against the engine's
  schema at their own span. `probabilities` and `policy` are no longer
  accepted, quoted keys and `unset` are gone with the `JsonRecord` syntax,
  and `unset` is no longer a keyword.

### Fixed

- **Probability arithmetic over unknowns was unchecked** (T1): `a + b`
  over two `Probability` defaults was accepted whatever they held.
- **Exact rational products could wrap** (T2): `ad * bd` of two `u64`
  denominators overflowed `i128` (a panic in debug, a silent wrap in
  release). Products now cancel across first and are exact, or L2006 when
  the reduced value does not fit.
- **`dist-exact` skipped nodes** (T4): a chance node with a self edge
  read the self edge's missing `p` as a hole and skipped the exact-sum
  check, and a non-chance node's `p` edges had no sum check. A node's
  `p` edges other than self edges now sum to at most 1, and to exactly 1
  when all carry a known `p` (always for a chance node).

## [0.5.0] - 2026-10-23

### Added

- **`Lit.lock` merges package by package** (SPEC §4.2, lg-flu.22).
  `.gitattributes` routes every `Lit.lock` through the `litlock` merge
  driver, `litigate lock --merge BASE OURS THEIRS`: a package row only one
  side changed takes that side, so parallel `lit/` changes no longer
  conflict in the locks of every dependent package and matter directory. A
  row both sides changed differently gets an unmerged digest, so the lock
  stays stale (L1007) until `just lit-sync`, which now regenerates every
  committed `Lit.lock` (and registers the driver; so does `just install`).
  `just lit-sync-check` fails on a stale `Lit.lock` as well.
- **Obligation instances in `.case` replay** (SPEC §5.3, §5.6). Each
  deadline trigger creates an obligation instance with a bearer (`bearer
  Actor`) under a declared trigger policy (`trigger first | latest | each |
  later_of(d)`; `first` is the default, so a second service no longer
  restarts a period). A `within` act discharges one instance borne by the
  acting party; another party's act cannot. A duty's instance borne by a
  party and open past due at `as_of` is reported as breached (new code
  L4009, a warning). `litigate check` reports every instance with its bearer
  and status in the deadline schedule, and the LSP's hints follow it.
- **Typed extension and waiver.** A deadline declares `extend` grants and
  a `waive { by Actor [, after_expiry] }` capability; an event type
  declares `extends d to p` or `waives d`. A cited event of such a type,
  by a grant's actor and within its window and bound, moves or releases
  every open instance of the deadline. This restores court-ordered
  extensions, which v0.4 scored as missed deadlines after excusal was made
  fail-closed, without the *Bowles* defect: a limit with no `extend` grant
  admits no extension, and an order beyond a bound is rejected.
- **An extension's bound is a sum** (FRCP 6(b)). A grant is bounded
  (`extend { by A, max N.days [, after_expiry] }`, a cap only where the
  declaring file imposes one), discretionary (`extend { by A, discretion
  S [, after_expiry] }`: no numeric ceiling, before expiry, or after it
  on motion), or non-extendable (`extend { never }`, FRCP 6(b)(2)). A
  deadline may declare several grants. New contextual keyword `never`.
- **FRCP extension capabilities.** `lit/frcp`'s extendable periods grant
  FRCP 6(b)(1)(A) and (B) (`discretion GoodCause`; `discretion
  ExcusableNeglect, after_expiry`), the Rule 50(b), 59(b), 59(e) and 60(b)
  periods are `extend { never }`, and the notice-of-appeal period grants
  FRAP 4(a)(5)(C)'s 30 days; `lit/core`'s Rule 81(c)(2)(C) and 14(a)(1)
  periods grant 6(b)(1). Each extendable frcp period has an
  `...Extended(until: Date)` event, done by the new `court` role, so a
  matter records a court-ordered extension.
- **Extension grants keyed on when the request was made** (lg-flu.24).
  A before-expiry discretionary grant that names a motion (`discretion
  GoodCause, on M()`) is on request: its window is judged on the
  request's date, not the order's, so a request made before the period
  expires licenses an order entered after it under good cause, and no
  excusable neglect is demanded (FRCP 6(b)(1)(A); 37 C.F.R. §§
  90.3(c)(1)(i), 2.145(e)(1)(i)). Each extended obligation records the
  licensing grant's standard and the request's date (`extended_under`,
  `request_made` in `litigate check`'s deadline schedule). A refusal for
  a missing motion is preferred to a window refusal. Data: every lit/frcp
  6(b)(1) period (and lit/core's two) gains the request grant; the PTAB
  appeal period (37 C.F.R. § 90.3(c), the version in force from Mar. 29,
  2024) and the TTAB civil-action period (§ 2.145(e)) are extendable by
  the Director only, on written request, before expiry for good cause and
  after it for excusable neglect, with no numeric ceiling; ptab and ttab
  gain a `director` role, so no court party may grant them.
- **Extension grants model the rule text exactly** (lg-flu.23). A
  bounded grant's bound is a closed date expression, `to max(...)`,
  `min(...)` over `prescribed_due` / `grant_order_entered` `+ N.days`
  (`max N.days` remains `to prescribed_due + N.days`), and may state a
  standard; FRAP 4(a)(5)(C) is exact: the later of 30 days after the
  prescribed time and 14 days after the order. A grant may require a
  motion (`on M() [, within W]`): an after-expiry discretionary grant must
  (FRCP 6(b)(1)(B), a motion made after expiry), and FRAP 4(a)(5)'s motion
  must come no later than 30 days after the prescribed time. The cause
  standard is a closed set (`GoodCause`, `ExcusableNeglect`,
  `GoodCauseOrExcusableNeglect`); any other identifier is L2007. `extend {
  never cite X }` names the rule forbidding extension and locks the period
  to its declaration's own grants; a lower authority's override that adds
  a grant is L3004. Data: FRCP 6(b)(1) now covers the Rule
  12(a)(1)(A)(ii) waiver and Rule 15(a)(1) periods; every FRAP 4(a)(1) and
  4(a)(3) period in frcp, frap and cafc grants FRAP 4(a)(5) under a FRAP
  26(b)(1) lock; FRAP 4(b)(4) on the criminal notice periods; FRAP
  26(b)(1) on the FRAP 5 petition period and 26(b)(2) on the ITC appeal
  periods. frap, cafc and frcrimp gain a `court` role.
- **Elections in `.case` replay.** A cited event whose type declares
  `elects p` (`p: String`) elects the edge `p` names: replay removes every
  alternative at each fork offering it (SPEC §5.6). cofc's `Elected`
  declares it.

### Changed

- **Event names carry no meaning.** `Struck`/`Forfeited`/`Withdrawn` remove
  an option only when their type declares `removes option`, and `Elected`
  elects one only when it declares `elects option`; `Waived`,
  `Extended` and `Excused` mean nothing by name. An event that takes no edge
  must fire an in-force deadline's trigger, be transition history, or
  declare an effect, and no such event is accepted where the matter has
  ended (the timing-only hatch is gone). The replay trace's `timing` effect
  is now `triggered`, `history`, `removed`, `elected`, `extended` or
  `waived`.
- **Obligation-instance semantics.** The default trigger policy is
  `first`: a repeated trigger while an instance is open restarts nothing
  unless the deadline declares `trigger latest` (or `each`,
  `later_of(d)`). A breach (L4009) needs a bearing party: an open duty
  instance no party bears lapses at `as_of` instead. An extension or a
  waiver applies to every open instance of the deadline (each
  defendant's), all or none.
- Six periods a later event restarts declare `trigger latest`: FRAP
  4(a)(1)(A) after a tolling motion, the 28 U.S.C. § 2101(c) cert period,
  the Speedy Trial Act clock, FRCrimP 29(c)(1), FRAP 4(b)(1)(A) and the
  PTAB appeal period. Inline `test` blocks pick the trigger by the same
  policy.

- **Tribunal rulings are chance nodes.** Every ruling a court, board,
  Director, agency or grand jury makes in the `lit/` forum packages is now a
  `chance ... discretion` node, not a `decision`: the party does not choose
  it. Generated packs carry `kind: "state"` for these nodes (the engine
  already drew their edges as nature's). Uncertain probabilities stay the
  pack's existing values or holes.
- **SPEC relaxations reverted.** `fact` is allowed only on `chance` nodes,
  a self edge never carries `p` (L2008), and a source `as_of` is a
  `YYYY-MM-DD` date. The data that relied on the relaxations is refactored:
  cofc's `limitations-check`/`section-1500-check` are chance fact nodes;
  ptab's mixed `petition-threshold-review` splits into the
  `petition-time-bar-check` fact node, the Office's compliance ruling and the
  petitioner's `petition-defect-noticed` choice; frcrimp's continuance
  self-loop is the `rule12-continuance-ruling` chance node; the cafc and ttab
  month-only source dates are their sourced days.

- **Real case events in ptab and cofc.** ptab's notice of filing date
  accorded is one event (`FilingDateNoticed`) on one edge out of the
  Office's compliance review, as in the proceeding: the invented
  `PetitionDocketed` and `FilingDateAccorded` events and the
  `petition-threshold-review`/`petition-filing-date-accorded` nodes are
  gone; `ipr-petition-filed` and `pgr-petition-filed` are the review
  (`IncompletePetitionNoticed` on the defect edge), the § 315(b) time-bar
  fact follows the notice, and joinder is a petitioner filing choice
  (`JoinderMotionFiled`). cofc matters elect their track with a cited
  `Elected` event instead of `Forfeited`; `patent-claim-fork` splits so a
  § 1498(a) claim is one option (`e-patentfork-patent`) ahead of the
  optional § 286 administrative claim (`patent-admin-claim-fork`).

### Fixed

- A `.case` `facts` edge written `pack::edge` is no longer qualified twice.
- **Corpus legal-modelling defects** (design review 2026-10-23 §5.2,
  lg-flu.15). FRCP periods start where a vendored source shows them in
  force: Rule 4(m) is 90 days from December 1, 2015 (not 2017) and 120
  days from 2007 to then; the Rule 12(a) and 15(a) periods date from the
  2009 texts and the FRAP 4(a) periods from 2009, so a 2010-2024 matter
  has its answer and appeal deadlines. A TTAB answer period before the
  sourced 60-day designation (June 2026) is § 2.105(a)'s 30-day floor. A
  cross-appeal is due 14 days after the first notice or with the notice of
  appeal, whichever is later (FRAP 4(a)(3), `later_of`, in `lit/frcp` and
  `lit/frap`), and `litigate check` reports that `floor` beside the
  counted due date. The notice of appeal cites 28 U.S.C. § 2107 and
  delegates nothing to a case order. `frcp-civil-procedure` gives each
  Rule 12 response track its own window (`waiver-response-window`,
  `removed-response-window`, `amended-response-window`) on its own
  clock. FRCP 6 has a 2005 version, and the pre-2005 versions mark the
  order of adding the mail days, contested until the 2005 amendment,
  UNVERIFIED. No fixed-date federal holiday is observed on another day before
  Executive Order 11582 (1971); the observed Monday is validated against
  *Derrington-Bey* (D.C. Cir. 1994).

## [0.4.0] - 2026-10-23

### Added

- **litigate** authoring toolchain in the release. The `litigate` CLI
  (check, build, query, diff, import, explain) and the `litigate-lsp` language
  server now ship in every release archive and the Homebrew formula
  (`brew install davidawad/tap/litgraph` installs `litgraph`, `litgraph-mcp`,
  `litigate` and `litigate-lsp`). Language reference: `docs/litigate/SPEC.md`.

### Changed

- **Data-driven deadline clock.** Counting rules (`CountingRule`) and holiday
  calendars (`HolidayCalendar`) are now data in `packs/clock/*.json` (the
  clock library, pack schema v3); a pack names its rule in `counting`. A
  generic interpreter replaces the hard-coded `RuleSet` enum, the
  `general`/`itc`/`holidays` modules and forum-string rule selection.
  `deadlines` outputs are unchanged. A pack with no `counting` rule, or an
  unknown one, is a `deadline-uncomputable` warning; `validate` gains the
  `clock` kind and rejects an unknown `counting` id; `schema clock` prints
  the library's JSON Schema. A zero-day backward court-day period from a
  closed day now rolls earlier (it rolled later, past the trigger).
- **Data-driven citation syntax.** Rule-body cite prefixes (FRCP, FRAP,
  RCFC, ...) and the ITC's dotted `cfr`/`usc` shorthands moved from the cite
  normalizer into each pack's new `citeSyntax` field (pack schema v3); the
  engine knows only U.S.C., C.F.R. and case citations. `cite::Family`'s
  per-body variants become `Family::Rules(heading)`, and
  `parse_cite_string` takes a `CiteGrammar` instead of a forum key. Lint
  output is unchanged. A new gate test, `no_forum_literals_in_engine`,
  fails on any pack id or forum key in `crates/*/src` outside test code;
  `describe`'s `pack(...)` metric example now reads `pack("x")`.

## [0.3.0] - 2026-09-30

### Added

- **TTAB pack** (`packs/ttab-trademark-trial-and-appeal-board.json`, forum
  `ttab`): an opposition to an application from the applicant's side, from
  the extension-of-time-to-oppose clock (30/90/60/final 60, 37 CFR 2.102(c))
  through standstill and settlement, institution and answer (40/60 days),
  the Section 18 restriction counterclaim, discovery, trial, briefs, final
  decision, and appeal (15 USC 1071), with exits at four stages (2.135
  judgment vs consented abandonment, suspension for settlement 2.117(c),
  ACR) and a parallel infringement-suit branch. New vendored sources
  `sources/cfr-37-part2.txt` and `sources/usc-15-lanham.txt`; named scenario
  `ttab-opposition-applicant-defense`; `cafc@ttab` instance and
  `ttab-appeal-to-cafc` / `cafc-remand-to-ttab` links (the cafc pack gains an
  `entry-ttab` origin, authored at remand probability 0 so existing outputs
  do not change); `ttab` forum maps to FRCP 6 day counting (37 CFR 2.196).
- **Famous-case tests** (`tests/cases/`, `crates/litgraph/tests/famous_cases.rs`):
  ten real, sourced procedural histories (Twombly, Iqbal, Celotex, eBay v.
  MercExchange, Oil States, SAS Institute, United States v. Arthrex, Hughes
  Aircraft (§ 1498), Blue & Gold Fleet (CoFC bid protest), Suprema v. ITC
  (§ 337)). Each one is replayed as a walk through the compiled graph, so
  links and state flags are honored. The tests check the terminal and flags
  it ends on, and check key deadlines through the `deadlines` op against
  hand-computed rule dates. Steps no pack can express are listed as
  expected-failure gaps, and the test fails once one is closed. Each case
  renders to an `expect-test` story snapshot (`<slug>.story.txt`) a lawyer
  can read.
- **Golden graphs** (`tests/golden/`, `crates/litgraph/tests/golden.rs`):
  seven 2-6 node packs with hand-computed `solve`, `chain`, `simulate`,
  `path` and min-cut answers, each file showing its arithmetic.
- `tests/README.md`: the test layers and how to add a case.
- `examples/famous-case-noa-holiday-roll.json`: the Oil States notice of
  appeal, a 63-day window that rolls past the observed July 4 holiday.
- Dev-dependency `expect-test` (MIT/Apache-2.0, one small transitive
  dependency, `dissimilar`). It provides the self-updating story snapshots
  (`UPDATE_EXPECT=1`).

### Fixed

- Deadline clock: Juneteenth is a legal holiday only from 2021 (Pub. L.
  117-17) and Martin Luther King, Jr. Day only from 1986 (Pub. L. 98-144).
  Previously both applied to every year, which moved historical ITC
  anchors and roll-forwards. For example, the Suprema petition-for-review
  deadline came out July 5, 2011 instead of July 1.
- `frcp-civil-procedure`: a dismissal with prejudice and a full summary
  judgment are final decisions (28 U.S.C. § 1291) but had no route to the
  notice-of-appeal window. New edges `dismissal-appeal-clock` and
  `summary-judgment-appeal-clock` add one.
- `frap-appellate-procedure`: new edges `no-transcript-certificate` (FRAP
  10(b)(1)(B), for appeals from rulings on the papers) and `no-stay-sought`
  (a stay is optional under FRAP 8(a)(1); this mirrors the CAFC pack's
  `motions-decline`). Before these, every appeal had to order a transcript
  or file an agreed statement, and had to move for a stay.
- `links.json`: `cafc@cofc` and `cafc@cofc-gov` now also patch
  `rehearing-time-expires` to 45 days, matching their `rehearing-file`. The
  `cafc@itc` instance gets both patches, because the Commission, a U.S.
  agency, is a party to every § 337 appeal (Fed. Cir. R. 40(a)(1)(B)).
- **Uncertain probabilities** (`docs/UNCERTAINTY.md`). Every chance draw
  (chance nodes, nature-first interrupts with their residual, act-or-wait
  waits) carries a Dirichlet belief whose mean is the resolved probability
  and whose strength is `scenario.uncertainty.concentration[node]`, else a
  calibration entry's `n`, else a default of 10 pseudo-observations
  (an estimate, warned `prior-concentration-estimated`).
  - `scenario.observe` (`{node: {edge: count}}`): conjugate update; every
    op then runs on the posterior mean (`provenance.modes.observe`).
  - `objective: {type: robust, credibility?, radius?, samples?, seed?}`:
    rectangular robust MDP (Iyengar 2005; Nilim & El Ghaoui 2005) over
    per-node L1 Bayesian credible sets (Petrik & Russel 2019). `solve`
    reports `robust.{nominal_value, robust_value, price_of_robustness,
    policy_changes, ...}`.
  - `posterior` op: Monte Carlo over the posterior; credible intervals on
    the case value and each option's Q, and P(each option is optimal).
  - `voi` op: EVPI of each chance node's outcome (exact) and probabilities
    (EVPPI), EVPI of everything, and EVSI of `studies` (`{node, k, cost |
    cost_edge}`) with `net` and `worth_paying`, all as expected regret, so
    exactly 0 when the decision can't change.
  - `Graph::sample_size` records calibrated edges' `n`.
- **`settle` op: settlement prediction** (`docs/SETTLEMENT.md`). From any
  node, each side's walk-away point is its certainty equivalent of
  litigating on under the solved general-sum equilibrium, under its own
  payoffs, costs and risk attitude (`scenario.objective` for `self`, the new
  `opponent_risk` for the opponent; `cvar` sides are estimated by seeded
  Monte Carlo). Reports the bargaining range (ZOPA) or the no-deal gap, and
  prices side by side: split-the-surplus midpoint, symmetric and weighted
  Nash bargaining (in each side's utility for sure money), and Rubinstein
  alternating offers (discount factors from per-side `discount_annual` or
  `delta` and the duration of the next procedural step, plus the
  short-round limit). `timing` recomputes the range along the likely line,
  names where the surplus peaks, and treats settling as an always-available
  action (optimal stopping) to show how the policy and value change. An
  optional `rule68: {offer, costs, eligible?}` models FRCP/RCFC 68(d)
  cost shifting on a plaintiff's judgment not more favorable than the offer
  (rule text verified on LII; *Delta Air Lines v. August* and *Marek v.
  Chesny* scope). Example: `examples/cofc-1498-settle.json`.

## [0.2.1] - 2026-09-29

## [0.2.0] - 2026-09-29

### Added

- Cargo feature `embed-sources` (default on) embeds the vendored L0
  source texts for cite verification. `litgraph-wasm` builds without it to
  keep the WebAssembly bundle small; `lint` then reports
  `sources-unavailable` unless `LITGRAPH_SOURCES` points at a `sources/`
  directory.

- **Calibration pipeline** (`docs/CALIBRATION.md`): `calibration/*.json`
  overlay files — embedded at build time like `packs/*.json`, overridable
  via `$LITGRAPH_CALIBRATION`/`--calibration-dir` — record a sourced
  probability, duration, or node attribute (value, source URL/title,
  vintage, sample size `n`, and derivation notes). `scenario.calibration`
  (a list of set names) applies them by mutating the compiled graph exactly
  as if the value had been hand-authored in the pack; `provenance.calibration`
  reports what was applied, from where, and with what `n`. The new
  `calibration` op ranks every *uncalibrated* probability/duration by
  decision sensitivity (the same one-at-a-time perturbation `tornado` uses),
  so an agent knows what to calibrate next. `litgraph schema calibration`
  and `litgraph validate --kind calibration` cover the new file kind. Ships
  with a first, real calibration set: USPTO PTAB institution/FWD rates,
  Federal Circuit affirmance/vacatur rates on CoFC-origin appeals, CoFC
  bid-protest volume, and AO district-court time-to-disposition — each
  sourced from one fetched, working document (see `docs/CALIBRATION.md` for
  citations and what's deliberately left uncalibrated).
- `litgraph-mcp` (`crates/litgraph-mcp`): a stdio MCP server, a thin
  wrapper over `litgraph::api::handle`. Every engine op is an MCP tool,
  each with an input schema sliced directly out of the engine's own
  `schemars` schema for `Op` (so a tool's shape cannot drift from what the
  engine accepts) and a description taken from `describe`'s own `"ops"`
  text; a generic `litgraph` tool takes a raw request verbatim. Packs,
  `links.json`, the named scenario library, and the manual are MCP
  resources (`litgraph://packs/<id>`, `litgraph://links`,
  `litgraph://scenarios` — an index of every named scenario —,
  `litgraph://scenarios/<id>`, `litgraph://describe`). Honors
  `LITGRAPH_PACKS`/`--packs-dir` and `LITGRAPH_SCENARIOS`/`--scenarios-dir`
  like the CLI. Engine errors
  (`ok: false`) become MCP tool errors carrying the same JSON envelope;
  the server never panics. Ships in the release tarballs, the container
  image, and the nix flake (`packages.litgraph-mcp`) alongside `litgraph`.
  See the README's "MCP server" section for `claude mcp add`, Claude
  Desktop, and generic-client setup.
- `clock` module: dependency-free deadline computation under `FRCP 6`,
  `RCFC 6`, `FRAP 26` (also governs the Federal Circuit), and `19 CFR
  210.6(a)` (ITC Section 337) — federal legal holidays (including
  RCFC-only Inauguration Day), forward/backward day counting,
  calendar-vs-court-day units, the 3-day mail/service rules (differing
  service-method sets per rule set), `FRCP`/`RCFC 6(a)(2)` hours-based
  periods, and `6(a)(3)` clerk-inaccessibility extensions. Ported from
  `civ-pro-the-gathering`'s `src/engine/clock/`, re-verified against the
  rule text.
- `deadlines` API op: concrete due dates, with a step-by-step computation
  trace, for an edge's authored `deadline`, a node's out-edges, or every
  deadline reachable from a node — with `service_method`,
  `additional_holidays`, and `clerk_inaccessible` options. Discoverable via
  `describe`/`schema request`.
- Scenario library (`scenarios/*.json`): named matter profiles (rates,
  stakes, perspective, payoffs, probabilities, policy, which packs to load),
  embedded into the binary like packs and overridable with
  `LITGRAPH_SCENARIOS`/`--scenarios-dir`. A request's `scenario` field can
  now be a bare name (`"scenario": "cofc-1498-patent-case"`) or a named
  scenario composed with inline overrides (`{"extends": "<id>",
  ...overrides}`, deep-merged per RFC 7386, `deny_unknown_fields` preserved
  on the merged result). CLI: `--scenario <name|file|json>`, `--set` composes
  onto a named scenario automatically. `describe` lists every scenario
  (`scenario_library`); `litgraph schema named-scenario` gives the file
  shape; `litgraph validate` accepts a scenario-library file directly
  (`named-scenario` kind). Ships four scenarios: a 28 U.S.C. §1498 patent
  case, a post-award bid protest (28 U.S.C. §1491(b)), an IPR defense at the
  PTAB with appeal to the CAFC, and a generic FRCP civil case; every shipped
  scenario is checked against the current packs by
  `crates/litgraph/tests/scenario_library.rs`. Public loader API:
  `Catalog::scenarios()` / `Catalog::scenario(name)`.
- **Cite verification against L0 sources** (`crates/litgraph/src/cite/`):
  `litgraph lint` now checks every node `cite` / edge `authority` string
  against the pack's own `sources` — normalizing surface forms ("FRCP
  12(b)(6)", "Fed. R. Civ. P. 12(b)(6)", "28 U.S.C. § 1498(a)", "35 USC
  315(e)", "37 C.F.R. § 42.108", "RCFC 56", compound citations like
  "28 U.S.C. 1291; 2106", case citations recognized and skipped as
  out-of-scope), resolving each to a vendored source span via
  `Pack.sources[].path`, and fuzzy-matching a quoted phrase in `note`
  against that span when one is given. Unverifiable cites are a new `warn`
  diagnostic, `unverifiable-cite`, with the exact reason. Vendored L0
  corpus under `sources/*.txt` (FRCP, FRAP, 19 U.S.C. § 1337, 37 C.F.R.
  Part 42, and 28/35 U.S.C. + 37 C.F.R. Part 1 fragments — see
  `sources/PROVENANCE.md` for why plain text was vendored instead of
  fetched at lint/test time, and each file's exact provenance/sha256),
  embedded into the binary the same way `packs/` is, with a
  `LITGRAPH_SOURCES` directory override mirroring `LITGRAPH_PACKS`.
  `cargo test --test cite_coverage_report -- --nocapture` reports current
  coverage across every embedded pack (see the next entry: every pack's
  `sources[].path` is now wired).
- **Matter facts** (`scenario.facts`): a chance node tagged `"fact"` in a pack
  represents a matter fact knowable at filing (time-barred? already pending
  elsewhere? a patent case?), not real uncertainty. `scenario.facts` (`{node
  ref: edge ref}`, the same shape as `policy`) forces that node's true branch;
  left unset, the engine falls back to the pack's authored prior and warns
  `fact-unset`. New lint rule `fact-no-prior` flags a `fact`-tagged node whose
  out-edges aren't all authored. Applied to `cofc::limitations-check` and
  `cofc::section-1500-check`. See `docs/PACK_SCHEMA.md#matter-facts-v2`.
- **District-court patent route**: `packs/links.json` gains a `cafc@district-court`
  instance (mirroring `cafc@cofc`/`cafc@ptab`/`cafc@itc`) and a link from
  `frcp-civil-procedure::notice-of-appeal-filed` to it, alongside the existing
  regional-circuit (FRAP) link, for 28 U.S.C. § 1295(a)(1) patent appeals.
  Which applies is a matter fact, not a free choice between forums — a
  scenario selects the applicable route with `scenario.remove_edges`.
- `frap-appellate-procedure`, `frcp-civil-procedure`, `frcrimp-criminal-procedure`,
  `itc-337`, `mpep-prosecution`, and `ptab-patent-trial-appeal-board` migrated
  to schema v2: authored terminal `payoff`/`outcome`, `roles`, `sources`
  (verified official URLs, several with `sha256`), stable `id`s on every
  parallel edge, and `duration` on every deadline-bearing edge plus several
  major no-deadline steps (sourced from the underlying rule's own timeline
  where one exists, otherwise a labeled estimate). `frap`/`frcrimp`/`ptab`
  also gain authored `hours` on every choice edge. `litgraph lint` reports no
  payoff/duration-authoring/role/source/parallel-id gaps on any of the six.
  `tests/fixtures/ts-parity/v1-packs/` freezes the pre-migration v1 pack
  content the TS-parity golden output was generated from, so
  `crates/litgraph/tests/parity.rs` keeps checking the engine against it
  independent of `packs/`'s own evolution.
- **State flags** on edges (`sets`/`clears`/`requires`/`forbids`, pack
  schema v2, additive): compiled into a product graph over `(node,
  flag-set)` so a node's identity can depend on procedural history — an IPR
  estoppel, a waived defense, a prior RCE — without any algorithm (`solve`,
  `chain`, `simulate`, `path`, `pareto`, `sweep`, `structure`) needing to
  change. Only reachable `(node, flag-set)` combinations are materialized; a
  pack that never declares a flag compiles to an identical graph. A hard cap
  (`CompileOptions.max_product_nodes` / request `max_product_nodes`, default
  20,000) fails compilation with a clear error instead of an unbounded
  compile. Flagged node ids are stable and readable
  (`ptab::fwd-issued{ipr-estopped}`); a compiled `Node` carries `flags` and
  `base_id` so API responses can project back to the base pack node.
  Terminals can carry `payoffByFlag` to vary payoff by which flag matched.
  `flag("x")` is available in edge and terminal expressions. New
  `CompileOptions.no_flags` / request `no_flags` reproduces the pre-flags
  compile exactly (used by `tests/parity.rs`). See
  `docs/PACK_SCHEMA.md#state-flags`.
- Modeled with real, sourced state-flag examples: IPR estoppel after a final
  written decision (35 U.S.C. § 315(e), `ptab-patent-trial-appeal-board.json`),
  a waived personal-jurisdiction/venue/process defense (FRCP 12(h)(1),
  `frcp-civil-procedure.json`), an RCE already filed (37 C.F.R. § 1.114,
  `mpep-prosecution.json`), and a fix for `cafc-federal-circuit.json`'s
  `cert-not-sought`/`cert-denied` terminals losing the panel's actual
  win/loss through a rehearing-or-cert detour (docs/CRITIQUE.md #13).
- `objective: {type: cvar, alpha, grid?, y_lo?, y_hi?}` optimizes `CVaR_alpha`
  of the total outcome (not just reports it, as `simulate` already did) via
  Rockafellar–Uryasev: a grid-discretized augmented-state backward
  induction, with the outer VaR-threshold search a lookup on the same grid.
  `y_lo`/`y_hi` (set together) override the default grid-bound heuristic for
  a graph where it's a poor fit or unnecessarily wide. Verified against
  brute-force enumeration of deterministic policies on small graphs
  (`tests/cvar.rs`/`tests/cvar_edge_cases.rs`, including a proptest).
  Documented exactness limits (grid discretization; doesn't compose with
  `discount_annual`, `fee_shift`, or a general-sum opponent) in
  `docs/CRITIQUE.md`.
- `scenario.opponent_objective` (a terminal expression): the opponent
  maximizes their own payoff (general-sum) instead of minimizing ours
  (zero-sum), solved as a subgame-perfect equilibrium by backward induction
  on the graph's SCC DAG (cyclic components iterate to a fixed point, with
  the same honest non-convergence reporting as `solve`). `self`'s
  `Objective::Cara`/`Worst` risk objective is honored (applied to self's
  aggregation over nature's draws only, via the same `mdp::aggregate` helper
  `mdp::solve` uses — never to the opponent's, which stays plain
  expectation). Reports both players' values (`solve`'s `opponent_value`/
  `opponent_values`) and gives `explain`'s per-option `regret` the mover's
  own criterion (the opponent's own `opp_q` at their node, not an assumed
  adversary). `opponent_objective: None` (default) reproduces the existing
  zero-sum answer exactly (`tests/general_sum.rs`). Documented in
  `docs/CRITIQUE.md`.
- Path-dependent terminal variables (`spent`, `elapsed_total`, `steps`) in
  utility (and `fee_shift.eligible`) expressions, for prejudgment interest
  and time-growing damages (e.g. `payoff * (1+r)^(elapsed_total/365)`).
  Exact in `simulate` (evaluated per sampled trajectory); `solve`/`chain`
  are Markov on the node and see them as `0`, with a `path-variable-in-markov`
  warning from `validate`/every scenario-resolving op when an expression
  depends on one. See `docs/COST_FUNCTIONS.md` and
  `examples/cofc-prejudgment-interest-sim.json`.
- `scenario.facts` now works on **mixed** (choice+chance) nodes, not just
  pure chance nodes: `litgraph lint`'s `fact-no-prior` check and the
  response's `fact-unset` warning both now look at a fact node's non-applicant
  interrupt edges specifically (`scenario/plan.rs`'s `chooser()`/`act_or_wait()`
  path), so a `fact`-tagged mixed node gets the same diagnostics a pure
  chance one always did, instead of the generic `mixed-node` hedge.
  `ptab-patent-trial-appeal-board.json`'s `petition-threshold-review`
  (35 U.S.C. §315(b) time-bar) is now tagged `fact` as the worked example.
  New `Graph::node_family`/`Graph::edge_family` (generalized from the
  calibration fix above) ensure a fact forced on a base node also forces
  every [state-flag](docs/PACK_SCHEMA.md#state-flags) product-graph copy of
  it — demonstrated on a real flagged copy,
  `petition-threshold-review{prior-petition-denied}` (General Plastic's
  follow-on-petition doctrine, 35 U.S.C. §314(a), on
  `institution-denied-merits`/`-fintiv`'s refile edges).
- New canary test (`every_embedded_pack_file_parses_individually_against_the_strict_pack_struct`)
  parses every embedded pack file individually against the strict `Pack`
  struct, so a structurally-invalid pack (e.g. a duplicate top-level JSON
  key introduced by an unlucky line-based merge -- syntactically valid to a
  generic parser, but rejected by serde's derived `Deserialize`) fails as
  one clearly-named test instead of a whole-suite cascade of near-identical
  panics.
- **A malformed pack or `links.json` no longer takes the whole catalog
  down.** `Catalog::from_files` (embedded set, `LITGRAPH_PACKS`/`--packs-dir`
  directories, `Catalog::load`) used to fail the entire catalog -- every
  pack, every op -- on the first file that didn't parse. It now skips that
  file and records it in the new `Catalog.load_errors` (`{file, message}`),
  keeping every other pack usable; a request naming the broken pack gets an
  ordinary `NotFound` (it was simply never loaded), not a global outage.
  `litgraph describe`'s new `pack_load_errors` names what failed and why, so
  the failure stays visible instead of silent. `Pack::from_json`'s own
  per-file strictness (schema, `deny_unknown_fields`) is unchanged -- this
  is about the blast radius of one bad file, not about being lenient with
  its content.
- **Cite verification actually wired**: every pack's `sources[].path` now
  points at the vendored L0 text it needs — `unverifiable-cite` warnings
  drop from ~1400 to 8 (98.5% resolved; ~99.5% of in-scope citations), the
  remaining 8 all one documented gap (`sources/PROVENANCE.md`: a handful of
  MPEP sections a two-column PDF layout couldn't cleanly isolate). New
  vendored files: `rcfc.txt` (112 RCFC rules), `fedcir-rules.txt` (55 Fed.
  Cir. local rules), `frcrimp.txt` (62 FRCrimP rules), `mpep.txt` (466 MPEP
  sections, ported from `civ-pro-the-gathering`'s conversion plus targeted
  PDF extraction), `cfr-19-part210.txt`/`cfr-37-part1.txt`/`cfr-37-part41.txt`
  (full eCFR XML parses, 79/363/75 sections), `usc-additional.txt` (57 more
  28/35/18/5 U.S.C. sections). Normalizer: citation ranges ("FRAP 28-31",
  "37 C.F.R. §§ 42.120-42.121") now decompose into one citation per member
  section instead of one unmatched literal; `itc-337.json`'s `usc1337.N`
  shorthand now resolves the same way its `cfr210.N` shorthand already did;
  fixed a carry-forward bug where an unrelated citation species ("Sup. Ct.
  R. 13.1" between two U.S.C. cites) left a stale family active for a later
  bare number. Also corrected two `mpep-prosecution.json` citations to a
  section that exists in the current MPEP (`§ 818.03` doesn't) and two
  `cofc-court-of-federal-claims.json` citations that embedded "[Not used]"
  inside the machine-parsed `cite` string instead of `note`.

## [0.1.0] - 2026-09-28

Initial public release.

litgraph models litigation procedure as a stochastic game on a graph and
answers the questions a litigator (or an agent) actually has: what to do,
what it will cost, how likely each ending is, what the answer is most
sensitive to, and what changes under any counterfactual.

Extracted and rewritten in Rust from the TypeScript engine in
[civ-pro-the-gathering](https://github.com/davidawad/civ-pro-the-gathering),
and verified against the original engine on all six original packs
(`crates/litgraph/tests/parity.rs`).

#### Engine

- One request envelope (`{op, scenario, ...}`) → one response envelope
  (`{ok, api_version, op, result, warnings, provenance, elapsed_ms}` or
  `{ok:false, api_version, op, error}`), served identically by the CLI and
  the library crate. `warnings` is grouped by code
  (`{code, count, example, at}`); `api_version` is the request/response
  contract version, independent of the crate's own semver release.
- Ops: `describe`, `packs`, `lint`, `validate`, `graph`, `metric`,
  `explain`, `solve`, `chain`, `simulate`, `path`, `pareto`, `sweep`,
  `tornado`, `structure`, `compare`, `batch`.
- CLI: `litgraph schema <request|response|scenario|pack|links>` (JSON
  Schema, draft 2020-12); `litgraph validate <file|->` (parses + resolves
  a pack/links/scenario/request, auto-detecting kind, exit 2 if invalid);
  `litgraph run <request.json>`; plus `q`, `describe`, and shorthand
  `<op> --packs --set --arg --scenario`. Packs are embedded in the binary
  at build time (`LITGRAPH_PACKS` or `--packs-dir` to override), so no
  `packs/` directory is needed after install.
- Parsing is strict everywhere (`deny_unknown_fields`): a typo in a
  request, scenario, op, pack, or links document is an error listing the
  valid fields, not a silently-ignored key.
- Custom cost/utility/probability/weight functions via a small expression
  language, evaluated everywhere a number is consumed (cost, utility, mask,
  probability transform, path weight, Pareto objectives, cut capacity, fee
  eligibility). Built-in metrics/utilities are themselves named expressions,
  visible via `describe`.
- Every inferred number (heuristic payoff, filled probability,
  deadline-as-duration, dead end, non-convergence) is reported as a
  structured warning with a code and location; `provenance.modes` records
  every modeling choice made.
- Packs compose into one namespaced graph via `packs/links.json`; pack
  **instances** (e.g. `cafc@cofc`) let a shared forum (the Federal Circuit)
  remember how it was entered — origin-specific remand routing, perspective
  flips, payoff transforms, edge patches.
- Solver: SCC-ordered Bellman backups (exact where the graph is acyclic,
  iterative within cycles), LU-solved absorbing chains, Dijkstra/Yen path
  search, Pareto frontiers, min-cut/dominators/betweenness structural
  analysis, seeded Monte Carlo simulation with CVaR/percentiles.
- Scenario fields make every v1-inherited modeling choice explicit and
  switchable: `mixed` node semantics, `prob_fill`, `opponent` model,
  `objective` (expected / CARA / worst-case), `perspective`, `fee_shift`,
  `discount_annual`.
- `lint` turns analysis findings (unquantified chance nodes, missing
  payoffs, dead ends, unsourced packs) into concrete pack-editing guidance.

#### Content packs (`packs/`)

- Eight forum packs: FRCP civil procedure, FRAP appellate procedure,
  Federal Circuit (CAFC), Court of Federal Claims (CoFC), PTAB, ITC § 337,
  USPTO prosecution (MPEP), FRCrimP criminal procedure.
- CoFC and CAFC packs are schema v2: authored terminal `payoff`/`outcome`
  tags, `sources` with official URL + sha256 + as-of date, `roles`, `attrs`.
  The other six packs are schema v1 (ported unchanged from
  civ-pro-the-gathering), with the engine's label-heuristic payoff fallback
  flagged on every response.
- `tests/fixtures/ts-parity/` — golden output from the original TypeScript
  engine, one file per pack, used by `cargo test` to verify the Rust port
  reproduces v1 behavior exactly (value iteration, absorbing chains,
  Dijkstra, SCCs, dominators, min-cut, Pareto frontiers).

#### Known limits (see `docs/CRITIQUE.md`)

- Markov on the node: litigation history (estoppel, waivers, RCE counts)
  isn't yet expressible as edge preconditions (planned: state-flag product
  graph).
- Payoffs are teaching estimates or placeholders outside the CoFC/CAFC
  packs; always set `scenario.payoffs` for the matter at hand.
- Mean-optimal by default; CVaR/percentiles are reported from simulation
  but not yet optimized.
- Court-day/holiday deadline computation (FRCP/RCFC Rule 6) is not yet
  ported from civ-pro-the-gathering's clock module.
