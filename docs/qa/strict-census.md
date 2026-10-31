# Strict package census

Baseline generated with `bash scripts/qa/strict-census.sh --limit 8`, using
`target/debug/litigate check --profile strict --format json` for each package.
The TSV records the command exit code, elapsed seconds, diagnostic counts by
code, deferred state-source L5002 count, and in-scope error count. The census
covers every immediate package directory under `lit/` except the 16 shared,
example, and primary federal packages excluded by the runner. Package
classification below uses `Lit.toml`'s `family` and `jurisdiction_id`, with
`bankr-*` and `bap*` identifiers for the bankruptcy appellate group.

## Totals

- Packages checked: 619 (all 619 returned strict-profile exit code 2).
- In-scope errors: 31,501.
- Deferred state-source errors: 0.
- Packages with zero in-scope errors: 0.
- Diagnostic codes: L5002 26,985; L9001 4,496; L9004 20.

## Errors by family

| Family | Packages | In-scope errors |
| --- | ---: | ---: |
| Bankruptcy courts and bankruptcy appellate panels (`bankr-*`, `bap*`) | 98 | 7,503 |
| Federal district civil (`family = "district-civil"`) | 94 | 5,046 |
| Federal district criminal (`family = "district-criminal"`) | 94 | 3,865 |
| State courts (`jurisdiction_id` authored in `Lit.toml`) | 307 | 13,921 |
| Federal appellate/specialized and state intermediate appellate (remaining packages) | 26 | 1,166 |
| **Total** | **619** | **31,501** |

## Errors by diagnostic code

| Code | Count | Meaning |
| --- | ---: | --- |
| L5002 | 26,985 | Strict profile found unresolved holes. |
| L9001 | 4,496 | A deadline or counting rule lacks an inline test. |
| L9004 | 20 | A file is not in canonical `litigate fmt` form. |

The deferred-state column is zero in this run: no L5002 diagnostic fell under
`lit/core/states/` (or `core/states/` in the emitted diagnostic path). Thus all
L5002 counts above are in scope for this baseline.
