# PTAB blended-terminal split (lg-pb1fe.52)

Base: `origin/main` at `98ecee7b5471beee5040ecf1909c0ee6f00bcae3`.

## Decision

DECIDED: Use `fwd-all-unpatentable` and `fwd-all-confirmed` as mutually exclusive flags. A mixed FWD clears both, so it and an unknown result share the neutral copies. The authoring verifier checks chance distributions in every flag context, including an unreachable context where both flags are present; the CAFC adverse copy requires `fwd-all-confirmed` without forbidding `fwd-all-unpatentable`, so that context still has the complete authored 0.8/0.2 distribution. On authored paths, every setter clears the other result flag and both flags are never set together.

The `federal-circuit-window` edges currently declare no `by` actor; their lowered `role: nature` comes from the engine's probability-fill behavior for those unassigned edges. They therefore resolve as a 0.5 / 0.5 chance in every result state. I left actors unchanged under the conditional instruction to copy an actor only if an edge has one. Whether the losing party should own that choice is David's decision.

## Result flags

| Edge | State action | Rule basis |
|---|---|---|
| `edge-fwd-issued-fwd-all-unpatentable` | Sets `fwd-all-unpatentable` and `ipr-estopped`; clears `fwd-all-confirmed`. | 35 U.S.C. § 318(a) |
| `edge-fwd-issued-fwd-all-confirmed` | Sets `fwd-all-confirmed` and `ipr-estopped`; clears `fwd-all-unpatentable`. | 35 U.S.C. § 318(a) |
| `edge-fwd-issued-fwd-mixed` | Sets `ipr-estopped`; clears both result flags. | 35 U.S.C. § 318(a) |
| `edge-rehearing-decision-rehearing-granted-modified` | Clears both; a granted rehearing modifies the operative FWD without a modeled winner. | 37 C.F.R. § 42.71(d) |
| `edge-director-review-decision-director-review-modified` | Clears both; the modified result is not modeled. | 37 C.F.R. § 42.75(e)(2) |
| `edge-director-review-decision-patent-owner-response-window` | Clears both; Director remand reopens proceedings without deciding the result. | 37 C.F.R. § 42.75(e)(2) |
| `edge-cafc-vacates-remands-fwd-issued` | Clears both before the Board issues a new FWD. | 35 U.S.C. § 319; 37 C.F.R. § 42.73 |

The result setters are mutually exclusive by construction: each sets one result flag and clears the other. Reopening edges clear both. The mixed result has no result flag, while `ipr-estopped` is preserved independently.

## Split terminals

All four directional payoffs are PLACEHOLDER values. The neutral terminals retain their original IDs, labels, outcomes, and package payoffs; the matter's existing stake keyed to an original terminal therefore still applies to that neutral copy.

| Blended terminal | Favourable copy | Adverse copy | Neutral copy |
|---|---|---|---|
| `appeal-forfeited-unappealed` | `appeal-forfeited-unappealed-win`, outcome adds `win`, payoff `$1,000,000` from `fwd-all-unpatentable` | `appeal-forfeited-unappealed-loss`, outcome adds `loss`, payoff `$0` from `fwd-all-confirmed` | Original ID and label, outcome `[final, appeal-forfeited]`, payoff `$400,000` |
| `cafc-affirms` | `cafc-affirms-win`, outcome adds `win`, payoff `$1,000,000` from `fwd-all-unpatentable` | `cafc-affirms-loss`, outcome adds `loss`, payoff `$0` from `fwd-all-confirmed` | Original ID and label, outcome `[affirmed, final]`, payoff `$450,000` |

Each no-appeal incoming edge copy retains the original authority, deadline, duration, and unprobabilized choice data. Each CAFC-affirmance copy retains authority and `p=0.8`; `forbids`/`requires` select exactly one outcome for each reachable flag context. The CAFC remand edge remains `p=0.2`, so its chance distribution remains 1 in known-win, known-loss, neutral, and verifier-only both-flags contexts.

