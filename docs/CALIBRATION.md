# Calibration

A pack's `probability`/`duration` fields are often authored as "teaching
estimates" — plausible numbers with a `note` saying so, not real data.
Calibration replaces specific ones with sourced values from published
statistics, without editing the pack: **data plus mechanism**, the same
split packs themselves use.

## The source: a `calibration` declaration

A calibration set is a `calibration` declaration in the `.lit` package it
calibrates (SPEC §5.9): `lit/ptab/calibration-ptab-fy2024.lit` for the
PTAB pack, `lit/links/calibration-cafc-fy2024.lit` for a `cafc@<origin>`
instance. The compiler resolves every target and types every value, so a
typo is an error with a span, never an entry that silently no-ops at
request time. `litigate build` lowers each declaration to an engine
calibration set (`packs/calibration/<id>.json`, internal IR the binaries
embed, SPEC §8.8); `$LITGRAPH_CALIBRATION` (or `litgraph
--calibration-dir`) still names a directory of such sets that replaces the
built ones for a request. `litigate import` turns a legacy set into the
declaration.

```lit
module ptab.calibration-ptab-fy2024

calibration ptab-fy2024: "PTAB IPR/PGR institution and FWD rates, FY2024" {
  note "What this set covers, in prose."
  source fy24-roundup "PTAB Trial Statistics FY24 End of Year Outcome Roundup — IPR, PGR" {
    url "https://www.uspto.gov/sites/default/files/documents/ptab_aia_fy2024__roundup.pdf"
    vintage [2023-10-01, 2024-10-01)
    retrieved 2026-09-28
  }
  p edge-institution-decision-institution-granted 0.681 {
    source fy24-roundup
    n 1087
    method "740 instituted / (740 + 347 denied) = 0.6807"
  }
  p edge-institution-decision-institution-denied-merits ? {
    source fy24-roundup
    n 1087
    method "Clears the pack's unsourced teaching estimate: the source gives no split."
  }
}
```

