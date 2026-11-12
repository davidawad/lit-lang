# Audit: packages with multiple roles defaulting to `self`

This inventory was generated from every `lit/*/Lit.toml` `[package.roles]` table on 2026-11-11. It lists all 129 packages with at least two roles mapped to `self`. The classification is based on the role names and the package's rule text: criminal procedure pairs `accused` with `prosecution` as opposing parties; PTAB's 35 U.S.C. § 311(a) expressly defines a petitioner as a person other than the patent owner. At TTAB, `respondent` and `appellant` in this package both identify the applicant's own procedural acts: 01-pre.lit:79 makes the applicant's consent controlling, 07-opposition-to-trial.lit:40 makes the applicant answer, and 12-decision-3.lit:7 says the applicant must move to amend; 08-decision.lit:31 is the applicant's choice to appeal after losing. The adverse TTAB party is `opposer`. In appellate packages, `appellant` and `petitioner` are usually the same party-facing side in alternative review tracks (appeal vs extraordinary review), rather than opposing litigants. `movant` is a procedural actor that can be either side and cannot be equated to the plaintiff/appellant without a matter's facts.

`litigate` has a matter-level `perspective` mechanism: per docs/litigate/SPEC.md §5.6 and §8.3, `perspective party` makes that party's actor `self` and other declared parties `opponent`, overriding only the primary package's role defaults. `parties` declares actor designations and entity types. FRCP uses `perspective plaintiff` and declares both plaintiff and defendant parties (cases/frcp-civil-case.case); L1016 rejects invalid/untyped actor designations. Thus packages can leave party defaults neutral and let each matter declare which side is `self`; no new mechanism is needed.

