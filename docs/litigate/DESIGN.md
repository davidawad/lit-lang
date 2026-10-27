# litigate: design

`litigate` is a strict, total, functional language for legal procedure. You
write a forum's rules once as `.lit` modules, and one matter as a `.case`
file. The compiler checks both as far as a machine can, then produces the
litgraph IR (pack, links and scenario JSON) plus other outputs. litgraph's
engine stays the analysis backend. litigate replaces hand-written JSON as the
way the engine's input is authored.

Status: design accepted 2026-10-22, epic in this repo's bead store
(epic `lg-litigate-epic-tuq`, `br list --label litigate`). This doc is the root of the epic: every bead
cites a section here, and an agent should be able to work from this doc and
the bead alone.

Pre-freeze design review (2026-10-23):
[`research/design-review-2026-10-23.md`](research/design-review-2026-10-23.md).
It lists defects on main and the items that must be decided before the
language is frozen. Its paradigm recommendation was ADOPTED on 2026-10-23
and is normative in [`SPEC.md`](SPEC.md) §0.

## 1. Why

The packs are hand-written, unversioned JSON snapshots of procedural rules.
Four problems follow:

1. **Rules change and nothing records it.** FRCP amends every Dec 1
   (28 U.S.C. § 2074). A pack is today's text, and a matter that spans an
   amendment cannot be modeled.
2. **Forum knowledge leaks into the engine.** Deadline counting
   (`clock/{general,itc,holidays}.rs`, the `RuleSet` enum,
   `ruleset_for_forum`) and pack-id literals in the CLI, cite normalizer,
   validator and library are Rust code. A new forum needs an engine change.
3. **Nothing checks the law-shaped invariants.** These are the effective-date
   overlaps, which authority may override which, and whether every
   "unless the court orders otherwise" really delegates. JSON Schema checks
   shape, not meaning.
4. **Matter facts can contradict procedure.** A scenario can claim a posture
   its own history could never reach.

Prior art covers pieces of this and nothing covers the whole. Catala has
default logic, OpenFisca has as-of dates, Governatori and Rotolo have temporal
defeasible logic, and session types model who may move when. None of them
compiles dated, precedence-ranked procedure into an analyzable game. The
research reports are under `docs/litigate/research/`.

## 2. The tower

Each layer depends only on the layers below it. Every id at every layer
resolves both up and down (§8), so an agent can enter anywhere.

```
 L7  Accretion    diagnostics with machine-applicable fixes → edit .lit/.case → recheck
 L6  Answers      litgraph explain/compare + litigate explain (value → rule → source span)
 L5  Analyses     litgraph ops (solve, chain, simulate, settle, ...) unchanged
 L4  View         litgraph scenario resolved against the graph, unchanged
 L3  Case         .case → compiled matter: parties, replayed event history, derived
                  posture, facts, stakes, beliefs  → lowers to scenario JSON
 L2  Graph        litgraph packs + links compiled into one namespaced graph, unchanged
 L1b Litigatable  a compiled, verified, content-hashed rule library as of an interval:
                  IR + verification report + source map → lowers to pack/links JSON
 L1a Lit source   .lit modules: typed, dated, ranked, cited rules (replaces hand JSON)
 L0  Sources      primary-law text with sha256 and as-of dates (exists: sources/)
```

L2 to L7 are litgraph today and do not change, except that forum-specific
code moves out of the engine (§9). L1a, L1b and L3 are new. From the flip
onward (§11) `packs/*.json` and `scenarios/*.json` are generated artifacts,
never edited by hand.

## 3. Principles

These are the tests a design choice has to pass.

1. **Illegal states are unrepresentable.** If the law forbids it, the types
   or the checker forbid it. Posture is derived from history, never asserted.
2. **Total and pure.** No general recursion, no IO, no ambient time. Every
   program terminates, and checking it is decidable and fast.
