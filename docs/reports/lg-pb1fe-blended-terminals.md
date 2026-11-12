# Blended terminal inventory and proposed splits

> The FRCP rows in the inventory show the known-direction copies; the final section records the neutral third copy and unknown-direction handling.

## Operational definition

A **blended terminal** is a `terminal` whose incoming edges, as shown by rule text or node/edge notes, admit both a result favourable to the package's `self` party and a result adverse to that party, while retaining one terminal outcome/payoff. I treated explicit notes using “blended,” “movant,” “losing party,” or “either party,” and converging edges from different actors or branches of a chance node as evidence. Counts below are source-level candidates; a common terminal reached from a prior result is included when the note or graph confirms that the prior result is not carried forward.

`self` here means the package's declared protagonist, not necessarily the named appellant or appellee in a particular matter. Proposed splits are design recommendations only for packages outside FRCP; those packages are not modified in this bead.

## Inventory

| Package | File:line | Terminal id (current outcome) | Incoming favourable / adverse edges | Proposed split and separating rule |
|---|---|---|---|---|
| frcp | `02-pleading-to-trial.lit:898` | `summary-judgment-for-plaintiff` (`win`) / `summary-judgment-for-defendant` (`loss`) | `sj-ruling`: two mutually exclusive Rule 56(a) “Granted in full” edges, selected by the movant flag set at discovery close. | Split by movant under FRCP 56(a): plaintiff or defendant is entitled to judgment as a matter of law. |
| frcp | `04-judgment-appeal.lit:108` | `appeal-window-blown-plaintiff-judgment` (`procedural-default, waiver-trap, win`) / `appeal-window-blown-defendant-judgment` (`procedural-default, waiver-trap, loss`) | The Rule 4(a) missed-deadline edge pair is selected by `judgment-for-defendant`; the losing party is defendant for a plaintiff judgment and plaintiff for a defendant judgment. | Split by carried judgment direction under FRAP 4(a)(1)(A) and 28 U.S.C. § 1291. |
| frcp | `04-judgment-appeal.lit:125` | `judgment-final-no-appeal-plaintiff` (`win`) / `judgment-final-no-appeal-defendant` (`loss`) | The no-appeal edge pair is gated by `judgment-for-defendant`; the losing party chooses not to appeal. | Split by carried judgment direction under 28 U.S.C. § 1291 and FRAP 4(a)(1)(A). |
| frcp | `04-judgment-appeal.lit:274` | `judgment-affirmed-plaintiff` (`affirmed, win`) / `judgment-affirmed-defendant` (`affirmed, loss`) | Both affirmance chance branches (0.55 and 0.10 net affirmance) are split by the carried judgment flag. | Split by carried judgment direction under 28 U.S.C. §§ 1291, 2106. |
| frcp | `04-judgment-appeal.lit:299` | `judgment-reversed-remanded` (`remand`) | The 0.35 reversal/remand edge can reverse either party’s judgment; it retains one remand outcome as directed. | Keep one `[remand]` terminal under 28 U.S.C. § 2106; the chart does not model the post-mandate judgment. |
| frap | `11-cert.lit:117` | `cert-not-sought` (`final`) | `certiorari-petition-window`: no petition leaves the panel disposition standing, whether affirmance or reversal/remand. | `cert-not-sought-after-affirmance` / `cert-not-sought-after-reversal` (and mixed if represented); carry the panel disposition under 28 U.S.C. § 1254(1), Sup. Ct. R. 13. |
| frap | `11-cert.lit:142` | `cert-denied` (`final`) | `certiorari-decision`: denial leaves either party's panel result in force; cert denial is not a merits decision. | `cert-denied-after-affirmance` / `cert-denied-after-reversal` (and mixed); carry the panel disposition under Sup. Ct. R. 10. |
| frap | `09-rehearing.lit:148` | `rehearing-en-banc-both-denied` (`final`) | Rehearing and en banc requests can follow different panel dispositions; “both denied” preserves each. | Disposition-specific denial terminals, keyed to affirmed/reversed/remanded/mixed panel result; FRAP 40(a)(1), 41(b). |
| ptab | `03-trial-to-appeal.lit:457` | `appeal-forfeited-unappealed` (`final, appeal-forfeited`) | FWD outcomes all feed the post-FWD appeal choice; unappealed FWD may favour petitioner or patent owner. | `appeal-forfeited-after-unpatentable`, `...-confirmed`, and `...-mixed`; preserve the operative FWD under 35 U.S.C. § 319 and 37 C.F.R. § 90.3(a). |
| ptab | `03-trial-to-appeal.lit:486` | `cafc-affirms` (`affirmed, final`) | CAFC affirmance can affirm an FWD finding claims unpatentable or patentable; the note explicitly says the preceding result varies. | `cafc-affirms-unpatentable`, `...-confirmed`, and `...-mixed`; carry the FWD result under 35 U.S.C. § 319. |
| itc | `08-commission-review-to-federal-circuit-appeal.lit:215` | `determination-final-unappealed` (`final`) | No appeal can leave either a no-violation result or an exclusion/remedy determination in force. | `no-violation-final-unappealed` / `violation-final-unappealed`; preserve the Commission determination under 19 U.S.C. § 1337(c). |
| itc | `08-commission-review-to-federal-circuit-appeal.lit:240` | `cafc-affirmed` (`affirmed, final`) | An affirmance may sustain a no-violation or violation/remedy determination. | `cafc-affirms-no-violation` / `cafc-affirms-violation`; preserve the Commission result under 19 U.S.C. § 1337(c). |
| cafc | `11-cert.lit:26` | `cert-not-sought` (`final`) | Panel disposition flags can represent affirmance, reversal, or mixed result; the note says the disposition stands. | Disposition-specific cert-not-sought terminals; preserve the panel result under 28 U.S.C. § 1254(1), Sup. Ct. R. 13. |
| cafc | `11-cert.lit:59` | `cert-denied` (`final`) | Cert denial leaves the panel's affirmed, reversed, or mixed result intact; payoff is already keyed by panel flags, but terminal outcome is not. | Disposition-specific cert-denied terminals; preserve the panel result under Sup. Ct. R. 10. |
| cafc | `09-decision-to-rehearing.lit:59` | `affirmed` (`loss`) | `panel-to-affirmed` and `panel-to-rule36` affirm; for the appellant self loses, but the appellee self wins. The declared `movant` perspective makes the former interpretation explicit only when movant is self. | Split by represented party perspective or retain the `movant` role in outcome resolution; FRAP 36(a). Review with the package's roles before assigning fixed `win`/`loss` outcomes. |
| cafc | `09-decision-to-rehearing.lit:48` | `rule36-affirmance` (`loss`) | Same affirmance for appellant/appellee sides; Rule 36 does not encode which party is `self`. | Split or encode party-relative result with the `movant` perspective; Fed. Cir. R. 36(a)-(b). |
| mpep | — | No candidate identified in reviewed terminal notes/edges. | No explicit blend note or opposing party outcomes converging on one terminal identified in the inventory search. | No split proposed. |