| Package | Roles mapped to `self` | Rule-text reading |
|---|---|---|
| `acca` | appellant, petitioner |
| `afcca` | petitioner, appellant |
| `ak-court-of-appeals` | appellant, petitioner |
| `ak-criminal-trial-court` | prosecution, accused |
| `ak-supreme` | appellant, petitioner |
| `al-court-of-criminal-appeals` | prosecution, accused |
| `al-criminal-trial-court` | prosecution, accused |
| `al-state-appellate` | appellant, petitioner |
| `ar-criminal-trial-court` | accused, prosecution |
| `ar-workers-compensation-commission` | movant, the |
| `as-criminal-trial-court` | accused, prosecution |
| `as-high-court-appellate` | appellant, petitioner |
| `az-court-of-appeals` | appellant, petitioner |
| `az-criminal-trial-court` | accused, prosecution |
| `bap1` | appellant, petitioner |
| `bap10` | appellant, petitioner |
| `bap6` | appellant, petitioner |
| `bap9` | appellant, petitioner |
| `bia` | appellant, petitioner |
| `ca-court-of-appeal-fifth-district` | appellant, petitioner |
| `ca-court-of-appeal-first-district` | appellant, petitioner |
| `ca-court-of-appeal-fourth-district` | appellant, petitioner |
| `ca-court-of-appeal-second-district` | appellant, petitioner |
| `ca-court-of-appeal-sixth-district` | appellant, petitioner |
| `ca-court-of-appeal-third-district` | appellant, petitioner |
| `ca-court-of-appeal` | appellant, petitioner |
| `ca-criminal-trial-court` | accused, prosecution |
| `caaf` | petitioner, appellant |
| `cafc` | movant, appellant |
| `cmcr` | petitioner, appellant |
| `co-criminal-trial-court` | prosecution, accused |
| `co-industrial-claim-appeals-office` | petitioner, appellant |
| `co-state-appellate` | appellant, petitioner |
| `cofc` | movant, plaintiff |
| `ct-criminal-trial-court` | prosecution, accused |
| `dc-criminal-trial-court` | prosecution, accused |
| `de-criminal-trial-court` | prosecution, accused |
| `de-supreme-civil` | petitioner, appellant |
| `fl-criminal-trial-court` | accused, prosecution |
| `fl-district-court-of-appeal-fifth` | appellant, petitioner |
| `fl-district-court-of-appeal-first` | appellant, petitioner |
| `fl-district-court-of-appeal-fourth` | appellant, petitioner |
| `fl-district-court-of-appeal-second` | appellant, petitioner |
| `fl-district-court-of-appeal-sixth` | appellant, petitioner |
| `fl-district-court-of-appeal-third` | appellant, petitioner |
| `fl-district-court-of-appeal` | appellant, petitioner |
| `frap` | movant, appellant |
| `frcrimp` | movant, accused, prosecution |
| `gu-criminal-trial-court` | prosecution, accused |
| `gu-supreme` | appellant, petitioner |
| `hi-criminal-trial-court` | prosecution, accused |
| `ia-criminal-trial-court` | accused, prosecution |
| `id-criminal-trial-court` | accused, prosecution |
| `id-supreme` | appellant, petitioner |
| `il-criminal-trial-court` | prosecution, accused |
| `in-criminal-trial-court` | accused, prosecution |
| `ks-court-of-appeals` | appellant, petitioner |
| `ks-criminal-trial-court` | accused, prosecution |
| `ky-court-of-appeals` | appellant, petitioner |
| `ky-criminal-trial-court` | accused, prosecution |
| `la-criminal-trial-court` | accused, prosecution |
| `ma-appeals-court` | appellant, petitioner |
| `ma-criminal-trial-court` | prosecution, accused |
| `md-appellate-civil` | appellant, petitioner |
| `md-criminal-trial-court` | prosecution, accused |
| `me-criminal-trial-court` | accused, prosecution |
| `mi-appellate` | appellant, petitioner |
| `mi-criminal-trial-court` | prosecution, accused |
| `mn-criminal-trial-court` | accused, prosecution |
| `mo-criminal-trial-court` | prosecution, accused |
| `mp-criminal-trial-court` | prosecution, accused |
| `mp-supreme` | appellant, petitioner |
| `ms-appellate` | appellant, petitioner |
| `ms-criminal-trial-court` | prosecution, accused |
| `mt-criminal-trial-court` | accused, prosecution |
| `mt-supreme` | appellant, petitioner |
| `mt-tax-appeal-board` | appellant, petitioner |
| `nc-criminal-trial-court` | prosecution, accused |
| `nd-criminal-trial-court` | accused, prosecution |
| `ne-criminal-trial-court` | prosecution, accused |
| `nh-criminal-trial-court` | prosecution, accused |
| `nh-supreme` | appellant, petitioner |
| `nj-criminal-trial-court` | accused, prosecution |
| `nm-criminal-trial-court` | prosecution, accused |
| `nm-state-appellate` | appellant, petitioner |
| `nv-criminal-trial-court` | accused, prosecution |
| `nv-supreme` | appellant, petitioner |
| `ny-appellate-terms` | appellant, petitioner |
| `ny-commission-of-appeals` | appellant, petitioner |
| `ny-criminal-trial-court` | accused, prosecution |
| `ny-supreme-civil` | petitioner, appellant |
| `oh-criminal-trial-court` | accused, prosecution |
| `ok-court-of-civil-appeals` | appellant, petitioner |
| `ok-court-of-criminal-appeals` | accused, prosecution |
| `ok-criminal-trial-court` | prosecution, accused |
| `or-criminal-trial-court` | accused, prosecution |
| `pa-criminal-trial-court` | prosecution, accused |
| `pr-criminal-trial-court` | prosecution, accused |
| `ptab` | patent-owner, petitioner |
| `ri-criminal-trial-court` | prosecution, accused |
| `ri-supreme` | appellant, petitioner |
| `sc-court-of-appeals` | appellant, petitioner |
| `sc-criminal-trial-court` | prosecution, accused |
| `sd-criminal-trial-court` | prosecution, accused |
| `tn-appellate` | appellant, petitioner |
| `tn-court-of-criminal-appeals` | accused, prosecution |
| `tn-criminal-trial-court` | prosecution, accused |
| `ttab` | appellant, respondent |
| `tx-court-of-civil-appeals` | appellant, petitioner |
| `tx-court-of-criminal-appeals` | prosecution, accused |
| `tx-courts-of-appeals` | appellant, petitioner |
| `tx-criminal-trial-court` | prosecution, accused |
| `ut-criminal-trial-court` | prosecution, accused |
| `ut-state-appellate` | appellant, petitioner |
| `va-court-of-appeals` | appellant, petitioner |
| `va-criminal-trial-court` | accused, prosecution |
| `vi-criminal-trial-court` | accused, prosecution |
| `vi-supreme-civil` | appellant, petitioner |
| `vt-criminal-trial-court` | accused, prosecution |
| `vt-supreme` | appellant, petitioner |
| `wa-court-of-appeals` | appellant, petitioner |
| `wa-criminal-trial-court` | prosecution, accused |
| `wawd-criminal` | prosecution, accused |
| `wi-court-of-appeals` | appellant, petitioner |
| `wi-criminal-trial-court` | accused, prosecution |
| `wv-criminal-trial-court` | prosecution, accused |
| `wv-supreme-court` | appellant, petitioner |
| `wy-criminal-trial-court` | accused, prosecution |
| `wy-supreme` | appellant, petitioner |

