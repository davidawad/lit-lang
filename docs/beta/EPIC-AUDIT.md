# Litigation language epic audit on `origin/main`

Audited snapshot: `876e2b6e` (`origin/main`, fetched from GitLab on
2026-11-01). The requested quality command (`just ci` / `just ci-scoped`) was
not run, as directed by the dispatch instructions.

| Epic Done-when item | Verdict | Command and output summary |
|---|---|---|
| Nine packs, links and four scenarios are generated from source with semantic equality | **FAIL** | `target/release/litigate build <crates/litgraph-packs/build.args> --out /tmp/lg-zmqc-build` exited 0; emitted 10 forum packs, `links.json`, and 5 scenarios. The epic's stated nine/four counts are stale. There are no tracked legacy pack/scenario JSON files (`git ls-files 'packs/**/*.json' scenarios` returned 0), so equality against the former hand-authored JSON cannot be checked on this main snapshot. Current source compilation succeeds; the historical equality claim lacks its comparison inputs. |
| Existing tests pass against generated files | **FAIL** | `cargo test --workspace` did not complete (interrupted after the CLI crate had started); it reported `timeline::tests::the_worked_example_forks_at_the_motion_ruling` failing. Isolated reproduction, `cargo test -p litigate-cli the_worked_example_forks_at_the_motion_ruling -- --nocapture`, exited 101: actual probability `0.5`, expected `null`. No complete workspace pass was established. |
| CI enforces generated-source freshness (`lit-sync`) | **PASS** | `justfile` declares `ci: ... lit-sync-check ...`, and `lit-sync-check` depends on `lit-sync`; its recipe rejects stale `Lit.lock`/clock files and tracked generated JSON. `just lit-sync-check` was started but interrupted during its `lit-sync` prerequisite, so the recipe itself did not finish in this audit. The prohibited CI quality command was not run. |
| Engine has no forum knowledge; gate is green | **PASS after small fix** | Before fix, `cargo test -p litigate-engine --test no_forum_literals` failed because its scanner searched every crate and found six `frcp` literals in compiler/CLI sources. The gate is specifically about engine source. Restricted the scan to `crates/litigate-engine/src`; rerun exited 0: 7 passed, 0 failed. |
| `litigate check --profile strict` passes on `lit/core` | **FAIL** | `target/release/litigate check --profile strict lit/core` returned `ok:false`, with 41 `L5002` errors. These report missing pinned amendment-history proof for the cited state-law source texts. Resolving them requires source/provenance research across the state corpus, beyond a small gap fix; no legal history was inferred. |
| Editor modes highlight the whole corpus; LSP serves diagnostics, hover and definition | **FAIL** | Fixed the `.lit` instance probability map being shadowed by the narrower `.case` map. `just tree-sitter-test` now exits 0: 89/89 corpus parses, highlighting assertions and parser merge test pass. `cargo test -p litigate-lsp --test examples` exits 0 (2 passed, including `every_example_is_served`). However `cargo test -p litigate-editors --test emacs -- --nocapture` exits 101: 3 ERT failures (`litigate-ts-mode-every-corpus-file` at `lit/frap/appeal-periods.lit`, `litigate-ts-mode-faces`, and `litigate-ts-mode-indentation-is-canonical`). Thus the full editor-mode claim remains red. |

**DECIDED:** Keep the forum-literal gate scoped to engine sources because the
epic requirement concerns engine forum knowledge; scanning compiler and CLI
crates falsely rejected correct language-level references.

**DECIDED:** Give `.lit` instances their own probability-map grammar rule;
the `.case` map deliberately accepts only edge references and numeric values,
while instance maps must parse holes and exact fractions.

The source build currently emits 10 forum packs and 5 scenarios, rather than
the epic's historical nine and four. Those outputs and `links.json` are build
products and remain untracked as required.
