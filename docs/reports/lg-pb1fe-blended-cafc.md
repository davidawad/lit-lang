# CAFC blended cert terminals

Origin for this change: `98ecee7b5471beee5040ecf1909c0ee6f00bcae3`.
The requested split was absent from that revision.

DECIDED: `appellant = self`, `movant = self`, `appellee = opponent` were already the package role mappings, so they stay. A panel affirmance and Rule 36 affirmance remain `[loss]`: `affirmed` and `rule36-affirmance` are dispositions against the appellant (`Fed. Cir. R. 36(a)-(b)`). The open question is how a matter whose self party is the appellee should resolve these fixed outcomes.

## Panel-result flags

| Edge | State action | Rule / reason |
|---|---|---|
| `panel-to-rule36`, `panel-to-affirmed` | Set `panel-affirmed`; clear `panel-reversed` and `panel-mixed`. | Fed. Cir. R. 36(a)-(b): affirmance decides the merits against the appellant. |
| `panel-to-reversed` | Set `panel-reversed`; clear `panel-affirmed` and `panel-mixed`. | FRAP 36(a): outright reversal decides the merits for the appellant. |
| `panel-to-mixed` | Set `panel-mixed`; clear `panel-affirmed` and `panel-reversed`. | FRAP 36(a): a mixed result has no single winner direction in this model. |
| `panel-to-remand` | Clear all three flags. | FRAP 36(a): vacatur/remand reopens the merits result and does not declare the post-remand winner. |
| `panel-to-dismissed` | Set and clear no panel-result flag. | 28 U.S.C. § 1295; FRAP 3: a jurisdictional/mootness dismissal ends without deciding or reopening the merits. A rehearing grant clears the former panel result before any fresh panel decision. |
| `rehearing-denied-edge` | Preserve the flags. | Fed. Cir. R. 40(a)(4): denial leaves the panel decision standing. |
| `rehearing-granted-to-decision` | Clear all three flags. | Fed. Cir. R. 40(e): the prior panel opinion is vacated and the new panel decision replaces it. This is the only rehearing-path clearing edge needed. |

Every merits setter clears the other two flags, so a replay cannot retain two result flags together. `panel-to-remand` clears rather than setting a direction because it vacates and reopens the result. `panel-to-dismissed` remains flagless because it is neither a merits result nor a reopening disposition.

## Cert-terminal copies

The neutral copy keeps the original terminal id, label, outcome, payoff and `payoff_by_flag`; its incoming edge forbids `panel-affirmed` and `panel-reversed`, so it is selected for `panel-mixed` or no result flag. The directional copies use the existing payoff for the panel result that sets the flag.

| Original id (neutral) | Neutral payoff | Affirmance copy | Reversal copy |
|---|---:|---|---|
| `cert-not-sought` | `0.usd`; `payoff_by_flag` remains 0 / 250,000 / 1,000,000 for affirmed / mixed / reversed | `cert-not-sought-after-affirmance`: `[final, loss]`, `0.usd` copied from `cert-not-sought`'s `panel-affirmed` payoff | `cert-not-sought-after-reversal`: `[final, win]`, `1,000,000.usd` copied from `cert-not-sought`'s `panel-reversed` payoff |
| `cert-denied` | `0.usd`; `payoff_by_flag` remains 0 / 250,000 / 1,000,000 for affirmed / mixed / reversed | `cert-denied-after-affirmance`: `[final, loss]`, `0.usd` copied from `cert-denied`'s `panel-affirmed` payoff | `cert-denied-after-reversal`: `[final, win]`, `1,000,000.usd` copied from `cert-denied`'s `panel-reversed` payoff |

The cert-denial chance retains the authored `0.99` denial / `0.01` grant distribution in every valid flag state. It has one enabled denial edge for each state: affirmed, reversed, mixed, or unset. Cert grant remains the original neutral transfer terminal. The cert-not-sought edges retain the existing Supreme Court Rule 13.1 authority and had no deadline or probability to copy.

DECIDED: no extra result flag is needed for a panel remand. `panel-to-remand` clears the prior result because vacatur reopens it. `panel-to-dismissed` sets none because the jurisdictional dismissal does not decide merits and ends the appeal. The sentence read is: “An edge that reopens the result without deciding it (rehearing granted, remand, vacatur) clears them all.”

## Matter solves

Release CLI solves use the scenario as authored; no stake, belief, or event in `cases/` or `tests/expect/` was changed. Values are the solve `value`; policy size is the engine's `policy_size`.