## Adverse pairs with a matter-level perspective

- **PTAB**: `patent-owner` and `petitioner` are adverse under 35 U.S.C. § 311(a). `cases/ptab-ipr-defense.case` declares patent-owner perspective, so the package defaults can be `nature` and the matter resolves the two actors as `self` and `opponent`.
- **TTAB**: `appellant` and `respondent` are both the applicant's own actions, not adverse roles: 01-pre.lit:79, 07-opposition-to-trial.lit:40, and 12-decision-3.lit:7 identify respondent actions as the applicant's; 08-decision.lit:31 makes the appeal choice for the applicant who lost. `opposer` is the adverse party and is already `opponent`. `cases/ttab-opposition-applicant-defense.case` declares applicant perspective; keep the package mappings of `appellant` and `respondent` as `self`.
- **Criminal packages**: `accused` and `prosecution` are adverse in the governing criminal rules, but no `cases/` matter with a declared perspective loads any of these packages, so no package-level role correction is made here.
- **Other packages**: packages with `appellant`/`petitioner` encode successive/alternative procedural capacities of the party on the prosecuting side; their rule text does not establish that the two actor labels are opposing parties. `cofc`'s `plaintiff` and generic `movant`, and `cafc`/`frap` `appellant` and `movant`, can refer to the same or different persons depending on a matter. No other audited package pair has both an adverse relationship established by its rule text and an existing perspective-bearing matter in `cases/` to safely correct in this change.

The TTAB matter uses the existing perspective mechanism, but both `appellant` and `respondent` refer to the applicant in this package. Their defaults remain `self`; `opposer` remains `opponent`. Criminal role defaults and context-dependent movant combinations remain for matters to resolve where such matters exist.

## Resolution in this bead

PTAB's default protagonist is the initiating petitioner (`self`), with the patent owner as `opponent`; its existing matter perspective assigns patent-owner to `self` and petitioner to `opponent` for this defense matter. This matches the FRCP convention in which the initiating plaintiff is the package default protagonist and a matter can override it. TTAB keeps `appellant` and `respondent` as `self` because the rule text identifies both as applicant actions; `opposer` is already `opponent`. Other audited criminal pairs have no perspective-bearing matter in `cases/`; no changes were made to them. The appellant/petitioner alternatives and context-dependent movant roles likewise remain unchanged.

For `cases/ptab-ipr-defense.case`, the release `solve` value was `$1,191,295.315355` both before and after. This unchanged result is explained by the matter's explicit `perspective { patent-owner: self, petitioner: opponent }`, which already overrides the package defaults. The post-change `explain` trace at `challenge-vehicle-decision{prior-petition-denied}` reports `control: opponent`; its three options are petitioner actions and the opponent selects the institution-granted path (`q = $980,380.473032`). At the matter's actual start, the time-bar fact resolves to the timely path and the following POPR alternatives are actor `either`, hence neutral nature draws. The correction therefore repairs package defaults for matters that do not specify a perspective without changing this already-perspectivized matter's value or policy. No inline PTAB test or snapshot expectation moved; all 35 PTAB tests pass.

The PTAB citation spellings in `.lit` cite directives are now uniform: 154 standard `37 C.F.R. §` fragments, 13 historically versioned `37 C.F.R. (2012) §` fragments, and 100 `35 U.S.C. §` fragments; there are zero legacy `37 CFR` fragments and zero bare-number `35 U.S.C.` fragments. The historical `(2012)` prefix is retained because `Lit.toml` declares its own dated source and section heading. Before normalization, the mixed counts recorded in the bead were 52 / 39 / 6 for the three ordinary CFR spellings and 43 / 12 for USC with/without §. `litigate check lit/ptab` had zero cite diagnostics before normalization and has zero cite diagnostics after; the post-change diagnostics are 105 L9011 and 24 L9002 warnings, none cite-resolution errors. `litigate lint lit/ptab` reports no unresolved/unverifiable cite warnings after normalization.

