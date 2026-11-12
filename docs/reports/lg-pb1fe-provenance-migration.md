# Typed provenance corpus migration (lg-pb1fe.57)

DECIDED: Migrate a note marker only when that note identifies one numeric field kind authored by its declaration. Leave notes that mix kinds, name no number, or describe legal interpretation untouched; never infer a vintage. `UNVERIFIED` has no typed basis, so an unambiguous numeric instance maps to `basis estimate` and remains explicitly counted as such.

Baseline: `origin/main` at `30d2114ecc2a703ccb744ee12ec7b6e856e9f5d6`.

## Inventory and migration

The read-only Rust inventory is `cargo run -p litigate-cli --bin provenance_inventory -- lit`. It walks note lines, their containing declaration and numeric fields. At baseline it found 442 marker-note lines: 193 unambiguous and 249 ambiguous. The current inventory finds 249 ambiguous and no remaining unambiguous candidates.

Marker counts are per note line and overlap when a line has more than one marker:

| Marker | Unambiguous before | Ambiguous before | Migrated typed values |
| --- | ---: | ---: | ---: |
| `PLACEHOLDER` | 117 | 2 | 117 `basis placeholder` |
| `basis: estimate` | 74 | 0 | 74 `basis estimate` |
| `UNVERIFIED` | 2 | 112 | 2 `basis estimate` |
| `teaching estimate` | 8 | 107 | Included in the 117 placeholder / 74 estimate values above |
| `APPROXIMATE` | 0 | 33 | 0 |
| `vintage <date>` | 0 | 1 | 0 |

Total: 193 typed values (117 placeholder, 76 estimate); 249 ambiguous note lines left. The two unverified numeric values are estimates because the basis lattice has no `unverified` label. No date was added unless the note stated one; therefore some new estimates and placeholders correctly produce L9012.

The four PTAB request-fee amounts with `verified` and a CFR cite remain explanatory note text. The cited provision's vendored amendment history does not prove the fee text for the rule version in force at the package's 2012-09-16 start date, so cite checking did not support `basis sourced(...)`. The note's stated 2025-01-14 effective/last-amended date remains unchanged. The independent hours values on those edges were typed as estimates.

Ten examples of intentionally retained ambiguous notes (file:line):

- `lit/mpep/03-post-grant.lit:93`
- `lit/mpep/03-post-grant.lit:100`
- `lit/mpep/03-post-grant.lit:127`
- `lit/mpep/03-post-grant.lit:158`
- `lit/mpep/03-post-grant.lit:189`
- `lit/mpep/03-post-grant.lit:202`
- `lit/mpep/03-post-grant.lit:209`
- `lit/mpep/03-post-grant.lit:236`
- `lit/frcp/03-post-trial-motions.lit:163`
- `lit/frcp/02-pleading-to-trial.lit:158`

## Grep counts and diagnostics

`git grep -i -c` totals across `lit/`:

| Marker | Before | After |
| --- | ---: | ---: |
| `PLACEHOLDER` | 175 | 175 |
| `UNVERIFIED` | 639 | 637 |
| Unique lines matching either marker | 814 | 812 |

The placeholder token count does not fall because the migrated word now appears in `basis placeholder`; the note inventory above is the measure of prose markers. Two numeric `UNVERIFIED` note markers were removed. Baseline `litigate check lit cases` had 0 L9012 warnings; the final full-corpus check has 201. These expected warnings mark estimate/placeholder values with no note-stated vintage.

## Identity check

Built the 9 touched packages with the release binary from the origin/main second clone and the final release binary into separate output directories. For each pack and source map, after removing only `lowered.basis[]`, note text, spans and source/world digest fields, the JSON was identical:

- `cafc`
- `cofc`
- `frap-appellate-procedure`
- `frcp-civil-procedure`
- `frcrimp-criminal-procedure`
- `itc-337`
- `mpep-prosecution`
- `ptab-patent-trial-appeal-board`
- `ttab-trademark-trial-and-appeal-board`

Thus no probability, payoff, duration, cost, effort, cite, rule fact, or edge moved. The builds also regenerated package `Lit.lock` rows via `just lit-sync`.

## Documentation references

`AGENTS.md` Rules 1, 5 and 8 now direct authors to per-number `basis` and `vintage` suffixes in declarations, with notes reserved for context. `docs/PACK_SCHEMA.md` applies this to matter facts and pack notes; `docs/CALIBRATION.md` explains typed estimate provenance.

## Final measurements

- `litigate check lit cases --format json`: exit 0, `ok: true`, no diagnostics/errors, L9012 = 201 (baseline 0).
- `litigate fmt --check lit cases`: exit 0, 0 diagnostics, 0 files changed.
- `litigate test` on all 9 touched packages: exit 0, 349 passed, 0 failed.
- `cargo nextest run -p litigate-cli --no-fail-fast -E 'binary(roundtrip) | binary(corpus) | binary(cases_port) | binary(core_stdlib)'`: exit 0, 47 passed, 0 skipped.
- `cargo build --release -p litigate-cli`: exit 0.
- `just lit-sync-check`: exit 0; `Lit.lock` files and the shared clock library are current, no generated file committed.
- `litigate lint` on the 9 touched packages: exit 0; 0 errors, 199 warnings (193 L9012 and 6 L9002).
