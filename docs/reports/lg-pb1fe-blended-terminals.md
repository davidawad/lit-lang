# Blended terminal inventory and proposed splits

> The inventory records each FRCP terminal as it appeared on origin/main before this bead's split; the split column records the resulting party-specific outcomes.

## Operational definition

A **blended terminal** is a `terminal` whose incoming edges, as shown by rule text or node/edge notes, admit both a result favourable to the package's `self` party and a result adverse to that party, while retaining one terminal outcome/payoff. I treated explicit notes using “blended,” “movant,” “losing party,” or “either party,” and converging edges from different actors or branches of a chance node as evidence. Counts below are source-level candidates; a common terminal reached from a prior result is included when the note or graph confirms that the prior result is not carried forward.

`self` here means the package's declared protagonist, not necessarily the named appellant or appellee in a particular matter. Proposed splits are design recommendations only for packages outside FRCP; those packages are not modified in this bead.

## Inventory

| Package | File:line | Blended terminal (main outcome before split) | Incoming favourable / adverse edges | Proposed split and separating rule |
|---|---|---|---|---|
| frcp | `02-pleading-to-trial.lit:898` | `summary-judgment-granted` (`final`; the “movant” may be plaintiff or defendant) | One `sj-ruling` chance edge (“Granted in full,” p=0.25) reaches it. The note leaves movant identity uncarried, so the same edge can mean judgment for plaintiff (self win) or defendant (self loss). | `summary-judgment-for-plaintiff` (`win`) / `summary-judgment-for-defendant` (`loss`), selected by movant under FRCP 56(a). |
| frcp | `04-judgment-appeal.lit:80` | `appeal-window-blown` (`procedural-default, waiver-trap, final`) | Plaintiff-verdict and defendant-verdict edges both converge at `final-judgment-entered`; one missed-notice edge then reaches this terminal. The note says that converged judgment may favor either side. | Party-direction win/loss copies plus the neutral `appeal-window-blown` (`final`) copy when neither flag is set; FRAP 4(a)(1)(A), 28 U.S.C. § 1291. |
| frcp | `04-judgment-appeal.lit:89` | `judgment-final-no-appeal` (`final`) | Plaintiff-verdict and defendant-verdict edges both converge at `final-judgment-entered`; one no-appeal edge reaches this terminal, so the waiver can leave either side's win or loss final. | `judgment-final-no-appeal-plaintiff` / `...-defendant` for known directions plus the neutral `judgment-final-no-appeal` when direction is unknown; 28 U.S.C. § 1291, FRAP 4(a)(1)(A). |
| frcp | `04-judgment-appeal.lit:182` | `judgment-affirmed` (`affirmed, final`) | The p=0.55 affirmance and p=0.10 net-affirmance edges both lead to this terminal after either plaintiff or defendant judgment has been collapsed at `final-judgment-entered`. | `judgment-affirmed-plaintiff` / `...-defendant` for known judgment direction plus the neutral `judgment-affirmed` when unknown; 28 U.S.C. §§ 1291, 2106. |
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

The lines below count terminal declarations whose own text contains one of “blended,” “movant,” “losing party,” or “either party” and whose outcome includes win or loss. These are scan candidates only; I did not audit all incoming-edge structures in these families.

