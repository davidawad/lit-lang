# ITC blended-terminal follow-up (lg-pb1fe.53)

DECIDED: ITC has no neutral copy because no unflagged or mixed-result state can reach either appellate decision; remove the two neutral terminals and keep the strict checker unchanged, as David ruled.

## Baseline and result

`origin/main` was `98ecee7b5471beee5040ecf1909c0ee6f00bcae3`; it did not change `lit/itc`, so the prior measured baseline remains applicable. There are no `.case` matters under `cases/` or `tests/expect` using `itc-337`.

| Measurement | Before (`origin/main`) | After |
|---|---:|---:|
| `litigate solve lit/itc` value | $540,260.353525 | $223,274.580971 |
| `policy_size` | 9 | 9 |
| Best-line terminal | `cafc-affirmed` | `determination-final-unappealed-violation` |
| Inline tests | 33 passed, 0 failed | 33 passed, 0 failed |
| Strict check | 0 diagnostics | 0 diagnostics |

After `solve` does not emit an `opponent_value` field for the package's default zero-sum model. Its implied opponent value is `-$223,274.580971`.

The best line reaches the Commission's violation finding, USTR's approval by silence, and the remedy-in-effect appeal window. Both available appeal-window edges under `violation-found` belong to `respondent` (`opponent`). The solver therefore gives the respondent the move. In `crates/litigate-engine/src/algo/equilibrium.rs`, `best_option` at lines 256–264 maximizes the opponent's `opp_q`; with the default zero-sum model `opponent_value` is the negation of self's value (lines 79–87). At the violation window the respondent compares filing (self Q $565,502.793179 after the filing cost) with no appeal ($400,000) and chooses no appeal, minimizing self's value. The old node was `by complainant`, so self chose to appeal and the old best line ended at the blended $700,000 `cafc-affirmed` terminal.

The full after best line is: institution → default track → violation ID → no petition for Commission review → final violation determination → remedy determined → USTR does not disapprove → remedy in effect → respondent does not appeal → `determination-final-unappealed-violation`.

On `origin/main`, both determination paths reached the same unflagged appeal-window choice: self appealed at $677,380.124461 instead of taking the blended $400,000 no-appeal payoff. The old CAFC review value was $680,980.124642: `0.7 × $700,000 + 0.3 × $636,600.415475` for the old remand loop, less the $3,600 appeal cost (within solver rounding). The old Commission chance value was $636,600.415912: `0.35 × $677,380.124461 + 0.65 × $614,642.111783`. The new Commission value is $263,675.977613, a $372,924.438299 decrease. The root solve folds in the preceding proceeding: the measured pre-Commission adjustment changes from −$96,340.062387 to −$40,401.396642, offsetting $55,938.665745 of the decrease. Thus the root moves by `−$372,924.438299 + $55,938.665745 = −$316,985.772554`, from $540,260.353525 to $223,274.580971.

After a no-violation determination, the complainant is self and chooses to appeal: its value is $75,502.793179, greater than the $0 unappealed adverse terminal. At CAFC review, direct affirmance pays $700,000 when the violation flag is set and $0 when no-violation is set. Remand has value $263,675.977263; the loop clears both flags, returns to `commission-final-determination`, and can produce the opposite merits result. The remand continuation's terminal distribution includes a $400,000 violation-unappealed outcome (p 0.653631), a $0 affirmed-no-violation outcome (p 0.273743), and presidential disapproval ($50,000, p 0.072626).

Arithmetic from the appellate decision back to the root:

- Violation review: `0.7 × $700,000 + 0.3 × $263,675.977263 = $569,102.793179`; subtract the $3,600 filing cost to get $565,502.793179. The respondent selects the $400,000 no-appeal edge.
- Presidential review: `0.9 × $400,000 + 0.1 × $50,000 = $365,000`.
- No-violation review: `0.7 × $0 + 0.3 × $263,675.977263 = $79,102.793179`; subtract $3,600 to get the complainant's $75,502.793179 appeal value.
- Commission final determination: `0.65 × $365,000 + 0.35 × $75,502.793179 = $263,675.977613`.
- The preceding proceeding reduces that continuation by $40,401.396642, yielding the measured root value $223,274.580971. The complete root solve also reports the standard fallback warnings below; the unflagged sink copies are unreachable and do not change the root value.

The root solve reports `compile` (1: flag-state expansion), `probability-fill` (29 single-outgoing-edge residual fills), and `sink` (3 unflagged non-terminal copies with no active edges, valued at 0). The sink copies are the unflagged clones of `federal-circuit-appeal-window`, `remedy-in-effect`, and `cafc-affirmed-result`; none is reachable from the start. The lower-level direct-affirm path checks below show only the matching flagged terminal. No payoff or probability was changed to tune the solve.

## Flags, terminal copies, and routing

