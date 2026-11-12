# Audit: packages with multiple roles defaulting to `self`

This inventory was generated from every `lit/*/Lit.toml` `[package.roles]` table on 2026-11-11. It lists all 129 packages with at least two roles mapped to `self`. The classification is based on the role names and the package's rule text: criminal procedure pairs `accused` with `prosecution` as opposing parties; PTAB's 35 U.S.C. § 311(a) expressly defines a petitioner as a person other than the patent owner; TTAB's opposition rules distinguish applicant from opposer/respondent. In appellate packages, `appellant` and `petitioner` are usually the same party-facing side in alternative review tracks (appeal vs extraordinary review), rather than opposing litigants. `movant` is a procedural actor that can be either side and cannot be equated to the plaintiff/appellant without a matter's facts.

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
- **TTAB**: `appellant` and `respondent` represent opposing parties in the opposition proceeding vocabulary; `cases/ttab-opposition-applicant-defense.case` declares applicant perspective. The package should leave these adversarial party defaults neutral and use the matter's perspective.
- **Criminal packages**: `accused` and `prosecution` are adverse in the governing criminal rules, but no `cases/` matter with a declared perspective loads any of these packages, so no package-level role correction is made here.
- **Other packages**: packages with `appellant`/`petitioner` encode successive/alternative procedural capacities of the party on the prosecuting side; their rule text does not establish that the two actor labels are opposing parties. `cofc`'s `plaintiff` and generic `movant`, and `cafc`/`frap` `appellant` and `movant`, can refer to the same or different persons depending on a matter. No other audited package pair has both an adverse relationship established by its rule text and an existing perspective-bearing matter in `cases/` to safely correct in this change.

The TTAB matter already uses the existing perspective mechanism, so its package role defaults were corrected in the same change as PTAB. Criminal role defaults and context-dependent movant combinations remain for matters to resolve where such matters exist.

## Resolution in this bead

PTAB's and TTAB's adverse-party role defaults are now `nature` in their `Lit.toml` packages. PTAB's existing matter perspective assigns its `patent-owner` actor to `self` and its petitioner actor to `opponent`; TTAB's applicant perspective assigns its `respondent` actor to `self` and its opposer remains `opponent`. Other audited criminal pairs have no perspective-bearing matter in `cases/`; no changes were made to them. The appellant/petitioner alternatives and context-dependent movant roles likewise remain unchanged.

For `cases/ptab-ipr-defense.case`, the release `solve` value was `$1,191,295.315355` both before and after. This unchanged result is explained by the matter's explicit `perspective { patent-owner: self, petitioner: opponent }`, which already overrides the package defaults. The post-change `explain` trace at `challenge-vehicle-decision{prior-petition-denied}` reports `control: opponent`; its three options are petitioner actions and the opponent selects the institution-granted path (`q = $980,380.473032`). At the matter's actual start, the time-bar fact resolves to the timely path and the following POPR alternatives are actor `either`, hence neutral nature draws. The correction therefore repairs package defaults for matters that do not specify a perspective without changing this already-perspectivized matter's value or policy. No inline PTAB test or snapshot expectation moved; all 35 PTAB tests pass.

The PTAB citation spellings in `.lit` cite directives are now uniform: 154 standard `37 C.F.R. §` fragments, 13 historically versioned `37 C.F.R. (2012) §` fragments, and 100 `35 U.S.C. §` fragments; there are zero legacy `37 CFR` fragments and zero bare-number `35 U.S.C.` fragments. The historical `(2012)` prefix is retained because `Lit.toml` declares its own dated source and section heading. Before normalization, the mixed counts recorded in the bead were 52 / 39 / 6 for the three ordinary CFR spellings and 43 / 12 for USC with/without §. `litigate check lit/ptab` had zero cite diagnostics before normalization and has zero cite diagnostics after; the post-change diagnostics are 105 L9011 and 24 L9002 warnings, none cite-resolution errors. `litigate lint lit/ptab` reports no unresolved/unverifiable cite warnings after normalization.
