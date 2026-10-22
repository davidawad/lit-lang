# litigate language specification

Status: normative for compiler implementation. This document refines
[`DESIGN.md`](DESIGN.md); where the design sketch is silent, this file fixes
the contract. Language version `1`, package lock format `1`, and litgraph
output schema version `2` are the initial targets. The clock IR described
below is additive schema v3 per DESIGN §9.1.

## 1. Compiler and dependency choices

The compiler is split into `litigate-base`, `litigate-syntax`,
`litigate-sema`, `litigate-verify`, `litigate-lower`, `litigate-import`,
`litigate-cli`, and `litigate-lsp`. Syntax uses `logos` for tokenization and
`rowan` for a lossless, recoverable CST. Semantic queries use `salsa`;
`litigate-lsp` uses `tower-lsp` on Tokio. Contract/output snapshots use
`insta`, property invariants use `proptest`, and human diagnostics use
`ariadne`. Dependencies are workspace-pinned by semver requirement; lockfile
pins exact resolved versions. Syntax and HIR public data types do not depend
on salsa internals.

## 2. Lexical rules

Input is UTF-8. Identifiers are Unicode XID identifiers with `_`, and may
contain `-` after the first character for stable kebab-case graph ids.
Keywords are reserved only in keyword position; escaped identifiers are not
provided in v1. Whitespace is insignificant except as a token separator.
`//` begins a line comment; `///` begins a documentation comment attached
to the next declaration. `/* ... */` is a non-nesting block comment.
Strings use double quotes and escapes `\\`, `\"`, `\n`, `\r`, and `\t`.
A date is exactly `YYYY-MM-DD` and must be a valid Gregorian date. Numeric
literals are non-negative decimal integers with optional `_` separators;
probability literals may be decimals or `numerator/denominator`. Signed
values are unary `-` followed by a number. `2_000_000.usd`, `21.days`,
`5.court_days`, `3.business_days`, and `8.hours` are single typed-literal
forms. `?` is a hole; `?("reason")` is a documented hole. `cite FRCP
12(a)(1)(A)(i)` is a citation literal whose grammar consumes the citation
text to a line break or enclosing `{`, `}`, or comma delimiter.

```ebnf
Letter       = XID_Start | "_" ; Continue = XID_Continue | "_" | "-" ;
Ident        = Letter, { Continue } ; Digits = Digit, { Digit | "_" } ;
Date         = Digit,Digit,Digit,Digit,"-",Digit,Digit,"-",Digit,Digit ;
Number       = Digits, [".", Digits] ; Rational = Digits, "/", Digits ;
String       = '"', { Escape | any UTF-8 scalar except '"', '\\' }, '"' ;
Escape       = "\\", ("\\"|"\""|"n"|"r"|"t") ;
LineComment  = "//", { any scalar except newline } ;
DocComment   = "///", { any scalar except newline } ;
BlockComment = "/*", { any scalar except "*/" }, "*/" ;
Hole         = "?" | "?(", String, ")" ;
Cite         = "cite", Authority, CitationText ;
Money        = SignedNumber, ".", Currency ;
Duration     = Number, ".", ("days"|"court_days"|"business_days"|"hours") ;
Probability  = Number | Rational | Hole ;
```

## 3. EBNF: rule modules and matters

The grammar below defines significant-token structure; comments and whitespace
may occur between tokens. `Id` is an `Ident` in declaration position;
`Ref` is a dotted or `::`-qualified identifier, optionally versioned. Lists
are comma-separated and may have a trailing comma.