## PTAB package-level values

The PTAB default is petitioner `self`, patent-owner `opponent`, following FRCP's initiating-party default. The explicit `perspective { patent-owner: self, petitioner: opponent }` in `cases/ptab-ipr-defense.case` continues to resolve the defense matter to `$1,191,295.315355` before and after.

| Test / snapshot | Old value | New value | Explain trace for the change |
|---|---:|---:|---|
| `every_package_compiles_and_solves_to_litgraph_final`, PTAB pack | `$516,355.604420` | `$510,737.442007` | The old best line starts with the petitioner filing a PGR petition as `self`. `litigate explain lit/ptab --arg node=patent-granted` now reports `control: opponent`; the patent-owner chooses “wait (let the proceeding move)” (`q = $510,737.442007`) instead of filing PGR directly (`q = $514,737.442007`) or considering a challenge (`q = $513,737.442007`). |
| `ptab_calibrated_institution` | `$572,733.783111` | `$569,035.765633` | The recorded op explains the institution chance node, whose three Board outcomes remain `nature`; its continuation values change because the package-start action at `patent-granted` has changed hands. The explain trace there shows the opponent choosing to wait rather than letting the former protagonist file directly. |
| `ptab_robust_institution` | `$312,883.368395` | `$246,794.612481` | The old robust best line begins “Consider challenging without suit” as `self` and pursues joinder. With distinct roles, the new robust line begins “File PGR petition directly” as `opponent`. The explain trace at `patent-granted` identifies the opponent as the controller and gives the competing choices and values, showing that patent-owner choice has replaced the old self-controlled opening. |
| `ptab_ipr_estoppel_flag` | no scalar solve value | no scalar solve value | This request is a `graph` op, so it has no decision value. The recorded graph changes its existing citation to `37 C.F.R. § 42.71(d)` and adds the `infringement-complaint-served` location to the `mixed-node` warning (count 5 → 6); the graph's post-FWD options remain tribunal/nature edges. |

The changed expectations are limited to these PTAB outputs; TTAB remains pinned at `-$74,025.00`, and the TTAB applicant matter remains `-$31,820.00` (`opponent_value = -$3,032.50`, `policy_size = 78`). The solve traces carry the pack's existing fallback warnings, including `probability-fill` (101 package nodes) and, in the PTAB matter, `fact-unset` (2 nodes) and `mixed-node` (6 nodes); these estimates are not authored certainty.

The `litigate fmt` change to `lit/ptab/03-trial-to-appeal.lit` is layout-only (6 insertions, 2 deletions). I built the PTAB package before and after formatting; the graph JSON is byte-identical after removing the package source digest and its dependent world digest. Those two content-address fields change because the source text's digest includes formatting, so the literal output files are not byte-identical; all procedural pack content is unchanged.

## PTAB opening nodes (follow-up)

**DECIDED:** `either` is the implicit actor only when no node or edge actor is supplied; SPEC §5.1 says, “An edge's actor is its own `by`, else the node's `by`, else `either`.” The suit edge is now explicitly `by patent-owner`: PTAB maps `either` to `nature` (`lit/ptab/Lit.toml:29`), and the SPEC says the engine uses that actor role to decide choice versus draw (§13.7). A complaint filing is the patent owner’s act, not a neutral draw. Each petitioner move from `patent-granted` and `infringement-complaint-served` is explicitly `by petitioner`.

**DECIDED:** The node `by` is inherited by every edge without its own `by` (SPEC §5.1). State-node audit, with verdicts on whether inherited edges belong to the declared node actor:

