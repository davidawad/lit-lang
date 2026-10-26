# Pack schema (v5)

A **pack** is one forum's procedure as a directed graph: nodes are procedural
postures, edges are transitions (moves, rulings, draws). Packs are JSON files in
`packs/`. v1 packs (the civ-pro-the-gathering statechart format) load unchanged;
v2 adds the fields marked **v2** below; v3 adds the optional `counting` reference
to the clock library (`packs/clock/`, below); v4 the `lowered` header; v5 a
designated deadline window and extension grants (**v5** below). Every added
field is optional, so older files remain valid.

The engine never guesses silently. Anything it has to infer (a payoff from a
label, a uniform draw where probabilities are missing, an elapsed time from a
deadline) is reported back in every query response under `warnings` /
`assumptions`, so an agent always knows which numbers are authored and which
are fallbacks.

**Packs are generated.** `packs/*.json`, `packs/links.json` and
`scenarios/*.json` are build outputs of the litigate authoring language:
each pack from its `lit/<forum>/` package, `links.json` from `lit/links/`,
each named scenario from `cases/<id>.case` (`docs/litigate/SPEC.md` §9.4
lists the exact mapping; `docs/litigate/TUTORIAL.md` walks through it).
They are never committed: `crates/litgraph-packs` builds and embeds them
at `cargo build` time, and `just lit-sync` writes the same build to
`target/lit-build/{packs,scenarios}/` to read. Edit the source, never the
JSON. This document stays the
reference for the JSON the compiler emits and the engine reads (and for
packs you load from elsewhere with `--packs-dir`): SPEC §8 says which `.lit`
construct lowers to each field below.

## Top level

