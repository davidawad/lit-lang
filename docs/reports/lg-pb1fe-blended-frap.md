# FRAP blended terminal split (lg-pb1fe.51)

DECIDED: use `panel-affirmed`, `panel-reversed`, and `panel-reversed-and-remanded` as mutually exclusive result flags -- each merits edge sets one and clears the other two; non-directional results and rehearing grants clear all three.

DECIDED: a directional split copy keeps the package's existing merits payoff for the result flag that reaches it; the original terminal stays the neutral copy with its existing label, outcome, and $400,000 placeholder.

DECIDED: keep the source's existing actors and role map on copied edges -- `[package.roles]` already maps `appellant = self`, `appellee = opponent`, and `movant = self`, while tribunal decisions are `by court`.

## Flags

| Source edge | Action | Authority / rule cite |
|---|---|---|
| `de-novo-review` → `affirmed` | Set `panel-affirmed`; clear `panel-reversed` and `panel-reversed-and-remanded` | FRAP 36(a) |
| `de-novo-review` → `reversed` | Set `panel-reversed`; clear the other two | FRAP 36(a) |
| `de-novo-review` → `reversed-and-remanded` | Set `panel-reversed-and-remanded`; clear the other two | FRAP 36(a) |
| `abuse-of-discretion-review` → the same three results | Apply the same flag action per result | FRAP 36(a) |
| `clear-error-review` → the same three results | Apply the same flag action per result | FRAP 36(a) |
| Any standard-of-review chance edge → `vacated-and-remanded` or `affirmed-in-part-reversed-in-part` | Clear all three; these results have no directional flag | FRAP 36(a) |
| `panel-rehearing-decision` → `rehearing-granted-amended-opinion` | Clear all three; the direction of the amended opinion is not modeled | FRAP 40(a)(4) |
| `en-banc-poll-decision` → `rehearing-granted-amended-opinion` | Clear all three; the direction of the amended opinion is not modeled | FRAP 35(a), 40(a)(4), 28 U.S.C. § 46(c) |

The chance edges on the three merits nodes retain their original probabilities. Rehearing and certiorari chance distributions are split into flag-gated copies. Each enabled context retains the original probabilities (0.05/0.95, 0.02/0.98, or 0.99/0.01); `check` verifies the chance distributions.

## Split terminals and payoffs

`self` is the appellant. Thus an affirmance preserves the appellant's loss; reversal and reversal-and-remand preserve its win. Reverse and reverse-and-remand use separate flags because the authored merits payoffs differ.

| Original neutral copy | After affirmance (`panel-affirmed`) | After reversal (`panel-reversed`) | After reversal and remand (`panel-reversed-and-remanded`) |
|---|---|---|---|
| `rehearing-en-banc-both-denied` — original ID, label, `[final]`, $400,000 | `rehearing-en-banc-both-denied-after-affirmance` — `[final, loss]`, $0 | `rehearing-en-banc-both-denied-after-reversal` — `[final, win]`, $1,000,000 | Same favorable copy — `[final, win]`, $600,000 by `payoff_by_flag` |
| `cert-not-sought` — original ID, label, `[final]`, $400,000 | `cert-not-sought-after-affirmance` — `[final, loss]`, $0 | `cert-not-sought-after-reversal` — `[final, win]`, $1,000,000 | Same favorable copy — `[final, win]`, $600,000 by `payoff_by_flag` |
| `cert-denied` — original ID, label, `[final]`, $400,000 | `cert-denied-after-affirmance` — `[final, loss]`, $0 | `cert-denied-after-reversal` — `[final, win]`, $1,000,000 | Same favorable copy — `[final, win]`, $600,000 by `payoff_by_flag` |

The original neutral IDs are selected only with no result flag. `vacated-and-remanded`, the mixed disposition, and granted rehearing therefore use the neutral copies. Cert/rehearing edges retain their original actor, authority, probability, and deadline fields. Every rehearing-denial copy retains the “Mandate clock resumes” continuation. The continuation compiler makes that node a decision and adds a synthetic `#end` terminal; the copied outcome and payoff appear on that final terminal.

Each favorable copy's authored `payoff 0` is only the fallback for a direct `scenario.start` with no flag, following the form in `lit/cafc/11-cert.lit`. The result-specific paths use `payoff_by_flag`, so the fallback does not assign the adverse merits value to the favorable path.

## BEFORE / AFTER

