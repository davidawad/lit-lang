# CLI error-probe matrix

**Status: complete.** The probe sweep covers all 22 CLI operations against five inputs (the smallest top-level matter, missing file, syntax error, unknown scenario setting, and unknown node), plus the separate `check --as-of` boundary. Every row ran under its own 10-second timeout; the sweep used `xargs -P 8` in one foreground command. A timeout is recorded as an individual row result.

Input `case` is `cases/e2e-ttab-unknown.case` (438 bytes, the smallest top-level `cases/*.case`). `missing-file` is `/__litigate_beta_missing__.case`; `syntax-error` is a temporary file containing `matter {`; `unknown-set` uses `--set definitely_unknown_beta_key=1`; and `unknown-node` uses the operation-specific `__unknown_beta_node__` where supported.

| op | input | exit | classification |
|---|---|---:|---|
| describe | case:cases/e2e-ttab-unknown.case | 124 | TIMEOUT |
| describe | missing-file | 1 | CLI_DIAGNOSTIC |
| describe | syntax-error | 2 | BARE_ERROR |
| describe | unknown-set | 124 | TIMEOUT |
| describe | unknown-node | 124 | TIMEOUT |
| packs | case:cases/e2e-ttab-unknown.case | 124 | TIMEOUT |
| packs | missing-file | 1 | CLI_DIAGNOSTIC |
| packs | syntax-error | 2 | BARE_ERROR |
| packs | unknown-set | 124 | TIMEOUT |
| packs | unknown-node | 124 | TIMEOUT |
| lint | case:cases/e2e-ttab-unknown.case | 124 | TIMEOUT |
| lint | missing-file | 1 | CLI_DIAGNOSTIC |
| lint | syntax-error | 2 | BARE_ERROR |
| lint | unknown-set | 124 | TIMEOUT |
| lint | unknown-node | 124 | TIMEOUT |
| validate | case:cases/e2e-ttab-unknown.case | 124 | TIMEOUT |
| validate | missing-file | 1 | CLI_DIAGNOSTIC |
| validate | syntax-error | 2 | BARE_ERROR |
| validate | unknown-set | 124 | TIMEOUT |
| validate | unknown-node | 124 | TIMEOUT |
| graph | case:cases/e2e-ttab-unknown.case | 124 | TIMEOUT |
| graph | missing-file | 1 | CLI_DIAGNOSTIC |
| graph | syntax-error | 2 | BARE_ERROR |
| graph | unknown-set | 124 | TIMEOUT |
| graph | unknown-node | 124 | TIMEOUT |
| metric | case:cases/e2e-ttab-unknown.case | 124 | TIMEOUT |
| metric | missing-file | 1 | CLI_DIAGNOSTIC |
| metric | syntax-error | 2 | BARE_ERROR |
| metric | unknown-set | 124 | TIMEOUT |
| metric | unknown-node | 124 | TIMEOUT |
| explain | case:cases/e2e-ttab-unknown.case | 124 | TIMEOUT |
| explain | missing-file | 1 | CLI_DIAGNOSTIC |
| explain | syntax-error | 2 | BARE_ERROR |
| explain | unknown-set | 124 | TIMEOUT |
| explain | unknown-node | 124 | TIMEOUT |
| solve | case:cases/e2e-ttab-unknown.case | 124 | TIMEOUT |
| solve | missing-file | 1 | CLI_DIAGNOSTIC |
| solve | syntax-error | 2 | BARE_ERROR |
| solve | unknown-set | 124 | TIMEOUT |
| solve | unknown-node | 124 | TIMEOUT |
| chain | case:cases/e2e-ttab-unknown.case | 124 | TIMEOUT |
| chain | missing-file | 1 | CLI_DIAGNOSTIC |
| chain | syntax-error | 2 | BARE_ERROR |
| chain | unknown-set | 124 | TIMEOUT |
| chain | unknown-node | 124 | TIMEOUT |
| simulate | case:cases/e2e-ttab-unknown.case | 124 | TIMEOUT |
| simulate | missing-file | 1 | CLI_DIAGNOSTIC |
| simulate | syntax-error | 2 | BARE_ERROR |
| simulate | unknown-set | 124 | TIMEOUT |
| simulate | unknown-node | 124 | TIMEOUT |
| path | case:cases/e2e-ttab-unknown.case | 124 | TIMEOUT |
| path | missing-file | 1 | CLI_DIAGNOSTIC |
| path | syntax-error | 2 | BARE_ERROR |
| path | unknown-set | 124 | TIMEOUT |
| path | unknown-node | 124 | TIMEOUT |
| pareto | case:cases/e2e-ttab-unknown.case | 124 | TIMEOUT |
| pareto | missing-file | 1 | CLI_DIAGNOSTIC |
| pareto | syntax-error | 2 | BARE_ERROR |
| pareto | unknown-set | 124 | TIMEOUT |
| pareto | unknown-node | 124 | TIMEOUT |
| sweep | case:cases/e2e-ttab-unknown.case | 124 | TIMEOUT |
| sweep | missing-file | 1 | CLI_DIAGNOSTIC |
| sweep | syntax-error | 2 | BARE_ERROR |
| sweep | unknown-set | 124 | TIMEOUT |
| sweep | unknown-node | 124 | TIMEOUT |
| tornado | case:cases/e2e-ttab-unknown.case | 124 | TIMEOUT |
| tornado | missing-file | 1 | CLI_DIAGNOSTIC |
| tornado | syntax-error | 2 | BARE_ERROR |
| tornado | unknown-set | 124 | TIMEOUT |
| tornado | unknown-node | 124 | TIMEOUT |
| calibration | case:cases/e2e-ttab-unknown.case | 124 | TIMEOUT |
| calibration | missing-file | 1 | CLI_DIAGNOSTIC |
| calibration | syntax-error | 2 | BARE_ERROR |
| calibration | unknown-set | 124 | TIMEOUT |
| calibration | unknown-node | 124 | TIMEOUT |
| posterior | case:cases/e2e-ttab-unknown.case | 124 | TIMEOUT |
| posterior | missing-file | 1 | CLI_DIAGNOSTIC |
| posterior | syntax-error | 2 | BARE_ERROR |
| posterior | unknown-set | 124 | TIMEOUT |
| posterior | unknown-node | 124 | TIMEOUT |
| voi | case:cases/e2e-ttab-unknown.case | 124 | TIMEOUT |
| voi | missing-file | 1 | CLI_DIAGNOSTIC |
| voi | syntax-error | 2 | BARE_ERROR |
| voi | unknown-set | 124 | TIMEOUT |
| voi | unknown-node | 124 | TIMEOUT |
| structure | case:cases/e2e-ttab-unknown.case | 124 | TIMEOUT |
| structure | missing-file | 1 | CLI_DIAGNOSTIC |
| structure | syntax-error | 2 | BARE_ERROR |
| structure | unknown-set | 124 | TIMEOUT |
| structure | unknown-node | 124 | TIMEOUT |
| deadlines | case:cases/e2e-ttab-unknown.case | 124 | TIMEOUT |
| deadlines | missing-file | 1 | CLI_DIAGNOSTIC |
| deadlines | syntax-error | 2 | BARE_ERROR |
| deadlines | unknown-set | 124 | TIMEOUT |
| deadlines | unknown-node | 124 | TIMEOUT |
| settle | case:cases/e2e-ttab-unknown.case | 124 | TIMEOUT |
| settle | missing-file | 1 | CLI_DIAGNOSTIC |
| settle | syntax-error | 2 | BARE_ERROR |
| settle | unknown-set | 124 | TIMEOUT |
| settle | unknown-node | 124 | TIMEOUT |
| compare | case:cases/e2e-ttab-unknown.case | 124 | TIMEOUT |
| compare | missing-file | 1 | CLI_DIAGNOSTIC |
| compare | syntax-error | 2 | BARE_ERROR |
| compare | unknown-set | 124 | TIMEOUT |
| compare | unknown-node | 124 | TIMEOUT |
| batch | case:cases/e2e-ttab-unknown.case | 124 | TIMEOUT |
| batch | missing-file | 1 | CLI_DIAGNOSTIC |
| batch | syntax-error | 2 | BARE_ERROR |
| batch | unknown-set | 124 | TIMEOUT |
| batch | unknown-node | 124 | TIMEOUT |
| check | out-of-interval-date | 124 | TIMEOUT |