- `lit/ptab/01-pre-petition-to-institution.lit:35` `patent-granted` — fixed: complaint edge is `patent-owner`; challenge evaluation and direct PGR filing are `petitioner`.
- `lit/ptab/01-pre-petition-to-institution.lit:62` `infringement-complaint-served` — fixed: challenge evaluation and direct IPR filing are `petitioner`.
- `lit/ptab/01-pre-petition-to-institution.lit:236` `popr-filed` — pass: Board decides institution.
- `lit/ptab/01-pre-petition-to-institution.lit:252` `popr-waived` — pass: Board decides institution.
- `lit/ptab/01-pre-petition-to-institution.lit:336` `institution-granted` — pass: Board issues scheduling order; settlement edge explicitly allows either party.
- `lit/ptab/02-post-institution.lit:72` `protective-order-ipr` — pass: Board-imposed protective-order transition proceeds into the response window.
- `lit/ptab/03-trial-to-appeal.lit:52` `patent-owner-response-filed` — doubtful: the “File Petitioner Reply” edge inherits `patent-owner`, though its label names the petitioner. Explicitly assigning it changes the defense matter value despite its opening posture; it is left untouched to preserve the bead’s required matter pin.
- `lit/ptab/03-trial-to-appeal.lit:124` `mta-filed` — pass: Board issues preliminary guidance; settlement edge explicitly allows either party.
- `lit/ptab/03-trial-to-appeal.lit:156` `mta-revised-filed` — pass: petitioner opposes the revised amendment.
- `lit/ptab/03-trial-to-appeal.lit:206` `adverse-judgment-request` — pass: Board enters the adverse judgment.
- `lit/ptab/03-trial-to-appeal.lit:243` `oral-hearing-held` — pass: Board issues the FWD.
- `lit/ptab/03-trial-to-appeal.lit:333` `rehearing-request-filed` — pass: Board panel considers rehearing.
- `lit/ptab/03-trial-to-appeal.lit:388` `director-review-request-filed` — pass: Director reviews the decision.
- `lit/ptab/03-trial-to-appeal.lit:469` `federal-circuit-notice-of-appeal` — doubtful: the node says `by board`, then hands to a Board-attributed Federal Circuit review chance node; this may be the intended tribunal/nature routing and is left unchanged.

The changed package choice is at `patent-granted`: the petitioner controls its own challenge choices, while the patent owner controls the decision to sue. The defense matter has an explicit perspective and replays from a later posture, so its solve remains unchanged. Recorded outputs and snapshot changes follow after running the release binary.

Recorded after the opening-node attribution change (baseline is the source/pin state after lg-pb1fe.33):

| Output | Old | New | Why it moved |
|---|---:|---:|---|
| PTAB package solve | `$510,737.442007` (policy 13) | `$514,737.442007` (policy 14) | At `patent-granted`, the petitioner now controls the two challenge moves and chooses “File PGR petition directly” (`q = $514,737.442007`). The opponent controls the suit edge; its best value there is `$510,737.442007`. The protagonist's direct PGR choice raises the package value by `$4,000`. |
| `cases/ptab-ipr-defense.case` solve | `$1,191,295.315355` (policy 211) | `$1,191,295.315355` (policy 210 observed) | The case still starts at the replayed §315(b) time-bar check and the value is unchanged. Policy size falls by one in the engine output despite the opening posture; no case expectation was re-recorded. |
| `ptab-calibrated-institution` | `$569,035.765633` | `$569,035.765633` | Scalar stays the same; its `mixed-node` warning drops from 6 to 5 because the infringement-complaint-served node no longer has petitioner moves inherited from the patent-owner actor. |
| `ptab-robust-institution` | `$246,794.612481` | `$296,756.444858` | Robust policy now begins with the self-controlled “Consider challenging without suit” option (`q = $296,756.444858`) and selects joinder downstream. The old policy began with opponent-controlled direct PGR. The robust objective therefore values the corrected petitioner-controlled strategy `$49,961.832377` higher. |
| `ptab-ipr-estoppel-flag` | no scalar solve value; `mixed-node` 6 | no scalar solve value; `mixed-node` 5 | This graph request has no decision value. It no longer reports infringement-complaint-served as a mixed actor node. |

The only changed recorded pins are the PTAB pack solve and the three PTAB requests above. Warnings remain material to interpretation: package solve reports `probability-fill` 101, `mixed-node` 5, and `fact-unset` 2; the defense matter reports `probability-fill` 425 and `mixed-node` 5. Its probabilities include fills, so these are model outputs with fallback estimates, not authored certainty.

