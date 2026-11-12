# `litigate query provenance` corpus report

Measured with the release CLI built from this branch:

```text
$ cargo build --release -p litigate-cli
Finished `release` profile [optimized] target(s) in 7m 26s
exit: 0
```

Both corpus queries loaded 635 packages from `lit/`.

```text
$ target/release/litigate --format json query provenance lit --basis placeholder
ok: true; total: 0; top ten packages: none

$ target/release/litigate --format json query provenance lit --basis estimate --no-vintage
ok: true; total: 1043
top ten packages by row count:
  frcp-civil-procedure: 219
  ttab-trademark-trial-and-appeal-board: 146
  mpep-prosecution: 144
  frap-appellate-procedure: 106
  itc-337: 92
  frcrimp-criminal-procedure: 89
  ptab-patent-trial-appeal-board: 81
  cofc: 78
  cafc: 71
  frcp-rule12-example: 8
```

These commands filtered the JSON response rows by the `package` field. The
placeholder query returned no rows in this corpus; the estimate query counts
numeric fields whose effective basis is `estimate` and whose vintage is absent.

## Verification

The required nextest filter was run on `origin/main` at `10b6a048` and on this
branch. The baseline ran 26 tests (23 passed, 3 failed); this branch ran 30
tests (27 passed, 3 failed). The only failed names on both were:

- `chance_odds_and_durations_come_from_docketed_steps_with_their_provenance`
- `anonymous_edges_to_the_same_node_are_numbered`
- `query_pages_and_filters`

The four new provenance tests passed. Workspace clippy passed with
`-D warnings`, and `cargo fmt --all --check` passed. Both nextest runs used:

```text
cargo nextest run -p litigate-cli --no-fail-fast -E 'binary(cov_cmd_query) | test(/query|describe|provenance|basis/)'
```

DECIDED: report zero placeholder rows and no top-ten packages because that is
what the release binary measured across the full 635-package load.
