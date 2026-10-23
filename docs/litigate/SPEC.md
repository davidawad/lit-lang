# litigate language specification

Status: normative for compiler implementation. This document refines
[`DESIGN.md`](DESIGN.md); where the two disagree, this file wins. Language
version `1`, `Lit.lock` format `1`. Lowering targets litgraph pack schema
v2, plus the additive v3 fields (`counting`, `citeSyntax`) and the engine's
landed clock library (§8.2). The Rust
contracts in §11 are the code form of this document. W1 beads implement
against both and need no further design decisions; a gap found later is a
SPEC bug and gets fixed here first.

## 0. Paradigm (ADOPTED)

Status: ADOPTED 2026-10-23, decided by the PM (dotfiles-3a) under David's
delegation, from the pre-freeze design review
([`research/design-review-2026-10-23.md`](research/design-review-2026-10-23.md)
§0, §1.2). Normative: this section governs the language's design. Where §1
onward still describes the language as built and conflicts with it, that text
is migrating to this paradigm through the `lg-flu` and `lg-962` beads, and
new work follows this section.

1. **Core:** a typed, stratified Datalog with priorities over bitemporal facts.
   Each declaration denotes a set of claims (declaration id, version, valid
   interval, transaction interval, applicability, authority, scope, value,
   relations, cite). Resolution is one fixed query over the union of claims.
   Layers merge as a set, and precedence comes from the instruments
   themselves; an incomparable conflict is an error, or a chance fork where
   the law is unsettled.
2. **Values:** a first-order, non-recursive expression language. Counting is
   a closed algebra of total combinators (anchor, count, roll, add, chain,
   clerk), not user code.
3. **Procedure:** a typed, bounded Petri net with discrete clocks. A posture
   is a marking plus open obligations; a `.case` is a firing sequence.
4. **Types:** ground-indexed and capability-typed first. SMT (QF_LIA/LRA)
   only for parameterised values and delegation bounds.

Not chosen: ASP as the core (it allows zero or several answer sets; clingo
stays an optional oracle for world enumeration), a general functional
language, or a dependent type theory.

"Error" below means a diagnostic of error severity with the named code (§9.1).
Strictness is the product: when a rule here doesn't permit something, reject
it.

## 1. Compiler and dependency choices

| Crate | Owns |
|---|---|
| `litigate-base` | `FileId`, `Span`, `LineIndex`, `Diagnostic`, `Severity`, `Fix`, `Edit`, `Label`, `Note`, `Derivation`, `WorldTag`, `Applicability`, `DiagnosticCode`, `Envelope`, `Profile` |
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
Duration     = Number, ".", ("days" | "court_days" | "business_days" | "weeks"
               | "months" | "years" | "hours") ;
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
  usable as ids. The one reserved keyword is `cite`. After it the lexer switches
  to citation mode: it skips spaces and tabs, then consumes `CiteText` up to,
  but not including, the first newline, `{`, `}`, `,`, or `//`. Trailing
  whitespace is trimmed and internal runs of whitespace collapse to one space.
  Empty citation text is L0001. A `cite` clause therefore ends its line or
  comes just before `{`. When the first character after the spaces is `?`,
  citation mode ends at once and the clause is a **cite hole**
  `cite ?("text")`: citation text that has no `CiteText` spelling (it
  contains `,`, `{`, `}`, `//`, or runs of whitespace) or that is not yet a
  resolvable citation is kept as the hole's reason (§8.1).
- **Reserved words.** `world`, `layer`, `precedent`, `overrule`,
  `delegable`, `lock`, `grant` and `characterization` are reserved for
  constructs a later edition adds (worlds and layers, precedent effects,
  delegation grants and locks). They lex as `Ident` but are L0007 wherever
  they appear, so no source written today collides with that syntax. Only
  the whole word is reserved: `world-court` and `"world"` are fine. The
  list is `litigate_syntax::RESERVED_WORDS`.
- **Edition marker.** A file may begin with `edition 2026` (`EditionDecl`,
  §3.1-§3.2), before `module` or `case`; anywhere else it is L0001. A year
  the compiler does not know is L0008. A file without a marker is edition
  2026, the first edition, permanently: a later edition is opt-in by
  marker, so a new edition never changes the meaning of an existing file.
  `Parse::edition` reports it; editions, like file kinds, are
  `#[non_exhaustive]` (§11).
- **Typed literals.** `2_000_000.usd`, `21.days`, `5.court_days`,
  `3.business_days`, `2.weeks`, `9.months`, `1.years`, and `8.hours` are
  single tokens. Period amounts must be integers (L0005), except decimal
  days in a `duration` value, which is an elapsed-time estimate (`duration
  4.5.days`); the parser enforces this, not the lexer.
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
LitFile      = [ EditionDecl ], ModuleDecl, { ImportDecl }, [ PackageDecl ], { Item } ;
EditionDecl  = "edition", Number ;                (* known years: 2026; else L0008 *)
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
EventDecl    = "event", Ident, "(", [ Param, { ",", Param }, [","] ], ")",
               [ EventEffect ] ;
EventEffect  = ( "removes" | "elects" ), Ident | "extends", Ref, "to", Ident | "waives", Ref ;
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
             | "payoff_transform", String | "stake", (SignedMoney | Hole)
             | "patch", Ident, JsonRecord ;

RuleDecl     = "rule", ("duty" | "power" | "liability"), Ident, { VersionClause
             | Relation | Cite }, "{", { Item, Sep }, "}" ;
DeadlineDecl = "deadline", Ident, ":", Duration, ("from" | "after" | "before"),
               Expr, "counted_by", Ref, { VersionClause | Relation | Cite },
               [ DeclOpts ] ;
DeclOpts     = "{", { ( "extendable", Bool | "extension_authority", String
             | "note", String | "bearer", Ref | "trigger", ( Ref | CallExpr )
             | ("extend" | "waive"), Capability ), Sep }, "}" ;
Capability   = "{", { ( "by", Ref | "max", Duration | "to", Expr
             | "discretion", Ident | "after_expiry" | "on", CallExpr
             | "within", Expr | "never", Cite ), Sep }, "}" ;
CountingDecl = "counting", Ident, { VersionClause | Relation | Cite },
               "{", { CountingItem, Sep }, "}" ;