3. **No silent defaults.** An unknown is an explicit hole (`?`) with a
   reason. Holes compile in `dev` profile with a warning and fail in
   `strict`. This mirrors litgraph's "no silent guesses".
4. **Every claim cites L0.** A rule without a citation that resolves to a
   span in `sources/` is a compile error in `strict`.
5. **One door, one envelope.** Every capability is a `litigate` subcommand
   with a JSON envelope. The CLI, LSP and MCP are thin wrappers over one
   library API.
6. **Deterministic and content-addressed.** Same inputs give byte-identical
   outputs. Litigatables carry hashes, and a `.case` pins them in a lockfile.
7. **Forum-agnostic core.** The compiler and engine know nothing about any
   forum. FRCP is a library, not a feature.
8. **Cheap enough to think with.** Incremental (salsa) queries. `check` on
   the full stdlib in well under a second, so an agent can loop on it.

## 4. Language: `.lit`

The surface syntax below is a sketch. `docs/litigate/SPEC.md` (bead
`spec`) is normative and may refine it. The concepts are fixed here.

### 4.1 Modules and packages

```lit
module frcp.rule12
import core.{time, money, roles}
import frcp.rule6.{frcp6}          // counting rule
```

A package is a directory with `Lit.toml` (name, version, forum key,
authority level, dependencies). `lit/` is the stdlib: `lit/core`,
`lit/frcp`, `lit/frap`, `lit/cafc`, `lit/cofc`, `lit/ptab`, `lit/itc`,
`lit/mpep`, `lit/frcrimp`, `lit/ttab`, `lit/links`. One package lowers to
one litgraph pack. Its namespace is the pack id, so ids stay
`frcp-civil-procedure::answer-due`.

### 4.2 Types

Nominal algebraic data types (`type`, `enum`) with exhaustive `match`. No
null, no implicit conversions, no subtyping except the authority lattice.

Built-in domain types, each closing off a class of mistakes:

| type | rules out |
|---|---|
| `Date` | timezones and times of day; dates only (`2025-03-03`) |
| `Days<CalendarDays>`, `Days<CourtDays>`, `Days<BusinessDays>`, `Hours` | adding court days to calendar days; converting needs an explicit counting rule |
| `Money<USD>` | stored as integer cents, so no floats for money and no mixed currencies |
| `Probability` | a literal outside [0, 1]; stored as an exact rational |
| `Dist<E>` over enum `E` | distributions that do not sum to exactly 1 or miss a variant (checked exactly) |
| `Cite` | a string parsed by the cite grammar into work (source binding), expression (its `as_of`, the text pinned by `path_sha256`) and pinpoint, resolved against L0 at compile time (SPEC §4.3) |
| `Interval<Date>` | half-open `[from, to)` effective windows |
| `Role` | actors that are not declared parties or roles of the forum |

### 4.3 Procedure

```lit
state answer_due: "Answer or Rule 12 motion due" { group pleadings }

decision respond by Defendant cite "FRCP 12(a)(1)(A)(i)" {
  answer       -> answered     { hours 8, within answer_period }
  move_dismiss -> mtd_pending  { hours 30, within answer_period, tags [dispositive] }
  default      -> default_risk { sets [defaulted] }
}

chance mtd_ruling by Court discretion cite "FRCP 12(b)(6)" {
  granted -> dismissed  p ?("no calibration yet")
  denied  -> answer_due p ?
}

terminal dismissed outcome [win, judgment]
```

`state`, `decision by <Role>`, `chance by <Role|Nature>` and
`terminal outcome [...]` map one to one onto litgraph node kinds. Edges carry
typed cost, hours, deadline refs, duration, flags
(`sets/clears/requires/forbids`), tags and attributes. Judicial discretion
("court may", "good cause") is always a `chance ... discretion` node with
explicit probabilities or holes, never logic.

### 4.4 Time: effective-dated rules

