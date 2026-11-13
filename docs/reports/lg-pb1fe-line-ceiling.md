# lg-pb1fe.9: 500-line source ceiling

DECIDED: Keep split declarations in their original Rust module via ordered `include!` fragments -- this preserves visibility and the public module surface while making each source fragment independently reviewable and under 500 lines.

## Before and after

Step 0 fetched `origin/main` at `30d2114e` (`30d2114ecc2a703ccb744ee12ec7b6e856e9f5d6`). The before scan found 68 source files above 500 lines among 408 `crates/*/src/*.rs` files. The after scan found 60; the only removals are the eight files split below. The ratchet file matches the after list.

Before scan (`find crates -path '*/src/*' -name '*.rs' | xargs wc -l | awk '$1>500' | sort -rn`, excluding the `wc` aggregate row):

```text
   2165 crates/litigate-cli/src/timeline.rs
   1136 crates/litigate-verify/src/assertions.rs
   1063 crates/litigate-sema/src/precedence/delegation.rs
   1000 crates/litigate-sema/src/hir/temporal.rs
    959 crates/litigate-sema/src/pkg/lock.rs
    953 crates/litigate-sema/src/types/build/clock.rs
    904 crates/litigate-verify/src/cite.rs
    901 crates/litigate-cli/src/clock.rs
    890 crates/litigate-cli/src/cmd/filing.rs
    888 crates/litigate-sema/src/db/parse/tree.rs
    887 crates/litigate-sema/src/case/replay/obligations.rs
    858 crates/litigate-sema/src/pkg/manifest.rs
    834 crates/litigate-cli/src/cmd/impact.rs
    811 crates/litigate-sema/src/case/replay.rs
    797 crates/litigate-sema/src/case/resolve/mod.rs
    795 crates/litigate-cli/src/matter.rs
    787 crates/litigate-sema/src/change/world.rs
    765 crates/litigate-cli/src/cli.rs
    748 crates/litigate-sema/src/case/graph.rs
    740 crates/litigate-engine/src/cite/normalize.rs
    736 crates/litigate-sema/src/types/refinement.rs
    732 crates/litigate-cli/src/cmd/check.rs
    729 crates/litigate-cli/src/ws.rs
    650 crates/litigate-cli/src/cmd/calibrate.rs
    644 crates/litigate-engine/src/clock/interp.rs
    629 crates/litigate-engine/src/clock/calendar.rs
    627 crates/litigate-sema/src/types/build/obligation.rs
    627 crates/litigate-engine/src/model/schema.rs
    623 crates/litigate-sema/src/types/build/cx.rs
    622 crates/litigate-lsp/src/lib.rs
    620 crates/litigate-syntax/src/parse/tests.rs
    619 crates/litigate-sema/src/temporal/world.rs
    619 crates/litigate-cli/src/cmd/filing/paper.rs
    616 crates/litigate-verify/src/context.rs
    615 crates/litigate-sema/src/hir/kind.rs
    600 crates/litigate-syntax/src/grammar/fields.rs
    592 crates/litigate-sema/src/types/build.rs
    589 crates/litigate-cli/src/cmd/worlds.rs
    582 crates/litigate-cli/src/mcp/authoring.rs
    578 crates/litigate-sema/src/resolve/sig.rs
    574 crates/litigate-base/src/diagnostic.rs
    573 crates/litigate-lower/src/pack.rs
    572 crates/litigate-lower/src/edge.rs
    559 crates/litigate-sema/src/guard.rs
    556 crates/litigate-smt/src/query.rs
    542 crates/litigate-lower/src/clock.rs
    541 crates/litigate-syntax/src/ast/tests/items.rs
    541 crates/litigate-cli/src/mcp/mod.rs
    538 crates/litigate-lsp/src/index.rs
    537 crates/litigate-cli/src/cmd/expect.rs
    536 crates/litigate-cli/src/cmd/import.rs
    533 crates/litigate-sema/src/temporal/version.rs
    533 crates/litigate-engine/src/clock/rule.rs
    529 crates/litigate-cli/src/cmd/build.rs
    528 crates/litigate-cli/src/cmd/coverage.rs
    526 crates/litigate-sema/src/types/expr.rs
    521 crates/litigate-engine/src/lint_tests.rs
    520 crates/litigate-import/src/node.rs
    519 crates/litigate-cli/src/inline_tests.rs
    516 crates/litigate-engine/src/lint.rs
    515 crates/litigate-verify/src/net.rs
    508 crates/litigate-engine/src/api/mod.rs
    506 crates/litigate-syntax/src/ast/items.rs
    505 crates/litigate-sema/src/resolve/scope.rs
    504 crates/litigate-lower/src/source_map.rs
    504 crates/litigate-lower/src/case.rs
    503 crates/litigate-sema/src/case/grant.rs
    501 crates/litigate-cli/src/cmd/run.rs
```

