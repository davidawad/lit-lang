# Timeline JSON (`litigate.timeline/1`)

The timeline op (lg-z2m.1) prints the timeline of a `.case` or `.lit` file as
JSON. The Emacs timeline buffer (lg-z2m.2) and any other UI only render it.
The schema is frozen: [`timeline.v1.schema.json`](timeline.v1.schema.json).
Later work (the lg-fq2 change layers, guard operations) fills in more data
under the same fields; it does not add fields or bump the version.

An illustrative document, written by hand rather than produced by the
engine, is [`examples/timeline-ttab-opposition.json`](examples/timeline-ttab-opposition.json).
Its dates show the shape only; it is not a statement of any matter's deadlines.

## Top level

| Field | Meaning |
|---|---|
| `schema` | Always `litigate.timeline/1`. |
| `source` | The file the timeline was built from, and whether it is a `case` or a `lit` file. |
| `as_of` | The date statuses are judged at. |
| `known_at` | The knowledge date for bitemporal queries, or `null` for "everything recorded". |
| `layers` | Every layer that contributed a row, enabled or not. |
| `worlds` | The docket world first, then one world per alternate timeline. |
| `rows` | Events, deadlines and fork points, each in exactly one world. |
| `edges` | Links between rows of one world. |
| `diagnostics` | Errors and warnings from building the timeline. |

## Layers

A layer is one rule source applied to the matter. `class` is a closed enum:

| `class` | Source |
|---|---|
| `base_rule` | A statute, regulation or court rule package. |
| `local_rule` | A court's local rules. |
| `standing_order` | A judge's or panel's standing order. |
| `general_order` | A court-wide general order. |
| `emergency_declaration` | A closure or emergency declaration. |
| `case_order` | An order in this matter. |
| `decision` | A decision that changes how a rule applies. |

`authority` is the layer's level in its package's declared authority
lattice (`Statute`, `Regulation`, `FederalRule`, `LocalRule`, ...), as a
string, because the lattice is declared in `.lit` and not fixed by the
schema. `enabled` is `false` for a layer the caller switched off; its rows
are omitted, but the layer is still listed so a UI can switch it back on.

## Worlds

`worlds[0]` is the docket as recorded (`parent` and `fork_row` are `null`).
Every other world names its `parent` world, the `fork_row` where it
diverges, and the `choice` that defines it: `assumed_order`, `grant`,
`denial`, `hypothetical_decision`, `election` or `chance` (with
`probability` when the model has one).