Baseline captured from origin/main `98ecee7b5471beee5040ecf1909c0ee6f00bcae3` using the release CLI built before the edits.

| Target | BEFORE | AFTER |
|---|---:|---:|
| `litigate solve lit/frap` value | 422,291.25 | 422,291.25 |
| `litigate solve lit/frap` policy_size | 19 | 31 |
| `litigate test lit/frap` | 42 passed, 0 failed | 42 passed, 0 failed |
| `litigate check --profile strict lit/frap` diagnostics | 0 | 0 |
| FRAP matters in `cases/` or `.case` matters selected by `tests/expect/frap` | none | none |

The root value and its selected line are unchanged because the solver stops at the panel merits terminals. `policy_size` rises by 12 because the compiled graph now carries result-flag product states and separate terminal copies; no matter stakes or outcomes were changed. The expectation records under `tests/expect/frap/` point to docket-history `.case` files using the separate `change-deadline-frap-base` test package, not `frap-appellate-procedure`; none enters a posture past this package's panel merits node.

Dependent package checks and inline test counts were captured on origin/main and after the edit:

| Package | Check diagnostics BEFORE → AFTER | Tests BEFORE → AFTER |
|---|---:|---:|
| `ca1` | 0 → 0 | 9 → 9 |
| `ca2` | 0 → 0 | 36 → 36 |
| `ca3` | 0 → 0 | 3 → 3 |
| `ca4` | 0 → 0 | 37 → 37 |
| `ca5` | 0 → 0 | 14 → 14 |
| `ca6` | 0 → 0 | 20 → 20 |
| `ca7` | 0 → 0 | 50 → 50 |
| `ca8` | 0 → 0 | 3 → 3 |
| `ca9` | 0 → 0 | 26 → 26 |
| `ca10` | 0 → 0 | 28 → 28 |
| `ca11` | 0 → 0 | 21 → 21 |
| `cadc` | 0 → 0 | 2 → 2 |
| `links` | 7 → 7 | 3 → 3 |

The package solve response after the edit reports `mixed-node` 7 and `probability-fill` 36. Those are engine fallbacks in existing unnumbered decision edges, not authored probabilities; the root value is consequently a modeled expected value, not a measured case result. There are no `payoff-not-authored` warnings after adding the direct-start fallback values described above. The graph also reports 42 additional product-graph nodes from the new state flags.

## Fixtures and references

No FRAP compiled-pack golden or snapshot exists under `crates/litigate-cli/tests/litgraph-final/packs/` or the snapshot directories, and no `tests/expect` values changed. The only added behavior test is `frap_blended_terminals_follow_the_panel_result_flags` in `crates/litigate-cli/tests/ops.rs`; it checks all three neutral IDs, all three affirmance-loss copies, all three reversal-win copies, each reversal payoff, reachability from the package start, and the final terminal after a continued terminal's `#end` twin.

`git grep` over `lit/`, `cases/`, `crates/`, `tests/`, and `docs/` finds the new copy IDs only in the FRAP source declarations and this test/report. The original IDs remain on the neutral copies, so existing references continue to resolve.

## Open for David

- Payoffs set under R3: for each of the three split terminals, after affirmance `$0` (copied from `affirmed`); after outright reversal `$1,000,000` (copied from `reversed`); after reversal-and-remand `$600,000` (copied from `reversed-and-remanded`). The reversal copies use one `payoff_by_flag` map for the two latter figures. Their `$0` direct-start fallback is the standard neutral/no-flag fallback and is not used by a directional path. The neutral copies keep the pre-existing `$400,000` placeholder. No other payoff or matter number changed.
- Actor decisions under R5: panel outcomes, rehearing rulings, certiorari rulings, and mandate issuance remain `by court`/nature where authored; filing choices and lapse edges continue to inherit `movant = self`. The package already declares `appellant = self` and `appellee = opponent`; neither role nor an actor was changed. The split copies preserve the incoming edge actors.
- The merits module says, “Both parties may have grounds for rehearing: the party who lost on the reversed portion may challenge the legal rule; the party who won only on the affirmed portion may seek full reversal.” The package's existing `movant = self` convention does not distinguish which of the two parties files a petition. This split preserves that existing actor model rather than changing roles or adding a second petition path.
- A granted rehearing's replacement direction is unspecified in the pack; the task says, “the amended opinion's direction is not modeled.” The grants clear all panel-result flags, so downstream cert paths use the neutral copy unless a later modeled merits result is added in a future change.
