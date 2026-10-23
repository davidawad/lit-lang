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
| `key` | Stable identity across worlds and runs, derived from what caused the row and never from its position, so inserting a row never renumbers others: a deadline or obligation is `<rule node id>@<key of its trigger row>`; a `.case` event is `event:<Name>@<declaration>`, where the declaration is the event's declared id, else its `<date>/<docket>` (date alone when it has no docket number), and an event a world assumes is `event:<Name>@<key of the fork row>`; a fork is `fork:<node id>@<key of its trigger row>` (a node can fork more than once in a matter, once per motion or appeal). An event without a declared id gets an info-level lint whose fix inserts a generated id, because a `<date>/<docket>` key changes when the date is corrected. Never derived from a label. |
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
