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