```ebnf
File          = LitFile | CaseFile ;
LitFile       = ModuleDecl, { ImportDecl | PackageDecl | StartDecl | GroupDecl | Declaration } ;
ModuleDecl    = "module", Ref ;
ImportDecl    = "import", Ref, [".", "{", IdList, "}"] ;
PackageDecl   = "package", Id, "{", "forum", Id, "authority", Id, "}" ;
StartDecl     = "start", Ref ;
GroupDecl     = "group", Id, ":", String ;
Declaration   = TypeDecl | EnumDecl | StateDecl | DecisionDecl | ChanceDecl
              | TerminalDecl | RuleDecl | DeadlineDecl | CountingDecl
              | CalendarDecl | TransitionDecl | AuthorityDecl | DefaultDecl
              | ExceptionDecl | TestDecl ;
TypeDecl      = "type", Id, "=", Type, ";" ;
EnumDecl      = "enum", Id, "{", IdList, "}" ;
StateDecl     = "state", Id, ":", String, ["{", StateField*, "}"], [VersionClause] ;
StateField    = "group", Id | "entry" | "fact", EnumType | VersionClause ;
DecisionDecl  = "decision", Id, "by", Role, ["cite", Cite], EdgeBlock, [VersionClause] ;
ChanceDecl    = "chance", Id, "by", (Role|"Nature"), ["discretion"],
                ["cite", Cite], EdgeBlock, [VersionClause] ;
TerminalDecl  = "terminal", Id, "outcome", "[", IdList, "]",
                ["payoff", Money], [VersionClause] ;
EdgeBlock     = "{", EdgeDecl+, "}" ;
EdgeDecl      = Id, "->", Ref, ["p", Probability], EdgeFields ;
EdgeFields    = "{", EdgeFieldList, "}" | ;
EdgeFieldList = EdgeField, {",", EdgeField} ;
EdgeField     = "hours", Duration | "cost", Money | "within", Ref
              | "deadline", Ref | "duration", Duration | "sets", IdList
              | "clears", IdList | "requires", IdList | "forbids", IdList
              | "tags", IdList | "authority", Ref | "attrs", Record ;
RuleDecl      = "rule", ("duty"|"power"|"liability"), Id,
                ["cite", Cite], "{", Declaration*, "}", [VersionClause] ;
DeadlineDecl  = "deadline", Id, ":", DaysType, ["cite", Cite],
                VersionClause, Relation*, "=", Number, Unit,
                "from", Expr, "counted_by", Ref ;
CountingDecl  = "counting", Id, ["cite", Cite], "{", CountingItem*, "}", [VersionClause] ;
CountingItem  = "exclude_trigger_day" | "include_last_day"
              | "roll_forward_on", IdList
              | "service_extension", Record, ["cite", Cite]
              | "holidays", Ref | "anchor", Id | "short_period", Expr ;
CalendarDecl  = "calendar", Id, "{", HolidayDecl*, "}", [VersionClause] ;
HolidayDecl   = Id, ":", HolidayRule, "effective", Interval,
                ["observed", Id] ;
TransitionDecl= "transition", Ref, "{", TransitionArm*, "}", [VersionClause] ;
TransitionArm = "unless", Expr, "->", Ref | "otherwise", "->", Ref ;
AuthorityDecl = "authority", Id, {">", Id} ;
Relation      = "subject_to", Ref | "notwithstanding", Ref
              | "supersedes", Ref | "revives", Ref | "delegable_to", IdList
              | "override", Ref ;
DefaultDecl   = "default", Id, "when", Expr, "=>", Expr ;
ExceptionDecl = "exception", Id, ("rebuts"|"undercuts"), Ref,
                "when", Expr, "=>", Expr, ["priority", Number] ;
TestDecl      = "test", String, ["cite", Cite], "{", TestStmt*, "}" ;
TestStmt      = "given", Expr, "on", Date | "expect", Expr, Compare, Expr ;

CaseFile      = "case", Id, UsesDecl, "as_of", Date,
                "perspective", Role, PartiesBlock, EventsBlock,
                [FactsBlock], [StakesBlock], [BeliefsBlock] ;
UsesDecl      = "uses", Use, {",", Use} ;
Use           = Ref, ("^", VersionRange | "=", Version) ;
PartiesBlock  = "parties", "{", PartyDecl+, "}" ;
PartyDecl     = Id, ":", Role, Type ;
EventsBlock   = "events", "{", Event*, "}" ;
Event         = Date, EventExpr, ("by"|"on"), Id ;
FactsBlock    = "facts", Record ;
StakesBlock   = "stakes", Record ;
BeliefsBlock  = "beliefs", "{", "observe", Ref, ":", DistRecord, "}" ;
Interval      = "[", Date, ",", (Date|".."), ")" ;
VersionClause = ["effective", Interval], ["known_at", Date] ;
Ref           = Id, {("."|"::"), Id}, ["@", Version] ;
IdList        = Id, {",", Id}, [","] ;
Role          = Ref ;
DaysType      = "Days<", ("CalendarDays"|"CourtDays"|"BusinessDays"), ">" ;
Unit          = "days"|"court_days"|"business_days" ;
Currency      = Ident ;
Type          = Ref | "Date" | "Hours" | "Money<", Currency, ">"
              | "Probability" | "Dist<", Ref, ">" | DaysType
              | "Interval<Date>" | "Role" ;
EnumType      = Ref ;
Version       = Digits, {".", Digits}, ["-", Ident] ;
VersionRange  = ("^"|"~"|">="), Version ;
Expr          = Match | BoolOr ;
BoolOr        = BoolAnd, {"||", BoolAnd} ;
BoolAnd       = CompareExpr, {"&&", CompareExpr} ;
CompareExpr   = AddExpr, [Compare, AddExpr] ;
Compare       = "=="|"!="|"<"|"<="|">"|">=" ;
AddExpr       = MulExpr, {("+"|"-"), MulExpr} ;
MulExpr       = Unary, {("*"|"/"|"%"), Unary} ;
Unary         = ["!"|"-"], Primary ;
Primary       = Literal | Ref | Call | Record | List | "(", Expr, ")" ;
Call          = Ref, "(", [ExprList], ")" ;
ExprList      = Expr, {",", Expr}, [","] ;
Match         = "match", Expr, "{", MatchArm+, "}" ;
MatchArm      = Pattern, "=>", Expr, [","] ;
Pattern       = Id | "_" ;
Record        = "{", [RecordField, {",", RecordField}, [","]], "}" ;
RecordField   = Id, ":", Expr ;
List          = "[", [ExprList], "]" ;
Literal       = Date | Money | Duration | Probability | String | Boolean
              | Hole | "unit" ;
Boolean       = "true"|"false" ;
SignedNumber  = ["-"], Number ;
Authority     = Ident ;
CitationText  = { any non-delimiter scalar } ;
EventExpr     = Call ;
DistRecord    = Record ;
HolidayRule   = Call | Date ;
```