## Other package families (count only)

The repository contains hundreds of additional package families, many of them unrelated state, bankruptcy, criminal, and specialty-court packages. No complete incoming-edge audit of those packages was performed for this focused follow-up; the explicit count of other-family candidates identified by this scan is **0**. This is a scan count, not a certification that every other package is blend-free. File a package-family follow-up when a concrete candidate is confirmed.

## FRCP split and three-state judgment direction

DECIDED: Use two mutually exclusive flags, judgment-for-plaintiff and judgment-for-defendant. Neither set means the judgment direction is unknown. Merits edges set one flag and clear the other. Rule 60 reopening clears both because it vacates the dismissal without deciding the claim. A generic post-trial grant clears both because its edge does not identify the new judgment's winner. The neutral copy is gated with forbids [judgment-for-plaintiff, judgment-for-defendant], so a third flag is unnecessary. The flags cannot both be set on a replay because every merits setter clears its opposite; the chance-node verifier's unreachable both-flags context follows the defendant copy to keep the authored distribution complete.

| Edge or transition | State action | Rule |
|---|---|---|
| edge-sj-ruling-summary-judgment-for-plaintiff / edge-sj-ruling-summary-judgment-for-defendant | Set the winner and clear the opposing flag; sj-by-defendant gates the defendant-movant outcome. Both grants retain p=0.25. | FRCP 56(a) |
| verdict-for-plaintiff / verdict-for-defendant | Set the winner and clear the opposing flag; p=0.5 each. | FRCP 52(a), 58 |
| edge-default-judgment-enters / edge-discovery-sanctions-default | Set plaintiff and clear defendant. | FRCP 55(b); 37(b)(2)(A)(vi) |
| Dismissal with prejudice → appeal clock | Set defendant and clear plaintiff. | FRCP 12(b)(6), 41(b); 28 U.S.C. § 1291; FRAP 4(a)(1)(A) |
| edge-rule60-relief-granted-reopened-case | Clear both; the reopened action has no asserted merits winner. | FRCP 60(b) |
| Post-trial motion denied → appeal clock | Leave flags unchanged. | FRAP 4(a)(4) |
| Post-trial motion granted → final-judgment-entered | Clear both; the generic amended/new judgment edge does not specify its winner. | FRCP 50(b), 59(a), 59(e) |
| No appeal / missed notice deadline, both FRAP 4 versions | Triple copies: plaintiff requires plaintiff flag; defendant requires defendant flag; neutral forbids both and has no actor. | FRAP 4(a)(1)(A); Bowles v. Russell; 28 U.S.C. § 1291 |
| Cross-appeal / no-cross-appeal, both FRAP 4 versions | Same triple; neutral copy has no actor. | FRAP 4(a)(3) |
| Court affirms / mixed net affirmance | Same-probability triple for known plaintiff, known defendant, and unknown; probabilities 0.55 and 0.10. | 28 U.S.C. §§ 1291, 2106 |
| Reversal/remand | Keep one remand terminal; post-mandate judgment direction is outside this chart. | 28 U.S.C. § 2106 |

