# How litgraph is tested

The Rust test code lives in `crates/litigate-engine/tests/` (integration tests, one
binary per file) and next to the code in `crates/litigate-engine/src/` (unit tests
in `#[cfg(test)]` modules). This directory holds the **data** the tests read:
readable fixtures and snapshots a reviewer can open without reading Rust.

```bash
cargo nextest run --workspace                          # everything
cargo test -p litigate-engine --test golden                   # just the golden graphs
litigate test tests/expect                             # the expect tests, famous cases included
```

## The layers, from most readable to most thorough

| Layer | Data | Test | What it proves |
|---|---|---|---|
| **Expect tests** | `tests/expect/<case>/` | `litigate test tests/expect` | Real, sourced matters (Bowles, Twombly, Celotex, eBay, Oil States, Arthrex, ...) give the recorded verdict, layers in force, filing outcomes and rule-computed deadlines in each world they run under. |
| **Golden graphs** | `tests/golden/<name>.json` | `golden.rs` | `solve`, `chain`, `simulate`, `path` and min-cut give the answer you get with a pencil on 2-6 node graphs. Each file shows its arithmetic. |
| **Features** | inline packs | `features.rs`, `general_sum.rs`, `cvar*.rs`, `path_vars.rs`, `calibration.rs`, `api_deadlines.rs`, `review.rs` | Each non-v1 capability behaves as documented on small hand-built graphs. `review.rs` pins defects an adversarial review found. |
| **Shipped content** | `packs/`, `scenarios/`, `calibration/` | `scenario_library.rs`, `calibration.rs`, `cite_coverage_report.rs` | Everything embedded in the binary still loads, resolves and cites. |
| **Parity** | `tests/fixtures/ts-parity/` | `parity.rs`, `clock_parity*.rs` | With v1 modes pinned, the engine reproduces the original TypeScript engine's numbers and deadline dates. |
| **Properties** | generated | `properties*.rs`, `clock_properties.rs` | Invariants over random graphs and dates (probabilities sum to 1, min cut = max flow, deadlines never land on a holiday, ...). |
| **Coverage gaps** | inline | `*_gaps*.rs`, `coverage_gaps.rs`, `api_coverage.rs`, `api_sweep_errors.rs` | Branches (error paths, degenerate inputs) the suites above don't reach; they keep line coverage at or above 97%. |
| **Proofs** | none | `src/proofs.rs` (Kani) | Properties proved for every input in a bounded space, not sampled. |

Read the table top-down to learn what the engine does. Read it bottom-up to
see how hard it has been pushed.

## Expect tests (`expect/`)

A matter's own expect test is a directory `tests/expect/<case>/`: its
`.case` and `Lit.lock`, one `.world` per circuit or forum it runs under,
and `expect.toml` recording, per world, the verdict, the layers in force,
each filing's outcome and each deadline's due date, status and cites
(docs/litigate/SPEC.md §5.8, expect tests; `bowles-v-russell/` is the
first). `litigate test tests/expect`, run by `just ci` and `just
ci-scoped`, fails on any value that moved, with a table of case, world,
field, expected and actual; `litigate test tests/expect --bless`
re-records them, and the commit says why.

The other files, `tests/expect/<forum>/<name>.toml`, list what a change to the law should
be read against: each names a matter and the world it runs in

```toml
case = "docs/litigate/change/bass-5cir-2000.case"
world = "docs/litigate/change/worlds/circuit-appellate-ca5.world"
paths = ["docs/litigate/change"]
```

or a package (`package = "lit/frcp"`) whose inline `expect` values count.
`litigate test --impact BASE`, run at the repository root, runs every one
at git revision BASE and in the working tree and prints, per forum, a
table of case, world, field (a filing's verdict, a deadline's due date
and status, an expectation), before and after, for each value that moved.
`just ci-scoped` posts it in the job log. A change is for the reviewer to
read, never a failure. The matters are public rulings
(sources/change-corpus/rulings); docs/litigate/expect-corpus.md
plans the leading decision per circuit per rule that extends them.

## Famous cases

The ten famous cases (arthrex, blue-and-gold, celotex, ebay, hughes,
iqbal, oil-states, sas, suprema, twombly) are expect tests,
`tests/expect/<slug>/` (lg-wvlf.3). Each records the orders its key
periods run from, and `expect.toml` the rule's date for each period (not
the day the parties filed), with the hand arithmetic and the historical
filing date in its leading comments. The periods are the package
`docs/litigate/famous`, counted by lit/core's rules in force on each
trigger's date; a period whose counting rule has no version in force then
(Sup. Ct. R. 30 before its 2023 edition) is recorded `unknown`.

## Golden graphs (`golden/`)

Each file holds one pack small enough to solve by hand, one op, the
expected slice of the op's `result`, and an `arithmetic` list showing how
each number was computed. Numbers must match to 1e-9. The exceptions are
sampled quantities from `simulate`, which get a `tolerance` of three
standard errors, also derived in `arithmetic`.

## Naming

Test names are sentences that state the behavior, e.g.
`suing_beats_settling_when_the_expected_verdict_exceeds_the_offer` or
`juneteenth_is_not_a_holiday_before_2021`. A failing test's name should
tell you what broke before you open the file.
