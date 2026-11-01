# CLI error-probe matrix

**Status: partial.** The full default matrix did not finish in this agent job.
The single script run reached only the `describe` operation and was stopped
while probing `cases/requests/cofc-1498-risk-averse-sim.case`. The script
covers every `.case` file plus missing-file, syntax-error, unknown-set,
unknown-node, and the `check --as-of` boundary for each operation. Thus all
rows below are observed results, but later `describe` inputs and every other
operation/input pair remain unrun. This is not a complete matrix.

Each probe has its own 20-second timeout. `124 / TIMEOUT` is an individual
probe result, not a harness stop condition. The table preserves the captured
TSV rows verbatim (columns: op, input, exit, classification).

| op | input | exit | classification |
|---|---|---:|---|
| describe | case:cases/cand-administrative-motion.case | 124 | TIMEOUT |
| describe | case:cases/cofc-1498-patent-case.case | 124 | TIMEOUT |
| describe | case:cases/cofc-bid-protest.case | 124 | TIMEOUT |
| describe | case:cases/e2e-ttab-ok.case | 124 | TIMEOUT |
| describe | case:cases/e2e-ttab-unknown.case | 0 | OK |
| describe | case:cases/examples/acme.case | 0 | OK |
| describe | case:cases/frcp-civil-case.case | 124 | TIMEOUT |
| describe | case:cases/ptab-ipr-defense.case | 124 | TIMEOUT |
| describe | case:cases/requests/cafc-cert-preserves-panel-outcome.case | 124 | TIMEOUT |
| describe | case:cases/requests/calibration-gaps.case | 124 | TIMEOUT |
| describe | case:cases/requests/cofc-1498-chain.case | 124 | TIMEOUT |
| describe | case:cases/requests/cofc-1498-named-scenario.case | 124 | TIMEOUT |
| describe | case:cases/requests/cofc-1498-risk-averse-sim.case | 124 | TIMEOUT |

## Findings

- `describe` times out on multiple matter files under the matrix's 20-second
  limit. Reproduce with:

  ```bash
  timeout 20s target/debug/litigate run cases/cofc-1498-patent-case.case --op '{"op":"describe"}'
  ```

## Run metadata

The probe script was run once with its default full-case settings. The CLI
binary was built with `CARGO_BUILD_JOBS=4 cargo build -p litigate-cli`. The
matrix run was interrupted after repeated per-input timeouts; no code fixes or
tests were run.

DECIDED: Preserve the captured rows and mark this matrix incomplete -- the
full serial run exceeds this job's execution window because most initial
`describe` matter probes consume the full 20-second timeout.