After scan (same command, excluding the `wc` aggregate row):

```text
    890 crates/litigate-cli/src/cmd/filing.rs
    888 crates/litigate-sema/src/db/parse/tree.rs
    887 crates/litigate-sema/src/case/replay/obligations.rs
    858 crates/litigate-sema/src/pkg/manifest.rs
    834 crates/litigate-cli/src/cmd/impact.rs
    811 crates/litigate-sema/src/case/replay.rs
    797 crates/litigate-sema/src/case/resolve/mod.rs
    795 crates/litigate-cli/src/matter.rs
    787 crates/litigate-sema/src/change/world.rs
    765 crates/litigate-cli/src/cli.rs
    748 crates/litigate-sema/src/case/graph.rs
    740 crates/litigate-engine/src/cite/normalize.rs
    736 crates/litigate-sema/src/types/refinement.rs
    732 crates/litigate-cli/src/cmd/check.rs
    729 crates/litigate-cli/src/ws.rs
    650 crates/litigate-cli/src/cmd/calibrate.rs
    644 crates/litigate-engine/src/clock/interp.rs
    629 crates/litigate-engine/src/clock/calendar.rs
    627 crates/litigate-sema/src/types/build/obligation.rs
    627 crates/litigate-engine/src/model/schema.rs
    623 crates/litigate-sema/src/types/build/cx.rs
    622 crates/litigate-lsp/src/lib.rs
    620 crates/litigate-syntax/src/parse/tests.rs
    619 crates/litigate-sema/src/temporal/world.rs
    619 crates/litigate-cli/src/cmd/filing/paper.rs
    616 crates/litigate-verify/src/context.rs
    615 crates/litigate-sema/src/hir/kind.rs
    600 crates/litigate-syntax/src/grammar/fields.rs
    592 crates/litigate-sema/src/types/build.rs
    589 crates/litigate-cli/src/cmd/worlds.rs
    582 crates/litigate-cli/src/mcp/authoring.rs
    578 crates/litigate-sema/src/resolve/sig.rs
    574 crates/litigate-base/src/diagnostic.rs
    573 crates/litigate-lower/src/pack.rs
    572 crates/litigate-lower/src/edge.rs
    559 crates/litigate-sema/src/guard.rs
    556 crates/litigate-smt/src/query.rs
    542 crates/litigate-lower/src/clock.rs
    541 crates/litigate-syntax/src/ast/tests/items.rs
    541 crates/litigate-cli/src/mcp/mod.rs
    538 crates/litigate-lsp/src/index.rs
    537 crates/litigate-cli/src/cmd/expect.rs
    536 crates/litigate-cli/src/cmd/import.rs
    533 crates/litigate-sema/src/temporal/version.rs
    533 crates/litigate-engine/src/clock/rule.rs
    529 crates/litigate-cli/src/cmd/build.rs
    528 crates/litigate-cli/src/cmd/coverage.rs
    526 crates/litigate-sema/src/types/expr.rs
    521 crates/litigate-engine/src/lint_tests.rs
    520 crates/litigate-import/src/node.rs
    519 crates/litigate-cli/src/inline_tests.rs
    516 crates/litigate-engine/src/lint.rs
    515 crates/litigate-verify/src/net.rs
    508 crates/litigate-engine/src/api/mod.rs
    506 crates/litigate-syntax/src/ast/items.rs
    505 crates/litigate-sema/src/resolve/scope.rs
    504 crates/litigate-lower/src/source_map.rs
    504 crates/litigate-lower/src/case.rs
    503 crates/litigate-sema/src/case/grant.rs
    501 crates/litigate-cli/src/cmd/run.rs
```

## Ratchet

`crates/.line-ceiling` records each current over-limit source path and its maximum line count. The Rust integration test `crates/litigate-base/tests/line_ceiling.rs` scans source lengths without git or network access, prints each violation with its count, and checks the committed tree. It also verifies an unlisted 501-line file fails and a listed file grown by one line fails. Run it with `cargo nextest run -p litigate-base --test line_ceiling`; `just line-ceiling-test` runs the same check and is wired into `just ci`.

