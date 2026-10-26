# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Legal change as declared instrument classes, per forum family, in
  worlds** (lg-fq2.1; docs/litigate/CHANGE.md, SPEC §5.7-§5.8). New
  classes `general-order` and `emergency-declaration`. `lit/core`'s
  `Lit.toml` carries the class matrix of the eight federal forum families
  as data (`[family.*]`: forums, binding courts, circuits, one cited cell
  per class with a real example, narrowed rows, the emergency window): a
  class its family lacks is the new L2011, as is an emergency grant
  beyond 90 days; a narrowed row is L2010. A precedent's binding scope is
  computed from its `court` and `designated` date, never declared, and
  its `override` is its waiver. A `.world` file is a set of layers, a
  location index and a modality: `litigate check` reports each world's
  layers, the precedents that do not bind at its circuit (L9009), the
  precedents an overruling withdraws, conflicts only the merged layers
  have, a malformed world (L1013), and a counterfactual world under
  strict or a synthetic layer in an actual one (L1014). One example
  package per class for the deadline domain, over vendored instrument
  text (`docs/litigate/examples/change/`, `sources/change-*.txt`).
- **A package's instrument class is part of its type** (lg-962.5).
  `kind` in `Lit.toml` (`base`, the default, `amendment`, `local-rule`,
  `standing-order`, `case-order`, `guidance`, `precedent`, `overruling`)
  indexes the package, each class has an effect row (SPEC §5.7), and a
  declaration with an effect outside its package's row is the new type
  error L2010: a local rule that orders the authority lattice, a case
  order that delegates or supersedes another package's text, a
  persuasive precedent that prescribes, overrides, interprets or states a
  fact, guidance that prescribes or sets the prior of a draw by an actor
  it does not bind. A non-base kind is justified by
  `[package.instrument]` (vendored `source`, `issuer`, `date`); a layer's
  kind fixes its `authority`; `hypothetical` is rejected as a kind (a
  world modality, design review §2.5); each is L1004. Every `precedent`
  is persuasive until lg-fq2.1 computes binding scope. No corpus package
  changes class.
- **One command line: `litigate`** (lg-9lrj). Every engine op is a
  `litigate` subcommand on a compiled matter or package: `litigate solve
  CASE`, `chain`, `simulate`, `path`, `pareto`, `sweep`, `tornado`,
  `calibration`, `posterior`, `voi`, `structure`, `deadlines`, `settle`,
  `compare`, `graph`, `metric`, `lint`, and `explain` given a path
  rather than a diagnostic code; `--arg k=v` sets op arguments and
  `--set k=v` overrides scenario params. `litigate run` and the op
  subcommands take a `.lit` file or package directory too (its pack under
  the default scenario; a links package, the graph it composes).
  `litigate engine` is the engine's raw JSON-in/JSON-out interface (the
  former `litgraph` CLI: `q`, `run`, `validate`, `schema`, shorthand ops),
  printing the engine response itself; its packs and named scenarios are
  compiled from `--lib` (default `lit`) and `--cases` (default `cases`)
  unless a directory or `$LITGRAPH_PACKS`/`$LITGRAPH_SCENARIOS` names one.
  `crates/litigate-cli/tests/round_trip.rs` compiles and solves every
  `lit/` package and every `cases/` matter.

- **Fee-shifting terminals offset our own hours, by statute and per
  matter** (lg-vwwr). A `.lit` terminal's `fee_shift f` (pack
  `feeShift`, a fraction in `[0, 1]`) is the governing statute's default
  share of our accumulated cost, the fee motion's own hours included,
  that a fee award recovers when the matter ends there; a `.case` stake
  takes `fee_shift f` (`j: 0.usd fee_shift 0.5`, or `j: fee_shift 0.5`
  alone) to override it for the matter (`scenario.fee_shifts`).
  `scenario.fee_shift` now replaces the packs' fractions; `solve` and
  `simulate` use the per-terminal fraction, so a fee motion is no longer
  valued at the gross award less its own cost. A fee-shift fraction
  outside `[0, 1]` is an error.
