# ITC blended terminals follow-up (lg-pb1fe.53)

DECIDED: carry `violation-found` and `no-violation` through the appeal and CAFC decision, and retain neutral copies behind both `forbids` guards -- this reflects R1/R2, but the package verifier rejects the neutral edges because there is no unflagged root path.

## Baseline

- `origin/main`: `98ecee7b5471beee5040ecf1909c0ee6f00bcae3` (contains the ITC terminals unsplit).
- `litigate solve lit/itc`: value `$540,260.353525`, `policy_size: 9`; the best line follows the violation finding, remedy in effect, appeal, and CAFC affirmance, ending at `cafc-affirmed`.
- Package matters: none. `git grep` found no `.case` matter in `cases/` or `tests/expect` that uses `lit/itc`; `tests/expect/itc/package.toml` covers the package's inline tests, not a matter.
- `litigate test lit/itc`: 33 passed, 0 failed.
- `litigate check --profile strict lit/itc`: 0 diagnostics.

## Result flags and split terminals

The final-determination chance sets exactly one result flag: the no-violation edge sets `no-violation` and clears `violation-found`; the violation edge sets `violation-found` and clears `no-violation`. The CAFC reversal/remand edge clears both because the result is reopened, and the remand return to the Commission also clears both before the Commission decides again. The § 1337(j) disapproval terminal needs no result flag because no appeal follows it.

| Original terminal | Favourable copy | Adverse copy | Neutral copy |
|---|---|---|---|
| `determination-final-unappealed` | `determination-final-unappealed-violation`, `[final, win]`, $400,000 | `determination-final-unappealed-no-violation`, `[final, loss]`, $0 | `determination-final-unappealed`, `[final]`, $400,000 |
| `cafc-affirmed` | `cafc-affirmed-violation`, `[affirmed, final, win]`, $700,000 | `cafc-affirmed-no-violation`, `[affirmed, final, loss]`, $0 | `cafc-affirmed`, `[affirmed, final]`, $700,000 |

The directional copies retain the blended payoff on the favourable violation result because the package has no separate merits-terminal payoff. The adverse no-violation result has no authored payoff, so its copy is $0. The original terminal IDs and payoffs remain on the neutral copies. All are marked `PLACEHOLDER` in their notes.

## Appeal actors

19 U.S.C. § 1337(c), vendored at `sources/usc-19-1337.txt:51`, says: “Any person adversely affected by a final determination of the Commission under subsection (d), (e), (f), or (g) may appeal such determination”. The no-violation branch's appellant is the complainant; the violation/remedy branch's appellant is the respondent. The latter appeal-window entry and filing edge are explicitly `by respondent`; the no-violation appeal edge remains `by complainant`. The node declaration remains `by complainant`, with the violation edge actor overriding it. No other `by complainant` decision node occurs downstream of `remedy-in-effect`; its appeal transition is explicitly assigned to respondent.

## Verification blocker

The required neutral terminal edges are guarded by `forbids [violation-found, no-violation]`. But both incoming final-determination edges set one of these flags, and the only edge that clears them goes to the remand state and back to a new final-determination draw, which sets a result flag again. Consequently, the appeal window and CAFC review are reached only with one result flag set. `litigate check --profile strict lit/itc` reports new `L5008` diagnostics for each neutral edge (three world instances each); `solve` stops on the same diagnostics. Baseline strict check had zero diagnostics.

Removing the neutral guards would make the neutral outcome selectable even when the Commission result is known, contrary to R2. Adding an unflagged path through the appeal would misrepresent the procedure. The current flag/neutral-copy rule and the strict-check acceptance requirement therefore conflict for this graph. No package solve after the edit, targeted terminal path test, fixture re-record, full check, format gate, or final CLI gates can pass until this conflict is resolved.

No matter stakes, beliefs, or events were edited. No compiled-package golden was updated because the package cannot compile past `L5008`. The split IDs occur only in the edited ITC rule file; no references in matters, crates, tests, docs, or `lit/links` require migration.

## Open for David

- Payoffs set/copy under R3: `$400,000` on `determination-final-unappealed-violation` (retained because violation stands); `$0` on `determination-final-unappealed-no-violation` (adverse result has no authored merits payoff); `$400,000` on neutral `determination-final-unappealed` (existing payoff retained); `$700,000` on `cafc-affirmed-violation` (retained because violation stands); `$0` on `cafc-affirmed-no-violation` (adverse result has no authored merits payoff); `$700,000` on neutral `cafc-affirmed` (existing payoff retained).
- Actor decisions under R5: complainant appeals after no violation; respondent appeals after violation/remedy takes effect. The law sentence read is quoted above. The `determination-vetoed` terminal remains non-appealable because § 1337(j) says the determination and action “shall have no force or effect.”
- Required design decision: how should an unreachable neutral copy be represented so it remains available only when no result flag is set or the result is mixed, while `litigate check --profile strict` emits no new `L5008` diagnostics?