## Splits

Each old source file now includes the listed fragments in order. The source counts below include test fragments; every resulting file is under 500 lines.

- `crates/litigate-cli/src/timeline.rs` (2165) -> parent (54), `timeline/model.rs` (197), `builder.rs` (391), `obligations.rs` (148), `events.rs` (296), `worlds.rs` (341), `tests_rows.rs` (340), `tests_worlds.rs` (401).
- `crates/litigate-verify/src/assertions.rs` (1136) -> parent (36), `assertions/counting.rs` (377), `expect.rs` (293), `clock.rs` (285), `tests.rs` (146).
- `crates/litigate-sema/src/precedence/delegation.rs` (1063) -> parent (22), `delegation/operations.rs` (429), `diagnostics.rs` (365), `claims.rs` (248).
- `crates/litigate-sema/src/hir/temporal.rs` (1000) -> parent (7), `temporal/interval.rs` (355), `extensions.rs` (276), `counting.rs` (362).
- `crates/litigate-sema/src/pkg/lock.rs` (959) -> parent, including its unchanged test module (363), `lock/digest.rs` (204), `file.rs` (392).
- `crates/litigate-sema/src/types/build/clock.rs` (953) -> parent (19), `clock/basic.rs` (468), `deadlines.rs` (210), `transitions.rs` (217), `helpers.rs` (43).
- `crates/litigate-verify/src/cite.rs` (904) -> parent (16), `cite/quotes.rs` (207), `measures.rs` (320), `sources.rs` (338).
- `crates/litigate-cli/src/clock.rs` (901) -> parent (33), `clock/fragments.rs` (333), `due.rs` (161), `tests.rs` (375).

## Identity and gates

The release `litigate check lit cases --format json` succeeded before and after the refactor. Normalizing only timing fields produced an empty diff (exit 0):

```sh
jq -S 'walk(if type == "object" then del(.elapsed_ms) else . end)' /tmp/lg-pb1fe.9/before-check.json > /tmp/lg-pb1fe.9/before-normalized.json
jq -S 'walk(if type == "object" then del(.elapsed_ms) else . end)' /tmp/lg-pb1fe.9/after-check.json > /tmp/lg-pb1fe.9/after-normalized.json
diff -u /tmp/lg-pb1fe.9/before-normalized.json /tmp/lg-pb1fe.9/after-normalized.json
```

Output: no diff. The unnormalized outputs were 825842 bytes before and 825847 bytes after.

Gate results:

- `cargo fmt --all -- --check`: exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
- `cargo nextest run -p litigate-base --test line_ceiling`: 3 passed, 0 failed.
- The end-of-workspace-set run recorded before the retry checkpoint was `cargo nextest run -p litigate-cli -p litigate-lsp -p litigate-verify -p litigate-base -p litigate-syntax -p litigate-sema -p litigate-engine`: 2446 tests, 2444 passed, 2 failed, 1 skipped (exit 100). The failures reproduce in isolation and are unrelated to the refactor: `every_embedded_calibration_entry_resolves` reports 1 unresolved entry in `frcp-fy2024.json`; `every_vendored_text_has_its_sha256_recorded` reports six source text files without current hashes in provenance.

## Retry finish verification

The final fetch advanced `origin/main` from `30d2114e` to `6ebc2f11`; the branch was rebased onto it. This base includes the parallel provenance migration (`c3aae65c`) and a formatting-only CLI inventory change. After rebase:

- `cargo nextest run -p litigate-base --test line_ceiling`: 3 passed, 0 failed (exit 0).
- `cargo nextest run -p litigate-cli -p litigate-sema -p litigate-verify -p litigate-base`: 1504 tests, 1503 passed, 1 failed, 0 skipped (exit 100). The failure is `litigate-cli::change_examples::every_example_layer_and_world_checks`, which now sees L9002/L9012 warning diagnostics. The same four-crate run passed 1504/1504 on base `30d2114e` before the final fetch; the warning assertion is outside this bead and appeared with the newer corpus base.
- `cargo fmt --all --check`: exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
- The two previously identified `litigate-engine` failures (`every_embedded_calibration_entry_resolves` and `every_vendored_text_has_its_sha256_recorded`) were not rerun; they were reported as pre-existing on `30d2114e` by the PM and are unrelated to this bead.
- The after-list remains 60 over-limit files; the rebase adds no over-limit source file. The ratchet test passed after rebase.