- **A designated due date the matter does not record is `unknown`, never
  `open`** (lg-flu.46). A notice of institution whose answer date is `?`
  leaves the TTAB answer period with status `unknown` at `as_of` and the
  reason "notice due date not recorded", naming the notice and its
  mailing date, in the schedule, in `explain-value` (which now reports a
  deadline's `status` and `reason`), in the LSP's inlay hint and on the
  timeline (`status.state` `unknown`, reason "InstitutionNoticed due date
  not recorded"); it never
  lapses however late `as_of` is. Three end-to-end TTAB matters,
  `cases/e2e-ttab-{ok,short,unknown}.case` (one notice mailed 2026-10-21
  designating 2026-11-30, 2026-11-05 and `?`), are golden-tested through
  `litigate check` and `explain-value`.
- **Every matter checks clean, as a gate** (lg-flu.48).
  `crates/litigate-cli/tests/matters_check.rs` checks `lit` and every
  `.case` under `cases/` (the TTAB end-to-end matters among them) and the
  timeline fixtures, and the `timeline` of each, requiring `ok: true` and no error;
  `cases/e2e-ttab-short.case`, wrong by design, must fail with L4001 alone.
  The PM's 5 L1010 on `Sup. Ct. R.` cites (`sources/scotus-rules.txt` "is
  not the text source `scotus` pins") came from a `litigate` built before
  lg-flu.41 re-vendored the 2026 Rules: a binary verifies pins against the
  `sources/` it embedded at build time, and that one held the 2023 text.
  Main's pin and text agree. `litigate check`, `build` and `where` now
  honor `LITGRAPH_SOURCES` (as AGENTS.md documents), an unreadable one
  failing every vendored cite closed, and the L1010 pin-mismatch note
  names the stale binary and both remedies.

- **The certiorari extension's timing follows Sup. Ct. R. 13.5, 30.2 and
  30.3, from the Rules effective March 16, 2026** (lg-flu.41).
  `sources/scotus-rules.txt` is re-vendored from the current
  `2026RulesoftheCourt_WEB.pdf`: Rules 13 and 30 read word for word as in
  the 2023 edition, and 29.2 (electronic submission) and 29.6 (ticker
  symbols) carry the 2026 revisions. The FRAP and Federal Circuit
  `cert-period`s now declare two grants: a Justice's good-cause extension
  on an application made at least 10 days before the due date, and one on
  extraordinary circumstances for a later application still within the
  period; a `CertExtensionDenied()` bars both, since a denied application
  "may not be renewed". The `extend` language gains what that needed
  (SPEC §5.3): `r - N.days` bounds, the `current_due` date reference (the
  due date as last extended), the `ExtraordinaryCircumstances` cause
  standard, and `unless D()`, the motion's denial. A motion is now spent
  by the extension it licenses, so a second extension needs a fresh
  motion. Rule 22's presentation and service, and an order on a timely
  application entered after the period has run, stay undeclared.
- **Filing-requirement syntax is reserved** (lg-962.9, SPEC §3.7, PM
  ruling 2026-10-24). A paper kind's length limits, certificates,
  exclusions and ECF event codes will be a cited, versioned `.lit`
  declaration that local rules, standing orders and case orders modify
  through the same relations and layers as deadlines: `limit
  principal-brief { words <= 13_000 cite "FRAP 32(a)(7)(B)(i)", requires
  certificate compliance, excludes [cover-page, ?("local rule")], ecf
  ?("...") } cite "FRAP 32(e)"`. The form parses into a new `LimitDecl`
  CST node and is rejected with the new code L0011, whose message begins
  `reserved: filing requirements`, until lg-962.10 implements it. `limit`,
  `certificate` and `excludes` are reserved words (L0011 anywhere,
  `FILING_WORDS`); `requires` stays the edge field keyword. No file in the
  corpus changes. The tree-sitter grammar (`limit_decl`) and the Emacs,
  Helix and Neovim highlights, indents and folds follow.
- **The Court of Federal Claims appeal window declares its adoption of
  FRAP 4 with 28 U.S.C. § 2522's words** (lg-flu.43). A deadline's
  `forum F` option may now carry the adopting rule and the words that
  adopt `F`'s period, `forum F cite "X" quote "..."` (SPEC §5.3); the cite
  resolves like any other, and `cite-resolves` requires the quoted words
  in the cited vendored section (whitespace runs collapsed), else L1010.
  A `quote` beside a cite hole is L2007. `lit/cafc`'s `cofc-noa-period`
  is `forum frap cite "28 U.S.C. § 2522" quote "within the time and in
  the manner prescribed for appeals to United States courts of appeals
  from the United States district courts"`, checked against the vendored
  § 2522 in `sources/usc-additional.txt`, so its counting under FRAP 26
  is a declared link, not an implicit reading. The FRAP 2016 transition
  stays undeclared until a source for it is vendored.
- **The TTAB answer time is extendable under FRCP 6(b)(1)(A), read
  through 37 C.F.R. § 2.116(a) as an interpretation** (lg-flu.45). An
  `extend` grant may now carry `interpretation "..."` (SPEC §5.3): the
  rule reaches the period only as the author reads it, so an extension
  the grant licenses stands with a new warning, L4012, naming the
  interpretation. `lit/ttab`'s `answer-period` declares Rule 6(b)(1)(A)'s
  two grants (the Board acting, or on an `AnswerExtensionMoved()`
  request made before the date) that way, since § 2.116(a) applies the
  Federal Rules only "wherever applicable and appropriate"; the Board's
  order is the new `AnswerTimeExtended(until)` event. The answering
  edges no longer leave the extension authority a hole: the generated
  pack's holes name "FRCP 6(b)(1)(A), applied through 37 CFR 2.116(a)
  (interpretation)". Rule 6(b)(1)(B) is not declared: an applicant past
  its answer time is in default, and the Board's § 2.106(a) reset stays
  its own `CaseOrder` path, unchanged. § 2.116's eCFR history (versions
  2017-01-01 and 2017-01-14, paragraph (a) unchanged) is added to
  `sources/cfr-37-part2-answer-time-history.txt`, and TBMP § 509 (June
  2026) to `sources/tbmp.txt`, cited as practice guidance, not authority.
- **`litigate import` restores a v2 scenario's parties and holes**
  (lg-flu.28, from lg-flu.14). A lowered scenario's `parties` come back as
  the `.case` `parties` block, and its `lowered.holes` as `unvalued` stakes
  and `as_of ?(..)` sources; a hole with no `.case` spelling is L6002. Its
  `events` come back only as a commented `events` block: the import still
  asserts the replayed `start`, `facts` and `remove_edges`, since which of
  those the events derive is known only to replay, and imported packages
  declare no events. Uncommenting the block and deleting what it derives
  replays to the same scenario against the authored packages (checked by hand
  on the four corpus matters with events).
- **Typed guard operations in patches** (lg-flu.36; SPEC §3.6). A later
  decision or order changes a fact edge's guard inside a links package's
  `patch`, explicitly and on its authority: `when and C` narrows it,
  `when or C` widens it, `supersedes when C` replaces it, each with
  `by Level` (its authority level; new), `effective [from, to)` and
  `cite`. Missing any of the three is L2007 and the operation is dropped;
  a cite hole keeps it unapplied. Operations on one edge apply in
  authority, then effective-date, order, never source or file order;
  non-commuting operations no authority or date orders are L3003. An
  operation needs a capability over the base node (L3004), may act only
  on a named edge of a `fact` node (L2007), and the composed guards stay
  disjoint (L2005). Replay at an instance posture reads the composed
  guard in force on the date it reads, and a fact step's trace records
  the operations (`Step.guard_ops`); a links source map records each
  patched guard's provenance (`guards`: base, ops with authority and
  whether in force, composed). `and`/`or` are keywords only there (L0010
  elsewhere). Tree-sitter `guard_op` gains `authority_clause`.