## 4. Package, imports, and lock semantics

A `Lit.toml` has `package.name`, `package.version` (SemVer),
`package.forum`, `package.authority`, `package.language = 1`, default
`package.in_force`, default `package.known_at`, and `dependencies` mapping
package names to SemVer constraints and source ids. It also has `package.title`,
`package.description`, `package.jurisdiction`, `package.roles`, and explicit `package.pack_id`
(the legacy litgraph id). These provide the corresponding pack metadata.
`[sources.<id>]` entries bind vendored L0 paths or public URLs, SHA-256, title,
and as-of dates; citations must resolve through those bindings. Each package
has exactly one top-level `start <node-id>` and may declare group labels with
`group <id>: "Label"`.
A `Lit.lock` is deterministic TOML containing each resolved package's exact
version, normalized source digest (SHA-256), dependency list, and compiler
language version. It has no timestamps or absolute paths. Imports resolve
only within the current package, its declared dependencies, or the named
stdlib package; filesystem traversal and undeclared transitive imports are
errors. Import graph cycles are errors. A package maps to one pack: explicit
`forum` is metadata and the stable litgraph `Pack.id` is the explicit legacy
id in `package.pack_id`. Never derive a pack id from a package name or current
directory. Module names do not affect node ids. Node and edge
ids preserve declared kebab-case; fully qualified graph identity is
`<pack-id>::<local-id>`.

Every rule-bearing declaration resolves an `in_force` half-open interval and
a `known_at` date. An explicit `VersionClause` overrides package defaults;
package defaults are required. `known_at` is the corpus recording date, not
the law's effective date. An as-of query supplies both axes (omitting
`known_at` means the latest known version).