Verification on the updated release binary: `litigate check lit cases` exited 0 with zero errors; `litigate fmt --check lit cases` exited 0; `litigate test lit/ptab` exited 0 with 35 passed and 0 failed. Strict PTAB check exited 2 with 9 errors; the attribution edits introduce no strict finding (they alter only edge actors), and the repository's strict baseline is already 9. The requested nextest selection ran 58 tests: 53 passed and only the five specified main-branch failures occurred (`ttab_check_human_output_is_limited_to_the_case_when_core_has_no_holes`; `the_director_extends_the_ttab_civil_action_period`; `the_director_not_the_court_extends_the_federal_circuit_ptab_window`; `ttab_opposition_matter`; `ttab_published_fixture_forks_with_unknown_deadlines`). PTAB roundtrip/pin tests passed. `lit-sync-check` must be rerun after committing the regenerated PTAB lock because its gate rejects any modified tracked lock in the working tree.

## PTAB reply and appeal nodes (follow-up 2)

**DECIDED:** 37 C.F.R. § 42.23(c)(1) calls the filing “Replies to patent owner responses to petitions”; § 42.23(b) limits the reply to arguments in the patent-owner response or institution decision. The `PetitionerReplyFiled()` transition after that response is the petitioner's filing, so `File Petitioner Reply` is explicitly `by petitioner` (`03-trial-to-appeal.lit:56`). The sur-reply is the patent owner's response to it and is explicitly `by patent-owner` (`:91`).

**DECIDED:** 37 C.F.R. § 42.107(a), (b), (e) assigns the preliminary response, waiver, and disclaimer options to the patent owner; `popr-window` is now `by patent-owner` (`01-pre-petition-to-institution.lit:211`). 37 C.F.R. § 42.120(a) says “A patent owner may file a single response”; the response, no-response, and settle choices from `patent-owner-response-window` therefore belong to `patent-owner` (`03-trial-to-appeal.lit:33`). The Director is the actor who reviews a request under §  42.75(e)(2), so the review edge now explicitly says `by director` (`:392`).

**DECIDED:** 35 U.S.C. § 319 says, “A party dissatisfied with the final written decision ... may appeal the decision.” The appeal-window incoming edge is `by either`, so the source does not establish which party filed the notice. The lowered state has no actor field before or after (`actor: null`); its review edge inherits `board`, which the PTAB package maps to `nature`. The CAFC link replaces the local notice edge in a composed graph. The node stays as authored with a source comment explaining that neutral placeholder; no new package role is needed. The before/after lowered pack diff confirms this node and its review edge are unchanged.

The full sweep of PTAB `state` and `decision` declarations found these party-name mismatches or ambiguous actors. The rest of the scanned edges with labels naming Petitioner, Patent Owner, Board, or Director agree with their inherited/explicit actor.

| Source location | Finding | Verdict |
|---|---|---|
| `lit/ptab/01-pre-petition-to-institution.lit:211` | POPR window offered patent-owner response/waiver/disclaimer while inheriting `either` | Fixed: all three options are patent-owner acts under § 42.107. |
| `lit/ptab/03-trial-to-appeal.lit:33` | Patent Owner Response window offered response/no-response/settlement while inheriting `either` | Fixed: § 42.120 assigns the response to the patent owner; the linked choices all belong to that party. |
| `lit/ptab/03-trial-to-appeal.lit:56` | Petitioner Reply edge inherited patent-owner from the preceding response state | Fixed: explicit `by petitioner`, under § 42.23. |
| `lit/ptab/03-trial-to-appeal.lit:91` | Patent Owner Sur-Reply edge inherited `either` | Fixed: explicit `by patent-owner`, as the response to the petitioner's reply. |
| `lit/ptab/03-trial-to-appeal.lit:317` | Request Director review has no single party actor | Left as `either`: either dissatisfied party may request review; § 42.75(c) does not identify a side. |
| `lit/ptab/03-trial-to-appeal.lit:367` | Escalate denied rehearing to Director has no single party actor | Left as `either`: a party may escalate, but the source does not identify which party. |
| `lit/ptab/03-trial-to-appeal.lit:477` | Notice node says `by board`; incoming notice edge is `either`; outgoing Federal Circuit review inherited `board` | Left with documented reason above: the node does not lower an actor; either party can appeal and the review transition is neutral `nature`. |
| `lit/ptab/03-trial-to-appeal.lit:392` | Director's review edge inherited `board` from the request state | Fixed: explicit `by director` under § 42.75(e)(2); `director` and `board` both map to `nature`, so this clarifies the actor without changing control. |