- **Pack schema v5: designated deadline windows and extension bounds**
  (lg-flu.33; SPEC §5.3, §8.1, §8.7; `docs/PACK_SCHEMA.md`). A pack
  `deadline` is now a counted `length` or a `designated` window
  `{minPeriod, sourceEvent, sourceEventArg, interpretation?}`: the due
  date the triggering notice or order sets in each matter, its least time
  in calendar days, never a counted `length`. Each `extend` grant of a
  `within` deadline lowers to `deadline.extensions`, a sum tagged by
  `bound`: `bounded` (date bound `until`, as FRAP 4(a)(5)(C)),
  `discretionary` (cause standard and window, as FRCP 6(b)(1)) or
  `nonExtendable` (the forbidding rule, as FRCP 6(b)(2)). No grant is no
  `extensions`: undeclared, never not-extendable. `links.json` is
  schema 3 for the same fields on a link. The engine rejects a window
  with both or neither of `length` and `designated`, and either new
  field in an older version. The `deadlines` op reports a designated
  window as `deadline-uncomputable`, naming the event and argument,
  instead of counting its least time. `litigate import` reports both
  fields as L6002 at their pointer: they come from a `deadline`
  declaration, whose trigger event and counting rule an edge window
  does not carry.
- **The 2012 text of 37 C.F.R. § 90.3 is vendored and its Director
  extensions granted** (lg-flu.25). `sources/fr-uspto-90-3-history.txt`
  holds § 90.3 as published in 77 FR 48677-48678 (effective Sept. 16,
  2012, 77 FR 48612). Its paragraph (c)(1) has the same two grants as the
  2024 text, so the PTAB appeal period's 2012-2024 version (`lit/ptab`'s
  `appeal-period`, `lit/cafc`'s `ptab-noa-period`) is now extendable by
  the Director on written request, before expiry for good cause and after
  it for excusable neglect, where it was refused. Under the 2012 text the
  request is filed "as provided in § 104.2", not vendored, so the 2024
  receipt-date rule is not applied to it.