- acca: 0
- afcca: 0
- ak-court-of-appeals: 0
- ak-criminal-trial-court: 0
- ak-rules: 0
- ak-superior: 0
- ak-supreme: 0
- akd: 0
- akd-criminal: 0
- al-court-of-criminal-appeals: 0
- al-criminal-trial-court: 0
- al-rules: 0
- al-state-appellate: 0
- al-state-civil: 1
- al-supreme-court: 0
- almd: 0
- almd-criminal: 0
- alnd: 0
- alnd-criminal: 0
- alsd: 0
- alsd-criminal: 0
- ar-circuit: 1
- ar-criminal-trial-court: 0
- ar-rules: 0
- ar-supreme-court: 0
- ar-workers-compensation-commission: 0
- ared: 0
- ared-criminal: 0
- arkctapp: 0
- arwd: 0
- arwd-criminal: 0
- as-criminal-trial-court: 0
- as-high-court: 0
- as-high-court-appellate: 0
- as-rules: 0
- az-court-of-appeals: 0
- az-criminal-trial-court: 0
- az-rules: 0
- az-superior: 0
- az-tax-court: 0
- azd: 0
- azd-criminal: 0
- bankr-ak: 0
- bankr-alm: 0
- bankr-aln: 0
- bankr-als: 0
- bankr-are: 0
- bankr-arw: 0
- bankr-az: 0
- bankr-cac: 0
- bankr-cae: 0
- bankr-can: 0
- bankr-cas: 0
- bankr-co: 0
- bankr-ct: 0
- bankr-dc: 0
- bankr-de: 0
- bankr-flm: 0
- bankr-fln: 0
- bankr-fls: 0
- bankr-gam: 0
- bankr-gan: 0
- bankr-gas: 0
- bankr-gu: 0
- bankr-hi: 0
- bankr-ian: 0
- bankr-ias: 0
- bankr-id: 0
- bankr-ilc: 0
- bankr-iln: 0
- bankr-ils: 0
- bankr-inn: 0
- bankr-ins: 0
- bankr-ks: 0
- bankr-kye: 0
- bankr-kyw: 0
- bankr-lae: 0
- bankr-lam: 0
- bankr-law: 0
- bankr-ma: 0
- bankr-md: 0
- bankr-me: 0
- bankr-mie: 0
- bankr-miw: 0
- bankr-mn: 0
- bankr-moe: 0
- bankr-mow: 0
- bankr-mp: 0
- bankr-msn: 0
- bankr-mss: 0
- bankr-mt: 0
- bankr-nce: 0
- bankr-ncm: 0
- bankr-ncw: 0
- bankr-nd: 0
- bankr-ne: 0
- bankr-nh: 0
- bankr-nj: 0
- bankr-nm: 0
- bankr-nv: 0
- bankr-nye: 0
- bankr-nyn: 0
- bankr-nys: 0
- bankr-nyw: 0
- bankr-ohn: 0
- bankr-ohs: 0
- bankr-oke: 0
- bankr-okn: 0
- bankr-okw: 0
- bankr-or: 0
- bankr-pae: 0
- bankr-pam: 0
- bankr-paw: 0
- bankr-pr: 0
- bankr-ri: 0
- bankr-sc: 0
- bankr-sd: 0
- bankr-tne: 0
- bankr-tnm: 0
- bankr-tnw: 0
- bankr-txe: 0
- bankr-txn: 0
- bankr-txs: 0
- bankr-txw: 0
- bankr-ut: 0
- bankr-vae: 0
- bankr-vaw: 0
- bankr-vi: 0
- bankr-vt: 0
- bankr-wae: 0
- bankr-waw: 0
- bankr-wie: 0
- bankr-wiw: 0
- bankr-wvn: 0
- bankr-wvs: 0
- bankr-wy: 0
- bap1: 0
- bap10: 0
- bap6: 0
- bap8: 0
- bap9: 0
- bia: 0
- ca-court-of-appeal: 0
- ca-court-of-appeal-fifth-district: 0
- ca-court-of-appeal-first-district: 0
- ca-court-of-appeal-fourth-district: 0
- ca-court-of-appeal-second-district: 0
- ca-court-of-appeal-sixth-district: 0
- ca-court-of-appeal-third-district: 0
- ca-criminal-trial-court: 0
- ca-rules: 0
- ca-superior: 0
- ca-superior-court-appellate-division: 0
- ca-superior-court-san-diego-county: 0
- ca-supreme-court: 0
- ca1: 0
- ca10: 0
- ca11: 0
- ca2: 0
- ca3: 0
- ca4: 0
- ca5: 0
- ca6: 0
- ca7: 0
- ca8: 0
- ca9: 0
- caaf: 0
- cacd: 0
- cacd-criminal: 0
- cadc: 0
- caed: 0
- caed-criminal: 0
- cand: 0
- cand-criminal: 0
- casd: 0
- casd-criminal: 0
- cavc: 0
- cgcca: 0
- cit: 0
- cmcr: 0
- co-criminal-trial-court: 0
- co-industrial-claim-appeals-office: 0
- co-rules: 0
- co-state-appellate: 0
- co-state-civil: 1
- co-supreme-court: 0
- cod: 0
- cod-criminal: 0
- cofc: 0
- core: 0
- ct-appellate-supreme: 0
- ct-compensation-review-board: 0
- ct-criminal-trial-court: 0
- ct-rules: 0
- ct-superior-civil: 0
- ctd: 0
- ctd-criminal: 0
- dc-criminal-trial-court: 0
- dc-rules: 0
- dc-state-appellate: 0
- dc-state-civil: 1
- dcd: 0
- dcd-criminal: 0
- de-court-of-chancery: 0
- de-court-on-the-judiciary: 0
- de-criminal-trial-court: 0
- de-family-court: 0
- de-rules: 0
- de-superior-civil: 0
- de-supreme-civil: 0
- ded: 0
- ded-criminal: 0
- examples: 0
- fisc: 0
- fiscr: 0
- fl-criminal-trial-court: 0
- fl-district-court-of-appeal: 0
- fl-district-court-of-appeal-fifth: 0
- fl-district-court-of-appeal-first: 0
- fl-district-court-of-appeal-fourth: 0
- fl-district-court-of-appeal-second: 0
- fl-district-court-of-appeal-sixth: 0
- fl-district-court-of-appeal-third: 0
- fl-rules: 0
- fl-state-civil: 1
- fl-supreme-court: 0
- flmd: 0
- flmd-criminal: 0
- flnd: 0
- flnd-criminal: 0
- flsd: 0
- flsd-criminal: 0
- frbp: 0
- frcp-2025: 0
- frcrimp: 0
- ga-criminal-trial-court: 0
- ga-rules: 0
- ga-state-civil: 1
- ga-supreme-court: 0
- gactapp: 0
- gamd: 0
- gamd-criminal: 0
- gand: 0
- gand-criminal: 0
- gasd: 0
- gasd-criminal: 0
- gu-criminal-trial-court: 0
- gu-superior: 0
- gu-supreme: 0
- gud: 0
- gud-criminal: 0
- habeas: 0
- hi-circuit: 0
- hi-criminal-trial-court: 0
- hi-ica: 0
- hi-rules: 0
- hi-supreme-court: 0
- hi-tax-appeal-court: 0
- hid: 0
- hid-criminal: 0
- ia-criminal-trial-court: 0
- ia-district: 1
- ia-rules: 0
- ia-supreme-court: 0
- iand: 0
- iand-criminal: 0
- iasd: 0
- iasd-criminal: 0
- id-court-of-appeals: 0
- id-criminal-trial-court: 0
- id-district: 0
- id-rules: 0
- id-supreme: 0
- idd: 0
- idd-criminal: 0
- il-circuit: 0
- il-criminal-trial-court: 0
- il-rules: 0
- il-supreme-court: 0
- ilcd: 0
- ilcd-criminal: 0
- illappct: 0
- ilnd: 0
- ilnd-criminal: 0
- ilsd: 0
- ilsd-criminal: 0
- in-criminal-trial-court: 0
- in-tax-court: 0
- in-trial: 0
- indctapp: 0
- innd: 0
- innd-criminal: 0
- insd: 0
- insd-criminal: 0
- iowactapp: 0
- jpml: 0
- ks-court-of-appeals: 0
- ks-criminal-trial-court: 0
- ks-district: 0
- ks-rules: 0
- ks-supreme-court: 0
- ksd: 0
- ksd-criminal: 0
- ky-circuit-court: 0
- ky-court-of-appeals: 0
- ky-criminal-trial-court: 0
- ky-rules: 0
- ky-supreme-court: 0
- kyed: 0
- kyed-criminal: 0
- kywd: 0
- kywd-criminal: 0
- la-appellate: 0
- la-criminal-trial-court: 0
- la-district-court: 0
- la-rules: 0
- la-supreme: 0
- laed: 0
- laed-criminal: 0
- lamd: 0
- lamd-criminal: 0
- lawd: 0
- lawd-criminal: 0
- links: 0
- ma-appeals-court: 0
- ma-appellate-tax-board: 0
- ma-criminal-trial-court: 0
- ma-department-of-industrial-accidents: 0
- ma-district-court-appellate-division: 0
- ma-rules: 0
- ma-sjc: 0
- ma-superior-civil: 1
- mad: 0
- mad-criminal: 0
- md-appellate-civil: 0
- md-circuit-civil: 1
- md-criminal-trial-court: 0
- md-rules: 0
- md-supreme-court: 0
- mdd: 0
- mdd-criminal: 0
- me-criminal-trial-court: 0
- me-law-court: 0
- me-superior-civil: 1
- med: 0
- med-criminal: 0
- mi-appellate: 0
- mi-circuit-court: 0
- mi-criminal-trial-court: 0
- mi-rules: 0
- mi-supreme-court: 0
- mied: 0
- mied-criminal: 0
- minnctapp: 0
- miwd: 0
- miwd-criminal: 0
- mn-criminal-trial-court: 0
- mn-district: 0
- mn-rules: 0
- mn-supreme-court: 0
- mnd: 0
- mnd-criminal: 0
- mo-circuit: 0
- mo-criminal-trial-court: 0
- mo-rules: 0
- mo-supreme: 0
- moctapp: 0
- moctapp-ed: 0
- moctapp-sd: 0
- moctapp-wd: 0
- moed: 0
- moed-criminal: 0
- mowd: 0
- mowd-criminal: 0
- mp-criminal-trial-court: 0
- mp-supreme: 0
- mp-trial: 0
- ms-appellate: 0
- ms-chancery-courts: 0
- ms-circuit-court: 0
- ms-criminal-trial-court: 0
- ms-rules: 0
- ms-supreme-court: 0
- msnd: 0
- msnd-criminal: 0
- mssd: 0
- mssd-criminal: 0
- mt-criminal-trial-court: 0
- mt-district: 0
- mt-rules: 0
- mt-supreme: 0
- mt-tax-appeal-board: 0
- mtd: 0
- mtd-criminal: 0
- nc-appellate-civil: 0
- nc-criminal-trial-court: 0
- nc-industrial-commission: 0
- nc-rules: 0
- nc-superior-civil: 0
- nced: 0
- nced-criminal: 0
- ncmd: 0
- ncmd-criminal: 0
- ncwd: 0
- ncwd-criminal: 0
- nd-criminal-trial-court: 0
- nd-district: 0
- nd-rules: 0
- ndctapp: 0
- ndd: 0
- ndd-criminal: 0
- ne-criminal-trial-court: 0
- ne-district: 0
- ne-rules: 0
- ne-supreme-court: 0
- nebctapp: 0
- ned: 0
- ned-criminal: 0
- nh-criminal-trial-court: 0
- nh-rules: 0
- nh-superior-civil: 1
- nh-supreme: 0
- nhd: 0
- nhd-criminal: 0
- nj-appellate-civil: 0
- nj-criminal-trial-court: 0
- nj-rules: 0
- nj-superior-law: 0
- nj-supreme-court: 0
- nj-tax-court: 0
- njd: 0
- njd-criminal: 0
- nm-criminal-trial-court: 0
- nm-rules: 0
- nm-state-appellate: 0
- nm-state-civil: 1
- nm-supreme-court: 0
- nmcca: 0
- nmd: 0
- nmd-criminal: 0
- nmid: 0
- nmid-criminal: 0
- nv-court-of-appeals: 0
- nv-criminal-trial-court: 0
- nv-district: 0
- nv-rules: 0
- nv-supreme: 0
- nvd: 0
- nvd-criminal: 0
- ny-appellate-division: 0
- ny-appellate-terms: 0
- ny-commission-of-appeals: 0
- ny-court-for-the-trial-of-impeachments: 0
- ny-court-of-appeals: 0
- ny-criminal-trial-court: 0
- ny-rules: 0
- ny-saratoga-chancery-court: 0
- ny-supreme-civil: 1
- nyed: 0
- nyed-criminal: 0
- nynd: 0
- nynd-criminal: 0
- nysd: 0
- nysd-criminal: 0
- nywd: 0
- nywd-criminal: 0
- oh-common-pleas: 0
- oh-court-of-appeals: 0
- oh-court-of-claims: 0
- oh-criminal-trial-court: 0
- oh-rules: 0
- oh-supreme-court: 0
- ohnd: 0
- ohnd-criminal: 0
- ohsd: 0
- ohsd-criminal: 0
- ok-court-of-civil-appeals: 0
- ok-court-of-criminal-appeals: 0
- ok-court-on-the-judiciary: 0
- ok-criminal-trial-court: 0
- ok-district: 1
- ok-judicial-ethics-advisory-panel: 0
- ok-rules: 0
- ok-supreme-court: 0
- oked: 0
- oked-criminal: 0
- oknd: 0
- oknd-criminal: 0
- okwd: 0
- okwd-criminal: 0
- or-circuit: 0
- or-court-of-appeals: 0
- or-criminal-trial-court: 0
- or-dor: 0
- or-rules: 0
- or-tax-court: 0
- ord-criminal: 0
- pa-commonwealth-court: 0
- pa-court-of-judicial-discipline: 0
- pa-cp-civil: 0
- pa-criminal-trial-court: 0
- pa-rules: 0
- pa-superior-court: 0
- pa-supreme-court: 0
- paed: 0
- paed-criminal: 0
- pamd: 0
- pamd-criminal: 0
- pawd: 0
- pawd-criminal: 0
- pr-criminal-trial-court: 0
- pr-primera-instancia-civil: 0
- pr-rules: 0
- pr-supreme-court: 0
- pr-tribunal-apelaciones: 0
- prd: 0
- prd-criminal: 0
- ri-criminal-trial-court: 0
- ri-rules: 0
- ri-superior-civil: 1
- ri-supreme: 0
- rid: 0
- rid-criminal: 0
- sc-circuit-civil: 0
- sc-court-of-appeals: 0
- sc-court-of-chancery: 0
- sc-criminal-trial-court: 0
- sc-rules: 0
- sc-supreme-court: 0
- scd: 0
- scd-criminal: 0
- scotus: 0
- sd: 0
- sd-circuit: 0
- sd-criminal-trial-court: 0
- sdd: 0
- sdd-criminal: 0
- tax-court: 0
- tn-appellate: 0
- tn-court-of-criminal-appeals: 0
- tn-criminal-trial-court: 0
- tn-rules: 0
- tn-supreme-court: 0
- tn-trial-court: 0
- tned: 0
- tned-criminal: 0
- tnmd: 0
- tnmd-criminal: 0
- tnwd: 0
- tnwd-criminal: 0
- ttab: 0
- tx-court-of-civil-appeals: 0
- tx-court-of-criminal-appeals: 0
- tx-court-of-review-tribunal: 0
- tx-courts-of-appeals: 0
- tx-criminal-trial-court: 0
- tx-district-court: 0
- tx-district-court-travis: 0
- tx-judicial-panel-on-multidistrict-litigation: 0
- tx-rules: 0
- tx-special-court-of-review: 0
- tx-supreme-court: 0
- txed: 0
- txed-criminal: 0
- txnd: 0
- txnd-criminal: 0
- txsd: 0
- txsd-criminal: 0
- txwd: 0
- txwd-criminal: 0
- ut-criminal-trial-court: 0
- ut-rules: 0
- ut-state-appellate: 0
- ut-state-civil: 0
- ut-supreme-court: 0
- utd: 0
- utd-criminal: 0
- va-circuit-civil: 0
- va-court-of-appeals: 0
- va-criminal-trial-court: 0
- va-rules: 0
- va-supreme-court: 0
- vaed: 0
- vaed-criminal: 0
- vawd: 0
- vawd-criminal: 0
- vi-criminal-trial-court: 0
- vi-superior-civil: 0
- vi-supreme-civil: 0
- vid: 0
- vid-criminal: 0
- vt-criminal-trial-court: 0
- vt-superior-civil: 1
- vt-supreme: 0
- vtd: 0
- vtd-criminal: 0
- wa-court-of-appeals: 0
- wa-criminal-trial-court: 0
- wa-rules: 0
- wa-superior: 0
- wa-supreme-court: 0
- waed: 0
- waed-criminal: 0
- wawd: 0
- wawd-criminal: 0
- wi-circuit: 0
- wi-court-of-appeals: 0
- wi-criminal-trial-court: 0
- wi-rules: 0
- wi-supreme-court: 0
- wied: 0
- wied-criminal: 0
- wiwd: 0
- wiwd-criminal: 0
- wv-appellate-civil: 0
- wv-circuit-civil: 0
- wv-criminal-trial-court: 0
- wv-rules: 0
- wv-supreme-court: 0
- wvnd: 0
- wvnd-criminal: 0
- wvsd: 0
- wvsd-criminal: 0
- wy-criminal-trial-court: 0
- wy-district: 0
- wy-rules: 0
- wy-supreme: 0
- wyd: 0
- wyd-criminal: 0

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

