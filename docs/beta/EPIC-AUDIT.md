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