| Matter | Before (value / policy size) | After (value / policy size) |
|---|---:|---:|
| `lit/cafc` | 21,268 / 28 | 21,268 / 28 |
| `cases/cofc-1498-patent-case.case` | 48,423.073363 / 17 | 48,423.073363 / 17 |
| `cases/cofc-bid-protest.case` | −56,862.0125 / 16 | −56,862.0125 / 16 |
| `cases/ptab-ipr-defense.case` | 1,500,000 / 216 | 1,500,000 / 218 |
| `cases/requests/cafc-cert-preserves-panel-outcome.case` | 19,468 / 28 | 19,468 / 28 |
| `cases/requests/cofc-1498-chain.case` | 4,116,516.29109 / 193 | 4,137,903.353491 / 195 |
| `cases/requests/cofc-1498-named-scenario.case` | −12,368.7625 / 17 | −12,368.7625 / 17 |
| `cases/requests/cofc-1498-risk-averse-sim.case` | 3,174,682.328115 / 193 | 3,188,131.662862 / 195 |
| `cases/requests/cofc-1498-settle.case` | 54,472.64367 / 17 | 54,472.64367 / 17 |
| `cases/requests/cofc-1498-tornado.case` | 4,116,516.29109 / 193 | 4,137,903.353491 / 195 |
| `cases/requests/cofc-1500-trap-compare.case` | 4,116,516.29109 / 193 | 4,137,903.353491 / 195 |
| `cases/requests/cofc-appeal-threshold-sweep.case` | 4,116,516.29109 / 193 | 4,137,903.353491 / 195 |
| `cases/requests/cofc-cvar-tail-averse.case` | −293,822.499182 / 0 | −293,822.499182 / 0 |
| `cases/requests/cofc-explain-dispositive-fork.case` | 4,116,516.29109 / 193 | 4,137,903.353491 / 195 |
| `cases/requests/cofc-general-sum-settlement.case` | −13,155 / 17 | −13,155 / 17 |
| `cases/requests/cofc-limitations-fact-known.case` | 3,452,912.539313 / 17 | 3,452,912.539313 / 17 |
| `cases/requests/cofc-posterior-observed-judge.case` | 48,423.073363 / 17 | 48,423.073363 / 17 |
| `cases/requests/cofc-prejudgment-interest-sim.case` | 3,668,011.734445 / 17 | 3,668,011.734445 / 17 |
| `cases/requests/cofc-voi-mock-sj-panel.case` | 48,423.073363 / 17 | 48,423.073363 / 17 |
| `cases/requests/famous-case-noa-holiday-roll.case` | 464,966.376624 / 223 | 464,966.376624 / 225 |
| `cases/requests/ptab-calibrated-institution.case` | 502,323.108026 / 18 | 502,323.108026 / 18 |
| `cases/requests/ptab-ipr-estoppel-flag.case` | 508,433.596697 / 18 | 508,433.596697 / 18 |
| `cases/requests/ptab-robust-institution.case` | 312,876.740975 / 18 | 312,876.740975 / 18 |

The four CoFC request families with changed values use the existing `stakes: 12`. The change is in the package branch values, not their matter stakes. For `cofc-1498-chain`, `explain` at `cafc@cofc::panel-decision` moves from 2,836,392.548915 to 2,844,572.839282. Node by node, `panel-to-remand` moves 5,346,526.010931 → 5,386,053.174118 because the remand clears the prior result before the case returns through its origin tribunal; `panel-to-rule36` and `panel-to-affirmed` each move 118,931.977357 → 119,504.597674 because a fresh merits result clears any stale earlier disposition. `panel-to-reversed` stays at 12,000,000, `panel-to-mixed` at 3,000,000, and `panel-to-dismissed` at 0. The changed CoFC request root values reflect those same nodes across the reachable appeal/remand cycles. The PTAB matter and holiday-roll request keep their values; their policy sizes grow by two because the compiled flag product includes the new outcome terminals.

The public-record `tests/expect` fixtures using `cafc.world` are timeline/replay expectations, not economic solve scenarios. The `tests/expect/ptab-ipr-defense` matter is listed above with its corresponding `cases/ptab-ipr-defense.case` solve. The `tests/expect/cafc/package.toml` runs CAFC inline tests and has no separate matter solve.

## Open for David

- PLACEHOLDER payoff set on each `...after-affirmance` copy: `0.usd`, copied from the original terminal's `panel-affirmed` payoff. PLACEHOLDER payoff set on each `...after-reversal` copy: `1,000,000.usd`, copied from its `panel-reversed` payoff.
- DECIDED under R5: with appellant=self and appellee=opponent, the no-cert edge after affirmance is `by appellant`; after reversal it is `by appellee`. The cert-review edges remain court decisions. The neutral no-cert edge retains the decision's existing `movant` actor (self) because the text does not assign a single losing party for a mixed/unknown result; confirm how a mixed panel disposition should divide the cert choice.
- The package's `[loss]` outcomes for `affirmed` and `rule36-affirmance` are appellant-relative. The bead directs these to remain unsplit; confirm how a future appellee-perspective matter should read them.
- The `by appellee` lapse edge causes the package solve to report one new `mixed-node` warning: the other side's no-petition move has no authored probability and is modeled by the engine as act-or-wait. No probability was added.

## Fixtures and gates

Release re-records changed these seven files and no others:

| File | Change and reason |
|---|---|
| `crates/litigate-cli/tests/litgraph-final/packs/cafc.json` | Package-only solve `policy_size` 169 → 171 as the compiled flag product adds the directional terminals; `mixed-node` warning count 8 appears because each linked CAFC instance now has an appellee-controlled, unprobabilized lapse edge. The package solve value and authored probabilities/payoffs stay the same. |
| `crates/litigate-cli/tests/litgraph-final/requests/cafc-cert-preserves-panel-outcome.json` | Same request value, paths and terminal result; adds `mixed-node:1` for the unprobabilized other-side lapse after a reversal. |
| `crates/litigate-cli/tests/litgraph-final/requests/cofc-1498-chain.json` | Cert absorption rows now end at the affirmed/reversed copies with `[final, loss]` / `[final, win]`; aggregate dollars 858,982.750732 → 833,698.983244, elapsed 590.448754 → 574.480058 days, hours 995.325211 → 968.710719, steps 51.681912 → 51.327053, utility 4,975,499.109326 → 4,971,602.404774, net 4,116,516.358594 → 4,137,903.421530. These are the current path distribution through CAFC's remand/redecision and the split cert endpoints; no scenario inputs changed. |
| `crates/litigate-cli/tests/litgraph-final/requests/cofc-1498-risk-averse-sim.json` | Same seed 7 and 50,000 runs. Net mean 4,085,656.2614 → 4,122,344.329; CVaR −1,347,476.548 → −1,293,840.016; `p_loss` 0.3424 → 0.34268; dollar mean 858,019.7386 → 832,791.671. The new `[win]` / `[loss]` terminal tags classify the sampled result; remand/redecision clears change the reachable paths. |
| `crates/litigate-cli/tests/litgraph-final/requests/cofc-1498-tornado.json` | Base value 4,116,516.29109 → 4,137,903.353491. The `stakes` low/high values at 9 and 15 become 2,872,641.532110 / 5,360,147.506046 → 2,895,002.770623 / 5,380,912.719363; the swing changes 2,487,505.973936 → 2,485,909.948739. It re-solves the same CAFC remand/redecision and cert branches at each stake. |
| `crates/litigate-cli/tests/litgraph-final/requests/cofc-1500-trap-compare.json` | Base 4,116,516.29109 → 4,137,903.353491; §1500-bar variant 2,547,458.061294 → 2,555,024.829507; compare delta −1,569,058.229796 → −1,582,878.523984. The matter's counterfactual and stakes are unchanged; only its surviving CAFC path uses the corrected remand/redecision flags and cert outcome copies. |
| `crates/litigate-cli/tests/litgraph-final/requests/cofc-appeal-threshold-sweep.json` | At stakes 0.25 the result remains −13,155; at stakes 20 it moves 7,431,648.47749 → 7,452,413.69081. The sweep repeatedly evaluates the same CAFC branch changes; no breakpoint or parameter was repinned. |
| `crates/litigate-cli/tests/litgraph-final/requests/cofc-explain-dispositive-fork.json` | At `cofc::patent-dispositive-motion-fork`, value 5,329,526.026721 → 5,369,053.190047; trial option Q moves by the same +39,527.163326, matching the `panel-to-remand` branch delta in `cofc-1498-chain`. |

No `tests/expect` file or snapshot moved. The source-reference grep over `lit/`, `cases/`, `crates/`, `tests/`, and `docs/` confirmed the retained original IDs remain referenced only where the neutral copy is intended; the four new directional ids are declared and tested in `lit/cafc/11-cert.lit` and the CLI test. `lit/links` needs no edits because its package-level terminal references are unchanged.

Release measurements before the edit: `lit/cafc` value 21,268, policy size 28; 36 inline tests; strict check had the same three pre-existing L5002 holes after excluding the stale lock diagnostic introduced by the source edit. After: value 21,268, policy size 28; 36 passed / 0 failed; strict check has those same three holes and no new diagnostics. `cofc`, `ptab`, and `links` check files/errors and inline test counts were unchanged: respectively 120/0 and 46/0, 114/0 and 35/0, 170/0 and 3/0.

Gate results: the focused `ops` regression passed 1/1; the release roundtrip suite passed 24/24. `just lit-sync-check` exited 0 after the changed CAFC lock was committed. `litigate check lit cases` exited 0 with zero errors; strict CAFC check reported only the same three pre-existing L5002 holes as BEFORE; `litigate test lit/cafc` passed 36/36; `litigate fmt --check lit cases` and `cargo fmt --all --check` exited 0; `cargo clippy -p litigate-cli --all-targets -- -D warnings` exited 0. The CoFC, PTAB and links check/test counts match BEFORE.

`cargo nextest run -p litigate-cli --no-fail-fast` completed once: 660 tests, 656 passed and 4 failed. Failures were `impact::a_layer_edit_lists_every_moved_deadline_verdict_and_expectation` (the known main-branch red), `timeline::a_restarted_or_absorbed_period_names_the_row_replacing_it`, `timeline::frcp_filed_fixture_forks_with_unknown_deadlines`, and `ttab_answer::the_generated_pack_carries_the_designated_answer_window`. Per the gate, the latter three were run on a clone pinned to origin/main `98ecee7b5471beee5040ecf1909c0ee6f00bcae3`; all three failed there with the same snapshot source-span offsets or FRCP authority value, confirming they are pre-existing. The CAFC split test passed in nextest.