The summary-judgment duration estimate is now attached to both party-specific edges, each at 410.91 days, with the same 21,861-case source and the existing Table C-5 basis. The generated calibration reference was updated for the two new edge ids, and `calibration-gaps.json` was updated for the changed FRCP response. The FRCP package solve fixture is intentionally re-recorded to $477,142.50 below. It is read by `roundtrip::every_package_compiles_and_solves_to_litgraph_final`, which compiles each current package and compares its fresh solve to the recorded response; leaving $478,445 would fail that comparison. It is a historical baseline, but this test deliberately checks current compiled behavior against it, so this intentional package-value change requires updating it.

## CLI regression filter comparison

The requested filter initially reported 53 failures on this branch (109 tests); the same filter on origin/main reported 52 failures (108 tests). Fifty names failed in both runs and are listed below as baseline failures. Two timeline tests failed only on origin/main (`timeline::frcp_civil_matter` and `timeline::frcp_filed_fixture_forks_with_unknown_deadlines`) and passed on this branch. The three branch-only failures were `frcp_extensions::a_request_after_the_order_does_not_license_it`, `roundtrip::calibration_gaps`, and `roundtrip::every_calibration_set_lowers_to_litgraph_final`; after correcting the typed-role test fixture and the calibration references, focused reruns passed all three (2/3 first, then the remaining calibration test 1/1). The full filter was not rerun after those fixes.

