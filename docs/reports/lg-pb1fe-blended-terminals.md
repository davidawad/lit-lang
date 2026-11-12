# Blended terminal inventory and proposed splits

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

## FRCP split

**DECIDED: Carry judgment direction in state flags and split terminals by the Rule 56 movant or the carried judgment direction.** This keeps one copy of the post-judgment and appeal subgraphs, uses no new probability, and lets the matter's existing `win`/`loss` stakes value the result. Outcome tags `final` were removed from the split terminals because the matter assigns `final` $0 while `win`/`loss` assign $500,000/$0; a terminal carrying conflicting outcome stakes requires an explicit per-terminal override.

### Engine questions and fixture

1. **Different edge actors:** actor roles are resolved per edge from the edge actor or its node default (`crates/litigate-engine/src/scenario/view.rs:203-209`; SPEC §5.1 at `docs/litigate/SPEC.md:1183-1188`). At a node with self and opponent moves, self chooses its edges and opponent/nature edges are interrupts (`crates/litigate-engine/src/scenario/plan.rs:113-153`). The default mixed handling uses act-or-wait when the interrupt has no authored probability (`plan.rs:264-331`). Therefore the two discovery-close Rule 56 edges can be assigned to plaintiff and defendant; this adds `mixed-node` warnings at nodes where the choices meet.
2. **`requires` / `forbids`:** they gate edge instantiations while flags expand the graph, before solve; the edge is absent when the flag condition fails (`crates/litigate-engine/src/model/flags.rs:41-49`; `docs/PACK_SCHEMA.md:391-394`). A small fixture reviewed before the FRCP edit is the `ESTOPPEL_PACK` at `model/flags.rs:208-226`, with reachable flagged copies and a `forbids` edge exercised at `flags.rs:266-296`.
3. **Outcome stakes:** each outcome stake is applied to every terminal declaring that tag (`crates/litigate-sema/src/case/resolve/stakes.rs:118-148,173-177`). If one terminal declares tags with different stakes, check reports L4004 unless that terminal gets an explicit override. The win/loss split therefore omits `final` from those outcomes and retains procedural tags such as `affirmed`, `procedural-default`, and `waiver-trap` where they do not name a conflicting matter stake.
4. **Terminal with an outgoing edge:** solve returns the terminal's utility before considering edges (`crates/litigate-engine/src/algo/mdp.rs:131-136`). The two summary-judgment terminals retain their appeal-clock edges as requested, but solve treats each as terminal, so it does not traverse those edges. This is an existing solver contract, not a duplicated appeal graph.

### Flags and per-edge disposition

`judgment-for-defendant` is absent when the judgment in force favours plaintiff. `sj-by-defendant` is set only when defendant is the Rule 56 movant. Gates are exclusive pairs on the same flag, so each chance context keeps the original total probability. An edge that does not establish who won leaves the judgment flag unchanged.

| Edge or edge family | Flag action | Rule text relied on |
|---|---|---|
| `discovery-closed` → plaintiff's Rule 56 motion | Does not set either flag; actor `plaintiff`. | FRCP 56(a), (b): a party may move. |
| `discovery-closed` → defendant's Rule 56 motion | Sets `sj-by-defendant`; actor `defendant`. | FRCP 56(a), (b): a party may move. |
| `sj-ruling` → plaintiff judgment | Forbids `sj-by-defendant`; clears `judgment-for-defendant`. | FRCP 56(a): judgment is for the successful movant. |
| `sj-ruling` → defendant judgment | Requires `sj-by-defendant`; sets `judgment-for-defendant`. | FRCP 56(a): judgment is for the successful movant. Both edges retain p=0.25. |
| `verdict-rendered` → plaintiff findings | Clears `judgment-for-defendant`. | FRCP 52(a), 58: the findings/judgment favour plaintiff. p=0.5. |
| `verdict-rendered` → defendant findings | Sets `judgment-for-defendant`. | FRCP 52(a), 58: the findings/judgment favour defendant. p=0.5. |
| `dismissed-with-prejudice` → appeal clock | Sets `judgment-for-defendant`. | Dismissal is adverse to plaintiff; 28 U.S.C. § 1291; FRAP 4(a)(1)(A). |
| `rule60-relief-granted-reopened-case` → `final-judgment-entered` | Leaves the flag unchanged. | FRCP 60(b) says the terms and further proceedings depend on the order; this edge does not specify the merits result. |
| `post-trial-ruling` denied → appeal clock | Leaves the flag unchanged. | FRAP 4(a)(4): denial ends tolling and restarts the clock; it does not say the underlying judgment changed. |
| `post-trial-ruling` granted → `final-judgment-entered` | Leaves the flag unchanged. | The source says amended or new judgment but does not say for which party. |
| `final-judgment-entered` / `post-trial-motion-window` → appeal clock; Rule 60 denial → appeal clock | Leaves the flag unchanged. | These transitions open an appeal period or appeal the order; they do not specify a merits winner. |
| Summary judgment terminal → appeal clock | Carries the flag unchanged. | 28 U.S.C. § 1291; FRAP 4(a)(1)(A). Kept on both split terminal declarations; see terminal solver behavior above. |
| Both `notice-of-appeal-window` versions: missed deadline / no appeal | Plaintiff-judgment gates are by defendant and forbid `judgment-for-defendant`; defendant-judgment gates are by plaintiff and require it. They do not change the flag. | FRAP 4(a)(1)(A); Bowles v. Russell, 551 U.S. 205 (2007); 28 U.S.C. § 1291. The losing party owns the appeal right. |
| Both `cross-appeal-window` versions: file / no cross-appeal | Plaintiff-winner gates are by plaintiff and forbid the flag; defendant-winner gates are by defendant and require it. They do not change the flag. | FRAP 4(a)(3): the winning appellee may cross-appeal a different part of the same judgment. |
| `court-of-appeals-decision` → affirmed | Each 0.55 and 0.10 branch has a plaintiff-judgment / defendant-judgment gated pair; neither changes the flag. | 28 U.S.C. §§ 1291, 2106. Affirmance leaves the judgment in force; the net-affirmance branch preserves that side. |
| `court-of-appeals-decision` → reversed/remanded | Leaves the flag unchanged and reaches the single `[remand]` terminal. | 28 U.S.C. § 2106; the chart does not model the judgment after remand. |