## Findings

- **Matter-backed operation probes exceed the 10-second row limit.** All 22 operations timed out on the small valid matter; the `unknown-set` and `unknown-node` rows also timed out before returning an operation-level result. The date-boundary check timed out as well. These rows are inconclusive about operation-level error formatting. Reproduce with:

  ```bash
  timeout 10s target/debug/litigate run cases/e2e-ttab-unknown.case --op '{"op":"describe"}'
  ```

- **Missing target paths return a CLI diagnostic (exit 1), not a JSON run response.** This is consistent across all 22 operations. Reproduce with:

  ```bash
  target/debug/litigate run /__litigate_beta_missing__.case --op '{"op":"describe"}'
  ```

- **Malformed `.case` input returns a structured compiler diagnostic (exit 2).** The output uses `diagnostics` with code `L0001` and message `expected \`case\`, found \`matter\``; it is not an engine-op error envelope. Reproduce with:

  ```bash
  printf 'matter {\n' >/tmp/lit-beta-syntax.case
  target/debug/litigate run /tmp/lit-beta-syntax.case --op '{"op":"describe"}'
  ```

No panic was reported in any completed row. No operation-level error-envelope conclusion can be drawn where the matter-backed probe timed out.

## Run metadata

Built with `CARGO_BUILD_JOBS=4 cargo build -p litigate-cli` (exit 0), then ran `scripts/beta/error-probes.sh` once (exit 0). The script is the probe harness; no tests or code fixes were run.

DECIDED: Use the smallest top-level matter and `xargs -P 8` with 10-second per-row timeouts -- this is the PM-approved approach and records slow rows without stopping the matrix.
