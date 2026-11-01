# Scoped gate timing

Measured on 2026-11-01 in this pod with `CARGO_BUILD_JOBS=4`. The baseline engine nextest command was `cargo nextest run --profile gate --no-tests=warn -p litigate-engine --message-format libtest-json` with `NEXTEST_EXPERIMENTAL_LIBTEST_JSON=1`; it ran 636 tests in 11.6s, 635 passed, 1 failed, and 1 skipped. The failing existing test is `clock_cycle::every_shipped_rule_rolls_to_an_open_day_from_every_date` (assertion at `crates/litigate-engine/tests/clock_cycle.rs:27`, rule `frcp6-connecticut`). No observed test exceeded 30s, so no additional test was removed from the gate profile. The existing gate profile filter continues to keep the known corpus-wide sweeps nightly-only; they remain in the default profile.

## 30 slowest measured engine tests

| Seconds | Test | Result |
|---:|---|---|
| 3.302 | `litigate-engine::litigate_engine$cite::corpus::tests::default_source_honors_litgraph_sources_override` | ok |
| 2.283 | `litigate-engine::scenario_gaps$lint_reports_bad_pack_start_when_a_packs_own_metadata_is_inconsistent` | ok |
| 1.572 | `litigate-engine::cite_coverage_report$report_cite_verification_across_all_embedded_packs` | ok |
| 1.497 | `litigate-engine::litigate_engine$cite::corpus::tests::embedded_corpus_loads_and_has_known_sections` | ok |
| 1.481 | `litigate-engine::coverage_gaps$lint_covers_every_diagnostic_code` | ok |
| 1.458 | `litigate-engine::coverage_gaps$lint_covers_v2_only_codes` | ok |
| 1.125 | `litigate-engine::litigate_engine$api::validate::tests::detects_and_validates_each_kind` | ok |
| 1.089 | `litigate-engine::litigate_engine$api::validate::tests::validates_clock_fragments_and_pack_counting_references` | ok |
| 1.069 | `litigate-engine::litigate_engine$lint::tests::lint_reports_unverifiable_cite_via_public_entry_point` | ok |
| 1.064 | `litigate-engine::api_coverage$lint_reports_diagnostics` | ok |
| 1.062 | `litigate-engine::litigate_engine$lint::tests::mixed_fact_node_without_a_full_interrupt_prior_is_flagged` | ok |
| 1.061 | `litigate-engine::api_coverage$batch_runs_every_sub_op` | ok |
| 1.049 | `litigate-engine::litigate_engine$lint::tests::a_plain_chance_node_without_the_fact_tag_still_gets_chance_unquantified` | ok |
| 1.042 | `litigate-engine::litigate_engine$lint::tests::a_gated_chance_node_sums_to_one_per_flag_context` | ok |
| 1.041 | `litigate-engine::litigate_engine$lint::tests::fact_node_with_a_full_prior_is_not_flagged` | ok |
| 1.035 | `litigate-engine::litigate_engine$lint::tests::fact_node_without_a_full_prior_is_flagged` | ok |
| 1.031 | `litigate-engine::litigate_engine$lint::tests::a_node_gated_on_too_many_flags_is_summed_as_if_ungated` | ok |
| 1.025 | `litigate-engine::litigate_engine$api::validate::tests::rejects_a_pack_whose_cite_syntax_names_an_unknown_source` | ok |
| 1.020 | `litigate-engine::litigate_engine$cite::corpus::tests::empty_and_embedded_corpora_report_emptiness` | ok |
| 1.007 | `litigate-engine::litigate_engine$lint::tests::mixed_fact_node_with_a_full_interrupt_prior_is_not_flagged` | ok |
| 0.573 | `litigate-engine::source_provenance$every_vendored_text_has_its_sha256_recorded` | ok |
| 0.434 | `litigate-engine::clock_cycle$every_shipped_rule_rolls_to_an_open_day_from_every_date` | failed |
| 0.390 | `litigate-engine::canonical_order$node_link_and_pack_order_leave_every_analysis_bit_identical` | ok |
| 0.168 | `litigate-engine::calibration$every_embedded_calibration_entry_resolves` | ok |
| 0.145 | `litigate-engine::api_gaps$full_catalog_surfaces_compile_notes_and_caps_warning_locations` | ok |
| 0.114 | `litigate-engine::uncertainty$voi_matches_the_beta_closed_forms` | ok |
| 0.106 | `litigate-engine::calibration$calibration_op_lists_gaps_ranked_by_swing` | ok |
| 0.102 | `litigate-engine::litigate_engine$api::catalog::tests::compile_without_links_uses_an_empty_link_file` | ok |
| 0.095 | `litigate-engine::uncertainty_api$a_node_on_a_cycle_gets_evppi_not_outcome_evpi` | ok |
| 0.087 | `litigate-engine::uncertainty_props$information_values_are_ordered_and_vanish_when_moot` | ok |

## Engine-only gate estimate

The stage measurements below used the gate-equivalent command where applicable. Expect is skipped for a pure Rust engine edit. Unmeasured global stages use the existing branch gate command paths; estimate is approximately 5 minutes on this warm pod, with repository-wide `cargo check` the largest measured item. A cold build can take longer.

| Stage | Seconds | Command / note |
|---|---:|---|
| fmt | 3 | `cargo fmt --all -- --check` |
| check | 135 | `cargo check --workspace --all-targets` |
| clippy | 30+ | scoped engine clippy attempt; did not pass because existing `litigate-cli` clippy diagnostics (`too_many_lines`, `case_sensitive_file_extension_comparisons`) |
| nextest | 12 | engine package, 636 tests, one failure noted above |
| doctest | 80 | `cargo test --doc -p litigate-engine` |
| expect | 0 | skipped on engine-only changes; ten-case clock/lowering smoke timing unavailable (manual attempt interrupted after it substantially exceeded 5 minutes) |