### Solve and explain

Using the release binary, `litigate solve cases/frcp-civil-case.case` changed from **−$16,655** to **$472,895**. Plaintiff now serves. Before the split, the best line stopped at the service deadline because the eventual shared `final` outcomes were valued at $0. After the split, the selected path reaches `judgment-final-no-appeal-plaintiff`, whose `[win]` outcome takes the existing $500,000 matter stake.

The selected line and its dollar contributions are:

1. Pre-filing investigation: `$482,645` downstream less 15 hours × `$650` = `$472,895`.
2. File complaint: `$489,550` downstream less `$6,905` = `$482,645`.
3. Clerk issues summons: no added cost; service-window value remains `$489,550`.
4. Serve defendant: service action cost `$2,100`; resulting value is `$489,550` from `$491,650` at service-completed.
5. Voluntary dismissal: 1 hour × `$650`; resulting value at that node is `$492,300` from `$491,650`.
6. Move for Rule 60(b) relief: 8 hours × `$650` + `$2,500` = `$7,700`; resulting value is `$500,000` at the motion node.
7. The selected Rule 60 grant returns to `final-judgment-entered`; no post-trial motion goes to the appeal window; its 0.5 no-appeal branch reaches the plaintiff-judgment `[win]` terminal at `$500,000`. The other 0.5 missed-window branch also retains the plaintiff-favourable judgment and its `[win]` stake.

`litigate explain` at `service-window` chooses “Serve defendant via formal process” (`q=$489,550`; waiver `$488,230`; fail to serve `$0`). At `voluntary-dismissal`, it chooses the Rule 60(b) motion (`q=$492,300`) over leaving the dismissal (`$0`), fees (`−$5,400`), or refiling (`−$10,300`). At `final-judgment-entered`, it chooses to proceed directly to appeal. This trace exposes a separate pre-existing modeling issue: `rule60-dismissal-motion` is a plaintiff decision and its “Court grants” edge is therefore treated as plaintiff-controlled; the note says the court’s discretionary ruling has no assigned probability. No probability was added in this bead.

The solve reports 8 state-flag compilation warnings, **18 `mixed-node` warnings** (6 before), and 73 `probability-fill` warnings. These warnings mean the result uses act-or-wait behavior at mixed nodes and default-filled probabilities at some nodes; the number is not a sourced expected result. The check measured 0 errors, 0 L2003 findings, and 49 passed / 0 failed FRCP inline tests.

The CLI test `crates/litigate-cli/tests/ops.rs::frcp_civil_case_solve_uses_party_relative_judgment_stakes` pins the matter at **−$16,655 before → $472,895 after**, and asserts that the best line serves the defendant. Existing CLI tests that named the prior terminal were changed to the new party-labelled ID in `frcp_extensions.rs`, `frcp_fee_moves.rs`, `pending_moves.rs`, and `rulings.rs`; they do not pin the solve value.

`NEW_BUG: Rule 60 dismissal ruling is plaintiff-controlled -- `rule60-dismissal-motion` is a plaintiff decision whose unprobabilized “Court grants/denies” edges let solve choose the court ruling; this path supplies the $500,000 win in the new trace.`