```lit
deadline answer_period: Days<CalendarDays>
  cite "FRCP 12(a)(1)(A)(i)"
  effective [2009-12-01, ..)
  supersedes answer_period@[1993-12-01, 2009-12-01)
  = 21.days from Served(complaint) counted_by frcp6
```

Every rule-bearing declaration has an effective interval. The checker proves
two things:

1. Versions of one declaration never overlap.
2. Their union covers the declaration's lifetime, or the gaps are explicit.

Resolution is always `as_of <Date>`. Versions are bitemporal (Snodgrass):
`in_force` is when the law applied, and `known_at` is when the corpus
recorded it. A query can therefore ask both what the law was on D and what we
believed on K.

When a matter straddles an amendment, a declared transition rule decides, not
a hard-coded one. FRCP 86(a) applies amendments to pending proceedings unless
the Court orders otherwise or doing so would not be "just and practicable".
In `.lit` that is a `transition` declaration attached to the amendment. Its
discretionary leg is a `chance ... discretion` node the case can resolve with
an order event. By default each event is replayed under the rules in force on
that event's date, as the declared transition rule directs.

Following LegalRuleML, three time axes are distinct: enactment, effect and
applicability. The core library and the spec name each one.

### 4.5 Precedence, delegation and exceptions

The core library declares the authority lattice as two chains that share
their ends (`lit/core/authority.lit`):

```lit
authority Statute > FederalRule > LocalRule > StandingOrder > CaseOrder
authority Statute > Regulation > CaseOrder
```

The first chain is the courts' rules:
- A statute outranks the rules of procedure (28 U.S.C. § 2072(b)).
- The rules of procedure outrank a court's local rules (FRCP 83(a)(1)).
- Local rules outrank a judge's standing orders.
- Standing orders outrank an order in one case.

The second chain is an agency's rules of practice, such as 19 C.F.R.
Part 210 (the ITC) or 37 C.F.R. Parts 1, 2, 41 and 42 (the USPTO and its
boards). Those rules answer to their enabling statute and bind the agency's
orders in one proceeding.

`Regulation` is deliberately incomparable with `FederalRule`, `LocalRule`
and `StandingOrder`. Agency rules and court rules never govern the same
deadline, so no canon may rank one over the other. A conflict between them
is a modelling error to fix in the data, not something to settle by an
order the law does not give. The lattice records the law's hierarchy; it is
never reshaped to make a package's data check.

A lower authority may change a higher one's rule only where the higher rule
delegates, by a typed capability naming the principal, the operations and
the bound (SPEC §5.5):

```lit
deadline answer_period ... delegable { to CaseOrder, op set }   // "unless the court orders otherwise"
```

A displacing relation (`override`, `notwithstanding`, `supersedes`,
`revives`) without a capability covering what it does to the target is a
compile error naming the missing capability. An emergency `grant` gives
capabilities for a bounded window, and a jurisdictional deadline admits no
waiver, unbounded extension or delegation at all. Within one authority, Catala-style `default` and
`exception` blocks express rule-plus-exceptions. Exceptions are checked to be
non-overlapping or explicitly prioritized. Together this is Governatori and
Rotolo's temporal defeasible logic, made static.

Every modification of base law is its own package with a declared
instrument class (`kind` in `Lit.toml`: amendment, local rule, general
order, standing order, case order, emergency declaration, guidance,
precedent, overruling), checked against the per-forum-family matrix in
`lit/core`, and layered in a world (a set of layers at a location, in a
modality): [CHANGE.md](CHANGE.md), SPEC §5.7-§5.8.

Refinements from the library research (`research/library-ideas.md`):

- **Two-stage conflict resolution.** First declared relations apply, then
  the canons:
  - Declared relations: `subject_to X`, `notwithstanding X`, `supersedes X`
    and `revives X`. Repeal does not revive a repealed rule unless the
    declaration says `revives` (Scalia and Garner, *Reading Law* §§ 13,
    27–29, 55–57).
  - Canons: higher authority, then specific over general, then later over
    earlier.
  - A conflict still left open is a compile error with both spans. It is
    never resolved silently.