The source digest is SHA-256 over the UTF-8 bytes of `Lit.toml` followed by
all package source files sorted by normalized package-relative POSIX path.
Each component is framed as unsigned 64-bit big-endian path length, path
bytes, unsigned 64-bit big-endian content length, then content bytes.
`Lit.lock` records each imported package digest
and is excluded from those digests. Line endings are not normalized. The
lockfile is rewritten only by `litigate lock`; checking a stale pin is an
error.

## 5. Construct semantics

- `module`, `import`, `package`, `Lit.toml`, and `Lit.lock` define an acyclic,
  pinned package graph and its one-to-one pack identity.
- `start` names exactly one local node id used as `Pack.startNodeId`.
  `group` defines a stable group id and display label; state `group` fields
  reference these declarations.
- `type`, `enum`, `match` define closed nominal sums and exhaustive pattern
  matching. No null, implicit conversion, subtyping, or general recursion.
- `state`, `decision`, `chance`, `terminal` lower to same-kind graph nodes.
  `state` fields include label/group/entry/fact. A `decision by Role` assigns
  outgoing edges to that actor; `chance by Nature|Court` is uncontrolled and
  requires a complete exact distribution except explicit dev holes. A
  terminal is absorbing and carries outcome tags and optional payoff.
- Edges carry destination, stable local id, actor, cite/authority, probability,
  costs, hours, duration/deadline refs, tags, attributes, and sets/clears/
  requires/forbids flags. Self-edge probability is forbidden. A pure chance
  node's exact branch probabilities sum to one.
- `deadline`, `counting`, `calendar`, and `transition` declare temporal law.
  Counting excludes/includes the trigger/final day, rolls final dates, adds
  service extensions, uses a calendar, and can define anchor and short-period
  behavior. Each declaration has `[from,to)` `in_force`, plus `known_at`
  corpus time. Query selection is `as_of` plus optional `known_at`; one
  version per declaration must resolve. A gap is legal only if explicitly
  declared. `transition` selects the rule version for each event. FRCP 86(a)
  applies an amendment to pending proceedings unless the court directs
  otherwise or application would be unjust and impracticable; the declaration
  expresses both exceptions as conditional arms and keeps the prior version
  for an exempt proceeding. This is data in `lit/core`/`lit/frcp`, never
  compiler code.
- `authority` defines a strict partial order. `A subject_to B` makes A yield
  when B applies; `A notwithstanding B` makes A control over B;
  `A supersedes B` terminates B's effect over A's declared effective
  interval; `A revives B` explicitly reactivates B after the superseding
  interval ends. These relations are applied before canons.
  Remaining conflicts resolve by higher authority, then more specific rule,
  then later rule; ties are errors. Repeal alone does not revive. An override
  is valid only if the overridden declaration lists the overriding authority
  in `delegable_to`.
- `default` introduces a rule that applies when its condition is true.
  `exception ... rebuts` concludes the contrary result; `undercuts` disables
  the referenced rule without asserting its opposite. Exceptions must be
  disjoint or declare a unique integer priority. Rule kinds are `duty` (must
  act within a duration), `power` (may act and creates a decision), and
  `liability` (missed duty imposes a flag or forced transition).
- Direction is part of duration typing: `N days before` is backward counting;
  `N days after/within` is forward counting. No implicit unit conversion.
- `test` blocks execute pure assertions against declared rules and dates.
  `given` creates a typed trigger; `expect` compares computed result.
- `.case` `uses` must resolve through lock pins; `as_of` chooses law in force;
  `perspective` selects protagonist. Parties hold one declared role each.
  Events are date-sorted and replay through legal edges. Posture cannot be
  asserted. Facts select variants of fact nodes; stakes value every reachable
  terminal or say `unvalued`; beliefs are exact observation counts and lower
  to `observe`. A missed duty requires a cited waived/extended/excused event.
  Submission/posture types are stage-indexed: a filing accepted at one stage
  cannot be submitted after judgment.

## 6. Types and permitted operations

All numbers are exact integers/rationals internally. `Money<Cur>` is integral
minor units (USD cents); display literals are converted exactly. `Probability`
is a reduced rational in `[0,1]`. `Dist<E>` is a total map over every enum
variant, with nonnegative exact weights summing exactly to `1`.

