# litigate language specification

Status: normative for compiler implementation. This document refines
[`DESIGN.md`](DESIGN.md); where the two disagree, this file wins. Language
version `1`, `Lit.lock` format `1`. Lowering targets litgraph pack schema
v2, plus the additive v3 clock IR (§8.2) once DESIGN §9.1 lands. The Rust
contracts in §11 are the code form of this document. W1 beads implement
against both and need no further design decisions; a gap found later is a
SPEC bug and gets fixed here first.

"Error" below means a diagnostic of error severity with the named code (§9.1).
Strictness is the product: when a rule here doesn't permit something, reject
it.

## 1. Compiler and dependency choices

| Crate | Owns |
|---|---|
| `litigate-base` | `FileId`, `Span`, `LineIndex`, `Diagnostic`, `Severity`, `Fix`, `Applicability`, `Note`, `DiagnosticCode`, `Envelope`, `Profile` |
| `litigate-syntax` | logos lexer, hand-written recursive-descent parser producing a rowan CST, `SyntaxKind`, typed AST wrappers, formatter |
| `litigate-sema` | salsa database, `Lit.toml`/`Lit.lock`, name resolution, type checking, version selection, HIR (`hir`) |
| `litigate-verify` | the obligations in §7 over HIR; `Obligation`, `Finding` |
| `litigate-lower` | HIR to litgraph JSON (§8), semantic diff, source maps |
| `litigate-import` | legacy pack/links/scenario JSON to `.lit`/`.case` (inverse of §8) |
| `litigate-cli` | the `litigate` binary (§9.4) |
| `litigate-lsp` | the language server over the same salsa database |

Crates: `logos` 0.15 (lexer), `rowan` 0.16 (lossless CST), `salsa` 0.26
(incremental queries), `tower-lsp` 0.20 on `tokio` (LSP), `insta` (snapshot
tests for diagnostics and outputs), `proptest` (invariants), `ariadne` 0.5
(human diagnostic rendering), `serde`/`serde_json` (all JSON), `toml` for
`Lit.toml`/`Lit.lock`, `sha2` for digests. Each is a `[workspace.dependencies]`
entry; crates use `workspace = true`. Public syntax and HIR types never expose
salsa types. The parser is hand-written (no parser generator) so that recovery
and the LL(3) decisions in §3.4 are explicit.

## 2. Lexical grammar

Input is UTF-8; a BOM or invalid UTF-8 is L0003. Line ends are `\n` or `\r\n`.
Tokens, in longest-match priority order:

```ebnf
Whitespace   = { " " | "\t" | "\r" | "\n" }- ;
LineComment  = "//", { char - "\n" } ;          (* not "///" *)
DocComment   = "///", { char - "\n" } ;
BlockComment = "/*", { char }, "*/" ;           (* non-nesting; unterminated = L0002 *)
Date         = D4, "-", D2, "-", D2 ;           (* not followed by an ident char *)
Money        = Number, ".", Currency ;
Duration     = Number, ".", ("days" | "court_days" | "business_days" | "hours") ;
Number       = Digits, [ ".", Digits ] ;
Digits       = digit, { digit | "_" } ;         (* "_" only between digits, else L0005 *)
Currency     = "usd" ;                          (* closed set in v1; other suffix = L0005 *)
String       = '"', { Escape | char - ('"' | "\\" | "\n") }, '"' ;
Escape       = "\\", ( "\\" | '"' | "n" | "r" | "t" ) ;
Ident        = IdStart, { IdContinue | "-", IdContinue } ;
IdStart      = XID_Start | "_" | { digit }-, XID_Start ;   IdContinue = XID_Continue ;
Underscore   = "_" ;                            (* a lone underscore *)
CiteText     = (* see below *) ;
Punct        = "::" | "->" | "=>" | "==" | "!=" | "<=" | ">=" | "&&" | "||" | ".."
             | "{" | "}" | "(" | ")" | "[" | "]" | ":" | "," | "." | "=" | "<" | ">"
             | "+" | "-" | "*" | "/" | "%" | "!" | "?" | "@" | "^" | "~" ;
```

- **Identifiers** may contain `-` only between two continue characters
  (`answer-due`, not `answer-`). Digits followed by a letter also start an
  identifier (`12b-motion-filed`); digits alone are a `Number`. The hyphen is greedy, so `a-b` is one
  identifier. Subtraction needs whitespace: `a - b`. When `a-b` doesn't
  resolve but `a` and `b` do, L1002 carries a machine-applicable fix to
  `a - b`.
- **Keywords are contextual.** The lexer emits `Ident` for every word. The
  parser remaps it to the keyword kind (`SyntaxKind::from_keyword`) only where
  the grammar expects that keyword. So `default`, `note`, `p` and others stay
  usable as ids. The one reserved word is `cite`. After it the lexer switches
  to citation mode: it skips spaces and tabs, then consumes `CiteText` up to,
  but not including, the first newline, `{`, `}`, `,`, or `//`. Trailing
  whitespace is trimmed and internal runs of whitespace collapse to one space.
  Empty citation text is L0001. A `cite` clause therefore ends its line or
  comes just before `{`. When the first character after the spaces is `?`,
  citation mode ends at once and the clause is a **cite hole**
  `cite ?("text")`: citation text that has no `CiteText` spelling (it
  contains `,`, `{`, `}`, `//`, or runs of whitespace) or that is not yet a
  resolvable citation is kept as the hole's reason (§8.1).
- **Typed literals.** `2_000_000.usd`, `21.days`, `5.court_days`,
  `3.business_days`, and `8.hours` are single tokens. Day amounts must be
  integers (L0005), except in a `duration` value, which is an elapsed-time
  estimate (`duration 4.5.days`); the parser enforces this, not the lexer.
  Hours may be decimal. Money may have at most the
  currency's minor-unit digits (2 for usd), else L2006. There is no rational
  token: `1/3` is a division of two numbers, folded exactly (§6). Negative
  values use unary `-`.
- **Dates** must be valid proleptic Gregorian dates (L0004).
- **Holes** are `?` or `?("reason")`.
- **Doc comments** attach to the next declaration. The joined lines (prefix
  `///` and one optional space removed) become its label when it has no
  `: "label"`. A doc comment followed by no declaration is L0006.
- Any other character is an `ErrorToken` (L0003). The CST stays lossless.

## 3. Syntax

Notation: ISO EBNF over significant tokens, with trivia between any two
tokens. Keyword strings are contextual keywords. `Ident` in declaring position
introduces a name. `Sep` is an optional comma: inside braces, items are
juxtaposed and may be separated by commas. Lists in brackets require commas
and allow a trailing comma.

### 3.1 Rule modules (`.lit`)