- **Two exception kinds** (Pollock 1987).
  - A *rebutting* exception concludes the opposite.
  - An *undercutting* exception only disables the rule.
  - They lower differently and are explained differently.
- **Rule kinds** (Hart): `duty` rules (must act within D), `power` rules
  (may move; creates a decision edge) and `liability` consequences (a missed
  duty sets a flag or forces an edge).
- **Direction-dependent counting.** "At least N days before" counts
  backward, and the counting rule's direction is part of its type.
- **Typestate.** Submission types are closed sums. A `.case` posture is
  typed by stage, so `Answer` cannot be filed against a case already in
  `Judgment`.

### 4.6 Deadline counting and calendars

Counting rules and holiday calendars are `.lit` declarations, not engine code:

```lit
counting frcp6 cite "FRCP 6(a)" {
  exclude_trigger_day, include_last_day,
  roll_forward_on [weekend, legal_holiday, inaccessible],
  service_extension { mail: 3.days, ... } cite "FRCP 6(d)",
  holidays federal_holidays
}
calendar federal_holidays {
  juneteenth: fixed(06-19) effective [2021-06-17, ..) observed nearest_weekday
  ...
}
```

They lower to the data-driven clock IR from §9. FRCP 6, RCFC 6, FRAP 26 and
19 CFR 210.6(a) are written in `lit/core` and `lit/<forum>`, including the
ITC business-day anchor and its short-period rule.

### 4.7 Inline tests

```lit
test "answer due 21 days after personal service" cite "FRCP 12(a)(1)(A)(i)" {
  given Served(complaint, Personal) on 2025-03-03
  expect deadline(answer_period) == 2025-03-24
}
```

Every deadline and every counting rule should carry tests. The lint
`rule-without-test` warns in `dev` and errors in `strict` for deadlines.
These tests are also the legal assertions a reviewer reads.

## 5. Language: `.case`

```case
case acme_v_widget
uses frcp ^2024.12, local.edtx =2024.12.01       // pinned in Lit.lock
as_of 2025-06-01
perspective Defendant

parties {
  acme:   Plaintiff  Corporation
  widget: Defendant  Corporation
}

events {
  2025-03-03  Filed(complaint) by acme
  2025-03-05  Served(complaint, Personal) on widget
  2025-03-20  Filed(motion_dismiss(12b6)) by widget
}

facts  { time_barred: false }
stakes { judgment(acme): -2_000_000.usd, dismissed: 0.usd }
beliefs { observe mtd_ruling: { granted: 3, denied: 1 } }
```

The checker enforces:

1. **Typed parties.** Each party holds exactly one role per proceeding, and
   the role must be declared by the forum.