| field | type | notes |
|---|---|---|
| `schemaVersion` | `1` to `5` | `4` is v3 plus `lowered`; `5` adds a `designated` deadline window and `extensions` (below); the compiler always emits `5` |
| `id` | string | pack id, also the namespace prefix when composing (`cofc::complaint-filed`) |
| `title`, `description`, `jurisdiction` | string | |
| `forum` | string | **v2** short forum key: `frcp`, `frap`, `cafc`, `cofc`, `ptab`, `itc`, `uspto`, ... |
| `counting` | string | **v3** id of a counting rule in the clock library (see [Clock library](#clock-library-v3-packsclockjson)); shipped ids are `frcp6`, `rcfc6`, `frap26` and `itc210`. The `deadlines` op resolves this field and never infers clock behavior from `forum` or pack id; a pack without it, or naming an id the library lacks, gets a structured `deadline-uncomputable` warning, and `validate` rejects an unknown id |
| `startNodeId` | node id | where walks and default analyses begin |
| `groups` | `[{id,label}]` | visual/hierarchical clustering |
| `roles` | object | **v2** actor → role mapping, see below |
| `sources` | `[{id,title,url?,path?,sha256?,asOf?}]` | **v2** primary sources the pack was authored from. `cite`/`authority` strings should be traceable to one of these. `path` is **repo-relative only** (e.g. a sibling pack file) — never a local/absolute filesystem path; cite the official `url` for everything else. `litgraph lint` errors (`local-source-path`) on an absolute `path` |
| `citeSyntax` | object | **v3** the citation syntax this pack's `cite`/`authority` strings use: rule-body prefixes and pack-private shorthands, see [Citation syntax](#citation-syntax-v3). The engine itself recognizes only U.S.C., C.F.R. and case citations |
| `nodes`, `edges` | arrays | Array order of `nodes` is not observable: the engine visits nodes in canonical (qualified id) order, whatever order packs are listed in, and a node's out-edges in this pack's `edges` order, then its `links.json` links by id |
| `filingLimits` | array | **v5, optional** each paper kind's filing requirements in force at the build dates, as a litigate `limit` lowers them (`docs/litigate/SPEC.md` §3.7): `{id, cite?, bounds?: [{measure: words\|pages\|lines, max?, cite?}], certificates?: [{kind, for?: [measure], cite?}], excludes?: {items: [part], cite?}, ecf?: [{code?, cite?}]}`. Bounds are alternatives (a paper within any one is within the limit); `lines` counts monospaced lines; a certificate with `for` is required only of a paper relying on those bounds. A value the sources do not settle is absent and listed in `lowered.holes` (`/filingLimits/i/bounds/j/max`, `/filingLimits/i/excludes/open/n`, `/filingLimits/i/ecf/j/code`). Data for checking a filed paper; no analysis reads it |
| `lowered` | object | **v4** build provenance of a compiled pack, see [The `lowered` header](#the-lowered-header) |

## The `lowered` header

Every document the compiler generates (pack `schemaVersion` 5, `links.json`
3, named scenario schema 2, clock fragment `schemaVersion` 4) carries a
`lowered` object; a hand-authored document has none. It is provenance,
not meaning: `litigate diff` ignores it (SPEC §8.6).

| field | type | notes |
|---|---|---|
| `pins` | `[{name,version,digest}]` | every package of the world the document was built in (the package and its dependencies; for a matter, the packages it `uses` and theirs), sorted by name; `digest` is the package's `sha256:<hex>` source digest |
| `asOf` | date | law-in-force date versions were selected at |
| `knownAt` | date | corpus date versions were selected among (absent: the latest) |
| `worldDigest` | `sha256:<hex>` | SHA-256 of the JCS text (RFC 8785) of `{"asOf", "knownAt", "pins"}` (`knownAt` omitted when absent): the identity of the world |
| `holes` | `[{pointer,reason?}]` | every explicit `?` the source left in an emitted field: `pointer` is the JSON pointer of that field in this document (`/edges/3/probability`, `/nodes/0/cite`, `/instances/cafc@cofc-gov/stake`, `/scenario/payoffs/<terminal>`), `reason` the `?("reason")` text. The field itself is absent, or for a cite or source date holds the reason text |
| `basis` | `[{pointer,basis,source?}]` | where numbers come from (design review 2026-10-23 §2.10), `basis` one of `sourced` < `calibrated` < `expert` < `estimate` < `hole` (best first; the join of two labels is the worse). A pack or link file labels each node or edge (`/nodes/i`, `/edges/i`, `/links/i`) whose source declares a `basis`, `source` the cite, `dataset@date` or expert; a number with no label is an `estimate`. A scenario has one label at `/scenario`: the join over every number of the packs it uses, a hole where the source wrote `?` and the probabilities of a node the matter gives a `prior` its own estimate |

Reading a document checks the header: `lowered` needs the schema version
above, `pins` must be sorted and unique, `worldDigest` must match its pins
and dates, and every hole and basis label must point into the document
(array indices in bounds; the field itself may be absent). A pack whose holes leave a number
open (`probability`, `cost`, `hours`, `payoff`, `payoffByFlag`, `duration`,
`attrs`, or an instance `stake`) adds one `authored-hole` warning per pack
to every response that compiles it: the number rests on a fill, not on the
pack. A `strict` build rejects holes, so only `dev` output carries them.

Generated files are written as RFC 8785 (JCS) canonical JSON, indented by
two spaces: members sorted by UTF-16 code units and numbers in ECMAScript
spelling (`1500`, not `1500.0`; `1e+21`). Removing the indentation gives
the exact JCS text that digests and litigatable hashes are taken over.

## Nodes

| field | type | notes |
|---|---|---|
| `id` | string | unique within pack, kebab-case |
| `kind` | `state` \| `decision` \| `terminal` | `terminal` = the matter ends here (for this pack) |
| `label` | string | |
| `cite`, `note`, `group`, `valence` (`good`/`caution`/`bad`/`neutral`), `courtListenerUrl` | | as v1 |
| `payoff` | number (USD) | **v2**, terminals only. Value of ending here **from the protagonist's (role `self`) perspective**, before costs already spent. Authored payoffs beat the engine's label heuristic, which only understands patent-prosecution vocabulary |
| `outcome` | string[] | **v2** machine tags for terminals, e.g. `["win","judgment","fee-eligible"]`, `["loss","procedural-default"]`, `["settlement"]`, `["remand"]`. Custom cost/utility functions can branch on these (`tag("fee-eligible")`) |
| `tags` | string[] | **v2** free-form node tags for any node (not just terminals), e.g. `["entry","router"]`. Expressions see a node's `outcome` ∪ `tags` as one set: `tag("x")` on a terminal, `to_tag("x")`/`from_tag("x")` on an edge testing its target/source node |
| `attrs` | `{string: number}` | **v2** free numeric attributes exposed to custom functions as `node.<name>` |
| `payoffByFlag` | `{string: number}` | **v2**, terminals only. Overrides `payoff` on a flagged product-graph copy of this terminal: if the copy carries one of these keys as a set flag, its payoff is that value instead of `payoff`. See [State flags](#state-flags) |
| `feeShift` | number in `[0, 1]` | **v2**, terminals only. Fee shifting by the governing statute's default: the fraction of our accumulated cost (the litigation hours and the fee motion's own) a fee award recovers when the matter ends here. `payoff` stays the gross amount; `solve`/`simulate` offset the cost of reaching it. A scenario's `fee_shift` replaces it at every terminal, its `fee_shifts` at one. See [docs/COST_FUNCTIONS.md](COST_FUNCTIONS.md#fee-shifting) |

## Edges

| field | type | notes |
|---|---|---|
| `id` | string | **v2** stable edge id, unique within pack. If absent the loader derives `from->to#n` (n = 0-based index among parallel edges). Author it whenever two edges share `from`/`to` |
| `from`, `to`, `label` | string | |
| `actor` | `applicant` \| `examiner` \| `office` \| `either` | v1 vocabulary kept for compatibility. Interpreted through `roles` |
| `authority` | string | the rule/statute that authorizes or compels this transition |
| `deadline` | `{length, unit?, extendable?, extensionAuthority?, note?, counting?}` or **v5** `{designated, extendable?, extensionAuthority?, extensions?, note?, counting?}` | `length` in `unit`s. **v5** `designated` `{minPeriod, sourceEvent, sourceEventArg, interpretation?}` replaces `length` (exactly one of the two): the due date is the `Date` argument `sourceEventArg` of the triggering notice or order `sourceEvent` in each matter, not a counted period (37 C.F.R. § 2.105(a)); `minPeriod` is the least time in calendar days from the trigger's date (`unit`, if given, is `calendar`), and `interpretation`, when present, says the rule names no start for it, so a shorter designation stands with a warning instead of being rejected. Such a window has no length for metrics (`days` is `0`, `has_deadline` is `1`), and the `deadlines` op reports it as `deadline-uncomputable` rather than count its least time. **v5** `extensions`: the rule's extension grants, each tagged by `bound`: `bounded` `{by, until, afterExpiry?, standard?, motion?, interpretation?}` (no later than date bound `until`), `discretionary` `{by, standard, window, motion?, interpretation?}` (any later date; `window` `beforeExpiry` or `afterExpiryOnMotion`), `nonExtendable` `{authority}` (rule `authority` forbids extending beyond the other grants). `standard` is `GoodCause`, `ExcusableNeglect`, `GoodCauseOrExcusableNeglect` or `ExtraordinaryCircumstances`; a date bound is `{"offset": {"from": R, "days": N}}` (N days after) or `{"before": {"from": R, "days": N}}` (N days before), `R` one of `prescribedDue`, `grantOrderEntered`, `currentDue` (the due date as last extended), or `{"max": [...]}` or `{"min": [...]}`; a `motion` is `{event, within?, unless?}`, `event` the motion's event pattern and `unless` the event that, once it precedes the order, bars the grant; a grant's `interpretation` is the reading it rests on when the rule reaches the window only as its author reads it (FRCP 6(b) through 37 C.F.R. § 2.116(a)). An absent `extendable` and no `extensions` are undeclared, never "not extendable". **v3** `counting`: the clock-library rule this window is counted by when it is not the pack's `counting` (a FRAP 4 period in a district-court pack, a certiorari period counted by `sup-ct-30`); a period is counted by the rule of the forum whose rules set it. **v2** `unit`: `calendar` (default) or `court` (court days); `week`, `month` or `year` (calendar months, counted by the rule's `months` provision; metrics read a month as 146097/4800 days and a year as 365.2425). The `deadlines` API op turns this into a concrete due date from a trigger date using the pack's v3 `counting` reference (see `docs/ARCHITECTURE.md`'s "Deadline clock") |
| `duration` | `{min?, mode, max?}` days | **v2** expected elapsed calendar time for this transition. Distinct from `deadline` (a window you must act inside). If absent the engine falls back to `deadline.length` and says so |
| `cost` | USD | out-of-pocket fees (filing fees, USPTO fees, bonds) |
| `hours` | number | attorney hours to execute (self/opponent moves only) |
| `probability` | 0..1 | only on non-`self` edges. Out-edges of a pure chance node must sum to 1 |
| `valence`, `courtListenerUrl`, `actionId`, `note` | | as v1 |
| `tags` | string[] | **v2** e.g. `["dispositive","sanctions","waiver-trap","settlement","appeal"]` |
| `attrs` | `{string: number}` | **v2** free numeric attributes for custom functions (`edge.<name>`), e.g. `{"opp_hours": 40, "fee_award_prob": 0.3}` |
| `replaces` | string[] | **v2, `links.json` only** — qualified (`pack::edge-id`) local edge ids this link edge supersedes when it's active. The compiled graph drops every listed edge once the link is loaded; error if any id doesn't resolve. Invalid on a pack's own edges (packs describe one forum; superseding is a composition concern, so it belongs in `links.json`) |
| `sets`, `clears` | string[] | **v2** state flags this edge sets/clears when taken. See [State flags](#state-flags) |
| `requires`, `forbids` | string[] | **v2** flags that must all be set / all be absent for this edge to exist in the compiled graph. See [State flags](#state-flags) |

## Roles (v2)

The game-theoretic reading of `actor`. Default mapping (the v1 semantics):

```json
"roles": { "applicant": "self", "examiner": "nature", "office": "nature", "either": "nature" }
```

Roles: `self` (the protagonist whose policy is optimized), `opponent` (an
adversary with its own choices — solved as a minimizer or with its own
objective, per query), `nature` (tribunal/agency/chance — draws per
`probability`). A pack whose `examiner` column is really "opposing party"
should say `"examiner": "opponent"`. Queries can override the mapping
(`perspective`), e.g. analyze the same FRCP pack as the defendant.

In packs where "applicant" is not a patent applicant (FRCP, CoFC, FRAP), the
convention is: `applicant` = the party whose moves we are analyzing (usually
plaintiff/appellant), `examiner` = the opposing party, `office` = the court or
clerk, `either` = either party / events not controlled by one side.

## Matter facts (v2)

Some chance nodes aren't real uncertainty — they're a **fact about this
matter** that just hasn't been told to the engine yet: whether a claim was
filed more than six years after accrual (`28 U.S.C. §2501`), whether the
same claim is already pending elsewhere (`28 U.S.C. §1500`), whether a
judgment arises under the patent laws (`28 U.S.C. §1295(a)(1)`). Modeling
these as an ordinary unauthored chance node makes them 50/50 (or whatever
`scenario.prob_fill` does), which is never right: the true answer is
either 0 or 1, the pack's author just doesn't know which until the
scenario says so.

**Tag the node** `"tags": ["fact"]` and **author a prior** — a labeled
estimate or sourced base rate — as the `probability` on every one of its
out-edges, same as any chance node, with the basis in each edge's `note`
(e.g. "estimate: counsel screens for this before filing"). `litgraph lint`
flags a `fact`-tagged node whose out-edges aren't all authored
(`fact-no-prior`) — the whole point of tagging it is that a real prior,
not a 50/50 default, should be there.

At query time, a scenario that knows the actual fact sets
**`scenario.facts`**: `{node ref: edge ref}`, exactly the shape of
`policy` (which forces *our* choice) but for a fact node nature controls —
it forces that edge's probability to 1 and its siblings to 0 (via the same
sibling-rescale `scenario.probabilities` already does; `facts` is sugar
over it, keyed by node so an agent doesn't have to compute the edge id by
hand). A `fact`-tagged node with no `scenario.facts` entry falls back to
the authored prior and a `fact-unset` warning names it, so "I don't know
this matter's facts yet" is always visible in the response, never silent.

```json
"scenario": { "facts": { "cofc::limitations-check": "cofc::e-limitations-timely" } }
```

Use `tags: ["fact"]` (not a new node `kind`) precisely because a fact node
is structurally an ordinary chance node — same edges, same probabilities,
same `fill` semantics — only its *epistemic status* differs, and every
existing algorithm, lint check and scenario field keeps working unchanged.

The pack schema also admits a **mixed** fact node (an applicant choice
alongside the fact-driven edges): `scenario.facts` forces the right edge
exactly as for a pure-chance fact node (it's the same
`scenario.probabilities` override underneath), and `litgraph lint`'s
`fact-no-prior` check and the `fact-unset` response warning both look at
just the non-applicant (interrupt) edges, so a mixed fact node needs the
same full authored prior on those and gets the same `fact-unset`
(not the generic `mixed-node`) warning when unset. No shipped pack has
one: the litigate language has no mixed nodes and allows `fact` only on
`chance` nodes (SPEC §5.1), so a fact and the choice that follows it are
two nodes, e.g.
`ptab-patent-trial-appeal-board.json`'s `petition-time-bar-check` (is the
petition time-barred under 35 U.S.C. §315(b)?) ahead of the Office's
compliance review and the petitioner's correct-and-refile choice. Its
value is computed, not drawn: the litigate source guards its edges with
the petition and service dates (`when`, SPEC §5.1), and a `.case` whose
events decide it lowers the derived edge to `scenario.facts`.

A matter fact is true regardless of procedural history, so it applies to
every [state-flag](#state-flags) product-graph copy of the node, not just
the one whose id happens to match `node_ref` exactly: `scenario.facts`
resolves `node_ref` to one node, then forces the same edge on every node
sharing its `base_id` (`Graph::node_family`/`Graph::edge_family`) — the
same reasoning as calibration's `Edge.base_id` handling above.

## Clock library (v3/v4, `packs/clock/*.json`)

Deadline counting is data. `packs/clock/` holds the **clock library**:
holiday calendars and counting rules, shared by every pack. A forum
package's own counting rules and calendars are generated into the built
`packs/clock/<pack file>.json` (today `mpep-prosecution.json`, 37 C.F.R.
§ 1.7, and `ttab-trademark-trial-and-appeal-board.json`, 37 C.F.R. §
2.196; a build output, not a file in the repository); the shared
fragments (`frap26`, `frcp6`, `frcrimp45`, `itc210`, `rcfc6`, `sup-ct-30`,
`us-federal-holidays`) are still hand-authored (SPEC §9.4). Each file is a
fragment:

```json
{ "schemaVersion": 3, "calendars": [ ... ], "countingRules": [ ... ] }
```

A generated fragment is `schemaVersion` 4 and carries a
[`lowered` header](#the-lowered-header); both versions are read.

Fragments are embedded at build time (key `clock/<file>`; the embedded
packs carry the hand-authored and generated fragments together) and read
from `<packs dir>/clock/` by `--packs-dir`/`LITGRAPH_PACKS`; in-memory
catalogs (`Catalog::from_files`, `litigate-wasm`'s `handle_with_packs`)
take them as files named `clock/<file>.json`. A source with no `clock/`
files gets the embedded hand-authored library (no generated fragment). Fragments merge in file-name order; a fragment that fails
to parse or validate, or redefines an id already loaded, is skipped and
listed in `load_errors`, as is a rule naming an unknown calendar. A
calendar a fragment restates unchanged is no redefinition: each fragment
whose rules close on a shared calendar carries it. Every
object is strict (`deny_unknown_fields`). `litgraph validate` detects the
kind `clock`; `litgraph schema clock` prints its JSON Schema.

### `HolidayCalendar`

| field | type | notes |
|---|---|---|
| `id`, `title` | string | `id` is what a rule's `holidays` names |
| `cite`, `sourceUrl` | string? | authority for the calendar |
| `holidays` | `[Holiday]` | ids unique within the calendar |

`Holiday`:

| field | type | notes |
|---|---|---|
| `id`, `name` | string | |
| `date` | `{"fixed": {month, day}}` or `{"nthWeekday": {month, weekday, nth}}` | `weekday` is `sunday`..`saturday`; `nth` is 1..=5, or -1..=-5 from the end of the month. A fifth weekday that does not exist in a month is skipped |
| `cycle` | `{every, anchorYear}`? | occurs only in years `anchorYear + k*every` (e.g. every 4 from 1965) |
| `observed` | `actual` (default) \| `nearest-weekday` | `nearest-weekday`: Saturday is observed Friday, Sunday Monday |
| `effective` | `{from?, until?}` | half-open `[from, until)` ISO dates, tested against the **observed** date |
| `cite` | string? | authority for the holiday or its effective date |

A date is a legal holiday under a rule if it is an observed holiday of any
of the rule's `holidays` calendars in that year or a neighboring one (an
observance can cross Dec 31/Jan 1), or one of the request's
`additional_holidays`; for a period counted forward, also of its
`forwardHolidays` calendars.

### `CountingRule`

| field | type | notes |
|---|---|---|
| `id` | string | what a pack's `counting` names; reported as `rule_set` |
| `label`, `sourceUrl` | string | reported as `rule_set_label` / `source_url` |
| `closedWeekdays` | `[weekday]` | days the forum is closed (a last day on one rolls) |
| `holidays` | `[calendar id]` | calendars the forum closes on (unioned), in either counting direction |
| `forwardHolidays` | `[calendar id]`? | calendars that close it only for a period counted forward (FRCP 6(a)(6)(C), FRAP 26(a)(6)(C): "for periods that are measured after an event") |
| `minLength` | `{days, message}`? | shorter periods fail with `message` |
| `steps` | `[Step]` | the pipeline below |
| `hours` | `{text, rollText, inaccessibleText?}`? | hours provision: count every hour from the trigger instant, then continue to the same time on the next open day. With `inaccessibleText`, a request setting `clerk_inaccessible` (office inaccessible during the last hour) continues to the same time on the next open day after that, reported as `clerk_inaccessibility_applied`; without it the request is ignored. Absent: hours-based periods are rejected |
| `months` | `{endOfMonth, text}`? | calendar-month provision: a `month` or `year` (twelve months) period runs from a `trigger` anchor to the corresponding day of the later month; when that month has no such day, `endOfMonth` `clamp` ends it on the month's last day and `roll` on the next month's first. The `count` step's periods are skipped; later steps apply. Absent: month and year periods are rejected. A `week` is seven calendar days under the `count` step's `calendar` period |

`steps` run in order over a working date. `anchor` must come first and
`count` second; `base` and `service` occur exactly once;
`clerkInaccessible` at most once; `roll` any number of times. Some weekday
must be open, or a roll would never end.

| `op` | fields | effect |
|---|---|---|
| `anchor` | `kind`, `text` | `trigger`: the trigger day is excluded (day 1 is the next day). `adjacent-business-day`: the first business day after (forward) or before (backward) the trigger is day 1; requires `minLength.days >= 1` |
| `count` | `periods: [{unit?, below?, count, text}]` | the first period whose `unit` (`calendar`/`court`, the deadline's authored unit; a week is 7 calendar days) and `below` (period shorter than this) match counts `n` days in `count` units: `calendar` (every day) or `business` (open days only). No match is an error |
| `service` | `methods: [{method, days}]`, `unrecognized?: [method]`, `text`, `backwardText` | total over the service methods (`personal`, `electronic`, `mail`, `clerk`, `other-consented`, `commercial-carrier`, `prison-mail`, `mail-foreign`, `express`, `express-foreign`): each is in exactly one list. A forward period's service method in `methods` adds `days` (with `days: 0` it moves nothing and emits nothing); one in `unrecognized` is an error naming the recognized ones; a backward period ignores it with `backwardText` |
| `base` | | records the working date as `base_date` |
| `roll` | `forward`, `backward`: `{toward, text}` | moves a closed day `later` or `earlier` (chosen by counting direction); `text` is emitted, and `last_day_rolled` set, only if the date moved |
| `clerkInaccessible` | `text?`, `ignoredText` | when the request sets `clerk_inaccessible`, a period moves to the next open day after the last day (`text`), regardless of counting direction; without `text`, `ignoredText` is emitted |

Step `text` templates may use `{trigger}`, `{anchor}`, `{date}` (this
step's result), `{n}` (period length), `{after}` (`after`/`before`),
`{forward}` (`forward`/`backward`), `{method}` (the service method's name,
e.g. `Mail`), `{days}` (days the method adds) and `{unit}` (`month(s)` or
`year(s)`, in the `months` text). Any other placeholder fails validation.
The result reports `nominal_date`, the period's last day as counted before
any roll or extension, beside the final `due_date`.

## Citation syntax (v3)

`citeSyntax` declares which rule bodies and shorthands `lint`'s cite
verifier recognizes in this pack. Forum-neutral shapes (`28 U.S.C. §
1498(a)`, `37 CFR 1.104`, `Bowles v. Russell, ...`) need no declaration;
every rule body does, so a new forum never needs an engine change. A pack
reads only its own declarations: a pack citing `FRAP 4` must declare FRAP
itself, or the citation is treated as unrecognized text.

```json
"citeSyntax": {
  "rules": [
    { "prefixes": ["FRCP", "Fed. R. Civ. P.", "Fed R Civ P", "F.R.C.P."],
      "heading": "Rule {}", "source": "frcp-rules" }
  ],
  "shorthands": [
    { "prefix": "cfr", "code": "cfr", "title": 19, "sectionParts": 2,
      "source": "cfr-210" }
  ]
}
```

| field | type | notes |
|---|---|---|
| `rules[].prefixes` | string[] | surface prefixes, matched case-insensitively at the start of a citation (one space in a prefix matches any run of spaces), then an optional `§` and the section number (`FRCP 12(b)(6)`) |
| `rules[].heading` | string | the vendored corpus heading (`## <heading>`) a section resolves to, with exactly one `{}` for the section number. Forms with the same heading are the same body, so a bare trailing number carries forward across them (`FRAP 28; 34`) |
| `rules[].source` | source id | optional; the `sources[]` entry this body normalizes into |
| `shorthands[].prefix` | string | a compact dotted form's literal prefix (`cfr210.16.b.4`) |
| `shorthands[].code` | `usc` \| `cfr` | the code the implied title belongs to |
| `shorthands[].title` | integer | the implied title number (nonzero) |
| `shorthands[].sectionParts` | integer | how many leading dotted parts form the section: `2` makes `cfr210.16.b.4` 19 C.F.R. § 210.16(b)(4); `1` makes `usc1337.b.1` 19 U.S.C. § 1337(b)(1). The first part must be all digits |
| `shorthands[].source` | source id | optional, as for `rules` |

Loading rejects a heading without exactly one `{}`, a rule with no
prefixes, an empty prefix, a zero `title` or `sectionParts`, a `source`
that is not a `sources[].id`, and two prefixes (rule or shorthand) where
one is a case-insensitive prefix of the other — so declaration order never
changes how a citation parses. Resolution still searches every vendored
`sources[].path` the pack declares.

Engine source carries no forum knowledge: the
`no_forum_literals_in_engine` test (`crates/litigate-engine/tests/no_forum_literals.rs`)
fails if any pack id or forum key from `packs/*.json` appears in
`crates/*/src` outside test code.

## Links (composition)

`packs/links.json` joins packs into one multi-forum graph. Its top-level
shape is `{"links": [...], "instances": {...}}`; the generated file adds
`"schemaVersion": 3` and a [`lowered` header](#the-lowered-header) (absent,
the version is `1`; `2` adds the header, `3` a link `deadline`'s
`designated` window and `extensions`, as pack v5).

```json
{ "links": [
  { "id": "cofc-to-cafc", "from": "cofc::judgment-entered", "to": "cafc::notice-of-appeal-filed",
    "label": "Appeal to the Federal Circuit", "actor": "applicant",
    "authority": "28 U.S.C. § 1295(a)(3); 28 U.S.C. § 2522",
    "deadline": { "length": 60 } }
]}
```

When composing, a terminal with outgoing link edges becomes a choice node with
an implicit zero-cost `accept` edge to a copy of itself holding its payoff
(for `self`-controlled links), so "stop here or appeal" is a real decision.

A link edge only loads once both endpoints' packs are loaded; it's silently
skipped otherwise, so a two-pack request never sees a link into a third,
unloaded pack. A link can carry `replaces` (see the edge table above) to
supersede a pack's own edge — for instance, a link representing "appeal to
the Federal Circuit" superseding a same-pack placeholder terminal edge.

## State flags

**v2.** Litigation has memory: an IPR estoppel, a waived Rule 12(h) defense, a prior
RCE all change what can happen next without changing where you structurally
are. A plain graph can't express "this edge only exists if X happened
earlier" — `docs/CRITIQUE.md` calls this the *Markov on the node* limit.
State flags are the fix: `sets`/`clears`/`requires`/`forbids` on an edge
(pack or `links.json`), compiled at load time into a **product graph** over
`(node, flag-set)`.

```json
{ "from": "fwd-issued", "to": "fwd-all-unpatentable", "label": "...",
  "sets": ["ipr-estopped"] },
{ "from": "later-invalidity-defense", "to": "...", "label": "...",
  "forbids": ["ipr-estopped"] }
```

- **`sets` / `clears`** — flags this edge adds to / removes from the current
  flag-set when taken. Flags are plain strings, not declared anywhere up
  front; a typo just means the flag is never set (no error, but `litgraph
  lint` can flag a `requires`/`forbids` that references one no edge ever
  sets).
- **`requires` / `forbids`** — gate the edge: it only exists in the compiled
  graph from a state where every `requires` flag is set and every `forbids`
  flag is absent. An edge whose `forbids` condition is always true wherever
  it's reachable (e.g. immediately after the flag that forbids it) simply
  never compiles anywhere — the block is absolute, not a soft preference.

**Compilation.** Every base node keeps its own empty-flag copy at its
original id and index — so a pack that never uses flags compiles to an
*identical* graph (`Graph::compile` skips the whole mechanism when no loaded
edge declares any of the four fields; a pack with a `sets`/`clears`/
`requires`/`forbids` field on `RawEdge` is still ordinary v1/v2 JSON,
these are additive, defaulted fields), and any node stays directly
addressable as `scenario.start` regardless of true reachability, same as
before flags existed. From there, a worklist walks `(node, flag-set)` states
reachable by forward flag propagation; the first time a state is reached its
node is materialized as `pack::local{flag1,flag2}` (flags sorted,
comma-joined — e.g. `ptab::fwd-issued{ipr-estopped}`), stable and readable.
**Only reachable combinations are materialized** — a pack with one flag used
in one corner of the graph gets one small pocket of duplicated nodes, not a
combinatorial blowup. A hard cap (`max_product_nodes`, default 20,000;
`CompileOptions.max_product_nodes` / request `max_product_nodes`) fails
compilation with a clear error naming the state it choked on, rather than
compiling forever or truncating silently.

**Everything downstream is unchanged.** Every algorithm (`solve`, `chain`,
`simulate`, `path`, `pareto`, `sweep`, `structure`) sees the product graph as
an ordinary graph — flags are compiled away into plain nodes before any
algorithm runs, so none of them know flags exist. A compiled `Node` carries
`flags` (its flag-set) and `base_id` (the unflagged id it's a copy of), so a
response can always project a flagged node back to the pack node an author
wrote (`api::render::node_ref` does this whenever `flags` is non-empty). A
compiled `Edge` carries `base_id` too (the pre-expansion edge it's an
instantiation of) — every flagged instantiation of the same authored edge
shares one `base_id`, which is how a calibration entry (`docs/CALIBRATION.md`)
targets one edge by its authored id and lands on every flagged copy at once,
not just the one whose exact id matches.

**`explain` groups a node's copies.** At a node with flagged copies,
`explain` details one copy and summarizes the rest. A base id resolves to
the replayed context: the copy carrying the start's flags (`from`, else
`scenario.start`) when the start reaches it (`context.basis: "replayed"`),
else the copy the start reaches in the fewest steps (`"reached"`), else
the base node itself (`"unreached"`); a flagged id is explained as named
(`"named"`). `other_contexts` lists the other copies the start reaches,
nearest first (then by id), each with its flags, `steps`, `value`,
`chosen` option and whether that option instantiates the same authored
edge (`same_choice`), and counts the copies it never reaches
(`unreached`). A node without flagged copies has neither field.

**Conditional chance draws.** A chance node's out-edges may carry
`requires`/`forbids` too, so an earlier choice changes the odds at a node
every option reaches: each product-graph copy keeps only the edges its
flag-set enables, and those are its distribution. `litgraph lint`'s
`probability-sum` checks the sum once per context (each set of the node's
gating flags, up to 12; a context enabling no edge is skipped) and names
the context that fails.

**Reading flags in expressions.** `flag("x")` is available in edge
(cost/mask/probability-transform) and terminal (utility) expressions — see
`docs/COST_FUNCTIONS.md`. On an edge it reads the source node's flag-set; on
a terminal it reads that terminal's own flag-set.

**Terminal payoff by flag.** A terminal can carry `payoffByFlag` (Nodes
table above): when a flagged copy of that terminal is materialized, a
matching key overrides `payoff` for that copy specifically (first match in
sorted key order if more than one flag matches — author mutually exclusive
flags). This is how `cafc-federal-circuit.json`'s `cert-not-sought`/
`cert-denied` restore the panel's actual win/loss through a rehearing-or-cert
detour that would otherwise re-zero it (`panel-affirmed`/`panel-reversed`/
`panel-mixed`, set on `panel-decision`'s outgoing edges and cleared on
`rehearing-granted-to-decision` since a fresh disposition replaces the one
being reheard).

**A reachability note.** Reachability lint (`litgraph lint`'s `unreachable`
diagnostic) checks per *base id*, not per exact compiled copy: a node that's
only ever reached with a flag set (nothing reaches its empty-flag copy in
practice) is not flagged as unreachable as long as *some* compiled copy of
it is reachable from the pack's start.

**Relation to `links.json` instances.** An instance (below) is a coarser,
same-mechanism cousin: both are "the product-graph construction for one
piece of history, expressed as data instead of code" (that phrase up front
in the Instances section was written before flags existed and is still
accurate for instances). An instance is a whole namespaced *copy of a pack*
— appropriate when the history changes which pack you're even in (CAFC
entered from the PTAB vs. from the CoFC needs a different remand target and
a flipped perspective) or needs its own `payoff_transform`/`roles` override.
A flag is a *value on one edge* — appropriate for memory that lives inside
one pack's own graph (an estoppel, a waiver, a count) and needs no identity
or perspective change. The two compose freely: an instance's own edges can
carry `sets`/`clears`/`requires`/`forbids` like any other edge (materialized
once the instance is compiled), and a link edge in `links.json` can too.
Reimplementing instances purely on top of flags was considered and rejected:
an instance's `remove_edges`/`probabilities`/`roles`/`payoff_transform`/
`patch_edges` rewrite a whole pack's data before compilation even starts,
which flags (a runtime product over an already-compiled edge set) have no
mechanism for.

## Instances (`links.json`, `"instances"`)

A **pack instance** is a namespaced copy of a base pack that remembers *how
it was entered* — the product-graph construction for one piece of history,
expressed as data instead of code. The Federal Circuit is one pack (`cafc`)
but four instances: entered from the Court of Federal Claims as the
plaintiff-appellant (`cafc@cofc`), from the CoFC as the government-appellant
(`cafc@cofc-gov`, perspective flipped), from the PTAB (`cafc@ptab`), and
from the ITC (`cafc@itc`) — each remanding only back to its own origin
forum, which the shared `cafc` pack alone can't express.

```json
{ "instances": {
  "cafc@cofc-gov": {
    "pack": "cafc",
    "note": "Federal Circuit as entered by the United States appealing a CoFC plaintiff win. Perspective flipped: the government (appellant) is the opponent; payoffs are the plaintiff's (stake − appellant payoff).",
    "remove_edges": ["origin-to-district-court", "origin-to-ptab", "origin-to-itc",
                      "remand-route-district-court", "remand-route-ptab", "remand-route-itc"],
    "probabilities": { "remand-route-cofc": 1.0 },
    "roles": { "applicant": "opponent", "examiner": "self" },
    "payoff_transform": "stake - payoff",
    "patch_edges": {
      "rehearing-file": { "deadline": { "length": 45, "note": "45 days when the United States is a party (Fed. Cir. R. 40)." } }
    }
  }
}}
```

| field | type | notes |
|---|---|---|
| `pack` | string | base pack id to copy |
| `note` | string | why this instance exists — required in practice, since the whole point is to say what's different |
| `remove_edges` | string[] | local edge ids (as the compiler derives them: authored `id`, else `from->to#n`) to drop from this instance |
| `probabilities` | `{edge id: p}` | override/author a probability on a local edge in this instance only |
| `roles` | `{actor: role}` | override the base pack's `roles` for this instance (a perspective flip) |
| `payoff_transform` | expression over `payoff` and `stake` | rewrites every terminal payoff (and `payoffByFlag` value), e.g. `"stake - payoff"` to convert an appellant's payoff into the original plaintiff's stake-relative payoff |
| `stake` | number (USD) | the value of `stake` in `payoff_transform`. Absent while the transform uses `stake`, every terminal payoff of the instance is left unauthored (the engine warns `payoff-not-authored`): what an appeal puts at stake is a fact of the matter, so `cafc@cofc-gov` leaves it to the matter's `scenario.payoffs` |
| `patch_edges` | `{edge id: JSON merge-patch}` | RFC 7386 merge-patch applied to one local edge's fields (deadline, cost, hours, tags, ...) — `null` deletes a field, an object merges, anything else replaces |

Referencing an edge id `remove_edges`/`probabilities`/`patch_edges` doesn't
recognize is an error naming the unknown id — instances are checked at
compile time, same as everything else.

## Scenario library (`scenarios/*.json`)

Generated from `cases/*.case` (see the note at the top). A **named scenario** is a matter profile: a human summary/notes, the packs it
needs, its sourced facts, and the engine scenario itself (rates, stakes,
perspective, masks, payoffs, probabilities, policy, modeling modes — the same
`scenario` object a request carries; `litgraph schema scenario` gives its
full shape). It is embedded into the binary the same way packs are
(`crates/litigate-engine/build.rs`); `$LITGRAPH_SCENARIOS` or `--scenarios-dir`
override it at runtime, independently of `$LITGRAPH_PACKS`/`--packs-dir`.

| field | type | notes |
|---|---|---|
| `id` | string | name a request references (`"scenario": "<id>"`); by convention matches the file stem |
| `summary` | string | one line; `litgraph describe`'s `scenario_library` lists every scenario by this |
| `notes` | string | facts, the basis for every rate/stake/probability (sourced vs illustrative), and any modeling judgment calls |
| `packs` | string[] | pack refs this scenario needs. A request naming this scenario with no `packs` of its own uses these; an explicit request `packs` always wins |
| `sources` | `[{title,url?,as_of?}]` | sourced facts backing this scenario's numbers |
| `scenario` | object | the engine scenario (`litgraph schema scenario`) |
| `schema_version` | `1` or `2` | absent is `1`; `2` adds the three fields below, and is what the compiler emits |
| `parties` | `[{id,actor,entity_type}]` | **v2** the matter's parties and the pack actor each plays |
| `events` | `[{date,docket?,action,relation,party,authority?}]` | **v2** the docket in docket order (by date, then `docket` number, never source order): `action` is the typed event as written (`Served(complaint, Mail)`), `relation` `by` or `on` the `party`. Replay already turned them into `scenario.start` and `scenario.remove_edges`; the engine reads them as record only |
| `lowered` | object | **v2** [`lowered` header](#the-lowered-header): the world the matter was replayed in (`asOf`/`knownAt` are the case's) and its holes (an `unvalued` stake, a source `as_of ?(..)`) |

`litgraph schema named-scenario` gives the full JSON Schema; `litgraph
validate target/lit-build/scenarios/your-scenario.json` (or `--kind named-scenario`) resolves
it against the current packs the same way a request is validated, reporting
unresolved node/edge refs and expression errors.

### Referencing a scenario from a request

A request's `scenario` field is one of:

- absent / `null` — the engine default (no overrides).
- an inline object — today's shape, unchanged; unknown fields are still a
  hard error naming the valid ones.
- a bare string `"<id>"` — the named scenario's `scenario`, unmodified; its
  `packs` apply when the request gives none of its own, so a request can be
  just `{"scenario": "<id>", "op": {...}}`.
- `{"extends": "<id>", ...overrides}` — the named scenario's `scenario`
  deep-merged (RFC 7386) with `overrides` (every other key); the merged
  result is still parsed as a real `Scenario`, so a typo in `overrides` is
  the same "unknown field" error as an inline scenario, not a silent no-op.

CLI: `--scenario <name|file.json|'{"inline":"json"}'>`; `--set k=v` composes
onto a named `--scenario` via `extends` automatically. `--scenarios-dir` (or
`$LITGRAPH_SCENARIOS`) points at a directory instead of the embedded library.

## Authoring rules

These rules hold for what the JSON says; since the flip you enforce them in
the `lit/` source, where `litigate check` turns most of them into
diagnostics (an unresolved cite is a hole, an error under `--profile
strict`; an unknown field does not parse), and `litgraph lint` runs on the
generated packs.

1. Every `cite` / `authority` must be checkable against a `sources` entry or a
   public citation. No invented rule numbers. A `sources[].path` must be
   repo-relative (or omitted) — never a local filesystem path; cite the
   official `url` instead. If `path` points at a vendored L0 text file
   under `sources/` (see `sources/PROVENANCE.md`), `litgraph lint`
   verifies the cite mechanically — normalizing the surface form ("FRCP
   12(b)(6)", "Fed. R. Civ. P. 12(b)(6)", "28 U.S.C. § 1498(a)", "37
   C.F.R. § 42.108", "RCFC 56", ... all resolve the same way; rule bodies
   come from the pack's [`citeSyntax`](#citation-syntax-v3)), resolving
   it to a `## <heading>` span in that file, and fuzzy-matching any
   quoted phrase in `note` against that span. A cite it can't verify gets
   an `unverifiable-cite` warning naming the exact reason (no local
   `sources[].path`, no matching heading, quote not found, ...) rather
   than silently passing; add new vendored text under `sources/` before
   citing a body of law that isn't there yet.
2. Probabilities are optional. Where you author one, put the basis in `note`
   (statistic + vintage, or "teaching estimate"). Unauthored is better than
   invented; the engine reports unauthored chance nodes.
3. Hours are BigLaw-honest estimates for the move; `cost` is cash only.
4. Terminals should carry `payoff` and `outcome` in v2 packs.
5. Parsing is strict: an unknown field anywhere in a pack, edge, source, or
   instance is a hard error (not a silently-ignored typo). Before opening
   a PR: `just lit-check` (or `litigate check lit/your-forum`), `just
   lit-sync`, then `litgraph validate target/lit-build/packs/your-pack.json` (or `--kind
   pack` to force detection) and `litgraph lint --packs your-pack` on the
   regenerated file.