- **`workflow-sound`: soundness checks on the procedure net** (lg-flu.18;
  design review §2.7, §2.8, §6 item 27; SPEC §7, §9.1, §13). A new
  verification obligation reads the lowered pack as a bounded workflow
  net, with one control place per node (the P-invariant "exactly one
  token") and one 1-safe place per state flag, `requires`/`forbids`
  honoured. It explores every marking reachable from `start` and the
  `entry` nodes, at most 65536 of them, and reports L5007 where a marking
  can no longer reach a terminal (a node the reached flags leave with no
  enabled edge, or a cycle with no way out), L5008 for an edge no reached
  marking enables, and L5009 for a `rule duty` obligation that an edge's
  `event` creates and that no later marking can discharge through a
  `within` edge. Each finding's note gives a firing sequence reaching it.
  A larger net is undecided, which is an L5007 error. Every package in
  `lit/` and `cases/` passes.
- **Declared `.case` event ids** (lg-flu.39; SPEC §5.6, PM ruling
  2026-10-23 on timeline keys). An event line may start with an id,
  `service: 2025-03-03 Served(Personal) on roadrunner`. The id is the
  event's timeline key, `event:<Name>@<id>` (`hir::Event::key`). Without
  an id the key is `event:<Name>@<date>[/<docket>]`, which changes when
  the date is corrected. An event without an id gets the new info lint
  L9006. Its machine-applicable fix (`litigate fix --code L9006`, or the
  editor quick fix) writes a generated id, `<event-name>-<n>: `, that no
  other event of the matter uses. A repeated id is L1001. Tree-sitter,
  Emacs, Helix and Neovim highlight the id.
- **Typed delegation capabilities, characterization and emergency grants**
  (lg-962.4; SPEC §5.3, §5.5, §13.8; design review 2026-10-23 §2.4, §6
  item 2). `delegable { to L, op O [, max N.unit] [, cite "X"] }` replaces
  `delegable_to [L]` (now L2007 with a fix): a displacing relation from a
  lower authority is licensed only by a capability covering the
  operations it performs (`extend`, `shorten`, `set`, `suspend`,
  `compute_method`, derived from its value) within its bound, each refusal
  an L3004 naming what is missing. `characterization jurisdictional |
  claim_processing [cite "X"]` indexes a deadline: a jurisdictional one
  admits no waiver, unbounded extension or delegation (new L2009), and a
  late act under a claim-processing one is L4013, a warning, not L4002.
  `grant g on [d] effective [from, to) { unlock, extend {...}, waive
  {...}, delegable {...} }` gives deadlines capabilities for a bounded
  window, never wider than its own authority (L3004), lifting locks with
  `unlock`; replay honours grants in force on an order's date. New
  contextual keywords `delegable`, `op`, `grant`, `unlock`,
  `characterization` (no longer reserved). Corpus: every `delegable_to`
  migrated to `delegable { to L, op set }`, and a test lowers the corpus
  with each clause widened back to every operation and finds every pack
  and scenario byte-identical; the certiorari periods (frap, cafc) keep
  lg-flu.29's bounded Justice extension on an application within the
  period (Sup. Ct. R. 13.5, 30.2; before the vendored 2023 Rules an
  application's timing is unknown, the period uncounted), and the
  notice-of-appeal periods (frap, frcp, cafc) are `characterization
  jurisdictional` (28 U.S.C. § 2107; Bowles); the Federal Circuit's
  district-court window loses its `CaseOrder` delegation. Delegation, characterization and grant cites
  are checked by `cite-resolves`.

- **A delegation's condition and procedure** (lg-962.4; SPEC §5.5;
  design review §2.4). `delegable { to L, op O, max N.unit, discretion S,
  on M() }` completes the five-part capability at the rule level:
  `discretion` names the cause standard (L2007 outside the closed set),
  `on` the motion an exercise requires (an event pattern, L1002). In a
  matter, an order licensed only by capabilities on motion has no effect
  unless the matter records the motion on or before the order's date
  (L3004 at the order package's `uses`), and the motion is legal history
  without an edge. Fixed: an override's trigger with an imported enum
  variant (`Served(complaint)`) no longer reads as a change of trigger,
  so it performs `extend` or `shorten`, not `set`.

- **Bitemporal claims** (lg-962.4; SPEC §4.1, §5.4; design review §2.6,
  §6 item 6, P9, L5). `retracted D` closes a version's transaction time,
  so a correction restates the window it corrects (disjointness is per
  transaction-time slice); `commenced [from, to)` makes a version apply
  only to proceedings commenced in the window (a matter is the proceeding
  commenced on its first event); `unknown_before D` (version clause, rule,
  or `Lit.toml`) makes a query before the encoded law a hole, new L3010
  (a warning in `dev`), in selection and replay. `lit/frcp` and
  `lit/frap` declare `unknown_before`.

- **Markings** (lg-962.4; SPEC §5.6; design review §2.8, §6 item 13). An
  edge may `spawn [nodes]` concurrent tracks and `joins [nodes]` others;
  replay runs over the marking as a typed firing sequence (a join is
  enabled only while the joined track is marked). `CheckedCase.marking`
  reports the tracks at `as_of`; several is L4014, a warning. Tracks do
  not lower to litgraph yet (L6001).


- **The tree-sitter parser merges without conflicts** (lg-flu.37). The
  generated `editors/tree-sitter-litigate/src/{parser.c,grammar.json,node-types.json}`
  stay committed (the crate's `build.rs` and editor packages compile
  them) and get the `tsgenerate` git merge driver, which keeps ours when
  both sides changed them, so two branches changing different grammar
  rules rebase cleanly. `just tree-sitter-generate` regenerates them from
  the merged grammar; `just tree-sitter-merge-driver` (and `just
  merge-drivers`, run by `just install` and `just lit-sync`) registers the
  driver. `just ci` now runs `just tree-sitter-test`, which fails on a
  stale parser and runs `script/merge-test`.
- **`LITGRAPH_PACKS` and `--packs-dir` stay CLI contract** (lg-flu.34; PM
  ruling 2026-10-23 on lg-flu.31). `litgraph` and `litgraph-mcp` load
  packs from `--packs-dir`, else `$LITGRAPH_PACKS`, else the packs built
  into the binary, and the scenario library the same way
  (`--scenarios-dir`, `$LITGRAPH_SCENARIOS`). Both go through the new
  `Catalog::resolve` (`litgraph_packs::resolve`); `Catalog::default_source`
  is `resolve` with no flags, and `litgraph_packs::with_scenarios` is gone.
  A flag or variable naming a directory that does not exist is now an `io`
  error naming it (`$LITGRAPH_PACKS: /x is not a directory`). A packs
  directory holding no pack file (a stale `LITGRAPH_PACKS=packs/`) is an
  `invalid` error, where it used to load an empty catalog. A variable set
  to the empty string counts as unset. The dev shells leave
  `LITGRAPH_PACKS` unset; `.env.example` points it at
  `target/lit-build/packs`.
- **Timeline op** (lg-z2m.1; docs/litigate/TIMELINE.md). `litigate
  timeline FILE` and the MCP tool `litigate_timeline` print the timeline
  of a `.case` or `.lit` file as the frozen `litigate.timeline/1` JSON
  (docs/litigate/timeline.v1.schema.json), the envelope's `result`: a
  matter's events and every deadline its events triggered, in date order,
  with due date, status and reason (a deadline the clock cannot count is
  a row with `due: null` and `unknown`), then the deadlines governing the
  options of its current posture and a fork row there, each option one
  alternate world. Options `--as-of`, `--known-at`, `--layers` (`ID`,
  `+ID`, `-ID`) and `--world`. Each package is one layer until the lg-fq2
  change layers land. The Rust types (`litigate_base::timeline`) join the
  schema bundle as kind `timeline`, so the generated `packages/litigate-ts`
  bindings export `Timeline`. Golden timelines of the TTAB opposition and FRCP matters
  and two fixtures (a fork and unknown deadlines each) are validated
  against the frozen schema, as is its example. Every row has a key derived
  from its cause (an election or removal by the option it names); keys are
  unique in every world view of every matter under `cases/`, and two rows
  of one world that still share one are an error (L1001) naming both. A
  layer whose authority is not in its package's authority lattice is an
  error (L1004) and the timeline has no result. An act the engine judges
  neither timely nor late (L4010) is `unknown` with the reason.
- **Interpreted least times** (lg-flu.32; SPEC §5.3, §9.1). A designated
  deadline may state `interpretation "..."`: the rule names no start for
  its least time, so measuring it from the trigger's date is the
  author's interpretation. A notice designating less then stands, with
  the new warning L4011 naming the shortfall and the interpretation,
  instead of being rejected. `lit/ttab`'s counterclaim answer (37 C.F.R.
  § 2.106(b)(3)(iii), "A time, not less than thirty days, will be
  designated by the Board") states one. The answer (§ 2.105(a)) and the
  cancellation answer (§ 2.113(a)) are still rejected below thirty days,
  because those rules count "from the mailing date of the notice". New
  contextual keyword `interpretation` (tree-sitter and editor highlights
  updated). The Board-order fixture
  (`crates/litigate-cli/tests/fixtures/ttab-board-order`) also resets
  the counterclaim answer time (§ 2.106(b)(3)(iv)) through the same
  `CaseOrder` layer.

### Changed

- **The engine crate is `litigate-engine`** (lg-dfv4.1). `crates/litgraph`
  is now `crates/litigate-engine`, an internal library of the `litigate`
  compiler (`publish = false`); Rust callers name it `litigate_engine::`
  instead of `litgraph::`. Its public API, behaviour, and the `litgraph`,
  `litgraph-mcp` and wasm frontends are unchanged.

- **A designated deadline's extension is no longer a hole** (lg-flu.33,
  superseding lg-flu.32's lowering). The answer edges of `lit/ttab` now
  carry their `designated` window, so `deadline/extendable` and
  `deadline/extensionAuthority` lower as they do for a counted window:
  stated values are emitted, and an unstated one is absent (undeclared),
  never `extendable: false`. The answer period's extension authority and
  its two FRCP 6(b)(1)(A) grants (lg-flu.45) are now in the pack, each
  grant with the 37 C.F.R. § 2.116(a) interpretation it rests on, where
  they used to be named only in a hole's reason.
- **Generated packs leave a designated deadline's extension undeclared**
  (lg-flu.32; SPEC §8.1). An edge `within` a designated deadline still
  lowers with no `deadline` window. Its `deadline/extendable` and
  `deadline/extensionAuthority` are now `lowered.holes` entries, whose
  reason names the deadline and whatever extension the source states.
  The extension used to be dropped with no trace, and it is never
  emitted as not-extendable. `lit/ttab`'s "File no answer" edge no
  longer states `extendable false` for an answer time the Board may
  reset (§ 2.106(a)). Its answering edges no longer state an extension
  authority, none being verified for the answer time:
  `extension_authority` may now be a hole (`?("reason")`, SPEC §3.1,
  tree-sitter updated), and they write
  `extension_authority ?("TTAB answer-time extension authority")`, which
  is the pack's `deadline/extensionAuthority` hole. The Board's reset
  (§ 2.106(a)) stays a `CaseOrder` layer.

### Deprecated

- **The `litgraph` binary** (lg-9lrj) is a shim forwarding to `litigate
  engine` with the packs and scenarios built in (same output and exit
  codes; a deprecation note on standard error). Call `litigate` instead;
  the shim goes with lg-dfv4.7.

### Fixed

- **59 cite holes are string cites; the 28 U.S.C. App. edition bindings
  carry their edition dates** (lg-flu.30). Holes kept only because the
  retired cite mode stopped at a comma (`cite ?("FRAP 3(a), (c)(1),
  (4)-(5)")`, `cite ?("35 U.S.C. § 286, last paragraph")`) are quoted
  cites where every citation in them is a rule or statute whose pinpoint
  is in the vendored text. The other 272 stay holes: case citations,
  unvendored texts (TBMP sections, RCFC appendices, Sup. Ct. R. 10, the
  MPEP, Federal Register notices), pinpoints the text lacks (37 C.F.R.
  § 42.108(a)(1)), commentary, and the FRAP cites of `lit/frcp`'s versions that
  left force on 2023-12-01, whose earlier FRAP text is not vendored. `lit/core`'s `frcp-2008`,
  `frcp-2011`, `frap-2008` and `frap-2011` sources held the fetch date
  (2026-10-22) as `as_of`; each now holds the effective date of the last
  amendment its rule's credits print (2007-12-01, 2009-12-01, 2005-12-01,
  2009-12-01), so the FRCP 6 and FRAP 26 versions whose window contains
  that date cite their own text and lose `historical`.
- **Remaining type, syntax and lowering bugs from the 2026-10-23 design
  review** (lg-flu.19; review §5.3-§5.5, §6 item 30). Sema: a type alias
  may name one declared after it, so an event parameter typed by a
  forward alias keeps its type and its arguments are checked (T3); an
  exception to an exception checks its conclusion against the root
  default's type (T6); a `Dist` record denotes only enums an
  `import m.{x}` list makes visible (T8); a hole in a `when`/`unless`
  condition or fact guard is L2001 even in `dev` (T12, SPEC §5.2); an
  unresolved `n-1` whose `n` resolves offers the fix `n - 1` (S4's
  non-breaking part). Syntax: a `.field` chain stops at the 256 cap
  operator chains have (S5); recovery anchors an indented item outside
  every bracket, not only at column 1 (S6); one recovery error (L0001,
  L0002) per position, and an error of another code there is no longer
  hidden (S8, SPEC §3.4). Tree-sitter rejects what the compiler rejects
  lexically: exact whitespace, `_` only between digits, real Gregorian
  dates (`invalid_date` in expressions), with conformance tests (S7).
  Lowering and clocks: replay refuses a business-day period, as edge
  lowering and the CLI do, instead of counting court days (LW3); a cite
  prefix binds only at a token boundary, so `FRCP 6` no longer binds
  `FRCP 60` (LW4); `litigate build` never writes an empty output (LW5);
  the clock interpreter refuses a closed run or a court-day count longer
  than one Gregorian cycle instead of looping (LW6). T5, T7 and T9-T11
  break source or output and stay with the identity-lock, edition and
  capability work.
- `litigate lint-filing` no longer reports L9006 for the filing line it
  splices into the matter: that line cannot declare an id, and the
  fix pointed at a line the `.case` file does not have.

## [0.6.0] - 2026-10-24

### Added

- **Five rulings of 2026-10-23 recorded as normative** (lg-litigate-epic-tuq.52;
  SPEC §5.4, §5.6, §9.3, §9.4; `crates/litigate-cli/tests/rulings.rs`
  pins each). An event dated after a matter's `as_of` is L4001. A
  cited excusing event covers a missed deadline only when its type
  declares `extends`/`waives` and an `after_expiry` grant admits it; a
  name or a cite alone covers nothing. The shared clock library
  (`packs/clock/*.json`) is hand-written. `just lit-sync` and
  `crates/litgraph-packs` build under the `dev` profile. A version's
  `known_at` is its recording date when no vendored source dates its
  promulgation (the frcp $505 fee versions, `known_at 2026-10-22`).
- **Deadline calendars and a filing linter** (lg-litigate-epic-tuq.29,
  DESIGN §10). `litigate calendar CASE` exports a matter's computed
  deadlines as an RFC 5545 `.ics` file (`--out`): all-day events with
  stable UIDs, the deadline's authority, the counting steps and optional
  `--remind` alarms; open deadlines unless `--all`. A deadline whose due
  date is unknown is listed under `unscheduled` with the reason, never
  given a date, and a matter with errors has no calendar. `litigate
  lint-filing CASE EVENT --date D --by PARTY` replays the matter with a
  proposed filing added: L4001 when it is not a legal move on that date,
  L4002 when it misses the obligation it discharges; otherwise the result
  lists what it discharges and the deadlines it triggers, with due dates.
- **A cite's vendored text must be proved its version's text** (lg-flu.40;
  SPEC §4.3, §7, §8.6; PM ruling 2026-10-23 on lg-flu.27-a3). A vendored
  text dated D is the text of a cite whose version takes effect at E < D
  only when the section's vendored amendment history (a `## versions
  <heading>` block in a pinned file) shows no amendment in (E, D]: an
  amendment exactly at E holds, one exactly at D or strictly between
  breaks. Otherwise the cite is a hole: inventoried by the `holes`
  obligation (L9002 in `dev`, L5002 in `strict`), counted in `check`'s
  `packages[].holes`, and a `lowered.holes` entry at the cite, its reason
  naming the breaking amendment (effective date, eCFR version and issue
  date, history file) or why there is no history. A proving history is
  recorded in the cite's source map as `history` (counting rules' and
  calendars' cites are now located in the clock source map too). The
  eCFR versioner's version list of each 37 C.F.R. section `lit/mpep`,
  `lit/ptab` and `lit/ttab` cite is vendored as
  `sources/ecfr-versions-title-37.txt` and bound as `ecfr-versions-37`:
  the 2017-01-14 TTAB answer-time cites (§§ 2.105(a), 2.106(b)(3)(iii),
  2.113(a)) and § 2.196 are proved, while 551 cites across the packages,
  54 of them in `lit/core`, are now holes (their texts postdate their
  versions and no history for FRCP, FRAP, RCFC, U.S.C. or 19 C.F.R. is
  vendored), so `litigate check --profile strict lit/core` reports them.
  An unvendored dated binding keeps lg-flu.6's window rule (L1010).
  `litigate timeline` names the evidence too: each history proving a
  row's cite is an L9008 note (new, `info`) at the row's source, naming
  the history's file, line, hash, read date and version on or before E;
  the frozen `litigate.timeline/1` schema is unchanged (data, not fields).

- **Law not encoded before a date, pre-2009 FRAP 4, declared USPTO
  closures, and outcomes read from the docket** (lg-flu.26; design review
  2026-10-23 item 6, L5, L7, L11, L13).
  - `unknown_before` (SPEC §5.4): a date before every recorded version of a
    declaration, and before its package's `in_force`, is the hole "law not
    encoded before D" (`Miss::Before`), not "no version in force". Replay
    names it for a stage and a deadline; a trigger then opens an instance
    whose due date is unknown (an L3007 warning), never counted under a
    later version, so an act against it is unjudged (L4010) and it is
    never breached. A declaration enacted later inside the package's corpus
    is still simply not yet law. The CLI schedule reports the reason.
  - FRAP 4 as printed in the 2008 and 2011 editions of 28 U.S.C. App.
    (`sources/usc-28-app-frap4-history.txt`): the notice-of-appeal and
    cross-appeal periods of `lit/frcp`, `lit/frap` and `lit/cafc` gain a
    version from December 1, 2002 (when 4(a)(5)(A)(ii) took the 2008
    edition's text) to December 1, 2009, whose 4(a)(5)(C) bound is 10 days
    after the order (in calendar days, the earlier date); `lit/cafc` no
    longer claims the 14-day bound from 1968 (district court) or 1992
    (CoFC). The Federal Circuit cross-appeal runs to the later of 14 days
    and the FRAP 4(a)(1)(A) period (`later_of(district-court-noa-period)`,
    new module `cafc.appeal-periods`).
  - Declared closures (L11): `lit/core`'s `uspto-closures`, the days the
    USPTO designated "a Federal holiday within the District of Columbia"
    (37 C.F.R. § 1.9(h); TBMP § 112), vendored from its notices
    (`sources/uspto-closures.txt`), close `uspto-1-7` and `ttab-2-196`. No
    FRCP/FRAP 6(a)(6)(B) day is declared.
  - The Federal Circuit `remand-destination` is a `fact` node whose guards
    read the origin's entry event; new `Order` variants `CofcJudgment` and
    `TtabDecision` open the CoFC and TTAB notice periods (so a CoFC matter
    no longer opens the district court's, nor a PTAB matter the TTAB's).
    The CoFC ADR outcome and patent settlement fork stay draws, decided on
    the docket by the Appendix H ¶3(h) concluding order (`AdrConcluded`)
    and the Patent Rule 17 paper (`SettlementDiscussionReported`), both
    vendored (`sources/rcfc-appendices.txt`). CoFC and PTAB matters state
    the remand destination.
  - litgraph: a `fact` node with one live out-edge (a links instance
    removed the rest) is decided and does not warn `fact-unset`.

- **The tree-sitter parser merges without conflicts** (lg-flu.37). The
  generated `editors/tree-sitter-litigate/src/{parser.c,grammar.json,node-types.json}`
  stay committed (the crate's `build.rs` and editor packages compile
  them) and get the `tsgenerate` git merge driver, which keeps ours when
  both sides changed them, so two branches changing different grammar
  rules rebase cleanly. `just tree-sitter-generate` regenerates them from
  the merged grammar; `just tree-sitter-merge-driver` (and `just
  merge-drivers`, run by `just install` and `just lit-sync`) registers the
  driver. `just ci` now runs `just tree-sitter-test`, which fails on a
  stale parser and runs `script/merge-test`.

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

- **Counting per forum** (lg-flu.8; design review 2026-10-23 §6 item 10,
  P8, L2-L4, L11). A counting rule's `service_extension` is total over the
  service methods, `Electronic` and `Personal` included: each adds `N.days`
  (`0.days`: none) or is `unrecognized`, and the engine refuses a period
  served by an unrecognized method instead of counting it as if nothing
  were added (a rule with no map recognizes none). `forward_holidays C
  cite X` closes only periods counted forward (FRCP 6(a)(6)(C)),
  `declared_holidays C cite X` names a declared-closure calendar of
  `once(date)` days (6(a)(6)(B)); the engine's `CountingRule` gains
  `forwardHolidays`. A counting rule names the `forums` whose periods it
  computes and a deadline is its package's forum's period unless `forum F`
  says otherwise: counting one forum's period by another's rule is L3009.
  A chained period runs from an explicit end of another's, `from
  deadline(d).nominal` or `.adjusted`, in replay, `litigate test`, the
  `tests-pass` obligation and `explain-value`; a bare `deadline(d)` is
  L2001. A pack edge's `deadline.counting` names the window's own rule
  when it is not the pack's.
- **lg-flu.8 follow-ups** (lg-flu.29). `lit/core` declares the FRCP
  86(a)(2)(B) transition for the 2016 amendment of Rule 6(d): an action
  pending on December 1, 2016 keeps electronic service's 3 days only on a
  `PriorServiceRuleOrdered()` order. The certiorari period (`lit/frap`,
  `lit/cafc`) is no longer `delegable_to [CaseOrder]`: a `justice` (a new
  role) extends it, for good cause and up to 60 days, on a
  `CertExtensionApplied()` application within the period (28 U.S.C. §
  2101(c); Sup. Ct. R. 13.5, 30.2), recorded by `CertPeriodExtended(until)`.
  The Federal Circuit's PTAB and TTAB notice-of-appeal windows count under
  the agencies' own rules, `uspto-1-7` (37 C.F.R. § 90.3(b)(2), 35 U.S.C.
  § 21(b)) and `ttab-2-196` (§ 2.145(d)(4)(ii)), not FRAP 26, and only the
  Director extends them (§§ 90.3(c), 2.145(e); `cafc` now depends on
  `mpep-prosecution` and `ttab-trademark-trial-and-appeal-board`); the
  CoFC window stays FRAP 26's, as FRAP 4's period through § 2522 and RCFC
  58.1 (`forum frap`). Chains in real rule data: the TTAB's defendant and
  reply briefs (37 C.F.R. § 2.128(a)(1)) run from the nominal end of the
  brief before (TBMP § 112, newly vendored `sources/tbmp.txt`; § 2.128
  dated in `sources/cfr-37-part2-history.txt`); MPEP 710.01(a)'s extended
  reply periods (`ssp-reply-extended-1..3`) count 4-6 months from the
  action, since a month chained on a period cut to a short month's last day
  would end early.
- **Rule history and per-forum rules.** `lit/core`'s FRCP 6 and FRAP 26
  gain the 2016 versions (electronic service adds nothing from December 1,
  2016) and their earlier versions recognize electronic service where it
  added 3 days (FRCP 2001-2016, FRAP 2002-2016); RCFC 6 splits at its
  August 1, 2017 amendment. New rules: `frcrimp45` (FRCrimP 45, from the
  vendored 2023 text) for the criminal pack, `sup-ct-30` (Sup. Ct. R. 30.1,
  newly vendored `sources/scotus-rules.txt`) for the FRAP and Federal
  Circuit certiorari periods, and `lit/ttab`'s `ttab-2-196` (37 C.F.R. §
  2.196 and § 2.119(c), dated by the newly vendored
  `sources/cfr-37-part2-history.txt`). PTAB counts under `uspto-1-7`
  (37 C.F.R. § 1.7 via § 42.1(a)) instead of `rcfc6`, and TTAB no longer
  under `frcp6`.

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

- **Designated deadlines** (lg-flu.27; SPEC §5.2, §5.3, §8.1). A
  deadline option `designated p` takes each instance's due date from the
  trigger's `Date` argument `p`, as the notice or order states it; the
  period is only the least time the rule allows, and a trigger designating
  less is rejected (L4001; L5006 in a test). The date rolls under the
  counting rule like any last day. A case event may write `?` for a
  `Date` argument: the deadline it designates is unknown, stays open, and
  the schedule reports it with `due: null` and the reason in `unknown`
  (`litigate check`, `explain-value`, the LSP's inlay hint). Any deadline
  the engine cannot count is now on the schedule that way instead of
  dropped. New contextual keyword `designated` (tree-sitter and editor
  highlights updated). An act through the `within` edge of such an
  instance fails closed: it is judged neither timely nor late, the
  instance ends `unjudged`, and `litigate check` passes with a warning
  naming the trigger that left the due date unknown (new code L4010,
  a warning in both profiles) instead of failing with L3009.

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
- **The TTAB answer period is the notice's** (lg-flu.27). `lit/ttab`
  no longer declares a flat 30- or 60-day answer period: the answer is due
  on the date the notice of institution designates
  (`InstitutionNoticed(due: Date)`, its date the mailing date), no less
  than thirty days from mailing (37 C.F.R. § 2.105(a)), and a Board order
  resetting it (§ 2.106(a)) is a `CaseOrder` package overriding
  `answer-period` (example: `crates/litigate-cli/tests/fixtures/ttab-board-order`).
  The counterclaim answer (§ 2.106(b)(3)(iii)) and the cancellation answer
  (§ 2.113(a)) are declared the same way, in force from January 14, 2017
  (`sources/cfr-37-part2-answer-time-history.txt`, the eCFR version
  history). The pack's answer edges carry no window. A matter can now
  record the path to an answer: `OppositionFiled()` takes the notice of
  opposition edges, `InstitutionNoticed(due)` the institution edge, and
  `AnswerFiled(Plain | WithCounterclaim)` the two answer edges.

### Fixed

- **A chain on a dependency's period opens in case replay** (lg-flu.29).
  Replay's world holds the used packages' dependencies, as `check` and
  lowering do: a chain on a dependency's deadline opens from the event
  starting it (a dependency event a matter may now write as
  `pack::Event(...)`), and a dependency's counting rule counts under its
  own versions and transitions rather than the shared library's. A
  dependency's nodes are never stages and its deadlines open no
  instances.
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
- **Dependency resolution with PubGrub** (SPEC §4.2, lg-flu.7). Every
  loaded package is a candidate, several versions of one name included;
  one resolution per package picks the highest version satisfying every
  requirement, and imports, `::` qualification, worlds, the lock and a
  matter's pins all read it, so load order no longer decides. No
  satisfying choice is the new L1011 with PubGrub's explanation (it was a
  misleading L1006).
- **`Lit.lock` v2, Cargo.lock model** (SPEC §4.2, lg-flu.7). Each row
  records only its own package: the source digest, a semantic digest of
  the span-free HIR (reformatting does not change it), the SHA-256 of
  every vendored L0 text under `sources/` its `[sources.*]` bindings name,
  and each dependency pinned as `name version`, which must name exactly
  one row. Every load rechecks every row and fails closed (L1007), never
  relocking silently; format 1 locks, and v2 locks that stored a `digest`,
  are stale: run `just lit-sync`. The Merkle digests and the lock's Merkle
  root are computed on read: `litigate lock --root DIR` prints the root,
  and a release records every lock's root in its notes and in the
  `LITLOCK-ROOTS` manifest of each archive (`scripts/release/lit-roots.sh`).
  Whether a package was verified against its dependencies is a build-cache
  fact (`target/litigate/verified/`, `LITIGATE_CACHE`), keyed by the
  Merkle root of its resolution so a dependency change invalidates its
  dependents; CI (`CI` set) re-verifies from scratch, ignoring it.
- **Stable identity with `formerly`** (SPEC §4.3). A node or named edge
  lists the ids it had before a rename (`formerly [old-id]`); a `.case`
  written against an old id still resolves, with the new L9007 warning
  and a machine-applicable fix (`litigate fix --code L9007`).
- **Computed compatibility classes: `litigate compat OLD NEW`** (SPEC
  §4.4). A release is classified against an earlier one as `interface`
  (an id or type removed, renamed or retyped), `value` (anything it
  computes may differ) or `none`; a version bump smaller than the class
  needs is the new L1012 with the least sufficient version as a fix. This
  is what `^` and `~` requirements now mean for a legal package.

### Removed

- **The `litlock` merge driver** (`litigate lock --merge`,
  `.gitattributes`, `just lit-merge-driver`; lg-flu.7). With each lock row
  holding only its own package's digests, two branches editing different
  packages rebase cleanly as plain text (`merge_safe`); when both changed
  one package its row conflicts like a Cargo.lock, and `just lit-sync`
  regenerates it.

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

- **Breaking (lg-flu.8):** every engine counting rule's `service` step is
  required and total (`unrecognized` lists the methods it does not
  recognize); a method a rule recognizes for no days adds no trace step.
  `ServiceMethod` gains `personal` and `electronic`. The `ptab`, `ttab`,
  `frcrimp` and certiorari deadlines change counting rule (above); the
  PTAB and FRCrimP packs now name a `counting` rule, so the `deadlines` op
  computes their windows instead of warning `deadline-uncomputable`.
  `dc-federal-holidays` moves from `lit/mpep` to `lit/core`.
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
