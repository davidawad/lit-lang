# Engine calibration typed provenance

DECIDED: Keep the op's current sensitivity descending within each basis, then
sort by source pointer; this preserves the existing sensitivity ranking while
making same-rank candidates repeatable.

`litigate calibration lit/frcp` was run with `target/release/litigate` before
and after this change. The original API output listed these first 15
probability inputs:

| Rank | Input | Swing |
|---:|---|---:|
| 1 | `p:frcp-civil-procedure::edge-amended-complaint-filed-amended-response-window` | 0 |
| 2 | `p:frcp-civil-procedure::edge-complaint-filed-service-window` | 0 |
| 3 | `p:frcp-civil-procedure::edge-cross-appeal-filed-appellate-briefing` | 0 |
| 4 | `p:frcp-civil-procedure::edge-default-entered-default-judgment-entered` | 0 |
| 5 | `p:frcp-civil-procedure::edge-discovery-closed-summary-judgment-motion` | 0 |
| 6 | `p:frcp-civil-procedure::edge-discovery-closed-daubert-motion-filed` | 0 |
| 7 | `p:frcp-civil-procedure::edge-discovery-open-protective-order-motion` | 0 |
| 8 | `p:frcp-civil-procedure::edge-discovery-open-voluntary-dismissal` | 0 |
| 9 | `p:frcp-civil-procedure::edge-final-pretrial-conference-trial-commences` | 0 |
| 10 | `p:frcp-civil-procedure::edge-notice-of-appeal-filed-cross-appeal-window` | 0 |
| 11 | `p:frcp-civil-procedure::edge-parties-joined-rule26f-conference` | 0 |
| 12 | `p:frcp-civil-procedure::edge-reply-evidence-ruling-sur-reply-allowed` | 0 |
| 13 | `p:frcp-civil-procedure::edge-reply-evidence-ruling-evidence-excluded` | 0 |
| 14 | `p:frcp-civil-procedure::edge-reply-evidence-ruling-request-denied` | 0 |
| 15 | `p:frcp-civil-procedure::edge-rule60-relief-granted-reopened-case` | 0 |

The operation's output value remains `base_value = 478445`; probability
perturbations, their low/high values, and their sensitivity values are
unchanged. After the change, equal-sensitivity rows sort by source pointer.
The first 15 probability inputs were:

| Rank | Input | Pointer | Basis | Swing |
|---:|---|---|---|---:|
| 1 | `p:frcp-civil-procedure::edge-final-pretrial-conference-trial-commences` | `/edges/109/probability` | estimate | 0 |
| 2 | `p:frcp-civil-procedure::edge-final-pretrial-conference-trial-commences{waived-12h-defenses}` | `/edges/109/probability` | estimate | 0 |
| 3 | `p:frcp-civil-procedure::edge-trial-commences-verdict-rendered` | `/edges/112/probability` | estimate | 0 |
| 4 | `p:frcp-civil-procedure::edge-trial-commences-verdict-rendered{waived-12h-defenses}` | `/edges/112/probability` | estimate | 0 |
| 5 | `p:frcp-civil-procedure::edge-notice-of-appeal-filed-cross-appeal-window` | `/edges/135/probability` | estimate | 0 |
| 6 | `p:frcp-civil-procedure::edge-notice-of-appeal-filed-cross-appeal-window{fee-motion-filed}` | `/edges/135/probability` | estimate | 0 |
| 7 | `p:frcp-civil-procedure::edge-notice-of-appeal-filed-cross-appeal-window{fee-motion-filed,fee-motion-supplemented}` | `/edges/135/probability` | estimate | 0 |
| 8 | `p:frcp-civil-procedure::edge-notice-of-appeal-filed-cross-appeal-window{waived-12h-defenses}` | `/edges/135/probability` | estimate | 0 |
| 9 | `p:frcp-civil-procedure::edge-notice-of-appeal-filed-cross-appeal-window{fee-motion-filed,waived-12h-defenses}` | `/edges/135/probability` | estimate | 0 |
| 10 | `p:frcp-civil-procedure::edge-notice-of-appeal-filed-cross-appeal-window{fee-motion-filed,fee-motion-supplemented,waived-12h-defenses}` | `/edges/135/probability` | estimate | 0 |
| 11 | `p:frcp-civil-procedure::edge-cross-appeal-filed-appellate-briefing` | `/edges/138/probability` | estimate | 0 |
| 12 | `p:frcp-civil-procedure::edge-cross-appeal-filed-appellate-briefing{fee-motion-filed}` | `/edges/138/probability` | estimate | 0 |
| 13 | `p:frcp-civil-procedure::edge-cross-appeal-filed-appellate-briefing{fee-motion-filed,fee-motion-supplemented}` | `/edges/138/probability` | estimate | 0 |
| 14 | `p:frcp-civil-procedure::edge-cross-appeal-filed-appellate-briefing{waived-12h-defenses}` | `/edges/138/probability` | estimate | 0 |
| 15 | `p:frcp-civil-procedure::edge-cross-appeal-filed-appellate-briefing{fee-motion-filed,waived-12h-defenses}` | `/edges/138/probability` | estimate | 0 |

The required CLI filter was run once on `origin/main` (`10b6a048`). It had
four failures: `anytime_filing::an_opposition_during_the_consent_decision_passes_lint_filing`,
`cov_cmd_calibrate::chance_odds_and_durations_come_from_docketed_steps_with_their_provenance`,
`lint_filing_paper::a_violation_fails_the_lint_of_an_accepted_filing`, and
`pending_moves::lint_filing_a_stipulation_lists_the_move_it_makes_pending`.
On this branch the final run had the same four failures (31 passed, 4 failed).
The new CLI provenance test initially failed due to its assertion reading the
outer CLI envelope instead of the nested engine result; that test was corrected.
`roundtrip::calibration_gaps`
was re-recorded only because calibration rows gained `pointer` and `basis`
and their equal-sensitivity order is now pointer-sorted. The old and new
snapshots retain the same `base_value`, low/high perturbation values, per-row
low/high outputs, swings, policy-change flags, truncation flags, and warnings;
no pinned numeric calibration value changed.

Measured checks: `cargo nextest run -p litigate-engine -p litigate-lower
--no-fail-fast` passed 807 tests (1 skipped); the final CLI filter had 31
passes and the four `origin/main` failures above; `cargo fmt --all --check`
and `cargo clippy --workspace --all-targets -- -D warnings` passed.