The neutral copies restore the original appeal-window-blown, judgment-final-no-appeal, and judgment-affirmed terminal IDs with outcome [final]. cases/frcp-civil-case.case assigns final $0, win the existing $500,000, and loss $0. The remand terminal retains the case's existing $250,000 override because the chart does not model the post-mandate judgment.

The terminal solver returns terminal value before considering outgoing edges. Therefore, the Rule 56 terminals retain their appeal-clock edges for the procedural graph, while solve stops at the terminal outcome.

## Court-ruling edge audit

DECIDED: rule60-dismissal-motion is by court; grants/denials are unprobabilized nature outcomes, so probability-fill applies. No probability was added. Other Court-labelled FRCP edges found by source search: remand and special-appearance waiver rulings (chance nodes), Rule 41(d) prior-cost orders, dismissal-fee prevailing-party findings, Rule 16(b) scheduling order, Rule 16(e) pretrial order, and Rule 55 default judgment. The Rule 55 judgment edge is explicitly by court. The remaining listed rulings are outside the selected solve trace.

## Solve and explain

Release solve on origin/main at 5f745fe8: value -$16,655; policy_size 103; elapsed_ms 11231.428 (11.231 seconds); best line ended in the 90-day service default. Corrected source: value $75,713.461431; policy_size 373; elapsed_ms 11075.427 (11.075 seconds). The current best line requests waiver, accepts the opponent's Rule 12(b) motion, proceeds to trial, and reaches a defendant verdict. Its verdict edge ID is verdict-for-defendant, which sets judgment-for-defendant and clears judgment-for-plaintiff. The best line ends in a loss, not a win.

The selected-line root value is $75,713.461431. The complaint node is $85,463.461431; pre-filing investigation is 15 hours × $650 = $9,750. The complaint filing value is $92,368.461431; the filing/effort cost is $6,905. Issuing summons adds no cost. The chosen service path requests a waiver and the opponent chooses a Rule 12(b) motion. The motion-denial court branch has p=0.45. The selected trial route takes the p=0.5 defendant verdict. Rule 60 can no longer be chosen as if the plaintiff controlled the court; a reopening without a new merits decision has neither direction flag and receives the neutral $0 final stake.

Every reachable win terminal requires a merits-setting edge: default-judgment-entered uses edge-default-judgment-enters; discovery-sanctions-default uses edge-discovery-sanctions-default; summary-judgment-for-plaintiff uses edge-sj-ruling-summary-judgment-for-plaintiff; a trial win uses verdict-for-plaintiff. Appeal-window-blown-plaintiff-judgment, judgment-final-no-appeal-plaintiff, and judgment-affirmed-plaintiff preserve a prior setter. A Rule 60 reopening clears both flags, so it cannot reach a win terminal without a new merits edge.

Solve warnings: compile 9 (675 additional reachable flag-state nodes), mixed-node 18 (act-or-wait at mixed nodes), probability-fill 99 (default-filled probabilities, including the Rule 60 ruling). The result is sensitive to these engine assumptions and is not a sourced expected litigation value.

The updated CLI test pins $75,713.461431 and, if the best-line terminal is a win, asserts a preceding merits-setting edge ID. The final FRCP package check reports 0 errors; its inline suite has 49 passed and 0 failed.
