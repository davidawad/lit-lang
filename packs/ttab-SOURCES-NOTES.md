# TTAB pack -- sources and authoring notes

`ttab-trademark-trial-and-appeal-board.json`, schemaVersion 2, forum `ttab`,
startNodeId `published`. 60 nodes / 111 edges / 13 terminals / 5 sources.

## What was read

- 37 CFR Part 2 (eCFR as of 2026-10-19): 2.6 (fees: notice of opposition and
  petition to cancel $600 per class electronic; statement of use $150 per
  class), 2.68, 2.88, 2.101, 2.102(c), 2.105, 2.106, 2.111, 2.116, 2.117,
  2.120, 2.121, 2.127(f) (no fees or contempt), 2.128, 2.129, 2.133, 2.135,
  2.136, 2.145(d)-(e), 2.196. Vendored as `sources/cfr-37-part2.txt`.
- Lanham Act (USPTO compilation 2026-07-15): 15 USC 1051(d), 1052(d), 1063,
  1064, 1068, 1071, 1114, 1117(a). Vendored as `sources/usc-15-lanham.txt`.
- TBMP June 2026 (not vendored; 6.9 MB): 207.03, 310.03(a) (answer 40 days,
  practice changing to 60), 313.02, 602.01, 605.02, 702.04, 902.02.

## Conventions a scenario must know

- Terminals carry attrs: `rb` (fraction of the rebrand cost incurred), `fx`
  (other applicant USD), `sh` (1 if the opposer pays a cost share), `sw`
  (opposer win fraction), `sfx` (opposer other USD). The authored `payoff` is
  computed at placeholder inputs (rebrand 100,000; cost share 30,000). A
  scenario recomputes value with a `utility` such as
  `-rebrand_cost*default(node.rb,0)+default(node.fx,0)+cost_share*default(node.sh,0)`
  and sets `opponent_objective` from `sw`/`sfx` (see
  `scenarios/ttab-opposition-applicant-defense.json`).
- Edge attrs `opp_hours` / `opp_fees` are the opposer's own cost; the scenario
  shadows the `opponent_dollars` metric to read them.
- State flags: `s18` (Section 18 counterclaim pleaded), `talked` (the one formal
  suspended-settlement round used), `sfail` (standstill talks failed). The
  scenario's `probability_fn` keys merits odds off the tags `merits-opposer`,
  `merits-applicant`, `restrict`, `cancel` and the deal odds off `deal`,
  `no-deal`, `inproc`.
- Every authored probability and hour is a teaching estimate, not a docket
  statistic. `tornado`, `sweep`, `voi` and `posterior` ignore
  `opponent_objective` (worst-case opposer); use explicit solves to vary the
  opposer's stake.

## Known gaps

- No summary-judgment branch (ACR is the proxy); one settlement round per case.
- The 15 USC 1071(b) civil action is abstract (not linked to the FRCP pack:
  de novo review on the Board record plus new evidence is not a generic civil
  case). The Federal Circuit route is linked through `cafc@ttab`.
- Ex parte appeals, cancellation proceedings as a first-class path, concurrent
  use and Section 66(a) oppositions are not modeled (a post-registration
  cancellation appears only as a risk branch).
- `sources/` has no TBMP text, so TBMP cites are not mechanically verified.