```ebnf
LitFile      = ModuleDecl, { ImportDecl }, [ PackageDecl ], { Item } ;
ModuleDecl   = "module", ModulePath ;
ModulePath   = Ident, { ".", Ident } ;
ImportDecl   = "import", ModulePath, [ ".", "{", Ident, { ",", Ident }, [","], "}" ] ;
PackageDecl  = "package", Ident, "{", "forum", Ident, Sep, "authority", Ident, "}" ;
Item         = StartDecl | GroupDecl | TypeDecl | EnumDecl | EventDecl | NodeDecl
             | LinkDecl | InstanceDecl | RuleDecl | DeadlineDecl | CountingDecl
             | CalendarDecl | TransitionDecl | AuthorityDecl | DefaultDecl
             | ExceptionDecl | TestDecl ;
StartDecl    = "start", Ident ;
GroupDecl    = "group", Ident, ":", String ;
TypeDecl     = "type", Ident, "=", Type ;
EnumDecl     = "enum", Ident, "{", Ident, { ",", Ident }, [","], "}" ;
EventDecl    = "event", Ident, "(", [ Param, { ",", Param }, [","] ], ")" ;
Param        = Ident, ":", Type ;

NodeDecl     = NodeKw, Ident, ":", String, { NodeClause }, [ NodeBody ] ;
NodeKw       = "state" | "decision" | "chance" | "terminal" ;
NodeClause   = "by", Ref | "discretion" | "outcome", IdList | VersionClause
             | Relation | Cite ;
NodeBody     = "{", { (NodeField | EdgeDecl), Sep }, "}" ;
NodeField    = "group", Ident | "note", String | "valence", Ident
             | "court_listener_url", String | "tags", IdList | "attrs", NumRecord
             | "payoff", (SignedMoney | Hole) | "payoff_by_flag", MoneyRecord ;
EdgeDecl     = (Ident | "_"), String, "->", Ref, { EdgeClause }, [ EdgeFields ] ;
EdgeClause   = "by", Ref | "p", ProbValue ;
ProbValue    = Number, [ "/", Number ] | Hole | "(", Expr, ")" ;
EdgeFields   = "{", { EdgeField, Sep }, "}" ;
EdgeField    = "event", CallExpr | "authority", (String | Cite) | "effort", Duration
             | "cost", (SignedMoney | Hole) | "within", Ref, [ DeadlineOpts ]
             | "deadline", Duration, [ DeadlineOpts ]
             | "duration", (Duration | "{", [ "min", Duration, Sep ], "mode", Duration,
                             [ Sep, "max", Duration ], "}")
             | "valence", Ident | "court_listener_url", String | "action_id", String
             | "note", String | "tags", IdList | "attrs", NumRecord
             | "sets", IdList | "clears", IdList | "requires", IdList
             | "forbids", IdList | "replaces", ReplaceList ;
ReplaceList  = "[", [ (Ref | String), { ",", (Ref | String) }, [","] ], "]" ;
DeadlineOpts = "{", { ("extendable", Bool | "extension_authority", String
             | "note", String), Sep }, "}" ;

LinkDecl     = "link", (Ident | "_"), String, ":", Ref, "->", Ref, { EdgeClause },
               [ EdgeFields ] ;
InstanceDecl = "instance", Ident, "@", Ident, "{", { InstanceField, Sep }, "}" ;
InstanceField= "pack", Ident | "note", String | "remove_edges", IdList
             | "probabilities", "{", { Ident, ":", ProbValue, Sep }, "}"
             | "roles", "{", { Ident, ":", Ident, Sep }, "}"
             | "payoff_transform", String | "patch", Ident, JsonRecord ;

RuleDecl     = "rule", ("duty" | "power" | "liability"), Ident, { VersionClause
             | Relation | Cite }, "{", { Item, Sep }, "}" ;
DeadlineDecl = "deadline", Ident, ":", Duration, ("from" | "after" | "before"),
               Expr, "counted_by", Ref, { VersionClause | Relation | Cite },
               [ DeadlineOpts ] ;
CountingDecl = "counting", Ident, { VersionClause | Relation | Cite },
               "{", { CountingItem, Sep }, "}" ;
CountingItem = "exclude_trigger_day" | "include_last_day" | "allow_hours"
             | "roll_forward_on", IdList
             | "service_extension", "{", { Ident, ":", Duration, Sep }, "}", [ Cite ]
             | "holidays", Ref | "anchor", Ident
             | "short_period", "{", "below", ":", Duration, Sep, "count", ":", Ident, "}" ;
CalendarDecl = "calendar", Ident, { VersionClause | Cite }, "{", { HolidayDecl, Sep }, "}" ;
HolidayDecl  = Ident, ":", CallExpr, "effective", Interval, [ "observed", Ident ] ;
TransitionDecl = "transition", Ident, "amends", Ref, { Cite },
               "{", { TransitionArm, Sep }, "}" ;
TransitionArm= "unless", Expr, "->", Ref | "otherwise", "->", Ref ;
AuthorityDecl= "authority", Ident, { ">", Ident } ;
DefaultDecl  = "default", Ident, { VersionClause | Relation | Cite },
               "when", Expr, "=>", Expr ;
ExceptionDecl= "exception", Ident, ("rebuts" | "undercuts"), Ref, { Cite },
               "when", Expr, [ "=>", Expr ], [ "priority", Number ] ;
TestDecl     = "test", String, [ Cite ], "{", { TestStmt }, "}" ;
TestStmt     = "given", Expr, "on", Date | "expect", Expr ;

VersionClause= "effective", Interval | "known_at", Date ;
Interval     = "[", Date, ",", (Date | ".."), ")" ;
Relation     = ("subject_to" | "notwithstanding" | "supersedes" | "revives"
             | "override"), Ref | "delegable_to", IdList ;
Cite         = "cite", ( CiteText | Hole ) ;
Ref          = Ident, [ "@", Ident ], { ("::" | "."), Ident }, [ "@", Date ] ;
IdList       = "[", [ Ref, { ",", Ref }, [","] ], "]" ;
NumRecord    = "{", { Ident, ":", SignedNumber, Sep }, "}" ;
MoneyRecord  = "{", { Ident, ":", SignedMoney, Sep }, "}" ;
JsonRecord   = "{", { (Ident | String), ":", JsonValue, Sep }, "}" ;
JsonValue    = SignedNumber | String | Bool | "unset" | JsonRecord
             | "[", [ JsonValue, { ",", JsonValue }, [","] ], "]" ;
SignedNumber = [ "-" ], Number ;   SignedMoney = [ "-" ], Money ;
Bool         = "true" | "false" ;
Hole         = "?", [ "(", String, ")" ] ;
Type         = Ident, [ "<", Type, { ",", Type }, ">" ] ;
```

### 3.2 Matters (`.case`)

Items appear in exactly this order. Only `uses`, `as_of`, and `summary` are
required.

```ebnf
CaseFile     = "case", Ident, UsesDecl, "as_of", Date, [ "known_at", Date ],
               "summary", String, [ "notes", String ], { SourceDecl },
               [ "start", Ref ], [ PerspectiveDecl ], [ PartiesBlock ],
               [ EventsBlock ], [ FactsBlock ], [ StakesBlock ], [ BeliefsBlock ],
               [ AnalysisBlock ] ;
UsesDecl     = "uses", UseItem, { ",", UseItem } ;
UseItem      = Ident, VersionReq ;
VersionReq   = ("^" | "~" | ">=" | "="), SemVer ;      (* tokens adjacent, no trivia *)
SourceDecl   = "source", String, [ "{", { ("url", String | "as_of", (Date | Hole)), Sep },
               "}" ] ;
PerspectiveDecl = "perspective", ( Ident | "{", { Ident, ":", Ident, Sep }, "}" ) ;
PartiesBlock = "parties", "{", { Ident, ":", Ident, Ident, Sep }, "}" ;
EventsBlock  = "events", "{", { Date, CallExpr, ("by" | "on"), Ident, [ Cite ] }, "}" ;
FactsBlock   = "facts", "{", { Ref, ":", Ref, Sep }, "}" ;
StakesBlock  = "stakes", "{", { Ref, ":", (SignedMoney | "unvalued"), Sep }, "}" ;
BeliefsBlock = "beliefs", "{", { "observe", Ref, ":", "{", { Ref, ":", Number, Sep },
               "}", Sep }, "}" ;
AnalysisBlock= "analysis", JsonRecord ;
```

A `SemVer` is `MAJOR.MINOR.PATCH[-pre]` built from adjacent `Number`, `.`,
`-`, and `Ident` tokens (`2024.12.0` lexes as `2024.12`, `.`, `0`; the parser
reassembles it). Trivia inside it is L0001.

### 3.3 Expressions