The 50 failures present on both revisions:

- `frcp_extensions`: `a_cert_extension_past_60_days_or_by_a_court_is_rejected`, `a_justice_extends_the_cert_period_by_up_to_60_days`, `a_late_federal_circuit_cert_application_needs_extraordinary_circumstances`, `an_unvendored_cert_extension_does_not_reach_an_expired_period`, `cert_extension_timing_follows_rules_13_5_and_30`, `court_ordered_extensions_are_recorded`, `frap_4a3_cross_appeal_runs_to_the_later_of_14_days_and_the_notice_period`, `frap_4a5_caps_the_notice_of_appeal_extension`, `frap_4a5_counts_the_cross_appeal_motion_window_from_the_later_date`, `frap_4a5_extends_the_cross_appeal_period`, `frap_4a5_motion_must_come_within_30_days_of_the_prescribed_time`, `frap_4a5_order_on_day_40_allows_filing_through_day_54`, `only_the_court_extends`, `rule_6b1_after_expiry_without_a_request_before_or_a_motion_after_is_rejected`, `rule_6b1_has_no_ceiling_and_reaches_an_expired_period_on_motion`, `rule_6b1a_turns_on_when_the_request_was_made`, `the_director_extends_the_ttab_civil_action_period`, `the_director_not_the_court_extends_the_federal_circuit_ptab_window`, `the_federal_circuit_cross_appeal_runs_to_the_later_of_14_days_and_frap_4a1`, `the_waiver_answer_and_rule_15a1_periods_are_extendable`.
- `frcp_fee_moves`: `a_fee_motion_after_the_fourteenth_day_is_late`, `a_later_scheduling_order_supersedes_the_supplement_date`, `a_motion_after_expiry_extends_for_excusable_neglect`, `a_protective_motion_and_its_supplement_count_the_hours_once`, `a_stipulation_after_the_due_date_licenses_no_order`, `a_stipulation_re_anchors_the_fee_clock`, `a_supplement_after_the_date_the_court_set_is_late`, `an_open_fee_clock_lists_its_protective_moves`, `the_full_motion_supplements_the_protective_one_and_discharges_its_deadline`, `the_post_trial_window_offers_the_fee_filing_beside_the_other_motions`.
- `pending_moves`: `a_motion_still_awaiting_its_order_past_the_due_date_reads_on_the_lapsed_row`, `a_stipulation_awaiting_its_order_is_the_fee_clocks_pending_move`, `lint_filing_a_stipulation_lists_the_move_it_makes_pending`, `the_order_ends_the_pending_move`.
- `roundtrip`: `every_package_compiles_and_solves_to_litgraph_final`, `ptab_calibrated_institution`, `ptab_ipr_estoppel_flag`, `ptab_robust_institution`.
- `rulings`: `a_cited_excusing_event_covers_a_missed_deadline`, `an_event_after_as_of_is_rejected`, `the_shared_clock_library_is_lit_core_lowered_and_sync_builds_under_dev`.
- `timeline`: `a_child_world_inserting_an_earlier_trigger_keeps_every_inherited_key`, `a_local_rule_raising_a_frap_limit_is_the_layer_of_its_row`, `a_restarted_or_absorbed_period_names_the_row_replacing_it`, `a_standing_order_sets_the_limit_and_its_row_names_every_layer`, `a_world_triggering_a_deadline_its_parent_left_untriggered_re_emits_it`, `an_alternate_world_filing_a_paper_shows_the_layer_that_set_its_limit`, `an_answer_to_a_notice_without_its_due_date_reads_unknown_with_why`, `ttab_opposition_matter`, `ttab_published_fixture_forks_with_unknown_deadlines`.