A world's view is its parent's view up to and including its fork row
(following the parent's edges), plus its own rows. Rows are matched across
worlds by `key` only:

- A row the child world changes (a different `due` or `status`) is re-emitted
  with a new `id` and the parent row's `key`; it replaces the parent row in
  the child's view.
- A row the child world drops is re-emitted the same way with
  `status.state` `superseded` (deadlines) or `none` (events) and a `reason`.
  A child world never drops a row by leaving it out.
- A row that exists only in the child world has a `key` the parent lacks.
- Rows that do not change are never copied.

A UI diffing two worlds pairs rows by `key`.

## Rows

| Field | Meaning |
|---|---|
| `key` | Stable identity across worlds and runs, derived from what caused the row and never from its position, so inserting a row never renumbers others: a deadline or obligation is `<rule node id>@<key of its trigger row>`; a `.case` event is `event:<Name>@<declaration>`, where the declaration is the event's declared id, else its `<date>/<docket>` (date alone when it has no docket number), and an event a world assumes is `event:<Name>@<key of the fork row>`; a fork is `fork:<node id>@<key of its trigger row>` (a node can fork more than once in a matter, once per motion or appeal). An event without a declared id gets an info-level lint whose fix inserts a generated id, because a `<date>/<docket>` key changes when the date is corrected. Until events declare ids (lg-flu.39): an election or removal event with no docket number is `event:<Name>@<date>/<option edge id>` (the edge it elects or removes, as written); a deadline nothing has triggered is `<rule node id>@event:<Name>` of its trigger pattern; a fork's trigger row is the event that moved the matter to its node, and a fork no event moved the matter to is `fork:<node id>@start`; an event a world assumes is named by its edge's event pattern, else the edge's id, else the node the edge leads to. Two rows of one world that still share a key are an error (L1001) naming both, never told apart by position. Never derived from a label. |
| `kind` | `event`, `deadline` or `fork`. |
| `date` | When an event happened; `null` for deadlines and unresolved forks. |
| `due` | A deadline's due date; `null` when it cannot be computed. |
| `status.state` | `met`, `open`, `late`, `lapsed`, `breached`, `unknown`, `superseded`, or `none` for events. |
| `status.reason` | Why, when the state needs one; always set for `unknown` and `superseded`. |
| `cite` | The rule text the row rests on. |
| `source` | The `.lit` or `.case` span that produced the row, or `null`. |
| `layer` | The class of the layer that decided the row: the last entry of `provenance`. |
| `provenance` | Layer ids in the order they applied, base first. Never empty. |
| `trigger` | The row that started this one, or `null`. |

A deadline the engine cannot compute is a row with `due: null`,
`status.state: "unknown"` and the reason. It is never left out.

## Edges

`sequence` orders rows in one world, `triggers` links a trigger to what it
starts, `supersedes` links a reset to the row it replaces, and `forks` links
a fork row to the first row of a child world.

## Producing a timeline

```bash
litigate timeline cases/ttab-opposition-applicant-defense.case   # the envelope; the document is `result`
litigate timeline lit/ttab/ttab.lit --as-of 2026-10-19
litigate timeline cases/ptab-ipr-defense.case --world w2 --layers=-cafc
```

Over MCP the same command is the `litigate_timeline` tool (`file`, `as_of`,
`known_at`, `layers`, `world`, `lib`). The document is the envelope's
`result` (the envelope's `diagnostics` are every diagnostic of the check;
the document's are the errors and the source file's own warnings). A
matter that does not resolve, a module whose package does not check, or
a timeline with a layer whose authority is not in its lattice has no
`result`.

| Option | Effect |
|---|---|
| `--as-of DATE` | A matter is replayed and judged at `DATE` instead of its own `as_of`; a module lists the deadlines in force on `DATE` (default: its package's `known_at`). |
| `--known-at DATE` | The knowledge date, as for `check`. |
| `--layers ID,+ID,-ID` | `ID` or `+ID` enables only the named layers; `-ID` disables one. A disabled layer stays listed with `enabled: false`; its rows, the worlds forking from them and their edges go. |
| `--world ID` | Only world `ID` and its ancestors, each ancestor up to the fork row its child branches at. Row ids are the same as in the full document. |

What fills the document today:

- **Rows of a matter.** Its events (`date`, the cite of the edge the event
  took, else the event's own authority); every obligation instance replay
  created, `due` the counted (or extended) date with the replayed status
  (`discharged` is `met`; `waived`, `merged` and a restarted period are
  `superseded` with the reason); then, at the replayed posture, the
  deadlines governing its options that no event has triggered, `unknown`.
  A matter with errors (a late act) still lists its counted deadlines.
- **Worlds.** One fork row at the replayed posture (`unknown` until an
  outcome is recorded) and one world per option still open: `chance` for
  a chance node (with `probability` when authored), `election` for a
  decision or a party's move, `hypothetical_decision` for a tribunal's.
  A world holds the hypothetical event of taking the option and the
  deadlines governing the options it leads to, `unknown` because no
  recorded event dates them. `assumed_order`, `grant` and `denial` wait
  for orders and decisions as layers (lg-fq2).
- **Layers.** Each package the matter uses (or the module's package) is
  one layer, `id` and `package` its name, `class` `base_rule` unless its
  authority is `LocalRule`, `StandingOrder` or `CaseOrder`, `effective`
  its `in_force` interval; every row's `provenance` is the one layer that
  declares it.
- **Rows of a module.** Each deadline the `.lit` file declares that is in
  force at `as_of`, `unknown` (a rule module records no events).
- **Keys.** A rule or node id is module-qualified (`ttab.opposition-period`,
  `frcp.pleading-to-trial.service-window`). `.case` events have no
  declared ids yet, so an event's key is `event:<Name>@<date>` (with
  `/<docket>` when it has a docket number, else `/<option edge id>` for an
  election or removal: `event:Elected@2026-09-14/e-casetype-patent`), and
  the lint that would insert ids does not exist yet. Two rows of one world
  that still share a key (two events of one name on one date, neither an
  election nor a removal, with no docket numbers) are an error, L1001,
  naming both: the document is still emitted and the exit is nonzero.
  Where the cause has no row, the key names it anyway: a deadline nothing
  has triggered is `<rule id>@event:<Name>` of its trigger pattern
  (`frcp.periods.answer-period@event:ProcessServed`), and a fork is keyed
  by the event that moved the matter to its node (an event that only
  triggers a deadline or is history does not), else `fork:<node id>@start`.
  An event a world assumes is named by its edge's event pattern, else the
  edge's id, else the node the edge leads to.
- **Unjudged acts.** An act against a deadline whose designated due date
  the trigger left unknown (L4010) is `unknown`, `due: null`, with the
  reason `timeliness unknown: <Trigger> due date not recorded; acted by
  <event> on <date>`. The schema has no other state for it.
- **Authorities.** Each layer's `authority` must be a name in the
  authority lattice its package sees (the `authority` chains of the
  package, its dependencies and `core`). A name outside it is an error,
  L1004 at the package's `Lit.toml`, and the timeline has no `result`:
  the name is never passed through.

The Rust types are `litigate_base::timeline`; the schema bundle publishes
them as kind `timeline` (TypeScript `Timeline` in `packages/litigate-ts`).
`crates/litigate-cli/tests/timeline.rs` validates the op's output and the
example above against the frozen schema.