```ebnf
Expr       = MatchExpr | OrExpr ;
MatchExpr  = "match", OrExpr, "{", { Pattern, "=>", Expr, Sep }, "}" ;
Pattern    = Ref | "_" ;
OrExpr     = AndExpr, { "||", AndExpr } ;
AndExpr    = CmpExpr, { "&&", CmpExpr } ;
CmpExpr    = AddExpr, [ ("==" | "!=" | "<" | "<=" | ">" | ">="), AddExpr ] ;
AddExpr    = MulExpr, { ("+" | "-"), MulExpr } ;
MulExpr    = Unary, { ("*" | "/" | "%"), Unary } ;
Unary      = ("!" | "-"), Unary | Postfix ;
Postfix    = Primary, { ".", Ident } ;
Primary    = Literal | Hole | CiteExpr | CallExpr | Ref | RecordExpr | ListExpr
           | "(", Expr, ")" ;
CallExpr   = Ref, "(", [ Expr, { ",", Expr }, [","] ], ")" ;
RecordExpr = "{", { Ident, ":", Expr, Sep }, "}" ;
ListExpr   = "[", [ Expr, { ",", Expr }, [","] ], "]" ;
Literal    = Number | Money | Duration | String | Date | Bool | "unit" ;
CiteExpr   = Cite ;
```

Comparison is non-associative: `a < b < c` is L0001. All binary operators
are left-associative.

### 3.4 Parsing decisions and recovery

- In a `NodeBody`, an item that starts with `Ident`/`_` followed by a `String`
  and then `->` is an `EdgeDecl`. Anything else is a `NodeField`. This is the
  only LL(3) decision. Everywhere else LL(1) is enough.
- Field values are atoms, so juxtaposed items are unambiguous. Wrap a compound
  expression in parentheses.
- Recovery: on an unexpected token, emit L0001 with the expected set and
  `span` = the token. Then skip to the next token in the current item's
  follow set (`,`, `}`, or a top-level item keyword at column 1), wrapping
  the skipped tokens in an `Error` node. Each error position reports one
  diagnostic. Unclosed delimiters are L0002 at the opener, with a
  machine-applicable fix inserting the closer at end of line.
- Lowering never runs on a file with L0xxx errors. Sema runs on the
  recovered tree so the LSP still works.

## 4. Packages, modules, names

### 4.1 `Lit.toml`

```toml
[package]
name = "frcp-civil-procedure"   # kebab-case package name
version = "2024.12.0"           # SemVer
language = 1
pack_id = "frcp-civil-procedure" # explicit litgraph Pack.id; never derived
forum = "frcp"                  # Pack.forum
authority = "FederalRule"       # default authority level
title = "..."                   # Pack.title
description = "..."             # optional, Pack.description
jurisdiction = "..."            # optional, Pack.jurisdiction
in_force = "2009-12-01"         # default effective start
in_force_until = "2030-01-01"   # optional default effective end (exclusive)
known_at = "2025-01-01"         # default corpus date
counting = "frcp6"              # optional; Pack.counting (v3): a counting-rule id

[package.roles]                 # actor -> "self" | "opponent" | "nature"; Pack.roles
applicant = "self"

[dependencies]                  # name -> { version = "^x.y.z", path = "../core" }
core = { version = "^1.0.0", path = "../core" }

[sources.frcp]                  # Pack.sources[] in key order of declaration
title = "Federal Rules of Civil Procedure"
url = "https://..."             # and/or path = "sources/frcp.txt" (repo-relative)
sha256 = "..."
as_of = "2024-12-01"
cite_prefixes = ["FRCP", "Fed. R. Civ. P."]
section_heading = "Rule {n}."   # optional; locates pinpoints in `path`

[[cite_syntax.rules]]           # Pack.citeSyntax.rules[], in order (v3)
prefixes = ["FRCP", "Fed. R. Civ. P."]
heading = "Rule {}"             # exactly one {} (the section number)
source = "frcp"                 # optional; a [sources.*] key

[[cite_syntax.shorthands]]      # Pack.citeSyntax.shorthands[], in order (v3)
prefix = "cfr"
code = "cfr"                    # "usc" | "cfr"
title = 19
section_parts = 2
source = "cfr-210"              # optional
```

`counting` and `cite_syntax` mirror the pack-level fields litgraph added in
schema v3 (`docs/PACK_SCHEMA.md`, "Citation syntax (v3)"). A
`cite_syntax.rules` entry with a `source` also binds its prefixes to that
source for cite resolution (§4.3), as the source's `cite_prefixes` would,
with `heading` locating the pinpoint. A **links package** (§8.4) has no
pack identity: its manifest has only `name`, `version`, `language`,
`in_force`, `known_at`, and `[dependencies]` on every package its links
and instances reference.

Unknown keys, missing required keys (`name`, `version`, `language`, `pack_id`,
`forum`, `authority`, `title`, `in_force`, `known_at`, `package.roles`), or
ill-typed values are L1004. A `.lit` `package` declaration is optional. When
present, its name, `forum`, and `authority` must equal `Lit.toml` (L1005). A
package's modules are all `*.lit` files under the directory holding
`Lit.toml`. Module names are unique within a package (L1001) and are
independent of file paths. They never affect node ids. **Module order** is
the order of the module files' package-relative POSIX paths; it orders
`Pack.groups`, `Pack.nodes`, and edges (§8.1). Node order is observable
(Monte Carlo analyses visit nodes in order), so it is part of the source.

### 4.2 `Lit.lock` and digests

`Lit.lock` is deterministic TOML: `version = 1`, then one `[[package]]` table
per resolved package, sorted by name, with `name`, `version`, `digest`
(`sha256:<hex>`), sorted `dependencies` (`"name version"` strings), and
`language`. It has no timestamps or absolute paths. Only `litigate lock`
writes it. Any other command finds a missing or stale lock and reports L1007.
The digest is SHA-256 over `Lit.toml` followed by every package source file,
sorted by package-relative POSIX path. Each component is framed as u64 BE
path length, path bytes, u64 BE content length, content bytes. `Lit.lock`
itself is excluded and line endings are not normalized.

### 4.3 Names and resolution

- Namespaces per package: **types** (enums, type aliases, event types),
  **values** (nodes, deadlines, counting rules, calendars, defaults,
  exceptions, transitions, rules), **groups**, **authorities**, and
  **actors** (from `[package.roles]`). Edge ids are scoped to their source
  node. Holiday ids are scoped to their calendar. Duplicates in one
  namespace are L1001. Two versions of one declaration (§5.4) share a name
  and are not duplicates.
- Declarations nested in a `rule` are hoisted into the module scope.
- Lookup order for an unqualified name: the current module, then names
  imported by `import m.{x}`, then (for `import m`) `m.x` only by its
  qualified form. No shadowing is allowed: a local that collides with an
  imported name is L1001. Enum variants resolve by expected type. When the
  type is unknown, use `Enum.Variant`.
- `.` paths name modules (`frcp.rule6.computing-time`). `::` qualifies by
  **pack id or instance** (`cafc::entry`, `cafc@cofc::entry-cofc`). It is
  valid only in links packages, `.case` files, and instance fields. A
  trailing `@<date>` selects the version whose `in_force` starts on that date
  (`answer-period@2009-12-01`).
- Imports may name only modules of the same package, declared dependencies
  (L1006 otherwise), or stdlib package `core`. Import cycles are L1003.
- Each package has exactly one `start` (L1008). Every actor used by `by`, and
  the implicit `either`, must be a key of `[package.roles]` (L1009).
- Cites resolve through `cite_prefixes`: the longest matching prefix wins,
  and no match is L1010. The pinpoint's first number (`12` in
  `12(a)(1)(A)(i)`) is located in the binding's `path` file with
  `section_heading`.

## 5. Construct semantics