## Rebase measurements (origin/main `bcc6bef1` → merged branch)

DECIDED: Keep origin/main's typed-provenance fixture as the merge base and re-record it with the merged release binary; this preserves lg-pb1fe.44's pointer/basis fields while recording the split's current values.

`calibration-gaps.json` was resolved from main's typed-provenance fixture and re-recorded with the merged release binary (`LITIGATE_RERECORD=1 cargo test --release -p litigate-cli --test roundtrip calibration_gaps`: 1 passed). Main's fixture had base value **478,445**, 10 probability rows and 10 duration rows. The merged fixture has base value **477,142.5**, with 10 probability and 10 duration rows; rows retain `pointer` and `basis` and remain ordered by basis, descending swing, then pointer.

Rows with pointers present on both sides whose numbers changed:

| Pointer | Main row and values | Merged row and values | Reason |
|---|---|---|---|
| `/edges/44/duration` | Rule 60 relief granted, 45–75 days | Same edge, 45–60 days | With the ruling now chance-controlled, the split/cleared judgment flags make the reopened neutral route part of the Rule 60 branch value. |
| `/edges/46/duration` | Rule 60 relief granted → reopened case, 45–75 days | Same edge, 45–60 days | The reopened case clears both judgment-direction flags; its post-reopen neutral judgment route changes this duration sensitivity. |
| `/edges/112/probability` | Trial commences → verdict rendered, 478,445 at both endpoints | Final pretrial conference → trial commences, 477,142.5 at both endpoints | The split inserts pack edges, so this array-index pointer now addresses a different edge. Its endpoint values track the base solve change caused by the Rule 60 court chance and neutral judgment direction; it is not a stable edge identity. |