| Type | Permitted operations |
|---|---|
| `Date` | equality/order; `Date + Days<U>` or `Date - Days<U>` only with an explicit counting rule; date interval membership |
| `Days<U>` | equality/order; addition/subtraction with same unit; multiply/divide by integer; explicit `counted_by` conversion among calendar/court/business days |
| `Hours` | equality/order; same-unit add/subtract; multiply/divide by integer; convert to duration only through explicit clock conversion |
| `Money<Cur>` | equality/order; same-currency add/subtract; multiply/divide by integer or rational; divide by same currency to dimensionless ratio; no currency mixing |
| `Probability` | equality/order; exact `+`, `-`, `*`, `/` when result remains `[0,1]`; `1-p`; no implicit float |
| `Dist<E>` | `probability(variant)`, exhaustive `map`, and conditioning on a nonzero event; normalization is never implicit |
| `Cite` | equality and source-span resolution only; no string concatenation |
| `Interval<Date>` | construction, equality, `contains(Date)`, intersection, disjointness; all intervals half-open |
| `Role` | equality and membership in declared roles; no arithmetic |

Strings, booleans, enums, and records support equality; records support field
projection; lists support length, indexing only with proven in-range index,
and map over a finite list. There is no IO, ambient clock, mutation, dynamic
eval, unrestricted recursion, floating point, or implicit cast. Division by
zero, overflow, invalid date, money precision loss, or out-of-range probability
is a type/evaluation error. Boolean expressions permit only `!`, `&&`, `||`,
and equality/inequality. Dates, days, hours, and money permit only the
comparisons listed in the table; `Probability` arithmetic is checked after
each operation. No other overloads or coercions exist.

## 7. Verification obligations and profiles

`litigate check` returns each named obligation with pass/fail evidence.
`well-typed` means all expressions typecheck, unit constraints hold, enums and
matches are exhaustive, and no forbidden operation exists. `cite-resolves`
means every cite maps to a normalized L0 source plus an exact source span.
`effective-disjoint` means all intervals for one logical declaration have
empty pairwise intersections. `effective-covered` means their union covers
the declared lifetime, except explicit gaps. `precedence-acyclic` is a DAG
check on declared precedence edges. `delegation-respected` means every
lower-authority override has a corresponding delegation. `exceptions-ordered`
means overlapping exceptions have unique explicit priority or are proven
disjoint. `dist-exact` means all variants occur once and rational weights sum
to one. `graph-exits` means each nonterminal has at least one active outgoing
edge. `graph-reachable` means every node is reachable from start or marked
`entry`. `deadline-satisfiable` means mandatory deadline constraints on every
reachable state have a nonempty intersection after counting/extension rules.
`case-replay` means events are ordered, legal, and timed under the selected
rule versions. `holes` reports every `?` and its reason.

`dev` permits explicit holes with L9002 warnings, permits missing cite source
resolution as warnings only for imported legacy declarations, and warns for
deadlines without tests. Structural/type errors remain errors. `strict`
rejects all holes, unresolved cites, untested deadlines/counting rules,
unvalued reachable terminals, unexplained date gaps, and every failed
obligation. Profile changes severity/acceptance, never inferred values.

## 8. Lowering contract

Lowering uses litgraph's `crates/litgraph/src/model/schema.rs` serde structs
and the field meanings in `docs/PACK_SCHEMA.md`; it must never serialize an
unrecognized approximation. Unknown/unrepresentable HIR is L6001. Source maps
and generated metadata are sidecars excluded from semantic equality.