### 5.1 Graph

- `state`, `decision`, `chance`, `terminal` declare litgraph nodes. Every node
  needs a label, and every kind may have outgoing edges (legacy terminals
  route onward through links). `chance` is sugar: litgraph kind `state`, but
  every edge must carry `p` (L2008) and the known probabilities must sum to
  exactly 1 (`dist-exact`). Holes are allowed only in `dev`. `discretion`
  marks a tribunal-discretion draw. It is verification metadata only.
- `outcome`, `payoff`, and `payoff_by_flag` are valid only on `terminal`
  (L2007).
- An edge's actor is its own `by`, else the node's `by`, else `either`.
  `p` is allowed on any edge, with any actor. A self edge (`to` = its node)
  must not carry `p` (L2008).
- `event` binds an edge to an event pattern for case replay (§5.6). `within d`
  ties the edge to deadline declaration `d`. `deadline 30.days` is a literal
  window with no trigger semantics, kept for legacy edges.
- `_` edges have no id. Lowering omits `id` and litgraph derives
  `from->to#n`. Such edges can't be referenced (L9003 info).
- Tags `entry` (an extra root for reachability) and `fact` (a matter fact,
  allowed only on `chance`, L2007) carry meaning. All other tags are opaque.
- `group` must name a declared group (L1002).

### 5.2 Types, enums, events, match

`type`, `enum`, and `event` declare closed nominal types. `match` must be
exhaustive with no unreachable arms (L2005). There is no null, implicit
conversion, subtyping, or recursion: a type alias may not refer to itself, and
neither may a default or exception, directly or transitively (L3006). An event
pattern in a deadline trigger or edge `event` may omit trailing arguments,
which then match anything. `given`, case events, and `has_event` arguments
must be complete.

### 5.3 Deadlines, counting, calendars

`deadline d: N.unit from|after T counted_by c` counts forward from the most
recent event matching trigger pattern `T`. `before` counts backward. Units are
the `Days<U>` unit of the literal. Counting rule `c` defines (FRCP 6(a)
vocabulary):

1. Anchor: `trigger_day`, or `next_business_day` (the first day that isn't a
   weekend or calendar holiday on or after the trigger).
2. `exclude_trigger_day`: the anchor day is day 0.
3. Count `N` units in the direction: calendar days count every day. Court and
   business days skip weekends and calendar holidays. With
   `short_period { below: K.days, count: U }`, a calendar-day period with
   `N < K` counts in unit `U` instead.
4. `include_last_day`, with `roll_forward_on`: when the last day falls on a
   listed kind (`weekend`, `legal_holiday`, `inaccessible`), move one day
   in the counting direction until it doesn't (FRCP 6(a)(1)(C), 6(a)(5)).
   `inaccessible` days come from case events. In a pure `.lit` test they are
   none.
5. `service_extension { Method: K.days }` adds `K` calendar days after step 4
   for a trigger whose service-method argument is `Method`, then repeats step
   4. It applies only to forward counting (FRCP 6(d)).
6. `allow_hours` permits `N.hours` deadlines (FRCP 6(a)(2)). Without it, an
   hours deadline is L3009.

A calendar holiday uses one of these rules: `fixed(month, day)`,
`nth(n, weekday, month)` (n in 1-4), `last(weekday, month)`,
`every(years, from_year, month, day)`, or `once(date)`. Weekdays are
lowercase English. `observed nearest_weekday` moves Saturday to Friday and
Sunday to Monday. `observed next_day` moves Sunday to Monday only. A holiday
counts only inside its `effective` interval. Inconsistent counting data
(unknown anchor/roll kind/unit, duplicate method) is L3009.

### 5.4 Versions and transitions

Every rule-bearing declaration has a half-open `in_force` interval
`[from, to)` and a `known_at` corpus date. The declaration's `effective` /
`known_at` clauses win over its enclosing `rule`'s, which win over the
package defaults. Declarations with the same name are versions of one
logical declaration. Their intervals must be pairwise disjoint (L3001) and
must cover the package lifetime. A gap is legal only when declared with
`supersedes`/`revives` (L3002). A query at `(as_of, known_at)` selects the
version with `from <= as_of < to`, among versions whose `known_at <=` the
query's `known_at` (latest when omitted). No version is L3007.

Refinements (temporal bead, `litigate-sema/src/temporal`):

- **Lifetime.** A declaration's lifetime runs from its earliest version's
  `from` to the package's end (`in_force_until`, else open). A declaration
  may begin after its package does; coverage is checked inside the
  lifetime only.
- **Declared gaps.** An uncovered day is legal when a declaration that
  `supersedes` this one covers it, when the version starting right after
  the gap `revives` it, or, for the trailing gap, when the version that
  ends there carries a relation other than `delegable_to` (a sunset or a
  temporary override: it declares what it amends). Disjointness ignores
  `known_at`: two versions never restate one window.
