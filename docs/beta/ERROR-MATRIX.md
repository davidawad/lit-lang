# Beta CLI error UX matrix

DECIDED: keep probes at the public CLI boundary and record the actual exit code and output classification -- this reproduces the errors a user sees without bypassing CLI handling.

## Probe contract

`scripts/beta/error-probes.sh` enumerates the 22 engine operations and every `cases/**/*.case`, then probes missing targets, malformed matter syntax, an unknown `--set` key and an unknown node. A response envelope is classified as `ENVELOPE` only when stdout parses as JSON with `ok:false` and string `error.code`, `error.message` and `error.hint`. Panics and backtraces are `PANIC`; other output is recorded separately. `run` does not accept `--as-of`, so the harness exercises out-of-interval date selection through `litigate check --as-of 0001-01-01`.

## Measured rows

Built with `cargo build -p litigate-cli` (exit 0). Ran `PROBE_SKIP_CASES=1 scripts/beta/error-probes.sh`, then interrupted it after the seven rows below; the full matrix did not finish. The script's normal mode remains the full probe requested by this bead. `OK` means the command returned success, not that the invalid input was correctly rejected.

| op | input | exit | observed result |
|---|---|---:|---|
| describe | missing file | 1 | CLI_DIAGNOSTIC |
| describe | malformed `.case` | 2 | BARE_ERROR |
| describe | unknown `--set definitely_unknown_beta_key=1` | 0 | OK (gap: key was accepted) |
| describe | unknown node | 0 | OK (not applicable to `describe`; harness currently sends an unused node field) |
| packs | missing file | 1 | CLI_DIAGNOSTIC |
| packs | malformed `.case` | 2 | BARE_ERROR |
| packs | unknown `--set definitely_unknown_beta_key=1` | 0 | OK (gap: key was accepted) |

No `PANIC` or Rust backtrace appeared in the rows that ran. No result is recorded for other op/input combinations; they remain unprobed. The malformed matter rows did not satisfy the requested JSON error-envelope check.

## Pre-existing failures

The previous attempt reported a failing timeline test in the full `litigate-cli` suite. This run did not run that suite or a targeted test: no implementation fix was completed, and the probe run stopped before the failure could be rechecked.

PARTIAL: complete and record all op/input rows, fix the envelope and unknown-key failures, and add/run one targeted regression test for each fix.
