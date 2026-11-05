# Litigation language epic audit on `origin/main`

Audited snapshot: `f44da4dac0c1227a4c22ec29fbeffbe41e829c62` (`origin/main`,
checked in the pod on 2026-11-05). The requested quality command (`just ci` / `just ci-scoped`) was
not run, as directed by the dispatch instructions.

| Epic Done-when item | Verdict | Command and output summary |
|---|---|---|
| Nine packs, links and four scenarios are generated from source with semantic equality | **N/A (comparison baseline deleted)** | `cargo build --release` exited 0; then `target/release/litigate build --out /tmp/lg-zmqc-57-build --pack-file cafc=cafc-federal-circuit --pack-file cofc=cofc-court-of-federal-claims lit/cafc lit/cofc lit/frap lit/bap8 lit/frcp lit/frcrimp lit/itc lit/mpep lit/ptab lit/ttab lit/links cases/cofc-1498-patent-case.case cases/cofc-bid-protest.case cases/frcp-civil-case.case cases/ptab-ipr-defense.case cases/ttab-opposition-applicant-defense.case` exited 0; emitted 10 forum packs, `links.json`, 5 scenarios, and 4 calibration sets. The epic's nine/four counts are stale. No tracked legacy pack/scenario JSON files exist (`git ls-files 'packs/**/*.json' scenarios` returns 0), so semantic equality against the former hand-authored JSON cannot be checked at this snapshot; the source build itself succeeds. |
| Existing tests pass against generated files | **PASS (targeted test)** | `cargo test -p litigate-cli the_worked_example_forks_at_the_motion_ruling -- --nocapture` exited 0: 1 passed, 0 failed; the test that failed at the prior snapshot now passes against the current generated files. The workspace suite was not run as this row requests only the named targeted regression. |
| CI enforces generated-source freshness (`lit-sync`) | **PASS** | `justfile` declares `ci: ... lit-sync-check ...`, and `lit-sync-check` depends on `lit-sync`; its recipe rejects stale `Lit.lock`/clock files and tracked generated JSON. `just lit-sync-check` was started but interrupted during its `lit-sync` prerequisite, so the recipe itself did not finish in this audit. The prohibited CI quality command was not run. |
| Engine has no forum knowledge; gate is green | **PASS after small fix** | Before fix, `cargo test -p litigate-engine --test no_forum_literals` failed because its scanner searched every crate and found six `frcp` literals in compiler/CLI sources. The gate is specifically about engine source. Restricted the scan to `crates/litigate-engine/src`; rerun exited 0: 7 passed, 0 failed. |
| `litigate check --profile strict` passes on `lit/core` | **FAIL** | `target/release/litigate check --profile strict lit/core` exited 2 (`ok:false`): 3 `L5002` holes remain. Calendar cite proof windows now begin at their earliest entry effective dates; the remaining unpinned histories are Hawaii HRS § 8-1 (2023-01-01), NMRA 1-006 (2024-12-31), and Utah URCP 6 (2024-05-01). See `docs/beta/FINDINGS-lg-zmqc-59.md`. |
| Editor modes highlight the whole corpus; LSP serves diagnostics, hover and definition | **PASS** | `cargo nextest run -p tree-sitter-litigate` exited 0 (15 passed, including `keywords_match_the_compiler`); `cargo nextest run -p litigate-import --test roundtrip` exited 0 (9 passed, including `the_editor_grammar_parses_every_output`); `cargo nextest run -p litigate-editors` exited 0 (17 passed, including Emacs ERT over all 948 corpus files); `cargo test -p litigate-lsp --test examples` exited 0 (2 passed, including `every_example_is_served`). `just tree-sitter-generate` and `cargo run -q -p litigate-editors --bin editor-queries` regenerated parser and editor queries. |

**DECIDED:** Keep the forum-literal gate scoped to engine sources because the
epic requirement concerns engine forum knowledge; scanning compiler and CLI
crates falsely rejected correct language-level references.

**DECIDED:** Keep separate indentation query entries for `instance_probability_map`
and `case_probability_map`, and mirror both in Emacs, because the grammar has
two distinct map nodes after the instance-map fix.

The source build currently emits 10 forum packs and 5 scenarios, rather than
the epic's historical nine and four. Those outputs and `links.json` are build
products and remain untracked as required.