- **Supersession.** `A supersedes B` deactivates the versions of B that
  took effect before A did, from A's `from`. When A's window ends, B stays
  inactive unless A also says `revives B` (B returns after A's window), or
  another declaration `revives B` (B returns at its `from`). A version of B
  enacted after A took effect is new law and is not suspended. `B@E
  supersedes B@D` documents succession and must name an earlier version
  (`D < E`, else L3006). `as_of` honours supersession: a superseded version
  is L3007 with a note naming the superseding declaration. Supersession and
  revival are bitemporal too: at `known_at` K only declarations recorded by
  K (their `known_at <= K`) suspend or revive.
- **Transitions.** For an event, the version in force on its date governs,
  unless that version took effect after the matter's first event and a
  transition amends it; then the arms decide. Each arm must select the
  amended version `d@D` or the version it replaces (the one ending on
  `D`), and `d@D` must replace one (L3008). One transition per amended
  version (L3008). Conditions may use only `has_event`, `true`/`false`,
  `!`, `&&`, `||`, and boolean `==`/`!=` (L3008).

`transition t amends d@D { unless C -> v, otherwise -> w }` handles
proceedings pending when version `d@D` takes effect. For each event on or
after `D` in a matter whose first event precedes `D`, the arms are evaluated
in order. The first `unless` whose condition is true selects its version,
else `otherwise` does. Exactly one `otherwise` must come last (L3008).
Conditions may use `has_event(Pattern)` (true when the matter has a matching
event on or before the evaluated one; always false outside a case). FRCP
86(a) is written this way in `lit/frcp`, never in compiler code.

### 5.5 Authority, precedence, defeasibility

`authority A > B > C` declares a strict partial order (cycles: L3006). Each
declaration's authority is its rule's or package's. When two in-force
declarations both apply to the same edge, deadline, or default target, the
winner is chosen as follows:

1. Explicit relations: `A subject_to B` (B wins), `A notwithstanding B` (A
   wins), `A supersedes B` (B is inactive during A's interval), and
   `A revives B` (B is active again after A's superseding interval ends).
   Repeal alone does not revive.
2. If still unresolved: higher authority, then the more specific declaration
   (inside a `rule` beats package level, and a narrower interval beats a
   wider one), then the later `from`. A remaining tie is L3003.
3. `A override B` with A's authority lower than B's requires B to list A's
   authority in `delegable_to` (L3004).

`default d when C => V` concludes V when C holds. `exception e rebuts d when C
=> V'` concludes V' instead. `exception e undercuts d when C` disables d
without concluding anything; it must have no `=>` (L2007). Exceptions on the
same target must be provably disjoint (their conditions can't both be true
for any assignment of the `has_event` atoms they mention) or carry distinct
`priority` (larger wins) (L3005). Rule kinds: `duty` requires each contained
deadline to be met (a case event missing it is L4002 unless excused). `power`
contains the decisions it grants. `liability` contains the consequence edges
taken when a duty is missed.

Refinements (precedence bead, `litigate-sema/src/precedence`):

- **Lattice.** The order is the transitive closure of every `authority`
  chain in the package and its dependencies (`lit/core` declares the
  shared one). A level no chain names is incomparable to every other.
- **Claimants.** The declarations that "apply to the same target" T are T
  itself and every declaration with `override T`, `notwithstanding T`, or
  `subject_to T`. On each day, the active claimant that beats every other
  active claimant wins. Stage 1: `notwithstanding` and `override` win,
  `subject_to` yields; a `@D` target applies only while version D is
  active. Stage 2, for pairs stage 1 leaves open: authority, then inside a
  `rule` over package level, then the shorter window (open-ended is
  longest), then the later `from`. L3003 is reported once per unordered
  pair, at the later declaration's relation clause with a note at the
  other. A pair whose declared relations contradict each other is L3006
  only, not also L3003. A declaration is inside a `rule` when the rule's
  source span contains it. Each package is checked with its dependencies;
  claimants in two packages that do not depend on each other meet only
  where a matter `uses` both, and are resolved there with the same order.
- **Cycles (L3006).** Declared wins (`notwithstanding`/`override` edges
  A→B, `subject_to` B→A), supersession between different declarations, and
  default/exception references (an exception to its target; a default or
  exception to every default or exception its expressions name) must each
  be acyclic.
- **Delegation.** `A override B` needs no delegation when A's authority
  equals or outranks B's. Otherwise every version of B that overlaps A's
  window must list A's authority, exactly, in `delegable_to` (L3004 at the
  `override` clause, naming the missing `delegable_to [Level]`, with a
  `maybe_incorrect` fix inserting it into B). `delegable_to X` must name a
  level strictly below the declaration's own (L3004). An override of a
  declaration never in force during the override's window is L3007, as is
  an exception never in force alongside its target.
- **Evaluation.** An exception applies only while its target's condition
  holds. Among an applicable node's applicable, undefeated exceptions the
  highest `priority` wins (an absent priority ranks lowest). A winning
  rebutting exception replaces the conclusion; a winning undercutting one
  disables its target, and a disabled exception has no effect on its own
  target.
- **Rule shapes (L2007).** A `duty` contains at least one deadline. A
  `power` contains at least one `decision`. A `liability` contains at least
  one node and no deadline.
- **Direction.** A deadline's direction is part of its type. Every version
  shares it (L2001), `within d` needs a forward `d` (L2001), and a backward
  deadline cannot use a counting rule anchored on `next_business_day`
  (L3009).

### 5.6 Matters (`.case`)

- `uses` pins resolve through `Lit.lock`. The first package is primary:
  unqualified refs resolve in it.
- `as_of` selects law in force (§5.4). It is compile-time only.
- `parties { id: Actor EntityType }`: the actor must be declared by the
  primary package (L4005). Party ids are unique.
- `perspective party` makes that party's actor `self`, every other party's
  actor `opponent`, and leaves remaining actors at the package roles.
  `perspective { actor: role }` sets roles explicitly. Unknown party or actor
  is L4005.
- Replay: begin at `start` if given, else the package start. Process events
  sorted by date. Ties keep source order, and out-of-order source is L4001
  with a fix. Before each event, while the current node is a `fact` chance
  node with a `facts` entry, follow that edge. An event that matches exactly
  one outgoing edge's `event` pattern takes it. An event that matches none
  but is the trigger pattern of an in-force deadline is timing-only. Anything
  else, or an ambiguous match, is L4001. An event taken through an edge with
  `within d` must fall on or before d's due date, computed from the latest
  matching trigger. Otherwise L4002, unless a prior event carries a `cite`
  and matches a `waived`/`extended`/`excused` event type declared by the
  package. The replayed node becomes the scenario `start`. An explicit
  `start` that disagrees with the replayed node is L4008.
- `facts { node: edge }`: the node must be a `fact` chance node and the edge
  must be its own (L4003).
- `stakes`: every terminal reachable from the replayed start needs a value
  or `unvalued` (L4004). `unvalued` is an error in `strict`.
- `beliefs { observe node: { edge: count } }`: node must be a chance node, edges
  its own (L4006).
- `analysis`: litgraph `Scenario` fields that have no dedicated block (§8.3).
- Posture is replayed, never asserted.

## 6. Types and permitted operations

All arithmetic is exact over i64 numerators/u64 denominators. Overflow,
division by zero, an invalid date, money precision loss, or a probability
outside `[0,1]` is L2006. `Money<Cur>` is integral minor units.
`Probability` is a reduced rational in `[0,1]`. `Dist<E>` is a total map over
the variants of enum `E` with exact non-negative weights summing to 1 (L2003).

| Type | Literal | Permitted operations |
|---|---|---|
| `Date` | `2025-03-03` | `== != < <= > >=`; `in(Interval<Date>)` |
| `Days<U>` (`U` = `CalendarDays`, `CourtDays`, `BusinessDays`) | `21.days`, `5.court_days`, `3.business_days` | comparisons; `+ -` with the same `U`; `* /` by `Number` with integral result; no unit conversion (L2004) |
| `Hours` | `8.hours` | comparisons; `+ -` with `Hours`; `* /` by `Number` |
| `Money<Cur>` | `2_000.usd` | comparisons; `+ -` same currency (L2004); `* /` by `Number` (exact, else L2006); `Money / Money` gives `Number` |
| `Probability` | `p` values, `Number` in `[0,1]` | comparisons; `+ - * /` checked to stay in `[0,1]`; `1 - p` |
| `Number` | `21`, `0.35`, `1/3` | comparisons; `+ - * / %` (`%` integers only) |
| `Dist<E>` | record `{ variant: p, ... }` | `prob(d, Variant)`; `given(d, [Variants])` conditions on a nonzero-mass subset |
| `Cite` | `cite ...` | `==` only |
| `Interval<Date>` | `[d, d)` in clauses | `in`, `intersect`, `disjoint` |
| `Role` | actor names | `==` only |
| `Bool` | `true`, `false` | `! && \|\| == !=` |
| `String`, enums, records | | `==`, `!=`; record field projection `r.f` |
| `List<T>` | `[a, b]` | `len(l)`, `contains(l, x)` |

The builtins (closed list) are `deadline(d) -> Date` (tests: due date from the
`given` events), `has_event(Pattern) -> Bool`, `prob`, `given`, `in`,
`intersect`, `disjoint`, `len`, and `contains`, plus the event constructors
from `event` declarations. There is no IO, ambient clock, mutation, float,
implicit cast, or overloading beyond this table (L2002 otherwise). Field
types: `p` is `Probability`, `effort` is `Hours`, `cost`/`payoff` are
`Money<usd>`, `duration` is `Days<CalendarDays>` (`min <= mode <= max`, decimal amounts allowed),
`deadline` is `Days<CalendarDays|CourtDays>`, and attrs are `Number`.

## 7. Verification obligations and profiles

`litigate check` reports every obligation as a `Finding`
`{obligation, passed, detail}`. Each failure also emits the listed code.

| Obligation | Definition | Code |
|---|---|---|
| `well-typed` | every expression and field typechecks, matches are exhaustive, no forbidden operation | L2xxx |
| `cite-resolves` | every cite resolves to a binding and, when the binding has `path`, to the pinpoint's heading span | L1010 |
| `effective-disjoint` | versions of each declaration are pairwise disjoint | L3001 |
| `effective-covered` | versions cover the package lifetime except declared gaps | L3002 |
| `precedence-acyclic` | authority order, relations, and default/exception references are acyclic | L3006 |
| `delegation-respected` | every lower-authority `override` is delegated | L3004 |
| `exceptions-ordered` | exceptions per target are disjoint or prioritized | L3005 |
| `dist-exact` | every chance node's and `Dist` value's known probabilities sum to exactly 1 and cover all variants | L2003 |
| `graph-exits` | every non-terminal node has at least one outgoing edge after `requires`/`forbids` are ignored | L5004 |
| `graph-reachable` | every node is reachable from `start` or an `entry`-tagged node | L5003 |
| `deadline-satisfiable` | for each reachable node, the due-date windows of mandatory (`duty`) deadlines on its outgoing edges intersect for some trigger date | L5005 |
| `tests-pass` | every `test` assertion holds | L5006 |
| `case-replay` | events order, match, and meet deadlines (§5.6) | L4001-L4008 |
| `holes` | lists every hole with reason and span | L9002 / L5002 |

Profiles change severity and acceptance, never values. `dev`: holes are L9002
warnings. A failed obligation is an error. Exceptions: `cite-resolves` on
declarations that `litigate import` marks `/// imported`, and missing tests
(L9001), are warnings. `strict` rejects every hole (L5002), every
`cite-resolves` failure, every untested deadline or counting rule (L9001 as
an error), `unvalued` stakes, and non-canonical formatting (L9004).
`litigate check --profile strict` passing on `lit/core` is the epic gate.

## 8. Lowering contract

Lowering produces values of litgraph's own serde types
(`crates/litgraph/src/model/schema.rs`, `model/links.rs`,
`scenario/library.rs`) and serializes them. It never writes JSON by hand.
Anything without a lowering is L6001.

### 8.1 Package to `Pack`

| Source | Target |
|---|---|
| `Lit.toml` `pack_id`, `title`, `description`, `jurisdiction`, `forum` | `Pack.id`, `title`, `description`, `jurisdiction`, `forum` |
| (derived) | `Pack.schemaVersion` = 3 if the package has a counting rule (manifest `counting`, or counting/calendar declarations), else 2 |
| manifest `counting` | `Pack.counting` (the counting-rule id, verbatim) |
| manifest `[[cite_syntax.*]]` | `Pack.citeSyntax.rules[]`/`shorthands[]` in order (`section_parts` is `sectionParts`) |
| `start` | `Pack.startNodeId` |
| `group` decls (module order, then source order) | `Pack.groups[]` |
| `[package.roles]` | `Pack.roles` |
| `[sources.*]` (declared order) | `Pack.sources[]` `{id, title, url, path, sha256, asOf}`; `cite_prefixes` and `section_heading` aren't emitted |
| node declarations (module order, then source order) | `Pack.nodes[]` |
| node id, label, cite | `RawNode.id` (kebab-case, no module prefix), `label`, `cite` (a cite hole lowers to its reason, below) |
| `state`/`chance` / `decision` / `terminal` | `RawNode.kind` `state` / `decision` / `terminal` |
| node `group`, `note`, `valence`, `court_listener_url`, `tags`, `attrs` | same-named `RawNode` fields (`courtListenerUrl`) |
| `outcome`, `payoff`, `payoff_by_flag` | `RawNode.outcome`, `payoff`, `payoffByFlag` (major units) |
| edge, in node order then edge order | `Pack.edges[]`: `from` = enclosing node, `id` (omitted for `_`), `to`, `label`, `actor` |
| `p`, `effort`, `cost` | `RawEdge.probability`, `hours`, `cost` |
| `authority` | `RawEdge.authority` (string, cite text, or cite-hole reason) |
| `within d` | `RawEdge.deadline` `{length: N, unit, extendable, extensionAuthority, note}` from d's version in force, edge options winning |
| `deadline N.unit {…}` | same, from the literal; `unit` omitted for calendar days, `"court"` for court days; business days are L6001 |
| `duration` | `RawEdge.duration {min, mode, max}` (a single duration is `{mode}`) |
| `valence`, `court_listener_url`, `action_id`, `note`, `tags`, `attrs`, `sets`, `clears`, `requires`, `forbids` | same-named `RawEdge` fields |
| `counting` / `calendar` | `Pack.counting` / `Pack.calendars` (§8.2) |
| `event`, authority lattice, relations, defaults, exceptions, transitions, tests, types | not emitted (verification only) |

Multiple versions: only the version selected by the build's `--as-of`/
`--known-at` is emitted. The default is the package `known_at` date for both.

A cite hole `cite ?("text")` lowers to `"text"` exactly, so imported
citations stay lossless; a cite hole without a reason lowers to an absent
field. Like every hole it is a warning in `dev` and an error in `strict`
(§7), so the porter replaces it with a resolvable citation.

### 8.2 Clock IR (pack schema v3, additive)

As landed by the engine bead (DESIGN §9.1), `Pack.counting` is a string:
the id of a counting rule in litgraph's shared *clock library*
(`packs/clock/*.json`, `docs/PACK_SCHEMA.md` "Clock library"), which holds
the rules and holiday calendars as step lists. The manifest's `counting`
key (§4.1) lowers to that id verbatim, which is all `litigate import`
needs. Lowering `counting`/`calendar` declarations to the clock library is
owned by the lowering bead; the sketch below predates the landed IR and is
reconciled there.

`Pack.counting` holds one `CountingRule` (the one named by the package's
deadlines; more than one distinct rule per pack is L6001).
`Pack.calendars` holds the calendars it references.

```json
{"counting":{"id":"frcp6","excludeTriggerDay":true,"includeLastDay":true,
  "rollForwardOn":["weekend","legalHoliday","inaccessible"],
  "serviceExtensions":{"Mail":{"amount":3,"unit":"calendar"}},
  "calendar":"federal-holidays","anchor":"triggerDay","shortPeriod":null,
  "allowHours":true},
 "calendars":[{"id":"federal-holidays","holidays":[
  {"id":"juneteenth","date":{"rule":"fixed","month":6,"day":19},
   "inForce":{"from":"2021-06-17","to":null},"observed":"nearestWeekday"}]}]}
```

`unit` is `calendar` | `court` | `business`. `anchor` is `triggerDay` |
`nextBusinessDay`. `shortPeriod` is `null` or `{threshold, unit}`. `date` is
one of `{"rule":"fixed",month,day}`, `{"rule":"nth",n,weekday,month}`,
`{"rule":"last",weekday,month}`, `{"rule":"every",years,fromYear,month,day}`,
or `{"rule":"once",date}`. `observed` is `nearestWeekday`, `nextDay`, or
absent. Direction and unit belong to each deadline query, not to the rule
(the engine already takes direction per query). The roll moves in the
counting direction. The generic interpreter consumes only this IR plus
`(trigger date, amount, unit, direction, service method, inaccessible dates)`
and never branches on a forum or pack id. The four current regimes (FRCP 6,
RCFC 6, FRAP 26, 19 CFR 210.6) must be expressible: RCFC adds
`inauguration-day: every(4, 1965, 1, 20) observed next_day`. FRAP 26(c)
service methods differ only in `service_extension` keys. ITC uses
`anchor next_business_day` and `short_period { below: 7.days, count:
business_days }`. Engine bead DESIGN §9.1 implements the interpreter
against this shape.

### 8.3 Case to `NamedScenario`

| Source | Target |
|---|---|
| case id, `summary`, `notes`, `source`s | `NamedScenario.id`, `summary`, `notes` (empty string when absent), `sources[] {title, url, as_of}`; an `as_of` hole lowers to its reason (`?("2026-06")` is `"2026-06"`) |
| `uses` | `packs[]`: pack ids in `uses` order |
| replayed/explicit start | `scenario.start` (qualified `pack::node`), omitted when it is the primary package's start and no events were replayed |
| `perspective` | `scenario.perspective` actor-to-role map (only entries that differ from the primary package's roles) |
| `facts` | `scenario.facts` qualified node to qualified edge |
| `stakes` | `scenario.payoffs` qualified terminal to major-unit number |
| `beliefs` | `scenario.observe` qualified node to `{qualified edge: count}` |
| `analysis { f: v }` | `scenario.f` = JSON of `v` (numbers exact to binary64; `unset` = JSON null) |
| `as_of`, `known_at`, parties, events | not emitted; recorded in the provenance sidecar |

`analysis` may contain any `Scenario` field except `start`, `perspective`,
`facts`, `payoffs`, and `observe`, which have dedicated blocks (L4007). The
lowerer assembles the object and deserializes it into
`litgraph::scenario::Scenario` (`deny_unknown_fields`). A serde error is
L4007 with the serde message as a note. Expression-valued fields (`mask`,
`probability_fn`, `opponent_objective`, `metrics`, `utilities`, `cost`,
`utility`) are parsed with litgraph's expression parser at lowering, and a
parse error is L6003.

### 8.4 Links package to `LinkFile`

A package containing `link`/`instance` declarations is a links package. It
must contain no nodes, and lowers to one `LinkFile`. A `link id "label": A ->
B` becomes a `links[]` `RawEdge` with qualified `from`/`to` and every
`EdgeField` as in §8.1, plus `replaces`: a `Ref` item lowers to its text,
and a `String` item verbatim (it names an anonymous edge by its derived
id, `"pack::from->to#n"`, which has no `Ref` spelling). Each `instance base@origin` becomes
`instances["base@origin"]` `{pack, note, remove_edges, probabilities, roles,
payoff_transform, patch_edges}`. `patch` records become `patch_edges[edge]`
verbatim as JSON merge patches (keys are `RawEdge` JSON names, `unset` is
null). The lowerer applies each patch to the base edge and deserializes the
result as `RawEdge` (L6001 on failure). `replaces` outside a links package is
L2007.

### 8.5 Numbers

Exact rationals become binary64 by round-to-nearest-ties-even. A value that
can't be represented as a finite binary64 is L6004. Money becomes major units
(`minor / 100` for usd). The importer reads each JSON number's decimal text
(never an already-rounded f64) into an exact rational, so import then lower is
the identity on every finite JSON number.

### 8.6 Semantic equality and determinism

`litigate diff` and `lit-sync` compare generated output with the expected JSON
by deserializing both into the litgraph serde types (which applies serde
defaults) and normalizing:

- absent `RawNode.kind` is `state`
- absent `Deadline.unit` is `calendar`
- absent edge `id` is litgraph's derived `from->to#n` (`local_edge_ids`)
- empty and absent collections are equal
- `Pack.edges` is stably ordered by the position of each edge's `from` in
  `Pack.nodes`: source groups edges under their node, so only the order
  within one source node is significant (it fixes derived ids and the
  engine's per-node action order; the whole test suite is unchanged under
  this regrouping)

Arrays are then compared in order, objects by key, and numbers by binary64
bit equality. Output is byte-deterministic: two-space-indented JSON with a
trailing newline, written atomically. Source maps (`*.map.json`: generated
JSON pointer to source span) and provenance are sidecars excluded from
equality.

### 8.7 Import (§8 read in reverse)

`litigate import` (crate `litigate-import`) turns legacy JSON into source
that lowers back to semantically equal JSON (§8.6). A value with no exact
spelling is L6002 at the input file, with its JSON pointer as a note; the
importer never approximates. Its output is canonical `fmt` output that
parses with no diagnostics, and it follows these conventions:

- A pack becomes `lit/<dir>/` where `<dir>` is the pack's `forum` key (the
  pack id when another imported pack shares the forum): `Lit.toml`, a root
  module `<dir>.lit` (module `<dir>`) holding the `group` declarations, and
  section modules `NN-<name>.lit` (module `<dir>.<name>`). A section is a
  contiguous run of nodes with one group, named after it without a
  `-group` suffix; runs whose nodes reach each other both ways are merged
  with every run between them (`<first>-to-<last>`), so imports stay
  acyclic (L1003). `NN` follows node order, so module order (§4.1)
  reproduces `Pack.nodes`. Each section imports the groups and the
  cross-section edge targets it uses, and the one holding the start node
  declares `start`.
- `Lit.toml`: `version = "0.1.0"`; `in_force = "0001-01-01"` and
  `authority = "Unclassified"` are explicit placeholders (legacy JSON
  records neither); `known_at` is the newest `sources[].asOf`.
- Every node and link carries the `/// imported` marker (§7). Edges nest under
  their source node in pack order. A node's `by` is its edges' most common
  actor; an edge repeats `by` only when it differs. A `state` whose every
  out-edge carries `p` and none loops back is written `chance`.
- A `cite`/`authority` string is citation syntax when it has a `CiteText`
  spelling and litgraph's citation parser recognizes every citation in it
  under the pack's `citeSyntax`; otherwise it is a cite hole carrying the
  original text. Notes stay `note` fields (they lower to `note`).
- `links.json` becomes the links package `lit/links/` (one module `links`).
- A named scenario becomes `cases/<id>.case`: `uses` every pack at
  `^0.1.0`, `as_of` the primary package's `known_at`, an explicit `start`
  (legacy scenarios assert posture; the porter replaces it with `events`),
  and every `Scenario` field without a dedicated block under `analysis`.

## 9. Diagnostics, JSON, CLI

### 9.1 Codes and severities

Ranges: L0xxx syntax, L1xxx names/modules/packages, L2xxx types, L3xxx
temporal/authority/precedence, L4xxx case, L5xxx verification, L6xxx
lowering/build, L9xxx lints. The closed registry is
`litigate_base::DiagnosticCode` (`ALL`, `as_str`, `explain`, `parse`,
`severity`). A new code is a SPEC change plus a registry change, never ad
hoc. Codes are never reused.

| Code | Name | Code | Name |
|---|---|---|---|
| L0001 | unexpected token | L3001 | effective overlap |
| L0002 | unterminated literal | L3002 | effective gap |
| L0003 | invalid token | L3003 | unresolved precedence |
| L0004 | invalid date | L3004 | missing delegation |
| L0005 | invalid number/suffix | L3005 | exception conflict |
| L0006 | dangling doc comment | L3006 | precedence cycle |
| L1001 | duplicate name | L3007 | no version in force |
| L1002 | unresolved name | L3008 | invalid transition |
| L1003 | import cycle | L3009 | invalid counting |
| L1004 | invalid manifest | L4001 | invalid case event |
| L1005 | package mismatch | L4002 | deadline missed |
| L1006 | undeclared dependency | L4003 | invalid case fact |
| L1007 | stale lock | L4004 | missing stake |
| L1008 | invalid start | L4005 | invalid party/perspective |
| L1009 | undeclared actor | L4006 | invalid belief |
| L1010 | unbound cite | L4007 | invalid analysis |
| L2001 | type mismatch | L4008 | start conflict |
| L2002 | invalid operation | L5001 | verification failed (generic) |
| L2003 | invalid distribution | L5002 | hole in strict |
| L2004 | unit mismatch | L5003 | unreachable node |
| L2005 | non-exhaustive match | L5004 | no exit |
| L2006 | arithmetic error | L5005 | deadline unsatisfiable |
| L2007 | invalid field | L5006 | test failed |
| L2008 | invalid probability | L6001 | lowering unsupported |
| L9001 | rule without test | L6002 | semantic diff |
| L9002 | hole present | L6003 | invalid engine expression |
| L9003 | anonymous edge | L6004 | non-finite number |
| L9004 | non-canonical format | L6005 | output error |
| L9005 | unused declaration | | |

Severity (`DiagnosticCode::severity(profile)`): every non-L9 code is
`error`. L9003 is `info`. L9005 is `warning`. L9001, L9002, and L9004 are
`warning` in `dev` and `error` in `strict`.

### 9.2 Shapes

```json
{"code":"L0001","severity":"error",
 "span":{"file":"cases/x.case","line":8,"col":3,"end_line":8,"end_col":9},
 "message":"expected `->`, found `{`",
 "notes":[{"message":"edges are written `id \"label\" -> target`","span":null}],
 "fixes":[{"span":{"file":"cases/x.case","line":8,"col":3,"end_line":8,"end_col":3},
           "replacement":"-> ","applicability":"machine_applicable"}]}