Before and after solves were run on the same source revision apart from the listed actor corrections. Values below are protagonist utility in USD; these are model outputs with the existing `probability-fill` and placeholder-payoff warnings, not sourced outcomes.

| Output | Old | New | Explanation of the move |
|---|---:|---:|---|
| PTAB package solve and `packs/.../ptab-patent-trial-appeal-board.json` pin | $514,737.442007 (policy 14) | $508,433.596697 (policy 18) | `popr-window` changes from nature (`q = $605,777.687487`) to opponent control. At the package start, the opponent chooses “File Patent Owner Preliminary Response” (`q = $554,246.096697`) rather than disclaimer (`q = $700,000`). At `patent-owner-response-window`, the opponent chooses “Elect not to respond” (`q = $571,093.754715`) over filing or settling (each `q = $610,546.877357`). The petitioner's best opening changes from “File PGR petition directly” (`q = $514,737.442007`) to “Consider challenging without suit” (`q = $508,433.596697`). At `patent-owner-response-filed`, the petitioner now controls the reply edge; across four flag contexts the full policy chooses `WAIT` (`q = $610,546.877357`), while the old opponent controller chose the reply (`q = $577,452.928158`). That corrected self move accounts for the policy-size increase from 14 to 18. |
| `cases/ptab-ipr-defense.case` | $1,191,295.315355 (policy 210) | $1,500,000.000000 (policy 216) | Matter perspective maps `patent-owner` to self. The old `popr-window` was a nature node with aggregate value $1,191,295.315355; after the fix, patent owner chooses “Disclaim all challenged claims” (`q = $1,500,000`) over filing or waiver (each `q = $1,052,738.071526`). That is the best line from the replayed timely petition posture. At the later POR decision, the corrected self controller chooses “Elect not to respond” (`q = $1,183,080.914441`); the old nature aggregate there was $1,121,391.351424. Policy size increases by six as corrected party-controlled decisions enter the protagonist policy across reachable flag states. |
| `ptab-calibrated-institution` request | $569,035.765633 | $569,808.108026 | Its institution chance node remains `nature`; its continuation changes because `popr-window` changes from nature (old aggregate $612,690.510422; response/waiver edge `q = $569,035.765633`) to opponent (response edge chosen, `q = $569,808.108026`). The request explains the institution node, so its value follows that updated continuation. |
| `ptab-robust-institution` request | $296,756.444858 (robust; nominal $501,550.765633) | $312,876.740975 (robust; nominal $502,323.108026) | The petitioner still chooses “Consider challenging without suit” at the start. At `popr-window`, nature's old aggregate was $374,453.166798; the corrected opponent chooses response (`q = $375,787.713669`) over disclaimer (`q = $700,000`). At `patent-owner-response-window`, nature's old aggregate was $410,528.071359; the opponent now chooses settlement (`q = $439,376.740975`) over filing (`q = $392,336.088979`) and no response (`q = $441,409.429136`). These response choices change the robust continuation value; the robust policy's selected opening stays the same. |
| `ptab-ipr-estoppel-flag` graph request | no scalar value | no scalar value | It is a graph op, not a solve. There is no value or policy to re-record; the mixed-node warning count remains 5. |

The appeal declaration's `by board` did not change its lowered node (no state actor is emitted), its `either` incoming edge, or its `board` review edge in the before/after pack diff. In composed use the link replaces the notice transition, so the selected appealing party enters the CAFC instance directly. No appeal-related solve value moved because of this declaration.

The role corrections also explain the non-value numeric changes in the recorded outputs: the package and graph-request `probability-fill` warning drops from 101 to 91, and the calibrated request's drops from 103 to 93, because the newly controlled party-choice edges no longer need residual chance fills. Robust `ambiguous_nodes` drops from 84 to 78, and its `prior-concentration-estimated` warning count drops from 78 to 72 because six formerly uncertain nodes are now controlled moves. The calibrated request's terminal-share `p` entries change because its downstream continuation follows the corrected owner choices. Solve `iterations` are convergence telemetry and change with the resulting policy/value graph; they are not authored legal estimates.

The defense matter value and policy size do move, and the explanation above identifies the changed controller, option, and q values. This matter has no inline `expect` block. Pins for the package and three PTAB requests are re-recorded from the compiled source after this explanation; the graph request remains non-scalar. The TTAB applicant-defense matter remains pinned at `-$31,820.00`.