`commission-final-determination` sets exactly one flag on each merits edge: the no-violation edge sets `no-violation` and clears `violation-found`; the violation/remedy edge sets `violation-found` and clears `no-violation`. The CAFC remand edge and the return to the Commission both clear both flags. The Commission then sets exactly one flag again. The presidential disapproval terminal ends the case and requires no result flag.

| Retired terminal id | Directional copy | Outcome | Payoff |
|---|---|---|---:|
| `determination-final-unappealed` | `determination-final-unappealed-violation` | `[final, win]` | $400,000 |
| `determination-final-unappealed` | `determination-final-unappealed-no-violation` | `[final, loss]` | $0 |
| `cafc-affirmed` | `cafc-affirmed-violation` | `[affirmed, final, win]` | $700,000 |
| `cafc-affirmed` | `cafc-affirmed-no-violation` | `[affirmed, final, loss]` | $0 |

There are only four directional terminals. The two retired ids are not aliases. Each directional copy carries the payoff the old blended terminal already assigned to a standing violation, or $0 where no adverse merits payoff was authored. All four payoff notes mark these numbers as placeholders.

### Why lit/itc has no neutral copy

R2's neutral copy applies only if an unflagged or mixed-result state can reach the old terminal. Every path into `federal-circuit-appeal-window` and `federal-circuit-review` carries exactly one of `violation-found` or `no-violation`. The only reopening transition is remand; it clears both flags and returns to `commission-final-determination`, which sets exactly one flag before the appeal path can be reached again. No unflagged or mixed result reaches either former blended terminal. Therefore the neutral edges and terminals are retired, and there is no unflagged route.

Attempt 1 retained neutral edges behind `forbids [violation-found, no-violation]`. Strict check reported L5008 unreachable-edge diagnostics for those edges (three world instances per neutral edge); the mainline baseline had zero diagnostics. The checker was correct. The amended R2 and David's ruling require removing the unreachable neutral copies, not exempting them from L5008.

The original terminal ids retired were `determination-final-unappealed` and `cafc-affirmed`. Their replacements are respectively `determination-final-unappealed-violation` / `determination-final-unappealed-no-violation` and `cafc-affirmed-violation` / `cafc-affirmed-no-violation`.

## Appeal actors

`lit/itc/Lit.toml` now maps `respondent = "opponent"`. At the shared appeal window, the `violation-found` notice and no-appeal edges both belong to respondent; the `no-violation` notice and no-appeal edges both belong to complainant. 19 U.S.C. § 1337(c), vendored at `sources/usc-19-1337.txt:51`, says: “Any person adversely affected by a final determination of the Commission under subsection (d), (e), (f), or (g) may appeal such determination”. Thus the respondent appeals a violation determination and the complainant appeals a no-violation determination. The only other `by complainant` decision downstream of `remedy-in-effect` is the appeal window itself; it is split per edge actor as the rule permits.

The composed Federal Circuit links are also split by result context. `itc-appeal-to-cafc-no-violation` replaces edge `itc-337::edge-federal-circuit-appeal-window-appeal-filed-1` as complainant; `itc-appeal-to-cafc-violation` replaces `...appeal-filed-2` as respondent. Both links carry mutually exclusive result guards. Downstream of `remedy-in-effect`, the only `by complainant` state is that state node itself; it has no party choice. The only downstream party decision is the appeal window, whose two violation moves now belong to respondent.

`git grep -l itc -- 'lit/*/Lit.toml'` also matches mentions in the `cafc`, `core`, and `frcp` manifests. Only `lit/links/Lit.toml` declares a dependency binding on `itc-337`; the other hits are prose, a vendored source entry, or ITC family metadata. No other dependent names either retired terminal id.

## Tests and fixtures

The `.lit` inline-test language supports event/deadline assertions, not graph-path assertions. The six graph checks therefore live beside the CLI ops tests, where `explain` traverses the compiled, flag-expanded graph and `graph` confirms each terminal's outcome. They cover both unappealed copies, both affirmance copies, the mutually exclusive appeal/affirmance copies, and a violation remanded to a second determination that may be no-violation. `litigate test lit/itc` still runs the package's 33 inline tests.

| Fixture | Reader test | Built from `lit/itc`? | Treatment |
|---|---|---|---|
| `crates/litigate-cli/tests/litgraph-final/packs/itc-337.json` | `roundtrip.rs::every_package_compiles_and_solves_to_litgraph_final` | Yes | Re-recorded from the compiled ITC source. The changed solve fields and reasons are listed after re-record. |
| `tests/fixtures/ts-parity/itc-337.json` | `parity.rs::value_iteration_matches`, `absorbing_chain_matches`, `shortest_paths_match`, `min_cut_and_dominators_match`, `cyclic_components_match`, `pareto_frontier_matches` | No; it is frozen TypeScript output | Left unchanged. |
| `tests/fixtures/ts-parity/v1-packs/itc-337.json` | `parity.rs::load` (used by the parity tests above) | No; it is the frozen v1 JSON input pack | Left unchanged. |