2. **Replay, never assert.** Events must be in date order, and each must be a
   legal transition from the posture at that date, under the rules in effect
   that day. The current posture is derived. Each trigger creates an
   obligation instance with a bearer; a missed deadline is a
   `deadline-missed` error unless a cited event of a type that `extends` or
   `waives` that deadline, within a grant the deadline declares (a
   numeric cap, the court's discretion, or `never`), transformed the
   instance first (SPEC §5.3, §5.6). Event names carry no meaning.
3. **Facts are typed.** They are checked against the forum's fact nodes, and
   each value must be a variant of that node's outcome enum.
4. **Stakes are exhaustive.** Every terminal reachable from the derived
   posture has a value or an explicit `unvalued` (an error in `strict`).
5. **Uses are pinned.** Versions come from `Lit.lock`, with content hashes.

A `.case` lowers to litgraph scenario JSON (`start`, `facts`, `payoffs`,
`observe`, `mask`, `remove_edges` and so on) and a deadline schedule.

## 6. Compiler architecture

```
crates/litigate-syntax   logos lexer, rowan lossless CST, error-recovering recursive-descent
                         parser, typed AST views, formatter (litigate fmt)
crates/litigate-sema     salsa db: packages/modules/lockfile, name resolution, types,
                         temporal resolution, precedence/delegation/exceptions,
                         case replay, verification obligations
crates/litigate-lower    HIR → litgraph Pack / LinkFile / Scenario (litgraph's own serde
                         types, so the output cannot drift from what the engine accepts),
                         clock IR, source maps; semantic JSON diff
crates/litigate-cli      `litigate` binary: one subcommand per op, JSON envelope
crates/litigate-lsp      tower-lsp server over the same salsa db
editors/tree-sitter-litigate  grammar.js, highlight/indent/fold/locals queries, corpus
editors/{emacs,helix,nvim}    modes built on the tree-sitter grammar
lit/                     stdlib (§4.1)
cases/                   .case ports of scenarios/ and examples/
```

### 6.1 Queries and the language server

One salsa database serves `check`, `build` and the language server, with
the granularity of design review §4.1 (lg-flu.17):

- **Position-independent item tree.** `item_tree(file)` anchors every range
  at its top-level declaration; `anchors(file)` is the side table that
  locates them (rust-analyzer's `AstIdMap`). Moving declarations, by
  blank lines, comments or reindentation, leaves the item tree equal, so
  salsa backdates it.
- **Typing per declaration.** `check_decl(DeclId)` types one top-level
  declaration and builds its HIR with position-free spans;
  `check_module(file)` runs the module-level checks and locates that HIR
  (`hir::spans`). An edit inside one declaration re-types only it.
- **Resolution per world.** The world is an interned key
  (`WorldId {as_of, known_at}`): `world_selection(package, world)` is what
  governs each of a package's declarations there, and
  `resolve_in(decl, world)` reads one, backdated per declaration.
  Switching worlds re-runs neither parsing nor typing.
- **Verification and replay.** `package_verdict(ws, package, build)` reads
  only the packages the package can see (its closure; all of them for a
  links package) and `case_check(ws, case, build)` replays one `CaseFile`
  input; clock fragments and the engine clock are lowered once per
  workspace.

The language server keeps that database across edits. Edits wait
`debounceMs` for the next; a check runs on a blocking thread, so a newer
edit arrives while it runs and cancels it between packages and matters,
the last complete analysis answering requests meanwhile. On top of it the
server offers rename (every version of a declaration and every reference,
across files), completion, and a world picker (`litigate/worlds`,
`litigate/selectWorld`) whose world hover reports what governs each
declaration in.

All crates live in the litgraph workspace. That gives one `cargo test`, one
`just ci`, one AGENTS.md, and lowering against litgraph's real types. The
language can split into its own repository later. Nothing here assumes the
monorepo beyond path dependencies.

The compiler's own parser is the source of truth for syntax. The tree-sitter
grammar is checked against it by a conformance test that parses every `.lit`
and `.case` file in the repo with both and requires zero tree-sitter `ERROR`
nodes. The two must agree on the shared corpus.

## 7. Verification

`litigate check` discharges named obligations and emits a
`VerificationReport`, which is embedded in every litigatable:

| obligation | meaning |
|---|---|
| `well-typed` | types, units, exhaustiveness |
| `cite-resolves` | every `Cite` resolves to an L0 span (reuses `litgraph::cite`) |
| `effective-disjoint`, `effective-covered` | version intervals neither overlap nor leave unexplained gaps |
| `precedence-acyclic`, `delegation-respected`, `exceptions-ordered` | §4.5 |
| `dist-exact` | chance distributions sum to exactly 1 over all variants |
| `graph-exits` | every non-terminal has an out-edge; no dead ends |
| `graph-reachable` | every node is reachable from the start, or marked `entry` |
| `deadline-satisfiable` | no state whose deadlines cannot all be met |
| `case-replay` | §5 rule 2 |
| `holes` | lists every `?`; fails in `strict` |

Beyond the checker:

- **Kani** proofs for date and counting arithmetic.
- **proptest**: parse(print(ast)) == ast, fmt idempotence, and lowering
  round-trips.
- **cargo-fuzz** on the parser.
- **cargo-mutants** on sema.

All of this follows `config/programming-languages/rust/README.md` in the
dotfiles and the swe-project-plugin-pack-rust gates: fmt, clippy
`-D warnings`, nextest, deny, audit, machete, 500-line file ceiling, and the
97% line-coverage floor on the `litgraph` engine. Every `litigate-*` crate is
held to 100% lines, regions and branches with no exclusions (`just
cov-litigate`, part of `just ci`).

## 8. Agent ergonomics

This contract is designed for an agent that knows nothing at the start.

- **`litigate describe`** returns the whole language as JSON: grammar
  productions, every construct with an example, built-in types, the stdlib
  index (packages, modules, declarations, effective ranges), every diagnostic
  code with its explanation, and the CLI contract. One call orients an agent.
- **Envelope.** Every subcommand with `--format json` returns
  `{schema_version, ok, op, result, diagnostics, provenance, elapsed_ms}`,
  the same shape as litgraph; `result` is kept alongside errors.
- **Diagnostics** have this shape (schema v2, SPEC §9.2):
  `{code: "L0412", severity, span: {file, line, col, end_line, end_col},
  message, labels, notes, fixes: [{edits: [{span, replacement}],
  applicability}], derivation, fingerprint, profile, world}`. Codes are
  stable. `litigate explain L0412` gives the long form. `litigate fix`
  applies machine-applicable fixes. An agent edits by fixes, not by guessing.
- **`litigate where <id>`** resolves any id at any layer: a litgraph node or
  edge, `forum::decl@version`, a diagnostic location, or a cite. It returns
  the declaration span, the L0 source span, the effective versions, which
  rules override it, and the litgraph ids it lowers to. Lowered JSON carries
  a `source_map` sidecar, so engine warnings map back to `.lit` lines.
- **`litigate explain-value`** traces a computed deadline or edge value back
  through the steps that produced it: counting steps, then the rule version,
  then any override and its delegation, then the L0 text.
- **`litigate diff A B`** is a semantic diff of two litigatables or two
  as-of dates. It lists which rules, deadlines and edges changed, with cites.
  "What did the 2024 amendments change?" is one call.
- **`litigate query`** runs structured queries such as rules citing X,
  declarations effective on D, overrides of Y, and holes in package P. Output
  is small by default (`--limit`).
- **`litigate import pack.json`** turns legacy pack JSON into `.lit` source.
  It is the migration accelerator and stays as an on-ramp.
- **MCP.** litgraph-mcp gains `litigate_check`, `litigate_build`,
  `litigate_where`, `litigate_explain` and `litigate_describe`.
- **Determinism.** No timestamps or absolute paths in outputs. `Lit.lock`
  pins content hashes.

## 9. Engine de-hardcoding (before the language)

The goal is that `crates/litigate-engine/src` contains no forum knowledge.

1. **Data-driven clock.** Add a `CountingRule` and `HolidayCalendar` IR to the
   pack schema (v3, additive), referenced by a pack's `counting` field. A
   generic interpreter replaces `RuleSet` and `ruleset_for_forum`. All four
   current rule sets become data in `packs/` (later lowered from `.lit`).
   Every existing clock test passes unchanged, bit for bit.
2. **No pack-id literals.** Move the forum and pack literals in
   `litgraph-cli/src/main.rs`, `cite/{verify,normalize}.rs`,
   `api/{validate,catalog,mod}.rs`, `scenario/library.rs`,
   `api/calibration/apply.rs`, `metrics.rs`, `litgraph-wasm` and
   `litgraph-mcp` into pack metadata, such as cite-normalization aliases and
   source bindings.
3. **Gate.** A test fails if any forum key or pack id appears in
   `crates/*/src` outside tests and fixtures. Elimination-ladder rung 2.

## 10. What else compiles from this (later epics)

These are not in this epic. Keep each as a separate follow-on task after the
core compiler lands; none is a prerequisite for compiling or analyzing a
`.lit` package or `.case` matter. The shared boundary is the verified case,
its replayed event history, derived deadlines and posture, plus the existing
litgraph analysis API. Each task consumes that boundary without adding forum
rules to the engine.

1. **Deadline calendars (`.ics`).** Consume a compiled case and emit an RFC
   5545 calendar for open deadlines, with reminders as an output option.
   Unknown due dates remain explicitly unscheduled; the export must not
   invent dates. Keep calendar serialization and CLI options in the calendar
   lowering, with tests for date-only deadlines and unscheduled obligations.
2. **Filing linter.** Accept a proposed filing, filing date and actor; replay
   it against the case to report whether it is a legal move, whether it is
   timely, which obligation it discharges, and which deadlines it triggers.
   Filing-specific checks include page and word limits, required
   certificates, and ECF code lists. It must use typed filing/event rules
   from the packages rather than infer legal meaning from event names.
3. **Docket matcher.** Match PACER/RECAP docket entries to declared event
   forms and advance a `.case` only when the match is supported. Preserve the
   source entry and match confidence for review; ambiguous or unmatched
   entries remain unresolved instead of silently changing posture. This is
   the NLP integration boundary, separate from deterministic case replay.
4. **Agent action sets.** Project the current typed marking onto a party and
   expose only the moves legal for that party and posture as MCP actions.
   Action sets depend on the markings and firing-sequence model in §2.8;
   they are not a separate source of procedural rules.
5. **RECAP replay calibration.** Derive calibration observations from
   replayed, sourced docket histories and emit the existing calibration
   format. Keep provenance and selection criteria with each estimate; do not
   treat docket matching as ground truth without review.
6. **Rule-amendment impact.** Recompile a corpus of pinned cases under
   selected rule versions and compare their outcomes, deadlines and paths.
   Report the selected version/world and changed results so the analysis is
   reproducible.
7. **`litigate-ts`.** Provide typed TypeScript bindings over
   `litigate-wasm`, generated from the public request/response contract.
   This is a language/tooling task and needs the repository's explicit
   TypeScript language-preference exception before implementation begins.

## 11. Migration and acceptance

1. Each pack in `packs/` gets a `lit/<forum>` package. Start from
   `litigate import`, then add effective dates, typed cites, delegation and
   tests.
2. **Equivalence.** For every pack, lowering its package must produce JSON
   that is semantically equal to the original: same nodes, edges, fields and
   values, ignoring key order and the added `source_map` and `generated`
   fields. `litigate diff --against packs/<id>.json` must be empty. The same
   applies to `lit/links` against `packs/links.json` and to
   `cases/*.case` against `scenarios/*.json`.
3. **The flip.** `packs/*.json`, `packs/links.json` and `scenarios/*.json`
   become generated. `just lit-sync` regenerates them, and `just ci` fails if
   they are stale. Every existing test (unit, parity, golden, famous cases,
   examples) passes unchanged against the generated files.
4. **Docs.** AGENTS.md, ARCHITECTURE.md and PACK_SCHEMA.md point to `lit/`
   as the authoring surface. The rule becomes: never edit `packs/*.json` by
   hand.

The epic is done when items 1 to 4 hold for all nine packs, links and the
four scenarios. In addition: `litigate check --profile strict` passes on
`lit/core` and every deadline has a test; editor modes highlight the whole
corpus; the LSP serves diagnostics, hover and go-to-definition; and the
engine gate in §9.3 is green.