| Source construct | Target field |
|---|---|
| `package` / `Lit.toml` | `Pack.id`, `forum`, `title`, `description`, `jurisdiction`, `schemaVersion`; package id is explicit legacy `pack_id` |
| `start` | `Pack.startNodeId` |
| `group` | `Pack.groups[]`; state group references lower to `RawNode.group` |
| `Lit.toml` roles | `Pack.roles` actor-to-`Role` mappings |
| `Lit.toml` source bindings | `Pack.sources[]` with repo-relative path or public URL, SHA-256, title, and as-of date |
| declaration id | `RawNode.id`; preserve kebab-case, no module prefix |
| state/decision/chance/terminal | `RawNode.kind`, `label`, `group`, `tags`, `outcome`, `payoff`, `attrs` |
| `entry` | node `tags: ["entry"]` |
| fact enum node | chance node `tags: ["fact"]` with authored prior on variant edges; `.case facts` forces the selected edge to probability 1 |
| role and actor | `Pack.roles`, `RawEdge.actor` |
| edge `p` | `RawEdge.probability` as finite binary64 using round-to-nearest ties-to-even; reject overflow/non-finite values |
| edge hours/cost/duration | `RawEdge.hours`, `cost`, `duration` (calendar-day triangular duration requires a single fixed/mode amount) |
| deadline ref | `RawEdge.deadline`, `Deadline.length/unit/extendable/extensionAuthority/note` |
| flags/tags/attrs | corresponding `RawEdge.sets`, `clears`, `requires`, `forbids`, `tags`, `attrs` |
| cites/rule source | `RawNode.cite`, `RawEdge.authority`, `sources[]`; normalized cites point to source bindings |
| `Lit.lock` hash | generated metadata and `provenance`; not a pack semantic field |
| authority lattice/relations | litigatable verification IR; only graph-relevant authority string is emitted to node cite/edge authority |
| counting/calendar | pack `counting` and `calendars` v3 data-driven clock IR: counting-rule id, unit/direction, trigger/day inclusion, roll set, extensions, holiday calendar, anchor, short-period rule; no forum branch in engine |
| `.case` `as_of` | compile-time rule-version selection; Scenario has no date field |
| `.case` perspective | `scenario.perspective` actor-to-litgraph-`Role` map; the named party's forum role is converted into protagonist/opponent actor assignments |
| parties/events/replayed posture | `scenario.start` (derived node id); roles and event schedule remain generated provenance/deadline sidecars, since Scenario has no event-history field |
| case facts | `scenario.facts` node-ref to edge-ref mapping |
| case stakes | `scenario.payoffs` terminal refs to major-unit JSON numbers; `unvalued` is a strict error |
| case beliefs | `scenario.observe` node refs to edge refs to numeric counts |
| case event counterfactuals | scenario `mask`, `remove_edges` only when the event/law semantics require those documented fields |
| `lit/links` package | one generated `LinkFile`; `LinkFile.links[]` holds fully qualified endpoints, `replaces`, flags, and all `RawEdge` fields; optional composition instances use `LinkFile.instances` |
| `test` | compiler verification results, never runtime pack fields |

`docs/PACK_SCHEMA.md` currently specifies schema v2 and
`model/schema.rs::Pack` has no `counting` field; implementing the v3 row is
owned by DESIGN §9. The lowerer contract is frozen here as the intended
additive v3 field. Semantic diff compares normalized JSON trees, ignores
object-key order and only the generated/source-map sidecars; arrays retain
order where the schema gives order meaning. Pack IDs map explicitly to legacy
pack IDs. Node and edge IDs stay local kebab-case; composed references use
`pack-id::local-id`.

The v3 clock IR is data-only. `Pack.counting` contains one `CountingRule` and
`Pack.calendars` contains the referenced `HolidayCalendar` declarations:
`{id, unit, direction, excludeTriggerDay, includeLastDay, rollForwardOn,
serviceExtensions, calendar, anchor, shortPeriod}`. `unit` is `calendar`,
`court`, or `business`; `direction` is `forward` or `backward`;
`rollForwardOn` is a list drawn from `weekend`, `legalHoliday`, and
`inaccessible`; `serviceExtensions` maps a service-method id to a typed day
amount; `calendar` is a `HolidayCalendar` id; `anchor` is `triggerDay` or
`nextBusinessDay`; `shortPeriod` is absent or `{threshold, unit}`. A
`HolidayCalendar` has `{id, holidays}` and each holiday is
`{id, dateRule, inForce, observed}`. `dateRule` is a fixed month/day or a
calendar expression; `inForce` is `{from,to}` with inclusive beginning and
exclusive end (null `to` means open-ended);
`observed` is `nearestWeekday`, `nextDay`, or absent. The generic clock
interpreter accepts only this IR plus a trigger/date operand and never
branches on a forum or pack id. The declarations in the source use readable
snake_case names; serialization uses the camelCase spelling shown here.