| item | notes |
|---|---|
| `p target q` | an edge's probability, in [0, 1]; `p target ?` clears the edge's own probability (needs `method`) |
| `duration target D` | an edge's elapsed time: `252.63.days`, or `{ min D, mode D, max D }` (min and max optional) |
| `attr target name x` | one numeric node attribute |
| target | `local` (this package), `pack::local`, or `base@origin::local` for one `links` instance only, not the base pack or its siblings. An edge target applies to every [state-flag](PACK_SCHEMA.md#state-flags) product-graph copy of that edge, not just the one whose id matches exactly: a calibration entry authors one value for the edge a pack describes, not one per flag-history it can be taken under (e.g. the PTAB pack's `edge-fwd-issued-fwd-all-unpatentable` and its `{ipr-estopped}` sibling, reachable after a Director-review remand for further proceedings, both get the calibrated value from a single entry) |
| `source` | a declared source: title, `url` (fetched and checked to resolve when the entry was authored), `vintage` (the half-open reporting period: `[2023-10-01, 2024-10-01)` is FY2024), `retrieved` (when you checked it) |
| `n` | sample size, or the count itself for a full-population annual total |
| `method` | required whenever the value is derived arithmetic from the source's raw counts (show the arithmetic) |

Authoring rules (same spirit as `docs/PACK_SCHEMA.md`):

1. Every number must come from a document you actually fetched, with the
   URL recorded and checked to work. Never invent a number.
2. If a source gives raw counts and only a derived rate is usable, show the
   arithmetic in `method`.
3. If you can't cleanly map a source's category onto the graph's edges
   (e.g. a source lumps two outcomes the pack models as separate edges),
   leave that edge uncalibrated rather than guessing a split. The `lint`
   warnings (`probability-fill`, `probability-renormalized`) and the
   `calibration` op both surface what's still unauthored.
4. Fewer, solid numbers beat many soft ones.

## Applying a set: `scenario.calibration`

```json
{
  "packs": ["ptab-patent-trial-appeal-board"],
  "scenario": { "calibration": ["ptab-fy2024"] },
  "op": { "op": "chain" }
}
```

Applying a set mutates the *compiled graph*, not the scenario's usual
override maps: each entry's value becomes **authored**, exactly as if it
had been hand-typed into the pack. That means the existing engine machinery
handles everything downstream for free — probability-fill for any sibling
edges the set didn't reach, `probability-renormalized` if the authored sum
drifts from 1.0, duration fallbacks, all unchanged. A ref that doesn't
resolve (its pack wasn't loaded in this request) is silently skipped, the
same convention `links.json` uses for a link into an unloaded pack — a
calibration set is expected to span multiple forums and be used with any
subset of packs.

`scenario.calibration` is a list: `["ptab-fy2024", "cafc-fy2024"]` applies
both, in order (a later set overwrites a ref an earlier one also set).

## Provenance: what was calibrated

Every response's `provenance.calibration` says exactly what happened,
whether or not the request asked for calibration (an empty array if not):

```jsonc
"calibration": [
  {
    "set": "ptab-fy2024",
    "applied": [
      {
        "target": "edge-probability",
        "ref": "ptab-patent-trial-appeal-board::edge-institution-decision-institution-granted",
        "value": 0.681,
        "n": 1087,
        "sourceTitle": "PTAB Trial Statistics FY24 End of Year Outcome Roundup",
        "sourceUrl": "https://www.uspto.gov/sites/default/files/documents/ptab_aia_fy2024__roundup.pdf",
        "vintageStart": "2023-10-01",
        "vintageEnd": "2024-09-30"
      }
      // ...
    ],
    "skipped": 0
  }
]
```

An agent reading a response can always tell an authored-from-real-data
number from a teaching estimate or an engine fallback, without reading the
pack source.

## Finding what to calibrate next: the `calibration` op

```json
{ "packs": ["frcp-civil-procedure"], "op": { "op": "calibration", "top": 10 } }
```

Ranks every **uncalibrated** probability and duration by decision
sensitivity, using the same one-at-a-time perturbation method `tornado`
uses for authored inputs (`crate::algo::sweep`), just aimed the other way:

- **probabilities**: perturbed by ±`dp` (default 0.1), ranked by
  `|ΔV(start)|` — how much the scenario's dollar value swings.
- **durations**: perturbed by ±`rel` (default 0.25, relative to the edge's
  deadline-length fallback, or a fixed 0–30 day band absent one), ranked by
  `|Δ expected elapsed days|` to absorption under the optimal policy.

`max_candidates` (default 300) bounds how many edges of each kind are
actually solved for, so a large multi-pack selection stays cheap; the
result says `probabilities_truncated`/`durations_truncated` when it had to
stop early rather than silently returning a partial ranking.

The two lists use different units (dollars vs. days) and are reported
separately rather than force-ranked into one list — see the source comment
in `crates/litigate-engine/src/api/calibration/gaps.rs` for why.

## The first calibration set

Four FY2024 sets ship, each sourced from one fetched, working document:

| set (file) | covers | source |
|---|---|---|
| `ptab-fy2024` (`lit/ptab/`) | IPR/PGR institution rate; FWD outcome split (all-unpatentable / mixed / all-confirmed) | USPTO PTAB Trial Statistics FY24 End of Year Outcome Roundup |
| `cafc-fy2024` (`lit/links/`) | `cafc@cofc` affirmance and vacatur/remand rates | CoFC FY2024 Statistical Report, "Appeals to the ... Federal Circuit" |
| `cofc-fy2024` (`lit/cofc/`) | annual bid-protest filing/disposition volume (node attrs) | same CoFC report |
| `frcp-fy2024` (`lit/frcp/`) | district-court time-to-disposition (voluntary dismissal, summary judgment) | AO Judicial Business Table C-5 |

Deliberately left uncalibrated (real gaps, not oversights — good first
targets for the next pass, and exactly what `litgraph q -` with
`{"op":"calibration"}` will surface):

- PTAB institution-denial reason (merits vs. Fintiv discretion): the source
  reports only a combined "denied" total.
- Federal Circuit Rule 36 rate, and the `reversed` vs.
  `affirmed-in-part-reversed-in-part` split for CoFC-origin appeals: no
  primary source with matching granularity was independently verified in
  this pass (a `panel-to-rule36` teaching estimate already cites
  `patentlyo-2024-datapack`; that citation was not re-verified here and is
  left as-is rather than duplicated into a calibration entry).
- CoFC bid-protest outcome rate (sustained vs. denied): the fetched
  statistical report gives filing/disposition volume only, not outcomes.