```

`file` is the repository-relative `/` path. Lines and columns are one-based.
Columns count Unicode scalars and the end is exclusive. The LSP converts to
UTF-16 through `LineIndex`. `severity` is `error|warning|info|hint`.
`applicability` is `machine_applicable|maybe_incorrect|has_placeholders|
unspecified`. Diagnostics are sorted by `(file, line, col, code)`.

Every JSON response is an envelope
`{ok, op, result, diagnostics, provenance, elapsed_ms}`. `ok` is false iff any
error-severity diagnostic was produced, and `result` is then absent.
`provenance` is `{compiler_version, language_version, packages:[{name,
version, digest}], as_of, known_at}`, with no timestamps or absolute paths.
Exit codes: `0` ok, `2` source/check failure (`ok: false`), `1` usage or I/O
failure.

### 9.3 CLI (`litigate`)

All commands accept `--format json|human` (default `human` on a TTY, else
`json`). Human output renders the same diagnostics with ariadne.

| Command | Arguments | `result` |
|---|---|---|
| `describe` | | `{language_version, grammar, types, builtins, diagnostic_codes:[{code, explain}], commands}` |
| `lock` | `[PKG_DIR]`, `--locked` | `{lockfile, changed, packages:[{name,version,digest}]}`; `--locked` fails (L1007) instead of writing |
| `check` | `PATHS...`, `--profile dev\|strict` (default `dev`), `--as-of`, `--known-at` | `{profile, files, obligations:[Finding], packages}` |
| `build` | `PKG_OR_CASE...`, `--as-of`, `--known-at`, `--out DIR` | `{outputs:{packs:[path], links:[path], scenarios:[path], source_maps:[path]}, litigatable_hash, obligations}` |
| `fmt` | `PATHS...`, `--check` | `{files:[{path, changed}]}`; `--check` reports L9004 instead of writing |
| `explain` | `CODE` | `{code, explanation, severity:{dev, strict}}` |
| `fix` | `PATHS...`, `--code CODE`, `--dry-run`, `--include-maybe` | `{edits:[{span, replacement}], changed_files}`; only machine-applicable fixes without `--include-maybe` |
| `where` | `ID` (qualified node/edge/declaration, code, or cite) | `{id, kind, declaration_span, versions:[{in_force, known_at, span}], lowered:[json_pointer]}` |
| `explain-value` | `--case FILE`, `--value EXPR` | `{value, type, steps:[{rule, cite, detail}], versions, overrides}` |
| `diff` | `A B` (generated or expected JSON files or package dirs), `--as-of` | `{equal, changes:[{pointer, before, after}]}` per §8.6 |
| `query` | `KIND` (`nodes\|edges\|deadlines\|holes\|cites\|versions`), `--match TEXT`, `--limit N`, `--cursor C` | `{matches:[{id, kind, span, cite, in_force}], next_cursor}` |
| `import` | `JSON...`, `--out DIR` | `{files:[{path, from}], warnings}`; output must round-trip under §8.6 (L6002 otherwise) |
| `lsp` | `--stdio` | LSP session (no envelope) |

`justfile` recipes: `lit-check` runs `litigate check --profile strict lit cases`.
`lit-sync` runs `litigate build` into a temp dir then `litigate diff` against
`packs/` and `scenarios/`. `lit-test` runs the litigate crates' tests. `just
ci` gains `lit-sync` at the flip (W3).

## 10. Canonical formatting (`litigate fmt`)

- Two-space indentation, LF line ends, no trailing whitespace, and one final
  newline.
- Header order: `module`, imports (sorted), `package`, then items in source
  order. Items are separated by exactly one blank line, except that
  consecutive single-line items of the same kind are contiguous. Doc
  comments don't count toward an item's line count.
- `.case` items go one per line in the §3.2 order, with a blank line before
  and after each multi-line block.
- Braced bodies: on one line `{ a, b }` when the whole item fits in 100
  columns. Otherwise one item per line, no commas, with the closing brace on
  its own line. A `cite` clause followed by a body puts `{` on the next line.
- One space around binary operators, `->`, `=>`, and after `:` and `,`. None
  inside brackets or parentheses.
- `parties` and `events` blocks always put one entry per line, even when
  they would fit on one.
- A `cite` clause runs to the end of its line, so any clause after it
  continues on the next line indented two more columns.
- Numbers keep their source digit grouping. Comments and doc comments are
  preserved verbatim. A comment on its own line attaches to the next item or
  entry. A comment after a token on the same line stays there, and a line
  comment always ends its line. Imports are sorted by module path, and
  separator commas are dropped from broken bodies (kept where required:
  `enum`, `import m.{...}`). Binary operators inside expressions are
  spaced, including `/`; the `p n/d` probability ratio is not.
- `fmt` is idempotent (proptest over the corpus) and never changes the CST
  modulo trivia.

## 11. Rust contracts

| Contract | Location |
|---|---|
| spans, diagnostics, codes, severities, envelope, profiles | `crates/litigate-base` |
| token/node kinds, contextual keywords | `crates/litigate-syntax/src/syntax_kind.rs` |
| typed CST views | `crates/litigate-syntax/src/ast.rs` |
| HIR (`Package`, `Manifest`, `Module`, `Declaration`, `Node`, `Edge`, `Case`, ...) | `crates/litigate-sema/src/hir.rs` and `hir/` |
| obligations | `crates/litigate-verify/src/lib.rs` |

W1 added `SyntaxKind`s `ImportList`, `Param`, `FieldExpr`, `Ratio`,
`KwUrl`, and the lookahead sentinel `Eof` (never stored in a tree), plus
typed views for nodes that previously had none. The `Expr` view also casts
`FieldExpr` and `Ratio`. All existing kinds and casts are unchanged.

The import bead added, with no new kinds: a `CiteExpr` may hold a
`HoleExpr` (`Cite::hole`), an `IdList` under `replaces` may hold string
`Literal`s (`IdList::strings`), a source `as_of` may be a `HoleExpr`, and
the forms in §2-§3 above (digit-led identifiers, decimal `duration` days,
optional `min`/`max`). The tree-sitter grammar matches.

HIR is plain serde data with every name resolved, every literal exact, and
JSON shape frozen by an insta snapshot. Changing any of these is a SPEC
change made by the owning bead.

## 12. Worked examples

[`lit/examples/Lit.toml`](../../lit/examples/Lit.toml) +
[`lit/examples/rule12.lit`](../../lit/examples/rule12.lit) form a
self-contained package. [`cases/examples/acme.case`](../../cases/examples/acme.case)
is a matter against it. Together they use every top-level construct except
links, instances, transitions, defaults, and exceptions, and they are the
shared W1 fixtures. Expected results: both parse with no diagnostics. Under
`dev`, check reports L9002 for the three holes and L9003 for the two `_` edges.
Under `strict`, check fails with L5002 for each hole. Replaying `acme.case`
starts at `time-barred`, follows the `not-barred` fact to `respond`, treats
`Served` as timing-only (due 2025-03-26), and takes `move-dismiss` on
2025-03-20, so the scenario `start` is `frcp-rule12-example::mtd-ruling`.
The two tests in `rule12.lit` pass (2025-03-24; 2025-03-27 with mail
service).