## Actor decision (R5)

DECIDED: No actor was added to the copied federal-circuit-window edges because those source edges have no actor declaration; the conditional in the instruction applies only when an edge already carries an actor. No `[package.roles]` mapping or node kind changed. The `either` discussed in the source comment belongs to the incoming notice-of-appeal link and is unchanged.

Open wording considered: “`federal-circuit-window` is the losing party's choice. If its edges carry an actor today, the copy after all-unpatentable is the patent owner's (`patent-owner`), after all-confirmed the petitioner's, the neutral copy stays as it is (the DECIDED comment above `federal-circuit-notice-of-appeal` explains the present `either`).” The current source has no `by` on either edge; it does not specify whether to introduce actors where none exist.

## Solve values

Release solve on the exact base above and after the split. Policy size is the engine's `policy_size`; the invalid expect matter has no solve value or policy size on either revision.

| Target | Before value / policy size | After value / policy size | Change and explanation |
|---|---:|---:|---|
| `lit/ptab` | `508433.596697` / 18 | `543489.544398` / 18 | `+$35,055.947701`. The package solve continues beyond the three FWD merits terminals through their outgoing post-FWD edges. At `fwd-issued`, the unpatentable option moves from `$716,640.632072` to `$939,914.489888`; confirmed moves from `$216,640.632072` to `$59,289.489888`; mixed moves from `$416,640.632072` to `$419,997.823221`. Specifically, after unpatentable, the no-appeal/affirm branches now pay `$1,000,000` and retain the neutral `$400,000` only on a cleared/mixed path; after confirmed, no-appeal/affirm now pay `$0` and only cleared/mixed paths retain the neutral payoffs. At the package root, the 0.55/0.25/0.20 FWD distribution and preceding costs then yield the recorded change. No matter stake was changed. |
| `cases/ptab-ipr-defense.case` | `1500000.0` / 216 | `1500000.0` / 288 | Same root value: its chosen line stops at `claims-disclaimed-preinstitution`, before the FWD. Policy size increases because the expanded reachable graph now includes the result-flag states. |
| `cases/requests/ptab-ipr-estoppel-flag.case` | `508433.596697` / 18 | `543489.544398` / 18 | Same package start and default scenario, so the same result-terminal branch changes as `lit/ptab`. |
| `cases/requests/ptab-robust-institution.case` | `312876.740975` / 18 | `341305.909603` / 18 | `+$28,429.168628`; unchanged matter assumptions, with the weighted FWD branch values changing at no-appeal/affirm as listed above. |
| `cases/requests/ptab-calibrated-institution.case` | `502323.108026` / 18 | `551360.206142` / 18 | `+$49,037.098116`; unchanged calibration and matter assumptions, with the weighted FWD branch values changing at no-appeal/affirm as listed above. |
| `cases/requests/famous-case-noa-holiday-roll.case` | `464966.376624` / 223 | `485562.967553` / 295 | `+$20,596.590929`; the solve's reachable post-FWD graph now carries result flags and directional terminal outcomes. Its filing/deadline data was not changed. |
| `tests/expect/ptab-ipr-defense/ptab-ipr-defense.case` | invalid; no value / policy size | invalid; no value / policy size | Existing main error: `applicant` is not an actor in the PTAB package (valid actors include `petitioner` and `patent-owner`). The expect matter was not edited. |

The instruction expected package values to remain unchanged because the solver was expected to stop at the FWD terminals. The release solve/explain output shows those terminals have post-FWD continuations: `fwd-issued`'s result options include no-appeal and CAFC branches. I preserved all matter stakes and documented the actual, rule-driven value changes instead of re-pinning any matter.

## Fixtures and verification