For `litgraph-final/packs/itc-337.json`, `LITIGATE_RERECORD=1 cargo test -p litigate-cli --test roundtrip every_package_compiles_and_solves_to_litgraph_final -- --exact` passed. The reviewed before/after fields are:

- `/result/value`: $540,260.353525 → $223,274.580971; the root value reflects the corrected opponent-controlled appeal and the no-violation payoff of $0.
- `/result/iterations`: 59 → 52, from the changed flag-conditioned graph and policy.
- `/result/best_line`: 19 edges → 17. Edges 0–11 remain the default track but their `q` values now reflect the lower downstream continuation: edge 0, $540,260.353525 → $223,274.580971; edges 1–5, $635,600.415912 → $262,675.977613; edges 6–11, $636,600.415912 → $263,675.977613. Edges 12–14 retain their labels but their `q` values change from $614,642.111783 to $365,000 (edge 12/13) and $677,380.124461 to $400,000 (edge 14) as the violation branch now chooses no appeal.
- Best-line downstream node references on edges 12–16 carry `{violation-found}`; edge ids 13–16 carry the matching flag suffix. Edge 15 changes `role: self` / `kind: choice` to `role: opponent` / `kind: opponent-choice`. Edge 16 changes from filing a notice of appeal (`appeal-filed`, with `deadline_days: 60`) to the no-appeal edge and `determination-final-unappealed-violation`; its role/kind are opponent/opponent-choice and its `q` changes from $677,380.124461 to $400,000. These fields change because respondent now controls both moves at that window.
- `/warnings`: the old `probability-fill` count 24 is now 29; flag compilation adds `compile` count 1 and the three unreachable no-flag sink copies add `sink` count 3. These are the fallback warnings described above; the unflagged copies are unreachable.

## Verification

- `litigate check lit cases`: exit 0, 0 errors. It reports the existing L4016 stale-`as_of` warnings in the example matters.
- `litigate check --profile strict lit/itc`: exit 0, 0 errors and 0 warnings.
- `litigate fmt --check lit cases`: exit 0, 0 errors and 0 warnings; all listed files were unchanged.
- `litigate test lit/itc`: exit 0, 33 passed, 0 failed.
- `litigate run lit/itc --op '{"op":"validate"}'`: exit 0, valid graph (69 nodes, 87 edges).
- `cargo test -p litigate-cli --test ops itc_ -- --nocapture`: exit 0, 6 passed, 0 failed.
- `LITIGATE_RERECORD=1 cargo test -p litigate-cli --test roundtrip every_package_compiles_and_solves_to_litgraph_final -- --exact`: exit 0, 1 passed.
- `just --verbose --jobs 1 lit-sync-check`: exit 0; `Lit.lock` files and shared clock library are current, with no generated file committed. Lock changes were committed separately as `bc7e05e4`.
- `cargo clippy -p litigate-cli --all-targets -- -D warnings`: exit 0.
- `cargo fmt --all --check`: exit 0.
- `litigate check lit/links` and strict: same baseline diagnostics (7 dev warnings, 4 strict info notes); no new diagnostics. The `origin/main` archive measured the same counts.
- `cargo nextest run -p litigate-cli --no-fail-fast`: exit 100; 665 run, 661 passed, 4 failed. `a_layer_edit_lists_every_moved_deadline_verdict_and_expectation` is the known main-branch failure. The other failures were `timeline::a_restarted_or_absorbed_period_names_the_row_replacing_it`, `timeline::frcp_filed_fixture_forks_with_unknown_deadlines`, and `ttab_answer::the_generated_pack_carries_the_designated_answer_window`.
- Ran those three unexpected failures separately from `/tmp/oldtree` at origin/main `98ecee7b`; all three fail there too. The timeline snapshots contain stale source spans/line numbers after source insertions, and the TTAB assertion expects `FRCP 6(b)(1)(A)` while the unchanged generated pack has `FRCP (2017 ed.) 6(b)(1)(A)`.

## Open for David

- Placeholder payoffs: $400,000 for unappealed violation; $0 for unappealed no-violation; $700,000 for affirmed violation; and $0 for affirmed no-violation. The $400,000 unappealed violation payoff is below the $700,000 affirmed violation payoff; that ordering is carried over from the old terminal and is not a new choice here.
- Actor decision: `respondent = "opponent"`; after a violation, both appeal and no-appeal moves are respondent's; after no-violation, both belong to complainant. The statutory sentence read is the § 1337(c) sentence quoted above.
- Retired ids: `determination-final-unappealed` and `cafc-affirmed` are removed, with no aliases; their four directional replacements are listed above.