CountingItem = "exclude_trigger_day" | "include_last_day" | "allow_hours"
             | "end_of_month", Ident
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
CaseFile     = [ EditionDecl ], "case", Ident, UsesDecl, "as_of", Date, [ "known_at", Date ],
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
and instances reference. A manifest with none of `pack_id`, `forum`,
`authority`, `title`, `package.roles` is a links manifest; one with some
but not all of them is L1004, as is a links manifest whose package
declares nodes. A link's actors are the `[package.roles]` keys of the pack
its `from` node belongs to. `either` may be a `[package.roles]` key (it
gives the implicit actor's role).

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

`type`, `enum`, and `event` declare closed nominal types. An event type
may declare one effect, which is the only way an event acts on a matter
besides taking an edge or firing a trigger (§5.6). An event's name never
gives it meaning: an event called `Extended` with no effect clause extends
nothing.

- `removes p`: an option-removing event. `p` is a parameter of type
  `String` naming the removed edge.
- `elects p`: an election. `p` is a parameter of type `String` naming the
  edge chosen; the alternatives to it are removed.
- `extends d to p`: an extension. `d` is a deadline and `p` a parameter of
  type `Date`, the new due date. It is licensed by `d`'s `extend`
  capability (§5.3).
- `waives d`: a waiver, licensed by `d`'s `waive` capability.

An unknown parameter is L1002, a parameter of the wrong type L2001, a
second effect or a `to` without `extends` L2007, and `extends` without
`to` L2007.

`match` must be
exhaustive with no unreachable arms (L2005). There is no null, implicit
conversion, subtyping, or recursion: a type alias may not refer to itself, and
neither may a default or exception, directly or transitively (L3006). An event
pattern in a deadline trigger or edge `event` may omit trailing arguments,
which then match anything. `given`, case events, and `has_event` arguments
must be complete.

### 5.3 Deadlines, counting, calendars

`deadline d: N.unit from|after T counted_by c` counts forward from the event
matching trigger pattern `T` that created the obligation instance being
judged (§5.6). `before` counts backward. Units are the `Days<U>` unit of the
literal.

A deadline declaration's options describe the obligation each trigger
creates (design review 2026-10-23 §2.4, §2.7):

- `bearer A`: the actor bearing each instance. A trigger done to (or by)
  a party playing `A` binds the instance to that party; otherwise one
  instance is created for each party playing `A`. With no `bearer`, or no
  party playing it, the instance is unbound.
- `trigger first | latest | each | later_of(e)`: what a matching trigger
  does while an instance for the same bearer is open. `first` (the
  default) ignores it, so a second service does not restart the period.
  `latest` supersedes the open instance and restarts the period (a tolling
  motion, a rehearing, a later appearance). `each` creates another
  instance. `later_of(e)` is `latest` and also absorbs the open instance of
  deadline `e` for the same bearer: the new instance is due no earlier
  than `e`'s (FRCP 15(a)(3), FRAP 4(a)(3)). `later_of` naming the deadline
  itself, or anything but one deadline, is L2007.
- `extend { ... }` declares one extension grant; a deadline may declare
  several (FRCP 6(b)(1)'s (A) and (B) are two). Its bound is a sum, and
  a grant states exactly one (otherwise L2007):
  - `extend { by A, max N.days | to B [, discretion S] [, after_expiry]
    [, on M() [, within W]] }` (bounded): actor `A` may extend an
    instance to a date no later than date bound `B`. Without
    `after_expiry` the extension must come on or before the current due
    date; with it, before or after. `max N.days` is `to prescribed_due +
    N.days`; giving both is L2007. A numeric cap belongs only in the file
    that imposes one, such as a local rule allowing one stipulated
    extension of N days, or FRAP 4(a)(5)(C), which a bounded grant states
    exactly with a standard and a motion:

    ```
    extend { by court, discretion GoodCauseOrExcusableNeglect,
             to max(prescribed_due + 30.days, grant_order_entered + 14.days),
             after_expiry, on AppealExtensionMoved(), within prescribed_due + 30.days }
    ```
  - `extend { by A, discretion S [, after_expiry, on M()] }`
    (discretionary): actor `A` may extend an instance to any later date.
    Its window is before expiry, on or before the current due date (FRCP
    6(b)(1)(A)); with `after_expiry` it is after expiry, on motion (FRCP
    6(b)(1)(B)), and then only after the due date. An `after_expiry`
    discretionary grant must name its motion `on M()` (L2007).
  - `extend { never cite X }` (lock): rule `X` forbids extending the
    period beyond the declaration's own other grants (FRCP 6(b)(2) with
    none; FRAP 26(b)(1) beside FRAP 4(a)(5)'s grant, "except as
    authorized in Rule 4"). `never` takes no other field (L2007). A
    lower authority's `override`, `notwithstanding`, `supersedes` or
    `revives` of a locked deadline, licensed only by `delegable_to`, may
    not declare an `extend` grant the locked version does not: L3004, and
    the relation has no effect on selection, so no local rule or order
    grants an extension there.

  The cause standard `S` is a closed set: `GoodCause`, `ExcusableNeglect`,
  or `GoodCauseOrExcusableNeglect` (FRAP 4(a)(5)(A)(ii)); any other
  identifier is L2007. A date bound (`to`, `within`) is a closed
  expression: `max(b, ...)`, `min(b, ...)`, or `r + N.days` (or `r`) with
  `r` one of `prescribed_due` (the instance's counted due date, raised to
  any `later_of` floor) and `grant_order_entered` (the date of the order
  exercising the grant); anything else is L2007, and days other than
  calendar days L3009. `on M()` names the motion the grant requires, an
  event pattern; `within W` bounds the motion's date (FRAP
  4(a)(5)(A)(i): no later than 30 days after the prescribed time expires,
  whenever the order comes) and needs `on` (L2007).

  A grant with an actor under `extendable false`, or a lock with no grant
  beside it under `extendable true`, is L2007.
- `waive { by A [, after_expiry] }`: actor `A` may release an instance,
  after its due date only with `after_expiry`. `max`, `to`, `discretion`,
  `on`, `within` and `never` are not allowed (L2007).

A deadline with no `extend` capability cannot be extended by any order: a
jurisdictional limit such as the notice of appeal (*Bowles v. Russell*, 551
U.S. 205 (2007)) simply declares none. A deadline declared inside a `rule
duty` is linear: each instance must be consumed exactly once. Elsewhere it is
affine and may lapse.

Counting rule `c` defines (FRCP 6(a) vocabulary):

1. Anchor: `trigger_day`, or `next_business_day` (the first day after the
   trigger that isn't a weekend or calendar holiday).
2. `exclude_trigger_day`: the anchor day is day 0; without it the anchor
   day is day 1. The engine counts `trigger_day` only with
   `exclude_trigger_day` and `next_business_day` only without it; other
   combinations are L6001 at lowering (§8.2).
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
7. A week is seven calendar days (step 3). Months and years (twelve months)
   are calendar months, counted from a `trigger_day` anchor to the
   corresponding day of the later month; when that month has no such day
   (January 31 plus one month), `end_of_month clamp` ends the period on its
   last day (MPEP 710.01(a)) and `end_of_month roll` on the first day of the
   next month. Steps 4-5 then apply. A month or year deadline counted by a
   rule of the same module without `end_of_month` is L3009; the engine
   rejects one whose rule has no `months` provision. The design review of
   2026-10-23 (§2.1) gives the units: they never convert into one another.

`deadline(d)` in a test is the due date `Due {nominal, adjusted}`: bare, or
as `deadline(d).adjusted`, it is the date after every roll and extension;
`deadline(d).nominal` is the last day as counted (step 3 or 7), before any
of them.

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

This is one query (`temporal::select`, supersession applied; precedence,
§5.5, picks among the claimants it finds with `precedence::select`, and
for a matter's event `temporal::governing` adds transitions). Lowering,
`check`, inline `test`s and `tests-pass`, case replay, the deadline
schedule, `explain-value` and the LSP select deadline, counting, calendar
and node versions through it, so none can describe different law. Package queries
(build, `check`, `test`, `tests-pass`) select among the versions recorded
by the package `known_at` unless `check`/`build --known-at` (the LSP's
`knownAt` initialization option) overrides it; a test counts each
deadline under the versions in force on its trigger date. A matter
selects at its own `as_of` and `known_at` (the latest corpus when it has
none) in replay, lowering and its schedule alike. A counting rule a loaded
package declares counts only under a version so selected; the shared clock
library stands in only for a rule no loaded package declares. A counting
version's clock fragment is lowered once, at its own `from` over the
latest corpus.

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
2. If still unresolved, and only between comparable authorities: higher
   authority, then the declaration with the narrower subject matter (its
   trigger or condition entails the other's, not conversely), then one
   inside a `rule` over package level, then the later `from`. The length
   of the in-force interval is never a canon. Incomparable authorities no
   relation orders, and any remaining tie, are L3003.
3. Every relation that displaces its target (`override`, `notwithstanding`,
   `supersedes`, `revives`) needs a capability: A's authority equals or
   outranks B's, or B lists A's authority in `delegable_to` (L3004). An
   unlicensed relation has no effect on selection.

`default d when C => V` concludes V when C holds. `exception e rebuts d when C
=> V'` concludes V' instead. `exception e undercuts d when C` disables d
without concluding anything; it must have no `=>` (L2007). Exceptions on the
same target must be provably disjoint (their conditions can't both be true
for any assignment of the `has_event` atoms they mention) or carry distinct
`priority` (larger wins) (L3005). Rule kinds: `duty` makes each contained
deadline's obligations linear (§5.3): a late act is L4002, and an instance
borne by a party and still open past due at `as_of` is breached (L4009). `power`
contains the decisions it grants. `liability` contains the consequence edges
taken when a duty is missed.

Refinements (precedence bead, `litigate-sema/src/precedence`):

- **Lattice.** The order is the transitive closure of every `authority`
  chain in the package and its dependencies. A level no chain names is
  incomparable to every other. `lit/core` declares the shared order, and
  every package sees it:

  ```lit
  authority Statute > FederalRule > LocalRule > StandingOrder > CaseOrder
  authority Statute > Regulation > CaseOrder
  ```

  `Regulation` (an agency's rules of practice, e.g. 19 C.F.R. Part 210, 37
  C.F.R. Parts 1, 2, 41, 42) is below `Statute` and above `CaseOrder`. It
  is incomparable with `FederalRule`, `LocalRule` and `StandingOrder`, so
  a claimant pair across the two chains is never decided by authority. A
  package's manifest `authority` (§4.1) names one of these levels. A
  package must not declare a chain that orders `Regulation` against a
  court level: such a conflict is fixed in the package's data. No
  diagnostic enforces this in language 1.
- **Claimants.** The declarations that "apply to the same target" T are T
  itself and every declaration with `override T`, `notwithstanding T`, or
  `subject_to T`. On each day, the active claimant that beats every other
  active claimant wins. Stage 1: `notwithstanding` and `override` win,
  `subject_to` yields; a `@D` target applies only while version D is
  active, and a displacing claim counts only when licensed (step 3).
  Stage 2, for pairs stage 1 leaves open and whose authorities are
  comparable: authority, then subject matter (a deadline whose trigger
  pattern extends the other's, e.g. `Served(complaint)` over `Served()`;
  a default or exception whose condition entails the other's), then
  inside a `rule` over package level, then the later `from`. L3003 is
  reported once per unordered
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
- **Selection.** The declaration that governs T on a date is its winning
  claimant; when that winner is itself displaced by a claim on it, the
  selection follows it (a cycle, L3006, stops where it closes). This one
  query (`precedence::select`, over the versions §5.4's `temporal::select`
  finds) is what lowering, case replay, inline tests and `tests-pass`, the
  matter schedule, `explain-value` and the LSP use, so a claim
  is executed, not only explained. Lowering emits a declaration only when
  it governs its own name at the build dates: a displaced node, deadline,
  counting rule or calendar is absent, and every reference to it (`within
  d`, `counted_by c`, `holidays k`, a matter's posture) resolves to the
  winner. A package's selection sees it, its dependencies and `core`; a
  links package sees every loaded package; a matter sees the packages it
  `uses`. A displacing deadline counts from its own trigger and under its
  own counting rule: replay and the schedule take the latest event matching
  it, and a test's `deadline(d)` fails unless the `given` that triggers `d`
  also matches the claimant's trigger.
- **Delegation.** A displacing relation (`override`, `notwithstanding`,
  `supersedes`, `revives`) needs no delegation when A's authority equals
  or outranks B's. Otherwise every version of B it acts on (for
  `override`/`notwithstanding`, those overlapping A's window; for
  `supersedes`/`revives`, those that took effect before A) must list A's
  authority, exactly, in `delegable_to` (L3004 at the relation clause,
  naming the missing `delegable_to [Level]`, with a `maybe_incorrect` fix
  inserting it into B). `delegable_to X` must name a
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
  node with a `facts` entry, follow that edge. An event whose type declares
  an effect (§5.2) applies it and does not move. Otherwise an event that
  matches exactly one outgoing edge's `event` pattern takes it. An event
  that matches no edge is legal only if it fires the trigger of a deadline
  in force on its date, or is matter history a transition arm's
  `has_event` reads. Anything else, or an ambiguous match, is L4001. Where
  the matter has ended (a terminal with no outgoing edge, links included),
  every event that takes no edge is L4001. The replayed node becomes the
  scenario `start`. An explicit `start` that disagrees with the replayed
  node is L4008.
- Obligation instances. After an event is handled, every in-force deadline
  whose trigger it matches creates instances under its `bearer` and
  `trigger` policy (§5.3). The declaration governing a deadline may be a
  claimant displacing it (§5.5): its trigger, policy, capabilities and
  counting then apply to the displaced deadline's instances, and a
  claimant displacing a deadline that the same event triggers creates no
  instance of its own. An event taking an edge with `within d`
  discharges the oldest open instance of `d` borne by the event's party
  (or unbound). It must fall on or before that instance's due date: the
  date counted from the instance's own trigger under the deadline and
  counting declarations governing the event (precedence and transitions
  applied, §5.4, §5.5), or the date an extension moved it to. Otherwise
  L4002, and the instance is closed as late. When only other parties' instances are open, the act
  discharges nothing and is L4001. With no instance open, no period is
  running and nothing is checked. At `as_of`, an open linear instance
  borne by a party and past its due date is breached (L4009, a warning: a
  breach is a fact of the matter, a liability of the bearer). Any other
  instance past due lapses. Each instance ends exactly once: discharged,
  late, breached, lapsed, waived, superseded (`latest`), or merged
  (`later_of`). `CheckedCase.obligations` reports them.
- Extension and waiver. An event whose type declares `extends d to p` or
  `waives d` must carry a `cite`. It transforms every open instance of `d`
  (an order extending the time to answer reaches each defendant's), all or
  none; with none open it is L4001. `d`'s version governing the event must
  grant the capability (an `extend` grant other than a `never` lock, or
  `waive`), and the event's `by` party must play the actor of at least one
  grant. One such grant must admit every open instance: its window holds
  the event's date (on or before each instance's current due date, or
  after it for `after_expiry`; a discretionary `after_expiry` grant only
  after it); the motion it names (`on M()`) is in the matter, an event
  matching `M` dated no earlier than the instance's trigger, no later
  than the order, no later than its `within` bound, and, for a
  discretionary `after_expiry` grant, after the current due date (FRCP
  6(b)(1)(B): "on motion made after the time has expired"); and an
  extension's new date is later than that due date and, for a bounded
  grant, no later than its bound, evaluated with `prescribed_due` the
  instance's counted due date and `grant_order_entered` the event's date.
  A discretionary grant has no ceiling. An event matching a grant's
  motion pattern is matter history: legal at any stage, moving nothing.
  When no grant admits the event, the refusal reported is the bound's in
  preference to the window's or the motion's. Each failure is L4001, and
  no instance changes; a lock with no grant beside it is L4001 for any
  extension. A cited
  event that is not of such a type transforms nothing, so no order can
  enlarge a limit that grants no capability (as in *Bowles v. Russell*,
  551 U.S. 205 (2007)).
- Option-removing events. A waiver or an order striking a defense does not
  move the matter; it takes an option away for the rest of it. An event
  whose type declares `removes p` is option-removing: its `p` argument
  names an edge by id, `edge` in the primary package or `pack::edge`
  otherwise (any version, link edges included). Replay handles it at the
  posture it finds, after following `facts`, and does not move: it records
  a removal `{edge, date, event, authority}` in `Case.removals`, in replay
  order. The event must carry a `cite` (otherwise L4001 and nothing is
  removed). A `p` argument naming no edge is L1002 with a `maybe_incorrect`
  fix to the nearest id. Removing an already removed edge is L4001 with a
  note at the first removal. A later event that matches only removed edges
  is L4001 (a note locates the removal) and replay stops. An event whose
  type declares `elects p` (§5.2) is an election: its `p` argument names
  the edge chosen, and replay removes every other edge (link edges
  included) at each node, any version, that offers it, as above, recording
  one removal per
  alternative with the election's date, event and cite. An alternative
  already removed stays removed once; electing an already removed edge is
  L4001 with a note at the removal; an alternative without an id cannot be
  removed and is L4001. Removals are derived, never asserted: checking
  replaces any `removals` in the HIR.
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
| `Days<U>` (`U` = `CalendarDays`, `CourtDays`, `BusinessDays`, `Weeks`, `Months`, `Years`) | `21.days`, `5.court_days`, `3.business_days`, `2.weeks`, `9.months`, `1.years` | comparisons; `+ -` with the same `U`; `* /` by `Number` with integral result; no unit conversion (L2004) |
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

The builtins (closed list) are `deadline(d) -> Due` (tests: due date from the
`given` events; bare it is the adjusted `Date`, and `.nominal`/`.adjusted`
project `Date`s, §5.3), `has_event(Pattern) -> Bool`, `prob`, `given`, `in`,
`intersect`, `disjoint`, `len`, and `contains`, plus the event constructors
from `event` declarations. There is no IO, ambient clock, mutation, float,
implicit cast, or overloading beyond this table (L2002 otherwise). Field
types: `p` is `Probability`, `effort` is `Hours`, `cost`/`payoff` are
`Money<usd>`, `duration` is `Days<CalendarDays>` (`min <= mode <= max`, decimal amounts allowed;
weeks, months and years are estimates in mean Gregorian days: 7, 146097/4800 and 365.2425),
`deadline` is `Days<CalendarDays|CourtDays|Weeks|Months|Years>`, and attrs are `Number`.

## 7. Verification obligations and profiles

`litigate check` reports every obligation as a `Finding`
`{name, status, count, examples, evidence}`, collected in a `VerificationReport`
`{obligations, profile}` that is embedded in every litigatable
(`litigate_verify::{Finding, VerificationReport}`; abridged):

```json
{"obligations":[
   {"name":"well-typed","status":"passed","count":0,"examples":[],
    "evidence":{"checked":28,"subjects":"declarations"}},
   {"name":"holes","status":"warning","count":1,
    "examples":["lit/x/main.lit:12:3: L9002 hole: no calibration yet"]},
   {"name":"case-replay","status":"skipped","count":0,
    "examples":["a rule package has no events; replay is checked per case"]}],
 "profile":"dev"}
```

- `obligations` lists every obligation of the table below, once, in table
  order. `name` is the obligation's kebab-case name.
- `status` is `passed` (checked, no finding), `warning` (only
  warning-severity findings: the profile accepts it), `failed` (at least one
  error-severity finding), or `skipped` (not checked for this artifact).
- `evidence` `{checked, subjects}` is the positive evidence a result rests
  on: how many subjects the check covered and what they are (`declarations`,
  `version sets`, `relations and authority chains`, `displacing relations
  and delegations`, `exceptions and precedence claims`, `chance nodes`,
  `non-terminal nodes`, `nodes`, `cites`, `inline tests`, `events`, `case
  entries`, `holes`, `deadline and counting rules`). Every `passed` finding
  carries it (`checked: 0` says the check ran over nothing). An obligation
  is never passed by the absence of diagnostics: one with no evidence and no
  finding is `skipped` (a package sema rejected has no HIR to count; a
  matter's report passes only its own `well-typed` and `case-replay`).
  Absent on skipped findings and on failures with no evidence.
- `count` is the number of findings (violations, holes, warnings); `0`
  when skipped.
- `examples` holds at most eight findings, sorted like diagnostics (§9.1),
  each `file:line:col: CODE message`. A skipped finding instead holds one
  string saying why: it does not apply (a rule package has no events, a
  matter carries no graph), or an earlier phase failed and the artifact
  never existed.
- The report is accepted iff no obligation is `failed`.

Each finding also emits the listed code as a diagnostic. Diagnostics of
earlier phases count against the obligation whose code the table lists
(`litigate_verify::obligation_of`); every L4xxx counts against
`case-replay`; L3003 against `exceptions-ordered`; L3007 and L3008 against
`effective-covered`; L9003, L9004, and L9005 belong to no obligation; every
other code (syntax, names, types, L3009, L5001, lowering) counts against
`well-typed`, so no error ever leaves every obligation passed.

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
| `case-replay` | sema's case checker accepts the matter: it resolves, its pins, parties, facts, stakes, and beliefs are valid, and its events order, match, and meet deadlines on the engine clock (§5.6) | L4001-L4009 |
| `holes` | lists every hole with reason and span | L9002 / L5002 |
| `rule-without-test` | every deadline and counting rule is exercised by a `test` | L9001 |

A rule package's report skips `case-replay`; a matter's report checks
`case-replay` (and counts its resolution errors against `well-typed`) and
skips every other obligation, which its packages carry.

Profiles change severity and acceptance, never values. `dev`: holes are L9002
warnings. A failed obligation is an error. Exceptions: `cite-resolves` on
declarations that `litigate import` marks `/// imported`, missing tests
(L9001), and `unvalued` stakes (L4004) are warnings. A declaration is
imported when one of its leading `///` lines is exactly `imported`; HIR
records it as `Metadata.imported`, and the marker line is never the
declaration's fallback label. The exemption covers the declaration's own
`cite` and, for a node, its edges' `authority` texts. `strict` rejects every
hole (L5002), every `cite-resolves` failure, every untested deadline or
counting rule (L9001 as an error), `unvalued` stakes, and non-canonical
formatting (L9004).
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
| `state`/`chance` / `decision` / `terminal` | `RawNode.kind` `state` / `decision` / `terminal`, always written |
| node `group`, `note`, `valence`, `court_listener_url`, `tags`, `attrs` | same-named `RawNode` fields (`courtListenerUrl`) |
| `outcome`, `payoff`, `payoff_by_flag` | `RawNode.outcome`, `payoff`, `payoffByFlag` (major units) |
| edge, in node order then edge order | `Pack.edges[]`: `from` = enclosing node, `id` (omitted for `_`), `to`, `label`, `actor` |
| `p`, `effort`, `cost` | `RawEdge.probability`, `hours`, `cost` |
| `authority` | `RawEdge.authority` (string, cite text, or cite-hole reason) |
| `within d` | `RawEdge.deadline` `{length: N, unit, extendable, extensionAuthority, note}` from d's version in force, edge options winning |
| `deadline N.unit {…}` | same, from the literal; `unit` omitted for calendar days, `"court"` for court days; business days are L6001 |
| `duration` | `RawEdge.duration {min, mode, max}` (a single duration is `{mode}`) |
| `valence`, `court_listener_url`, `action_id`, `note`, `tags`, `attrs`, `sets`, `clears`, `requires`, `forbids` | same-named `RawEdge` fields |
| `counting` / `calendar` | `Pack.counting` (the rule id) plus a clock-library fragment (§8.2) |
| `event`, authority lattice, relations, defaults, exceptions, transitions, tests, types | not emitted (verification only) |

Multiple versions: only the declaration the precedence selection (§5.5)
picks at the build's `--as-of`/`--known-at` is emitted, with supersession
and licensed displacing claims applied (§5.4-§5.5); a declaration with no
version in force then, or displaced by a claimant, is absent, and
overlapping versions, or an edge or `start` naming an absent node, are
L3007. A reference resolves to the selected claimant; unordered claimants
are L3003. The default is the package `known_at` date for both. Node order is
module order (the module files' paths, §4.1), then source order.

A cite hole `cite ?("text")` lowers to `"text"` exactly, so imported
citations stay lossless; a cite hole without a reason lowers to an absent
field. Like every hole it is a warning in `dev` and an error in `strict`
(§7), so the porter replaces it with a resolvable citation.

### 8.2 Clock IR (pack schema v3)

The engine's landed clock (DESIGN §9.1, `crates/litgraph/src/clock`) is the
target. `Pack.counting` is a string: the id of a `CountingRule` in
litgraph's shared *clock library* (`packs/clock/*.json`,
`docs/PACK_SCHEMA.md` "Clock library"), a set of fragments
`{schemaVersion: 3, calendars: [HolidayCalendar], countingRules:
[CountingRule]}` merged by id. Rules are step pipelines; every step carries
its cited trace `text`.

- A package that declares no `counting` lowers the manifest's `counting`
  key (§4.1) to `Pack.counting` verbatim: it names a rule of the shared
  library, which is all `litigate import` needs.
- A package that declares one counting rule `c` gets `Pack.counting =
  "c"`. One that declares several and whose manifest names none is a
  **counting-rule library** (`lit/core`): its pack has no `counting`, and
  packages that import its rules name them in their own manifests. A
  manifest `counting` naming a rule the package does not declare, while it
  declares some, is L6001. Lowering also emits one clock fragment holding
  every declared rule (the versions in force at the build dates) and, once
  each, the calendars they name, written beside the pack as
  `clock/<pack id>.json`. Library ids are global: a generated fragment
  replaces the hand-authored one with the same id (the engine skips and
  reports a duplicate).

A counting rule lowers field by field; anything outside this table is
L6001:

| `counting c` | `CountingRule` |
|---|---|
| id, `cite` | `id`, `label` (the cite, else the id); each step's `text` is generated as `"<cite>: ..."` |
| the `url` of the source its cite binds to (§4.3) | `sourceUrl` (required: no bound url is L6001) |
| (always) | `closedWeekdays: ["saturday", "sunday"]` |
| `holidays k` | `holidays: ["k"]`, and calendar `k` in the fragment |
| `anchor trigger_day` + `exclude_trigger_day` | `{"op": "anchor", "kind": "trigger"}` |
| `anchor next_business_day` (no `exclude_trigger_day`) | `{"op": "anchor", "kind": "adjacent-business-day"}`, `minLength: {days: 1}` |
| `short_period { below: K.days, count: U }` | first `count` period `{unit: "calendar", below: K, count}` (`calendar` for calendar days, else `business`) |
| (always) | `count` periods `{unit: "calendar", count: "calendar"}`, `{unit: "court", count: "business"}` |
| `include_last_day`, `roll_forward_on [weekend, legal_holiday]` | `roll` before `base`: forward `toward: later`, backward `toward: earlier` |
| `service_extension { M: K.days }` | `service` `{method, days: K}`, then a second `roll`; `M` in kebab case must be an engine method (`mail`, `clerk`, `other-consented`, `commercial-carrier`, `prison-mail`, `mail-foreign`, `express`, `express-foreign`); non-calendar days are L6001 |
| `inaccessible` in `roll_forward_on` | `clerkInaccessible` step (and, with `allow_hours`, `hours.inaccessibleText`: FRCP 6(a)(3)(B), FRAP 26(a)(3)(B)) |
| `allow_hours` | `hours` provision |
| `end_of_month clamp` / `roll` | `months` provision `{endOfMonth: "clamp"/"roll", text}` |

The pipeline is therefore `anchor, count, roll, base, service, roll,
clerkInaccessible` (absent steps dropped): §5.3 steps 1-6 in order. The
engine always includes the last day and rolls off any closed day, so a
rule without `include_last_day`, or whose `roll_forward_on` lists only one
of `weekend`/`legal_holiday`, is L6001. Service days are added after the
roll, as FRCP 6(d) and FRAP 26(c) read ("3 days are added after the
period would otherwise expire under Rule 6(a)"), and `base` records that
day: the period's own last day after its roll, before any added days. The
shipped `frcp6`, `rcfc6`, and `frap26` fragments use the same order, so a
lowered rule reproduces their `due_date`, `base_date` and
`last_day_rolled`.

| `calendar k` / holiday | `HolidayCalendar` / `Holiday` |
|---|---|
| id, label, cite | `id`, `title` (the label, else the id), `cite`, `sourceUrl` (bound url, if any) |
| `h: … effective [a, b)` | `Holiday {id: h, name: h, effective: {from: a, until: b}}` |
| `fixed(m, d)` | `date: {fixed: {month, day}}` |
| `nth(n, w, m)` / `last(w, m)` | `date: {nthWeekday: {month, weekday, nth}}`, `nth` = n / -1 |
| `every(y, from, m, d)` | `fixed` plus `cycle: {every: y, anchorYear: from}`, effective from no earlier than January 1 of `from` |
| `once(date)` | `fixed` on that month and day, effective `[date, date + 1)` (widened by a day each side under `nearest_weekday`) |
| `observed nearest_weekday` / none | `observed: "nearest-weekday"` / `"actual"` |
| `observed next_day` | L6001: the engine has no Sunday-only observance |

The generic interpreter consumes only this IR plus `(trigger date, amount,
unit, direction, service method, inaccessible dates)` and never branches on
a forum or pack id. The four current regimes are expressible: FRCP 6 and
FRAP 26 differ only in `service_extension` keys; RCFC 6 adds
`inauguration-day: every(4, 1965, 1, 20) … observed nearest_weekday` (as the
landed `us-inauguration-day` calendar has it); 19 CFR 201.14/210.6 uses
`anchor next_business_day` and `short_period { below: 7.days, count:
business_days }`. Lowering validates every fragment with the engine's own
loader (`ClockLibrary::from_json`), so an invalid one is L6001.

### 8.3 Case to `NamedScenario`

| Source | Target |
|---|---|
| case id, `summary`, `notes`, `source`s | `NamedScenario.id`, `summary`, `notes` (empty string when absent), `sources[] {title, url, as_of}`; an `as_of` hole lowers to its reason (`?("2026-06")` is `"2026-06"`) |
| `uses` | `packs[]`: pack ids in `uses` order |
| replayed/explicit start | `scenario.start` (qualified `pack::node`): the explicit `start`, else the replayed node; omitted when neither exists (the primary package's start applies) |
| `perspective` | `scenario.perspective` actor-to-role map: `perspective party` lists only the actors whose role differs from the primary package's roles (which must be loaded, else L6001); an explicit `{ actor: role }` map is emitted as written |
| `facts` | `scenario.facts` qualified node to qualified edge |
| `stakes` | `scenario.payoffs` qualified terminal to major-unit number |
| `beliefs` | `scenario.observe` qualified node to `{qualified edge: count}` |
| `Case.removals` (§5.6) | `scenario.remove_edges`: any `analysis.remove_edges` entries in order, then each removed edge not already listed; the source map has one entry per removed edge (`/scenario/remove_edges/i`) with the removing event's span and its cite located in L0 under the edge's package. A non-list `analysis.remove_edges` is left for litgraph to reject (L4007) |
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
payoff_transform, stake, patch_edges}`. `patch` records become `patch_edges[edge]`
verbatim as JSON merge patches (keys are `RawEdge` JSON names, `unset` is
null). The lowerer applies each patch to the base edge and deserializes the
result as `RawEdge` (L6001 on failure); the base pack must be loaded with
the links package, else a patching instance is L6001, and a patch naming an
edge the base lacks is L1002. `payoff_transform` must parse as a litgraph
expression whose only variables are `payoff` and, when the instance
declares one, `stake` (L6003). A `stake` the transform does not use is
L2007. A known `stake` lowers to `stake` in USD; a `stake` hole lowers to
an absent `stake`, and the engine then leaves every terminal payoff of the
instance unauthored (`payoff-not-authored`) instead of guessing one: the
amount at stake is a fact of the matter. `replaces` outside a links
package is L2007.

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
bit equality. Clock fragments compare the same way. Output is
byte-deterministic: two-space-indented JSON with a trailing newline,
written atomically; a scenario omits `Scenario` fields equal to litgraph's
defaults (serde restores them). Source maps and provenance are sidecars
excluded from equality.

`semantic_diff` (crate `litigate-lower`, the `litigate diff` oracle)
reports `{kind, pointer, id, before, after}` with `kind` one of `added`,
`removed`, `changed`, `order`. Nodes, edges, links, groups, sources,
calendars and counting rules are matched by id, so pointers name ids
(`/nodes/answer-due/label`, `/edges/respond->answered#0/cost`) and a pure
permutation is one `order` change (`/nodes/order`, `/edges/order/<from>`
for edges within one source node, or `/links/order` for links, whose whole
order counts since they have no node list to regroup under). Other arrays
are compared by index.

The sidecar `<file>.map.json` is `{generated: {by: "litigate-lower",
version, language}, kind, id, package: {name, version, digest}, as_of,
known_at, entries}`; each entry is `{pointer, id, span, decl, cite}` for
one generated node, edge, link, instance, counting rule or calendar:
`pointer` its JSON pointer, `id` its litgraph id (derived for `_` edges),
`span` its declaration, `decl` the emitted version `id@in-force-from`, and
`cite` the `cite`/`authority` text with each citation litgraph's citation
parser recognizes (under the pack's `citeSyntax` and the sources'
`cite_prefixes`/`section_heading`) as `{heading, source, path, line}`: the
`## <heading>` line in the vendored L0 file. The `generated` object is the
generated-file marker; JSON documents carry none, since litgraph's types
reject unknown fields.

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
`decode`, `severity`, `derivation_rule`). A new code is a SPEC change plus a
registry change, never ad hoc. Codes are never reused. Decoding is
tolerant: a well-formed code (`L` and four digits) outside the registry, from
a newer compiler or retired, decodes to `DiagnosticCode::Unknown`, re-encodes
unchanged, and is graded `error`; anything else is rejected.

| Code | Name | Code | Name |
|---|---|---|---|
| L0001 | unexpected token | L3001 | effective overlap |
| L0002 | unterminated literal | L3002 | effective gap |
| L0003 | invalid token | L3003 | unresolved precedence |
| L0004 | invalid date | L3004 | missing delegation |
| L0005 | invalid number/suffix | L3005 | exception conflict |
| L0006 | dangling doc comment | L3006 | precedence cycle |
| L0007 | reserved word | | |
| L0008 | unknown edition | | |
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
| | | L4009 | obligation breached |
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

Severity (`DiagnosticCode::severity(profile)`): every non-L9 code except
L4009 is `error`. L9003 is `info`. L4009 and L9005 are `warning`. L9001, L9002, and L9004 are
`warning` in `dev` and `error` in `strict`.

### 9.2 Shapes

Envelope and diagnostic schema version 2 (`litigate_base::SCHEMA_VERSION`):

```json
{"code":"L3003","severity":"error",
 "span":{"file":"lit/x/main.lit","line":15,"col":67,"end_line":15,"end_col":82},
 "message":"`b@2009-12-01` and `a@2009-12-01` both apply to `tg` on [2009-12-01, ..), and neither prevails",
 "labels":[{"span":{"file":"lit/x/main.lit","line":14,"col":67,"end_line":14,"end_col":82},
            "message":"`a@2009-12-01` is declared here"}],
 "notes":[{"message":"no declared relation (`notwithstanding`, `override`, `subject_to`) orders them"}],
 "fixes":[{"message":"rename `old` to `new`",
           "edits":[{"span":{"file":"lit/x/main.lit","line":3,"col":10,"end_line":3,"end_col":13},
                     "replacement":"new"},
                    {"span":{"file":"lit/y/main.lit","line":8,"col":4,"end_line":8,"end_col":7},
                     "replacement":"new"}],
           "applicability":"maybe_incorrect"}],
 "derivation":{"rule":"conflict","fact":"conflict(`tg`, `a@2009-12-01`, `b@2009-12-01`) on [2009-12-01, ..)",
   "span":{"file":"lit/x/main.lit","line":15,"col":67,"end_line":15,"end_col":82},
   "premises":[{"rule":"undefeated","fact":"undefeated(`a@2009-12-01`, `tg`) on [2009-12-01, ..)",
                "premises":[{"rule":"claim","fact":"claim(`a@2009-12-01`, `tg`)"},
                            {"rule":"no-declared-win","fact":"¬declared_win(`b@2009-12-01`, `a@2009-12-01`)"},
                            {"rule":"no-canon","fact":"the canons tie: …"}]}]},
 "fingerprint":"6b222b9a9158b43f","profile":"dev",
 "world":{"name":"actual","as_of":"2025-06-01"}}
```

`file` is the repository-relative `/` path. Lines and columns are one-based.
Columns count Unicode scalars and the end is exclusive. The LSP converts to
UTF-16 through `LineIndex`. `severity` is `error|warning|info|hint`.
`applicability` is `machine_applicable|maybe_incorrect|has_placeholders|
unspecified`. Diagnostics are sorted by `(file, line, col, code)`.

- `labels` are secondary locations, in any file, each saying what is there.
  `notes` are unlocated explanation. Both are omitted when empty.
- A fix is one or more non-overlapping `edits`, possibly across files,
  applied together or not at all (`litigate fix`, LSP code actions); its
  optional `message` says what it does.
- `derivation` is present on every L3xxx error: the proof of the defect in
  the resolution semantics (§0; review §3.3 strata S0-S5). Each step is
  `{rule, fact, span?, premises}`. The precedence checks give full trees:
  L3003 and L3005 `conflict` over two `undefeated` claimants (each a
  `claim` with `no-declared-win`/`no-defeat` and `no-canon`, or an
  `exception` with `no-priority` and `not-disjoint`), L3004 `unlicensed`
  over the displacing `claim`, the authority `rank`, and each
  `no-capability` (missing delegation), L3006 `cycle` over each `declared`
  `prevails` edge. Every other L3xxx error carries the one-step derivation
  of its code's rule (`DiagnosticCode::derivation_rule`: `overlap`, `gap`,
  `no-winner`, `transition`, `counting`) whose premises are its labels
  (`premise` steps).
- `fingerprint` (set in envelopes) identifies a diagnostic across runs and
  across edits that only move lines: 16 hex digits of FNV-1a-64 over the
  code, the file, the message, and its occurrence among diagnostics equal in
  those three.
- `profile` is the checking profile that graded the severity (`check`,
  `build`, `run`); `world` is the world it was found in: `name` `actual`
  (the packages as loaded; named worlds arrive with `.world` files) at the
  command's `as_of`/`known_at` coordinates.

Every JSON response is an envelope
`{schema_version, ok, op, result, diagnostics, provenance, elapsed_ms}`.
`schema_version` is `2`. `ok` is false iff any error-severity diagnostic was
produced. `result` is kept alongside errors whenever the command produced
one: a failing `check` still reports its obligations, and a failing `build`
reports `{obligations}`; it is absent only when there is none.
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
| `build` | `PKG_OR_CASE...`, `--as-of`, `--known-at`, `--out DIR`, `--pack-file ID=STEM` (repeatable) | `{outputs:{packs:[path], links:[path], clock:[path], scenarios:[path], source_maps:[path], litigatables:[path]}, litigatable_hash, litigatables, obligations}` |
| `fmt` | `PATHS...`, `--check` | `{files:[{path, changed}]}`; `--check` reports L9004 instead of writing |
| `explain` | `CODE` | `{code, explanation, severity:{dev, strict}}` |
| `fix` | `PATHS...`, `--code CODE`, `--dry-run`, `--include-maybe` | `{fixes:[{code, message?, edits:[{span, replacement}], applicability}], changed_files, skipped_overlapping, dry_run}`; only machine-applicable fixes without `--include-maybe`; a fix applies whole or not at all |
| `where` | `ID` (qualified node/edge/declaration, code, or cite) | `{id, kind, declaration_span, versions:[{in_force, known_at, span}], lowered:[json_pointer]}` |
| `explain-value` | `--case FILE`, `--value EXPR` | `{value, type, steps:[{rule, cite, detail}], versions, overrides}` |
| `diff` | `A B` (generated or expected JSON files or package dirs), `--as-of` | `{equal, changes:[{pointer, before, after}]}` per §8.6 |
| `query` | `KIND` (`nodes\|edges\|deadlines\|holes\|cites\|versions`), `--match TEXT`, `--limit N`, `--cursor C` | `{matches:[{id, kind, span, cite, in_force}], next_cursor}` |
| `import` | `JSON...`, `--out DIR` | `{files:[{path, from}], warnings}`; output must round-trip under §8.6 (L6002 otherwise) |
| `lsp` | `--stdio` | LSP session (no envelope) |

`build` writes pack `ID` to `packs/ID.json` (its source map to
`packs/ID.map.json`, its clock fragment to `packs/clock/ID.json`); `--pack-file
ID=STEM` writes those as `STEM` instead. Both sides are kebab-case, an id is
mapped once, `links` is reserved, and a mapping naming no built pack, or two
outputs landing on one path, is a usage failure (exit 1).

`justfile` recipes: `lit-check` runs `litigate check --profile strict lit cases`.
`lit-sync` regenerates the repository's generated files (§9.4); `lit-sync-check`
runs it and fails when `git status` shows any change under `packs/` or
`scenarios/`, and is part of `just ci`. `lit-diff` builds into a temp dir and
runs `litigate diff --check` against `packs/` and `scenarios/` without
writing. `lit-test` runs the litigate crates' tests.

### 9.4 Generated repository files

Since the flip (DESIGN §11.3) these files are build outputs; edit `lit/` or
`cases/` and run `just lit-sync`, never the JSON:

| Generated file | From |
|---|---|
| `packs/<file>.json` | each forum package `lit/<forum>/` (not `lit/core`, a library, nor `lit/examples`) |
| `packs/links.json` | `lit/links/` |
| `packs/clock/<file>.json` | a forum package's own counting rules and calendars (today only `mpep-prosecution`) |
| `scenarios/<id>.json` | each top-level `cases/<id>.case` (not `cases/requests/` nor `cases/examples/`) |

`<file>` is the pack id, except where `lit-sync` maps it with `--pack-file`
to the legacy file name the engine, its tests and external users already
load: `cafc` → `cafc-federal-circuit`, `cofc` → `cofc-court-of-federal-claims`.
The input list and the mappings are the justfile's `lit_sync_inputs` and
`lit_sync_names`; a new forum package is added there. `lit-sync` deletes
`packs/*.json` and `scenarios/*.json` before copying, so a removed package's
file disappears; source maps and litigatables are not copied. Not generated:
`lit/core`'s counting rules and calendars are not yet the engine's clock
library, so `packs/clock/{frap26,frcp6,itc210,rcfc6,us-federal-holidays}.json`
stay hand-authored (with `packs/clock/SOURCES-NOTES.md` and
`packs/*-SOURCES-NOTES.md`), as do `examples/*.json` requests, which the
`cases/requests/` matters reproduce through `litigate run`. Generated JSON is
the canonical rendering of §8.6, so its bytes need not match a hand-written
predecessor; `litigate diff` is the equality that matters.

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

The edition bead (lg-flu.12) added `KwEdition` and `EditionDecl`, the
`Edition` type and `Parse::edition`, and `RESERVED_WORDS`. `FileKind` and
`Edition` are `#[non_exhaustive]`, so a `.world` file kind or a new
edition is additive: `parse(text, kind)` keeps its signature because the
edition comes from the source, not the caller. The tree-sitter grammar
matches.

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
`Served` as a trigger that creates the `answer-period` obligation (due
2025-03-26), and takes `move-dismiss`, which discharges it, on
2025-03-20, so the scenario `start` is `frcp-rule12-example::mtd-ruling`.
The two tests in `rule12.lit` pass (2025-03-24; 2025-03-27 with mail
service).

## 13. Type-system classification of checks (type system v2 audit)

Status: analysis for epic lg-962 (bead lg-962.1). Nothing here changes
compiler behaviour; §§1-12 stay normative for the current compiler. This
section classifies every §7 obligation, every registered diagnostic code
(§9.1) and every SPEC rule that has no code of its own, so the v2 beads
(lg-962.3 onwards) can move each one into the type checker or keep it a
whole-program check on purpose.

### 13.1 Classes and the decidability budget

- **(a) Types.** Rejected by ordinary syntax-directed typing: closed
  nominal ADTs (§5.2), exhaustive `match`, phantom indices drawn from a
  closed finite set (`Days<U>`, `Money<Cur>`, a deadline's direction),
  sorts that make an illegal combination unrepresentable (a sum type in
  place of two independent flags), and the grammar itself. No arithmetic
  reasoning. Includes checks done today by the lexer or parser (L0xxx):
  the grammar is the term language, so these are not type obligations
  but are listed for completeness.
- **(b) Decidable refinement or indexed type.** A judgement
  `Γ ⊢ e : {v: B | φ}`, or a type indexed by a compile-time value, where
  `φ` and the index live in this fixed fragment and nowhere else:
  - quantifier-free linear arithmetic over exact rationals and integers
    (QF_LRA / QF_LIA), with dates as integer day numbers and money as
    integer minor units;
  - propositional logic over `has_event` atoms and boolean constants
    (the §5.4 transition and §5.5 exception condition language);
  - finite-set membership, equality and cardinality over enum variants,
    declared names, authority levels and service methods;
  - the authority order as a fixed partial order on type-level
    constants (DESIGN §4.2: the one admitted subtyping).

  Γ holds only the declaration's own signature and the signatures (never
  the bodies) of the declarations it names, plus the *family* the
  resolver already groups by name: the versions of one declaration, or
  the exceptions on one target. No transitive closure, fixpoint or
  execution. Because litigate values are compile-time constants, most
  (b) predicates are ground and are discharged by the exact evaluator
  (§6) with no solver; only predicates over holes (`?`) and `has_event`
  atoms go to the SMT backend of lg-962.3. Every query is finite and
  quantifier-free, so every type check terminates. Products of two
  non-constant terms (`p * q` with both holes) are outside the fragment
  and stay an L2006 check at evaluation.
- **(c) Global whole-program check.** Needs one of: reachability, cycle
  detection or another fixpoint over a graph; day-by-day activity over a
  whole package (supersession, revival, claimants); execution (tests,
  replay, counting through holiday calendars); an artefact outside the
  source (L0 text, `Lit.lock` digests, the engine's loader or expression
  parser, an expected JSON file, the filesystem); or the profile. The
  reason column names which.

Every row keeps its code. A check moved into types still reports the
same `DiagnosticCode`, against the same §7 obligation, with the same span
and fixes (DESIGN §8), so `litigate explain`, `litigate fix` and the
VerificationReport contract do not change; only the phase that finds it
does. A refinement failure names the violated predicate and the ground
values (or, under SMT, a counter-model) in a note. Each (a) or (b) row
that a v2 bead moves gets negative tests showing the type checker rejects
it, and `just cov-litigate` stays at 100% (bead lg-962). Kani, Verus,
mutation testing and mechanized metatheory are out of scope here
(lg-litigate-epic-tuq.38).

### 13.2 Obligations (§7)

| # | Obligation | Class | Target type form (b) / reason (c) / mechanism (a) |
|---|---|---|---|
| O1 | `well-typed` | a+b | Union of the L2xxx rows (C18-C32) and the codes §7 counts here (L0xxx, L1xxx, L3009, L6xxx): mostly (a); arithmetic domain (L2006), distributions (L2003), probability range (L2008), `fact` placement and rule shapes (L2007) are (b). |
| O2 | `cite-resolves` | c | Needs the vendored L0 text and the manifest's source bindings: resolving a pinpoint to a heading span reads a file. |
| O3 | `effective-disjoint` | b | Family refinement on the versions of one declaration: `Versions<d> = [Interval]` with `∀ i<j. to_i ≤ from_j ∨ to_j ≤ from_i` (QF_LIA, pairwise, finite). lg-962.3. |
| O4 | `effective-covered` | c | Coverage minus declared gaps depends on which other declarations `supersedes`/`revives` this one, transitively and bitemporally (§5.4): activity over the whole package. |
| O5 | `precedence-acyclic` | c | Cycle detection over the authority chains (with dependencies), the relation graph, and default/exception references. |
| O6 | `delegation-respected` | b | Per declaration (bead lg-962.4 comment: each delegating declaration carries `delegable_to`): see C36. |
| O7 | `exceptions-ordered` | b | Family refinement on the exceptions of one target: see C37. |
| O8 | `dist-exact` | b | `Dist<E> = {m: E → Q | ∀e. m(e) ≥ 0 ∧ Σ_e m(e) = 1}`, total over `E`; a chance node's edges are `Dist<Edges(n)>`. Holes make it an SMT query (`Σ known ≤ 1`). lg-962.3. |
| O9 | `graph-exits` | a | Node kind index: the body of a non-terminal node is `NonEmpty<Edge>`. Checked per version of the node. |
| O10 | `graph-reachable` | c | Graph reachability from `start` and `entry` nodes over the lowered graph. |
| O11 | `deadline-satisfiable` | c | Needs reachability (only reachable nodes) and due-date windows computed through counting rules and holiday calendars for some trigger date. |
| O12 | `tests-pass` | c | Execution of `test` blocks (counting through calendars). |
| O13 | `case-replay` | b+c | Event order and posture typestate are (b) (C47, C48); deadline timing, stakes and start agreement are (c) (C49, C51, C55). |
| O14 | `holes` | a | A hole is a distinct inhabitant `Hole<T>` of each field type; listing them is a traversal of the typed tree. |
| O15 | `rule-without-test` | c | Which deadlines and counting rules a test exercises is a whole-package relation between `test` bodies and declarations. |

### 13.3 Diagnostic codes (§9.1)

"Today" names the crate (`sema`, `verify`, `lower`, `syntax`, `cli`,
`import`) that emits the code. A code with several distinct conditions
gets one row per condition when the conditions classify differently.

| # | Code: condition (SPEC §) | Today | Class | Target type form (b) / reason (c) / mechanism (a) |
|---|---|---|---|---|
| C1 | L0001 unexpected token (§3) | syntax | a | Grammar. |
| C2 | L0002 unterminated literal (§2) | syntax | a | Grammar. |
| C3 | L0003 invalid token, BOM, invalid UTF-8 (§2) | syntax, sema | a | Lexical grammar. |
| C4 | L0004 invalid date (§2) | syntax, sema, lower | a | `Date` has only valid proleptic Gregorian inhabitants; the literal is a smart constructor. |
| C5 | L0005 `_` placement, unknown suffix, non-integral day amount (§2) | syntax, sema | a | Lexical grammar; `Days<U>` carries an integer (decimal only in `duration`, an elapsed-time estimate). |
| C6 | L0006 dangling doc comment (§2) | syntax | a | Grammar. |
| C7 | L1001 duplicate name in a namespace; local shadows import (§4.3) | sema | a | Scope formation: a namespace is a finite map; versions share a name by construction (§5.4). |
| C8 | L1002 unresolved name, enum variant, option edge, patch edge (§4.3, §5.6, §8.4) | sema, lower | a | Name resolution is part of typing: every `Ref` has a declaration type. The cross-package forms (`pack::edge`, a patch's base edge) read only the referenced package's signature. |
| C9 | L1003 import cycle (§4.3) | sema | c | Cycle detection over the module import graph. |
| C10 | L1004 manifest: unknown/missing/ill-typed key (§4.1) | sema | a | Manifest is a closed record (`deny_unknown_fields`). |
| C11 | L1004 manifest: some but not all pack-identity keys; links manifest with nodes (§4.1) | sema | a | `Manifest = Pack {pack_id, forum, authority, title, roles} \| Links {}`; a links package's items are a sort without `NodeDecl`. |
| C12 | L1005 `package` declaration disagrees with `Lit.toml` (§4.1) | sema | b | Singleton index: the module is typed `Module<name, forum, authority>` and must equal the manifest's constants. |
| C13 | L1006 import or `uses` names an undeclared dependency (§4.3) | sema | b | Finite-set membership: `import m` requires `pkg(m) ∈ Deps ∪ {self, core}`. |
| C14 | L1007 missing or stale `Lit.lock` (§4.2) | sema, cli | c | Compares SHA-256 digests of files on disk. |
| C15 | L1008 not exactly one `start` per package; case start invalid (§4.3) | sema | c | A count over every module of the package; no single declaration owns it. Becomes (a) only if `start` moves into `Lit.toml` or the `package` declaration (a syntax change, out of scope). |
| C16 | L1009 actor not in `[package.roles]` (§4.3) | sema | a | `Role` is a closed enum generated from `[package.roles]` (plus `either`). |
| C17 | L1010 cite unbound (§4.3, §7) | verify | c | Needs the L0 file to locate the pinpoint heading (O2). The prefix-binding half alone is (b) finite-set membership over `cite_prefixes`. |
| C18 | L2001 type mismatch (§6) | sema, lower, verify | a | Nominal typing. |
| C19 | L2001 deadline direction: versions disagree; `within d` with backward `d` (§5.5) | sema | a | Phantom direction index `Deadline<Fwd\|Bwd>`; a declaration family has one signature; `within : Deadline<Fwd>`. |
| C20 | L2002 operation not in the §6 table, IO, float, cast, overloading | sema | a | Closed operator signatures. |
| C21 | L2003 `Dist` value not total or not summing to 1; `given` on a zero-mass subset (§6) | sema, verify | b | `Dist<E>` as O8; `given(d, S)` requires `Σ_{v∈S} d(v) > 0` (QF_LRA, ground). |
| C22 | L2003 chance node probabilities do not sum to 1 (§5.1) | sema, verify | b | O8 with `E = Edges(n)`, excluding the self edge (C32). |
| C23 | L2004 mixed day units or currencies (§6) | sema | a | Phantom indices `Days<U>`, `Money<Cur>`; no conversion operator. |
| C24 | L2005 non-exhaustive match or unreachable arm (§5.2) | sema | a | Exhaustiveness over closed sums. |
| C25 | L2006 i64/u64 overflow (§6) | sema | b | `{v: Q \| -2^63 ≤ num(v) < 2^63 ∧ 0 < den(v) < 2^64}`; ground for constants. |
| C26 | L2006 division by zero; `%` on non-integers; `Days`/`Money` scaled to a non-integral result (§6) | sema | b | Divisor `{n \| n ≠ 0}`; `%` operands `Int = {n: Number \| n ∈ Z}`; `Days<U> * n : {r \| r ∈ Z}`; `Money * n : {r \| r·100 ∈ Z}`. Linear because one operand is a literal; two holes multiplied leave the fragment (13.1). |
| C27 | L2006 invalid date from arithmetic; money with more than 2 minor digits (§2, §6) | sema | b | `Date` arithmetic result `{d \| valid(d)}` over day numbers; money literal `{v \| v·100 ∈ Z}`. |
| C28 | L2006 probability arithmetic leaves `[0,1]` (§6) | sema | b | `Probability = {q: Q \| 0 ≤ q ≤ 1}`; `+ - * /` results checked against it. lg-962.3. |
| C29 | L2007 field unknown, duplicated, or wrong for the declaration kind; `outcome`/`payoff` off a terminal; `undercuts` with `=>`; `replaces` outside links (§5.1, §5.5, §8.4) | sema, lower | a | One record type per declaration kind (node kind index on fields: `Node<Terminal>` alone has `payoff`); `Exception = Rebut {…, then} \| Undercut {…}`; `replaces` exists only in the links sort. |
| C30 | L2007 `fact` tag on a node with no `p` edge (§5.1) | sema | b | Indexed by node kind and edge set: `fact` requires `kind = chance ∨ ∃e ∈ Edges(n). has_p(e)` (finite). |
| C31 | L2007 rule shapes: `duty` without a deadline, `power` without a decision, `liability` without a node or with a deadline (§5.5) | sema | b | Cardinality refinement on the rule body: `Duty: {items \| #deadline ≥ 1}`, `Power: {#decision ≥ 1}`, `Liability: {#node ≥ 1 ∧ #deadline = 0}`. |
| C32 | L2008 `p` outside `[0,1]`; `p` on a chance node's self edge; `p` missing at a chance node (§5.1) | sema | a+b | Range is (b) `Probability` (C28). Self edge and missing `p` are (a): a chance node's edges are `Edge<Chance>`, whose target is `Node \ {n}` for the distribution and whose `p` field is required (a hole inhabits it in `dev`). |
| C33 | L3001 versions of one declaration overlap (§5.4) | sema | b | O3. |
| C34 | L3002 undeclared gap in version coverage (§5.4) | sema | c | O4. |
| C35 | L3003 two claimants left unordered (§5.5) | sema | c | Claimants are every declaration naming the target in any relation, across the package and its dependencies, compared day by day over active versions after supersession; a whole-package fixpoint over activity. |
| C36 | L3004 lower-authority displacing relation (`override`, `notwithstanding`, `supersedes`, `revives`) without delegation; `delegable_to` not strictly below own level (§5.5) | sema | b | Indexed by authority: B's signature carries `Delegable<L ⊆ Levels>` per version. `A override B` (likewise each displacing relation, over the versions it acts on) checks `auth(A) ≥ auth(B) ∨ ∀v ∈ Versions(B). overlaps(v, A) → auth(A) ∈ delegable(v)` (finite set, QF_LIA overlap). `delegable_to X` requires `X < auth(self)` in the fixed order. Per declaration, so no global search. |
| C37 | L3005 exceptions on one target neither disjoint nor prioritized (§5.5) | sema | b | Family refinement: for each pair on a target, `UNSAT(C_i ∧ C_j) ∨ priority_i ≠ priority_j` (propositional SAT over `has_event` atoms; small and finite). |
| C38 | L3006 type alias, default or exception refers to itself (§5.2) | sema | c | Transitive self-reference is a cycle in the reference graph. (A direct `type T = T` alone is (a).) |
| C39 | L3006 authority order, declared wins, supersession, default/exception references cyclic (§5.5) | sema | c | O5. |
| C40 | L3006 `B@E supersedes B@D` with `D ≥ E` (§5.4) | sema | b | Version index order: `supersedes : Version<d, E> → Version<d, D> → {D < E}`. |
| C41 | L3007 no version in force at the build or event date; `@date` names no version (§4.3, §5.4) | sema, lower | b+c | Plain selection is (b): `d@t` requires `∃v. from_v ≤ t < to_v ∧ known_v ≤ K` (finite disjunction, QF_LIA). The supersession-aware form (a superseded version is not in force) is (c), as C34. |
| C42 | L3007 override or exception never in force alongside its target (§5.5) | sema | b | Interval refinement: `window(A) ∩ ∪ window(Versions(B)) ≠ ∅` (QF_LIA). |
| C43 | L3008 transition arms: not exactly one trailing `otherwise`; condition outside the allowed fragment (§5.4) | sema | a | `Arms = [Unless] ++ Otherwise`; conditions have their own type `Cond` with only `has_event`, booleans, `! && \|\| == !=`. |
| C44 | L3008 arm selects neither `d@D` nor the version it replaces; `d@D` replaces none; two transitions on one version (§5.4) | sema | b | Version indices: each arm's target `∈ {d@D, pred(d@D)}` and `∃ pred(d@D)`; uniqueness is a family refinement over transitions per amended version (`# = 1`). |
| C45 | L3009 counting: anchor/exclude combination; unknown anchor, roll kind or unit; duplicate service method; `short_period` unit (§5.3) | sema, lower | a | `Anchor = TriggerDay {exclude_trigger_day} \| NextBusinessDay` makes the illegal combinations unrepresentable; roll kinds and units are enums; service extensions are a map keyed by the closed `ServiceMethod` enum. |
| C46 | L3009 `N.hours` without `allow_hours`; backward deadline on a `next_business_day` rule; `nth` outside 1-4; unknown calendar (§5.3, §5.5) | sema | b | `Counting<hours: Bool, anchor: Anchor>` as indices: an hours deadline needs `hours = true`; `Deadline<Bwd>` needs `anchor ≠ NextBusinessDay`; `nth: {n \| 1 ≤ n ≤ 4}`; calendar is a resolved `Ref` (a). |
| C47 | L4001 case events out of source order (§5.6) | sema | b | Events list `{es \| ∀i. date_i ≤ date_{i+1}}` (QF_LIA, ground); the fix is unchanged. |
| C48 | L4001 event matches no edge or several; takes no edge without a declared effect, a firing trigger or transition history, or where the matter has ended; matches only removed edges; removal or transform without `cite`; removal repeated; discharge of another party's obligation; extension or waiver with no target, no capability, the wrong principal, after expiry, or beyond its bound (§5.6) | sema | b | Typestate (DESIGN §4.5) as an indexed type: the matter is `Case<n, R, O>` (posture node `n`, removed edge set `R`, open obligation instances `O`); an event `e` types only as `Case<n, R, O> → Case<to(edge), R, O'>` for the unique edge of `n` not in `R` whose pattern it matches, or as `Case<n, R', O'>` by its declared effect or the triggers it fires (C41). The capability check is a ground comparison of the principal and of dates against the bound. Finite and ground: one step per event. |
| C49 | L4002 act discharges an obligation instance after its due date (§5.6) | sema | c | The due date comes from counting through holiday calendars and inaccessible days from the instance's trigger: execution, not a type. |
| C50 | L4003 `facts` node not a `fact` chance node or edge not its own (§5.6) | sema | b | `facts {n: Edges(n)}` with `n : Node<Chance, fact>` (index on the referenced node's signature). |
| C51 | L4004 reachable terminal without a stake or `unvalued` (§5.6) | sema, verify | c | Reachability from the replayed start. `unvalued` being an error in `strict` is a profile rule (§7), not a type. |
| C52 | L4005 unknown party actor, duplicate party id, unknown perspective party or actor (§5.6) | sema, lower | a | `Actor` is the closed enum of the primary package's actors; `parties` is a map; `perspective` takes a party id or a map over `Actor`. |
| C53 | L4006 belief not on a chance node or edge not its own (§5.6) | sema | b | As C50 with `n : Node<Chance>`. |
| C54 | L4007 analysis field unknown, duplicated by a dedicated block, or ill-shaped (§8.3) | sema, lower | a | Type the `analysis` record against litgraph `Scenario` minus the dedicated fields (today serde at lowering does it). |
| C55 | L4008 explicit `start` disagrees with the replayed node (§5.6) | sema | c | Depends on the replay result (C48, C49). |
| C56 | L5001 verification failed (generic) | none | n/a | Registered, never emitted. Kept so codes are never reused (§9.1). |
| C57 | L5002 hole under `strict` (§7) | verify | a | `strict` types fields without the `Hole<T>` inhabitant (O14). |
| C58 | L5003 unreachable node | verify | c | O10. |
| C59 | L5004 non-terminal without an exit | verify | a | O9. |
| C60 | L5005 mandatory deadlines unsatisfiable | verify | c | O11. |
| C61 | L5006 test failed | verify | c | O12. |
| C62 | L6001 business days in an edge deadline; `observed next_day`; non-calendar service days; service method not an engine method; rule without `include_last_day` or both roll kinds (§8.1, §8.2) | lower, sema | a | Target-capability sorts: the edge `deadline` field is `Days<CalendarDays\|CourtDays>`; the lowering-target counting type requires `include_last_day` and `{weekend, legal_holiday} ⊆ roll`; `ServiceMethod` is the engine's closed enum; `Observance` without `next_day` in the engine sort. |
| C63 | L6001 manifest `counting` names an undeclared rule while some are declared; cite with no bound `url` (§8.2) | lower | b | Finite-set membership over the package's counting declarations and its bound sources. |
| C64 | L6001 clock fragment rejected by `ClockLibrary::from_json`; patching instance without its base pack loaded; patch fails `RawEdge` (§8.2, §8.4) | lower | c | Runs the engine's own loader or depends on the set of packs in the build. |
| C65 | L6002 import has no exact spelling; generated output differs from the expected document (§8.6, §8.7) | import, cli | c | Compares against an external JSON document. |
| C66 | L6003 embedded litgraph expression does not parse (§8.3, §8.4) | lower | c | A foreign language checked by litgraph's parser. Becomes (a) only if litigate types the engine expression language itself. |
| C67 | L6004 value has no finite binary64 (§8.5) | lower | b | `{v \| \|v\| ≤ f64::MAX}`; implied by C25 for i64/u64 rationals, so it guards future wider numerals. |
| C68 | L6005 output not written (§8.6) | lower | c | Filesystem effect, not a property of the source. |
| C69 | L9001 deadline or counting rule without a test (§7) | verify | c | O15. |
| C70 | L9002 hole present (§7) | verify | a | O14. |
| C71 | L9003 anonymous edge (§5.1) | verify, cli | a | Syntactic: an `_` edge has no id in the `Ref` namespace. |
| C72 | L9004 non-canonical format (§10) | syntax, cli | c | Whole-file comparison with the formatter's output, not a typing property. |
| C73 | L9005 unused declaration or import | none | c | Registered, never emitted; when added it needs whole-package use analysis. |
| C74 | L0007 reserved word used as an identifier (§2) | syntax | a | Lexical grammar. |
| C75 | L0008 unknown edition (§2) | syntax | a | Grammar: `Edition` is a closed set per compiler. |
| C76 | L4009 a duty's instance borne by a party is open past its due date at `as_of` (§5.6) | sema | c | Linearity of the instance is by construction (each ends once); whether it is past due needs the clock at `as_of`, as C49. |

### 13.4 SPEC rules with no code of their own

These rules hold today by construction or are folded into a code above;
they are listed so the v2 type system keeps them.

| # | Rule (SPEC §) | Class | Target type form (b) / reason (c) / mechanism (a) |
|---|---|---|---|
| R1 | No null, implicit conversion, subtyping or recursion (§5.2) | a | No `Null` type; no coercion rules; authority order is the only subtyping (DESIGN §4.2). |
| R2 | Event patterns in triggers and edge `event` may omit trailing arguments; `given`, case events and `has_event` arguments are complete (§5.2) | a | Two types per event: `Pattern<E>` (prefix) and `Event<E>` (complete). |
| R3 | `Money<Cur>` is integral minor units; `Money / Money` is `Number` (§6) | a | Representation plus the operator table. |
| R4 | `Probability` is a reduced rational (§6) | a | Canonical representation; `Probability` refinement is C28. |
| R5 | Field types: `p`, `effort`, `cost`/`payoff`, `duration`, `deadline`, attrs (§6) | a | Field signatures per declaration kind. |
| R6 | `duration` satisfies `min ≤ mode ≤ max` (§6) | b | `{(a, m, b) \| a ≤ m ≤ b}` (QF_LRA, ground). |
| R7 | Deadline amounts are non-negative day counts (§5.3) | b | `Days<U> = {n: Int \| n ≥ 0}` for deadline windows. lg-962.3. |
| R8 | Intervals are half-open `[from, to)` with `from < to` (§5.4) | b | `Interval = {(f, t) \| f < t}` (`..` is `t = +∞`). lg-962.3. |
| R9 | A declaration's own `effective`/`known_at` beats its rule's, which beats the package's (§5.4) | a | Elaboration order of defaults; no check can fail. |
| R10 | Selection at `(as_of, known_at)` (§5.4) | b | As C41. |
| R11 | `has_event` in a transition is evaluated against events on or before the evaluated one; false outside a case (§5.4) | a | Semantics of the `Cond` type (C43). |
| R12 | Supersession and revival semantics, bitemporal (§5.4) | c | O4. |
| R13 | Precedence stages and tie-breaks (§5.5) | c | C35. |
| R14 | Exception evaluation: target condition must hold; highest priority wins (§5.5) | a | Semantics; the ordering it needs is C37. |
| R15 | Option-removing event recognition: an event type declaring `removes p` or `elects p` with `p: String` (§5.2, §5.6) | a | A declared effect on the event type, checked when the type is built; `p` should become `EdgeRef`, making the L1002 there (C8) a type error. |
| R16 | Posture is replayed, never asserted; HIR `removals` derived (§5.6) | a | No source syntax for posture or removals. |
| R17 | `::` only in links packages, `.case` files and instance fields (§4.3) | a | Two `Ref` sorts per file kind. |
| R18 | `analysis` cannot set `start`, `perspective`, `facts`, `payoffs`, `observe` (§8.3) | a | C54. |
| R19 | `stakes` values are money or `unvalued` (§3.2) | a | Grammar. |
| R20 | A `power` rule contains the decisions it grants; a `liability` the consequence edges (§5.5) | b | C31. |
| R21 | `service_extension` applies only to forward counting (§5.3) | a | The service step exists only for `Deadline<Fwd>`; a backward deadline ignores it by type. |
| R22 | Typestate: a submission cannot be filed against a matter in a later stage (DESIGN §4.5) | b | C48. |
| R23 | Each package lowers deterministically; node order is source order (§4.1, §8.6) | c | Whole-package property of the lowering, tested by `lit-sync-check`. |

### 13.5 Summary

Counting each row once by its primary class (a mixed row counts as its
first class): obligations O1-O15 are 3 (a), 5 (b), 7 (c); codes C1-C76
are 29 (a), 24 (b), 22 (c), one n/a; rules R1-R23 are 14 (a), 6 (b),
3 (c).

- The (c) rows share six reasons: graph reachability (O10, O11, C51,
  C58, C60), cycle detection (O5, C9, C38, C39), whole-package activity
  under supersession and precedence (O4, C34, C35, C41's second half),
  execution through calendars (O11, O12, C49, C55, C61), aggregation over
  every module of a package (C15, O15, C69, C73, R23), and external
  artefacts or the profile (O2, C14, C17, C64-C66, C68, C72). None of
  these fits the 13.1 fragment without a fixpoint or an IO step, so they
  stay obligations.
- The (b) rows lg-962.3 discharges first are the ones named in that
  bead: `Probability` (C28, C32), `Interval` (R8), deadline amounts (R7),
  `Dist<E>` (O8, C21, C22) and version disjointness (O3, C33). The rest
  (C36 delegation, C37 exceptions, C44 transitions, C46 counting indices,
  C47/C48 typestate) are ground or propositional and need no new solver
  theory.
- Two (c) rows can become (a) only through a syntax change, recorded for
  review rather than proposed: C15 (`start` into the package header) and
  C66 (type the engine expression language in litigate).