`frcp-fy2024.json` also changed: the 3-entry main calibration now has 4 entries, with the prior single summary-judgment duration (410.91 days) applied to both `edge-sj-ruling-summary-judgment-for-plaintiff` and `edge-sj-ruling-summary-judgment-for-defendant`. The two voluntary-dismissal entries remain 252.63 days. No authored duration changed; the same Table C-5 source, sample count (21,861), and median basis apply to both split edges.

Release solve measurements, main first and merged branch second:

| Target | Value | Opponent value | Policy size | Best line |
|---|---:|---:|---:|---|
| `lit/frcp` package | 478,445 → 477,142.5 | Not emitted by `solve` (the default scenario has no `opponent_objective`) | 103 → 373 | Before: voluntary dismissal → Rule 60 relief granted → final no-appeal. After: Rule 60 relief denied (p=0.5) → appeal window → final no-appeal. |
| `cases/frcp-civil-case.case` | −16,655 → 75,713.461431 | Not emitted (no `opponent_objective`) | 103 → 373 | Before: formal service path reaches the 90-day service default. After: request service waiver → opponent files Rule 12(b) motion → 0.45 motion-denial branch → proceed through trial → defendant verdict (p=0.5) and defendant-judgment loss. |

The current package-level best line ends at the neutral `judgment-final-no-appeal` terminal; its default package payoff is still 500,000. A neutral terminal remains reachable with positive probability from the root: voluntary dismissal → Rule 60 motion → relief granted (default-filled p=0.5) → reopened case (both flags cleared) → final judgment → notice-of-appeal window → neutral no-appeal. The matter overrides the neutral `final` outcome to $0, so that branch does not inherit the package's 500,000 default.