```json
{"counting":{"id":"frcp6","unit":"calendar","direction":"forward","excludeTriggerDay":true,"includeLastDay":true,"rollForwardOn":["weekend","legalHoliday","inaccessible"],"serviceExtensions":{"mail":{"amount":3,"unit":"calendar"}},"calendar":"federal-holidays","anchor":"triggerDay"},"calendars":[{"id":"federal-holidays","holidays":[{"id":"juneteenth","dateRule":"fixed(06-19)","inForce":{"from":"2021-06-17","to":null},"observed":"nearestWeekday"}]}]}
```

## 9. Diagnostics, JSON, and CLI

Diagnostic ranges are closed: L0xxx lexical/parser errors; L1xxx module,
package, and names; L2xxx type/unit errors; L3xxx temporal, authority, and
precedence; L4xxx case replay; L5xxx verification; L6xxx lowering/build;
L9xxx lints. Codes are stable and unknown codes are rejected. Current public
registry and explanations are in `litigate-base::DiagnosticCode`.

```json
{"code":"L0412","severity":"error","span":{"file":"cases/x.case","line":8,"col":3,"end_line":8,"end_col":9},"message":"...","notes":[],"fixes":[{"span":{"file":"x","line":1,"col":1,"end_line":1,"end_col":2},"replacement":"...","applicability":"machine_applicable"}]}
```

Every JSON operation returns `{ok,op,result,diagnostics,provenance,elapsed_ms}`;
`result` is absent on failure. `provenance` carries package hashes, lock
versions, source ids, and compiler version; it contains no timestamp or
absolute path. Exit codes match litgraph: `0` successful operation, `2`
source/check failure, `1` usage or I/O failure.

| Command | Arguments | Result |
|---|---|---|
| `describe` | `--format json` (default) | `{language_version, grammar, types, stdlib, diagnostic_codes, commands}` |
| `lock` | package path, `--locked` | `{lockfile, changed, package_hashes}`; `--locked` fails on drift without writing |
| `check` | paths, `--profile dev|strict`, `--format json` | `{files, profile, obligations, package_hashes}` plus diagnostics |
| `build` | package paths, `--as-of DATE`, `--known-at DATE`, `--out DIR` | `{litigatable_hash, outputs:{packs,links,scenarios,source_map}, verification}` |
| `fmt` | files/directories, `--check` | `{files:[{path,changed}]}`; formatted bytes are written only without `--check` |
| `explain CODE` | stable code | `{code, explanation, notes, fixes_guidance}` |
| `fix` | files, `--code CODE`, `--dry-run` | `{edits:[{file,span,replacement}], changed_files}` |
| `where ID` | qualified node/edge, declaration, diagnostic, or cite | `{id, declaration_span, source_span, versions, overrides, lowered_ids}` |
| `explain-value` | `--case FILE --value EXPR` | `{value, unit, counting_steps, rule_versions, overrides, citations}` |
| `diff A B` | two packages/as-of dates, `--against JSON` | `{equal, changes:[{id,field,before,after,cites}]}` |
| `query` | query kind, `--limit N` | `{matches:[{id,kind,span,cites,effective}], next_cursor}` |
| `import` | legacy `pack.json` or links/scenario JSON, `--out DIR` | `{files:[{path,source}], semantic_warnings}` plus diagnostics |
| `lsp` | `--stdio` | language-server protocol session; no JSON envelope on stdio |

`--format json` applies to every non-LSP command; human format renders the
same diagnostic contract through ariadne. Build writes deterministic files
atomically and never updates source files. `fix` only applies
machine-applicable edits unless explicitly passed `--include-maybe`.

## 10. Worked examples

The normative sample module is [`lit/examples/rule12.lit`](../../lit/examples/rule12.lit);
the matter sample is [`cases/examples/acme.case`](../../cases/examples/acme.case).
They use only constructs in this specification and are the shared parser
fixtures for W1. The sample is illustrative: unresolved probabilities are
holes and therefore fail the strict profile until authored or calibrated.