`crates/litigate-cli/tests/litgraph-final/packs/ptab-patent-trial-appeal-board.json` was re-recorded with `LITIGATE_RERECORD=1 cargo test --release -p litigate-cli --test roundtrip every_package_compiles_and_solves_to_litgraph_final -- --exact` (1 passed). It changes the PTAB solve value, ancestor `q` values that reflect the changed terminal outcomes, state-expanded node IDs, and the compiler/probability-fill warning counts. The three PTAB request fixtures were re-recorded with `LITIGATE_RERECORD=1 cargo test --release -p litigate-cli --test roundtrip ptab_` (3 passed): `ptab-calibrated-institution.json` changes its explain value and downstream probabilities/IDs; `ptab-ipr-estoppel-flag.json` changes the FWD flag-state nodes and transition IDs; `ptab-robust-institution.json` changes the robust and nominal values, best-line `q` values, and result-state IDs. All four PTAB fixture tests pass again without `LITIGATE_RERECORD`. No other package fixture, snapshot, matter/expect file, authored probability, or duration changed.

`lit/ptab/Lit.lock` was regenerated by `just lit-sync`; it changes the digest for `03-trial-to-appeal.lit` only. The generated pack output remains in `target/lit-build/` and is not committed.

Before/after controls: `lit/ptab` inline tests 35 passed / 0 failed on both; `lit/links` check had 0 errors and its inline tests had 3 passed / 0 failed on both. The base strict PTAB check had 2 `L5002` errors; the post-change strict check retains those same two extension-period errors and introduces no new diagnostic. The base expect-case actor error noted above is also unchanged.

Final checks: `litigate run lit/ptab --op '{"op":"validate"}'` exited 0 (`valid: true`); `litigate lint lit/ptab` exited 0 with its existing content diagnostics. `litigate check lit cases` exited 0 with 0 errors; `litigate fmt --check lit cases` and `cargo fmt --all --check` exited 0; `cargo clippy -p litigate-cli --all-targets -- -D warnings` exited 0; targeted PTAB ops, link composition, package roundtrip, and the three PTAB request roundtrips passed. The one full `cargo nextest run -p litigate-cli --no-fail-fast` run reported 653 passed and 7 failed. The three PTAB request failures were corrected by the re-records above. The known main failure `impact::a_layer_edit_lists_every_moved_deadline_verdict_and_expectation` remains. Three other failures reproduced on the unmodified base clone: `timeline::a_restarted_or_absorbed_period_names_the_row_replacing_it`, `timeline::frcp_filed_fixture_forks_with_unknown_deadlines`, and `ttab_answer::the_generated_pack_carries_the_designated_answer_window`.

The unrelated baseline failures are stale fixtures: the two FRCP timeline snapshots record older line/byte offsets in `lit/frcp/periods.lit` and `lit/frcp/fees.lit`; the TTAB test expects `FRCP 6(b)(1)(A)` while the unchanged current rule source emits `FRCP (2017 ed.) 6(b)(1)(A)`. Their `.snap` files and TTAB rule were left untouched.

## Open for David

- Review the four placeholder payoffs introduced by R3: `appeal-forfeited-unappealed-win` `$1,000,000`; `appeal-forfeited-unappealed-loss` `$0`; `cafc-affirms-win` `$1,000,000`; `cafc-affirms-loss` `$0`. Each directional payoff is copied from the corresponding operative FWD terminal, as R3 directs; no neutral payoff changed.
- The `federal-circuit-window` edges carry no actor, so the engine fills the appeal choice as a 0.5 / 0.5 chance in every result state; whether the losing party should own that choice is David's. The existing source sentence says: “Only actual trial participants may appeal — no alternative civil-action route like 35 U.S.C. § 145 exists for AIA trials.” The comment also says: “The incoming notice edge is `either`, so it does not identify a particular appealing party.”
- The solver follows post-FWD continuations from the FWD merits terminals, contrary to the expected-unchanged note in step 1(d). The changed branch and node values are reported above; no matter input was changed.

The PTAB CLI assertion now lives in `crates/litigate-cli/tests/blended_ptab.rs`; `crates/litigate-cli/tests/ops.rs` is unchanged from `origin/main` to avoid recurring test-list conflicts.