The package value changed because Rule 60 is now a court-controlled chance node, not a choice by `self`; the denied branch and granted/reopened branch are both included. On the matter best line, actor correction is visible edge by edge: formal-service/default on main is replaced by waiver request, the opponent's Rule 12(b) move, the court's p=0.45 denial branch, and the p=0.5 defendant-verdict edge. That edge sets `judgment-for-defendant`, clears `judgment-for-plaintiff`, and selects the loss payoff. The expected matter value rises by $92,368.461431; it is not a new plaintiff-win outcome.

The FRCP package golden in `crates/litigate-cli/tests/litgraph-final/packs/frcp-civil-procedure.json` moved from $478,445 to $477,142.5 because `roundtrip::every_package_compiles_and_solves_to_litgraph_final` compares it to a fresh solve. It is not an immutable external reference. The frozen `litgraph-final` provenance is historical context, not a reason to pin an intentionally changed current result.

## Final gate evidence

- `just lit-sync`: exit 0; regenerated locks and build outputs. `just lit-sync-check`: exit 0, locks and shared clock current, no generated files committed.
- Release `litigate check lit cases`: `ok: true`, 0 errors. `litigate test lit/frcp`: 49 passed, 0 failed. `litigate fmt --check lit cases`: exit 0.
- `cargo clippy -p litigate-cli --all-targets -- -D warnings`: exit 0. `cargo fmt --check`: exit 0.
- Focused release re-records passed: `calibration_gaps` 1/1 and `every_package_compiles_and_solves_to_litgraph_final` 1/1. The corrected line-number snapshot test `cov_cmd_import::a_number_the_typed_source_cannot_hold_fails_the_check` passed 1/1.
- Final `cargo nextest run -p litigate-cli --no-fail-fast`: 655 tests, 654 passed, 1 failed. The only final failure was `cov_cmd_value::an_edge_without_a_deadline_by_package_name`; it also failed on the separate `origin/main` clone with the same three new L9003 info entries from unchanged `lit/cov/cov.lit`, so it is a main baseline failure. The first full run additionally failed `cov_cmd_import::a_number_the_typed_source_cannot_hold_fails_the_check` because this bead shifted a label in `cases/frcp-civil-case.case` from line 34 to 41; its snapshot was updated, and it passed in the final run. All five PM-listed main reds (`ttab_check_human_output_is_limited_to_the_case_when_core_has_no_holes`, the two named director-extension tests, and the two named TTAB timeline tests) passed on the merged branch.

FOLLOWUP: split blended terminals in `lit/frap`
FOLLOWUP: split blended terminals in `lit/ptab`
FOLLOWUP: split blended terminals in `lit/itc`
FOLLOWUP: split blended terminals in `lit/cafc`
