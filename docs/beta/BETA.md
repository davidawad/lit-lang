# Beta tester guide

**Snapshot: 2026-11-01.** This guide describes the source tree at that date. Rule coverage and its effective date are package-specific; this is not a statement that every package is current through 2026-11-01.

## What litgraph is

litgraph models a court or agency forum's procedure as a directed graph and composes forums into a stochastic game: what should a protagonist do, what might it cost, how long might it take, what endings are modeled, and how sensitive is the answer to assumptions? It is intended for litigators and law students exploring procedure, agents that need a sourced and explicitly hedged model, and builders of litigation-adjacent tools. (Adapted from the [README](../../README.md).)

> **Not legal advice.** litgraph's probabilities are teaching estimates (vintage noted in the pack, where authored at all) and most payoffs are placeholders, not real damages figures. Every response reports which numbers are authored law/data vs. engine fallbacks (`warnings`, `provenance.modes`) — read them before trusting a number. This project does not represent anyone, does not predict real case outcomes, and is not a substitute for counsel.

## Modeled scope and dates

At this snapshot, `lit/` contains 635 package directories represented in the strict baseline. This is the catalog of modeled forums, courts, and jurisdictions; package identifiers are the directory names below. Some packages share rules or compose through `lit/links`. Read each package's `Lit.toml` and `.lit` declarations for its exact procedure and authorities. Source declarations carry their own as-of/effective intervals; there is no single catalog-wide law date. The build selects rule versions for their declared intervals. A package may therefore cover only a slice of a court's procedure, and source coverage can be older than the snapshot date.

<details>
<summary>All package identifiers (635)</summary>

`acca`, `afcca`, `ak-court-of-appeals`, `ak-criminal-trial-court`, `ak-rules`, `ak-superior`, `ak-supreme`, `akd`, `akd-criminal`, `al-court-of-criminal-appeals`, `al-criminal-trial-court`, `al-rules`, `al-state-appellate`, `al-state-civil`, `al-supreme-court`, `almd`, `almd-criminal`, `alnd`, `alnd-criminal`, `alsd`, `alsd-criminal`, `ar-circuit`, `ar-criminal-trial-court`, `ar-rules`, `ar-supreme-court`, `ar-workers-compensation-commission`, `ared`, `ared-criminal`, `arkctapp`, `arwd`, `arwd-criminal`, `as-criminal-trial-court`, `as-high-court`, `as-high-court-appellate`, `as-rules`, `az-court-of-appeals`, `az-criminal-trial-court`, `az-rules`, `az-superior`, `az-tax-court`, `azd`, `azd-criminal`, `bankr-ak`, `bankr-alm`, `bankr-aln`, `bankr-als`, `bankr-are`, `bankr-arw`, `bankr-az`, `bankr-cac`, `bankr-cae`, `bankr-can`, `bankr-cas`, `bankr-co`, `bankr-ct`, `bankr-dc`, `bankr-de`, `bankr-flm`, `bankr-fln`, `bankr-fls`, `bankr-gam`, `bankr-gan`, `bankr-gas`, `bankr-gu`, `bankr-hi`, `bankr-ian`, `bankr-ias`, `bankr-id`, `bankr-ilc`, `bankr-iln`, `bankr-ils`, `bankr-inn`, `bankr-ins`, `bankr-ks`, `bankr-kye`, `bankr-kyw`, `bankr-lae`, `bankr-lam`, `bankr-law`, `bankr-ma`, `bankr-md`, `bankr-me`, `bankr-mie`, `bankr-miw`, `bankr-mn`, `bankr-moe`, `bankr-mow`, `bankr-mp`, `bankr-msn`, `bankr-mss`, `bankr-mt`, `bankr-nce`, `bankr-ncm`, `bankr-ncw`, `bankr-nd`, `bankr-ne`, `bankr-nh`, `bankr-nj`, `bankr-nm`, `bankr-nv`, `bankr-nye`, `bankr-nyn`, `bankr-nys`, `bankr-nyw`, `bankr-ohn`, `bankr-ohs`, `bankr-oke`, `bankr-okn`, `bankr-okw`, `bankr-or`, `bankr-pae`, `bankr-pam`, `bankr-paw`, `bankr-pr`, `bankr-ri`, `bankr-sc`, `bankr-sd`, `bankr-tne`, `bankr-tnm`, `bankr-tnw`, `bankr-txe`, `bankr-txn`, `bankr-txs`, `bankr-txw`, `bankr-ut`, `bankr-vae`, `bankr-vaw`, `bankr-vi`, `bankr-vt`, `bankr-wae`, `bankr-waw`, `bankr-wie`, `bankr-wiw`, `bankr-wvn`, `bankr-wvs`, `bankr-wy`, `bap1`, `bap10`, `bap6`, `bap8`, `bap9`, `bia`, `ca-court-of-appeal`, `ca-court-of-appeal-fifth-district`, `ca-court-of-appeal-first-district`, `ca-court-of-appeal-fourth-district`, `ca-court-of-appeal-second-district`, `ca-court-of-appeal-sixth-district`, `ca-court-of-appeal-third-district`, `ca-criminal-trial-court`, `ca-rules`, `ca-superior`, `ca-superior-court-appellate-division`, `ca-superior-court-san-diego-county`, `ca-supreme-court`, `ca1`, `ca10`, `ca11`, `ca2`, `ca3`, `ca4`, `ca5`, `ca6`, `ca7`, `ca8`, `ca9`, `caaf`, `cacd`, `cacd-criminal`, `cadc`, `caed`, `caed-criminal`, `cafc`, `cand`, `cand-criminal`, `casd`, `casd-criminal`, `cavc`, `cgcca`, `cit`, `cmcr`, `co-criminal-trial-court`, `co-industrial-claim-appeals-office`, `co-rules`, `co-state-appellate`, `co-state-civil`, `co-supreme-court`, `cod`, `cod-criminal`, `cofc`, `core`, `ct-appellate-supreme`, `ct-compensation-review-board`, `ct-criminal-trial-court`, `ct-rules`, `ct-superior-civil`, `ctd`, `ctd-criminal`, `dc-criminal-trial-court`, `dc-rules`, `dc-state-appellate`, `dc-state-civil`, `dcd`, `dcd-criminal`, `de-court-of-chancery`, `de-court-on-the-judiciary`, `de-criminal-trial-court`, `de-family-court`, `de-rules`, `de-superior-civil`, `de-supreme-civil`, `ded`, `ded-criminal`, `examples`, `fisc`, `fiscr`, `fl-criminal-trial-court`, `fl-district-court-of-appeal`, `fl-district-court-of-appeal-fifth`, `fl-district-court-of-appeal-first`, `fl-district-court-of-appeal-fourth`, `fl-district-court-of-appeal-second`, `fl-district-court-of-appeal-sixth`, `fl-district-court-of-appeal-third`, `fl-rules`, `fl-state-civil`, `fl-supreme-court`, `flmd`, `flmd-criminal`, `flnd`, `flnd-criminal`, `flsd`, `flsd-criminal`, `frap`, `frbp`, `frcp`, `frcp-2025`, `frcrimp`, `ga-criminal-trial-court`, `ga-rules`, `ga-state-civil`, `ga-supreme-court`, `gactapp`, `gamd`, `gamd-criminal`, `gand`, `gand-criminal`, `gasd`, `gasd-criminal`, `gu-criminal-trial-court`, `gu-superior`, `gu-supreme`, `gud`, `gud-criminal`, `habeas`, `hi-circuit`, `hi-criminal-trial-court`, `hi-ica`, `hi-rules`, `hi-supreme-court`, `hi-tax-appeal-court`, `hid`, `hid-criminal`, `ia-criminal-trial-court`, `ia-district`, `ia-rules`, `ia-supreme-court`, `iand`, `iand-criminal`, `iasd`, `iasd-criminal`, `id-court-of-appeals`, `id-criminal-trial-court`, `id-district`, `id-rules`, `id-supreme`, `idd`, `idd-criminal`, `il-circuit`, `il-criminal-trial-court`, `il-rules`, `il-supreme-court`, `ilcd`, `ilcd-criminal`, `illappct`, `ilnd`, `ilnd-criminal`, `ilsd`, `ilsd-criminal`, `in-criminal-trial-court`, `in-tax-court`, `in-trial`, `indctapp`, `innd`, `innd-criminal`, `insd`, `insd-criminal`, `iowactapp`, `itc`, `jpml`, `ks-court-of-appeals`, `ks-criminal-trial-court`, `ks-district`, `ks-rules`, `ks-supreme-court`, `ksd`, `ksd-criminal`, `ky-circuit-court`, `ky-court-of-appeals`, `ky-criminal-trial-court`, `ky-rules`, `ky-supreme-court`, `kyed`, `kyed-criminal`, `kywd`, `kywd-criminal`, `la-appellate`, `la-criminal-trial-court`, `la-district-court`, `la-rules`, `la-supreme`, `laed`, `laed-criminal`, `lamd`, `lamd-criminal`, `lawd`, `lawd-criminal`, `links`, `ma-appeals-court`, `ma-appellate-tax-board`, `ma-criminal-trial-court`, `ma-department-of-industrial-accidents`, `ma-district-court-appellate-division`, `ma-rules`, `ma-sjc`, `ma-superior-civil`, `mad`, `mad-criminal`, `md-appellate-civil`, `md-circuit-civil`, `md-criminal-trial-court`, `md-rules`, `md-supreme-court`, `mdd`, `mdd-criminal`, `me-criminal-trial-court`, `me-law-court`, `me-superior-civil`, `med`, `med-criminal`, `mi-appellate`, `mi-circuit-court`, `mi-criminal-trial-court`, `mi-rules`, `mi-supreme-court`, `mied`, `mied-criminal`, `minnctapp`, `miwd`, `miwd-criminal`, `mn-criminal-trial-court`, `mn-district`, `mn-rules`, `mn-supreme-court`, `mnd`, `mnd-criminal`, `mo-circuit`, `mo-criminal-trial-court`, `mo-rules`, `mo-supreme`, `moctapp`, `moctapp-ed`, `moctapp-sd`, `moctapp-wd`, `moed`, `moed-criminal`, `mowd`, `mowd-criminal`, `mp-criminal-trial-court`, `mp-supreme`, `mp-trial`, `mpep`, `ms-appellate`, `ms-chancery-courts`, `ms-circuit-court`, `ms-criminal-trial-court`, `ms-rules`, `ms-supreme-court`, `msnd`, `msnd-criminal`, `mssd`, `mssd-criminal`, `mt-criminal-trial-court`, `mt-district`, `mt-rules`, `mt-supreme`, `mt-tax-appeal-board`, `mtd`, `mtd-criminal`, `nc-appellate-civil`, `nc-criminal-trial-court`, `nc-industrial-commission`, `nc-rules`, `nc-superior-civil`, `nced`, `nced-criminal`, `ncmd`, `ncmd-criminal`, `ncwd`, `ncwd-criminal`, `nd-criminal-trial-court`, `nd-district`, `nd-rules`, `ndctapp`, `ndd`, `ndd-criminal`, `ne-criminal-trial-court`, `ne-district`, `ne-rules`, `ne-supreme-court`, `nebctapp`, `ned`, `ned-criminal`, `nh-criminal-trial-court`, `nh-rules`, `nh-superior-civil`, `nh-supreme`, `nhd`, `nhd-criminal`, `nj-appellate-civil`, `nj-criminal-trial-court`, `nj-rules`, `nj-superior-law`, `nj-supreme-court`, `nj-tax-court`, `njd`, `njd-criminal`, `nm-criminal-trial-court`, `nm-rules`, `nm-state-appellate`, `nm-state-civil`, `nm-supreme-court`, `nmcca`, `nmd`, `nmd-criminal`, `nmid`, `nmid-criminal`, `nv-court-of-appeals`, `nv-criminal-trial-court`, `nv-district`, `nv-rules`, `nv-supreme`, `nvd`, `nvd-criminal`, `ny-appellate-division`, `ny-appellate-terms`, `ny-commission-of-appeals`, `ny-court-for-the-trial-of-impeachments`, `ny-court-of-appeals`, `ny-criminal-trial-court`, `ny-rules`, `ny-saratoga-chancery-court`, `ny-supreme-civil`, `nyed`, `nyed-criminal`, `nynd`, `nynd-criminal`, `nysd`, `nysd-criminal`, `nywd`, `nywd-criminal`, `oh-common-pleas`, `oh-court-of-appeals`, `oh-court-of-claims`, `oh-criminal-trial-court`, `oh-rules`, `oh-supreme-court`, `ohnd`, `ohnd-criminal`, `ohsd`, `ohsd-criminal`, `ok-court-of-civil-appeals`, `ok-court-of-criminal-appeals`, `ok-court-on-the-judiciary`, `ok-criminal-trial-court`, `ok-district`, `ok-judicial-ethics-advisory-panel`, `ok-rules`, `ok-supreme-court`, `oked`, `oked-criminal`, `oknd`, `oknd-criminal`, `okwd`, `okwd-criminal`, `or-circuit`, `or-court-of-appeals`, `or-criminal-trial-court`, `or-dor`, `or-rules`, `or-tax-court`, `ord-criminal`, `pa-commonwealth-court`, `pa-court-of-judicial-discipline`, `pa-cp-civil`, `pa-criminal-trial-court`, `pa-rules`, `pa-superior-court`, `pa-supreme-court`, `paed`, `paed-criminal`, `pamd`, `pamd-criminal`, `pawd`, `pawd-criminal`, `pr-criminal-trial-court`, `pr-primera-instancia-civil`, `pr-rules`, `pr-supreme-court`, `pr-tribunal-apelaciones`, `prd`, `prd-criminal`, `ptab`, `ri-criminal-trial-court`, `ri-rules`, `ri-superior-civil`, `ri-supreme`, `rid`, `rid-criminal`, `sc-circuit-civil`, `sc-court-of-appeals`, `sc-court-of-chancery`, `sc-criminal-trial-court`, `sc-rules`, `sc-supreme-court`, `scd`, `scd-criminal`, `scotus`, `sd`, `sd-circuit`, `sd-criminal-trial-court`, `sdd`, `sdd-criminal`, `tax-court`, `tn-appellate`, `tn-court-of-criminal-appeals`, `tn-criminal-trial-court`, `tn-rules`, `tn-supreme-court`, `tn-trial-court`, `tned`, `tned-criminal`, `tnmd`, `tnmd-criminal`, `tnwd`, `tnwd-criminal`, `ttab`, `tx-court-of-civil-appeals`, `tx-court-of-criminal-appeals`, `tx-court-of-review-tribunal`, `tx-courts-of-appeals`, `tx-criminal-trial-court`, `tx-district-court`, `tx-district-court-travis`, `tx-judicial-panel-on-multidistrict-litigation`, `tx-rules`, `tx-special-court-of-review`, `tx-supreme-court`, `txed`, `txed-criminal`, `txnd`, `txnd-criminal`, `txsd`, `txsd-criminal`, `txwd`, `txwd-criminal`, `ut-criminal-trial-court`, `ut-rules`, `ut-state-appellate`, `ut-state-civil`, `ut-supreme-court`, `utd`, `utd-criminal`, `va-circuit-civil`, `va-court-of-appeals`, `va-criminal-trial-court`, `va-rules`, `va-supreme-court`, `vaed`, `vaed-criminal`, `vawd`, `vawd-criminal`, `vi-criminal-trial-court`, `vi-superior-civil`, `vi-supreme-civil`, `vid`, `vid-criminal`, `vt-criminal-trial-court`, `vt-superior-civil`, `vt-supreme`, `vtd`, `vtd-criminal`, `wa-court-of-appeals`, `wa-criminal-trial-court`, `wa-rules`, `wa-superior`, `wa-supreme-court`, `waed`, `waed-criminal`, `wawd`, `wawd-criminal`, `wi-circuit`, `wi-court-of-appeals`, `wi-criminal-trial-court`, `wi-rules`, `wi-supreme-court`, `wied`, `wied-criminal`, `wiwd`, `wiwd-criminal`, `wv-appellate-civil`, `wv-circuit-civil`, `wv-criminal-trial-court`, `wv-rules`, `wv-supreme-court`, `wvnd`, `wvnd-criminal`, `wvsd`, `wvsd-criminal`, `wy-criminal-trial-court`, `wy-district`, `wy-rules`, `wy-supreme`, `wyd`, `wyd-criminal`

</details>


## First matter in ten minutes

From the repository root, install the CLI and run the example matter. Rust/Cargo must be installed; Cargo's bin directory must be on `PATH`. These commands use the Court of Federal Claims example and its linked Federal Circuit procedure. Copy the commands and outputs below; the output is from this source snapshot.

```sh
cargo install --path crates/litigate-cli
litigate solve cases/cofc-1498-patent-case.case
litigate explain cases/cofc-1498-patent-case.case --arg node=cofc::sj-ruling
litigate run cases/cofc-1498-patent-case.case --op '{"op":"simulate","runs":20}'
```

The install completed with:

```text
Installed package `litigate-cli v0.6.0 (/work/repo/crates/litigate-cli)` (executable `litigate`)
```

The CLI prints a compile envelope with `diagnostics` and nests the engine response under `result`. The blocks below are excerpts from those real responses; diagnostic detail and other fields are omitted for readability.

### `solve`

```text
{
  "ok": true,
  "api_version": 1,
  "op": "solve",
  "result": {
    "start": {"id": "cofc::claim-accrues", "label": "Claim Against the United States Accrues"},
    "value": 48423.073363,
    "converged": true,
    "best_line": [
      {"id": "cofc::e-accrual-to-patentfork", "p": 1.0, "q": 48423.073363},
      {"id": "cofc::e-patentfork-nonpatent", "p": 1.0, "q": 48423.073363},
      {"id": "cofc::e-limitations-timely", "p": 0.95, "q": 50971.656172},
      {"id": "cofc::e-1500-clear", "p": 0.95, "q": 53654.374917}
    ]
  },
  "warnings": [
    {"code": "fact-unset", "count": 2},
    {"code": "mixed-node", "count": 5},
    {"code": "probability-fill", "count": 27}
  ]
}
```

### `explain`

```text
{
  "ok": true,
  "api_version": 1,
  "op": "explain",
  "result": {
    "node": {"id": "cofc::sj-ruling", "label": "Court Rules on Summary Judgment"},
    "control": "chance",
    "value": 475000.0,
    "options": [
      {"label": "Granted in full for plaintiff (liability and damages both resolved)", "p": 0.15, "q": 1000000.0},
      {"label": "Motion denied; case proceeds to trial", "p": 0.5, "q": 500000.0},
      {"label": "Granted in part; remaining issues proceed to trial", "p": 0.15, "q": 500000.0},
      {"label": "Granted in full for the government", "p": 0.2, "q": 0.0}
    ]
  },
  "warnings": [
    {"code": "fact-unset", "count": 2},
    {"code": "mixed-node", "count": 5},
    {"code": "probability-fill", "count": 27}
  ]
}
```

### `simulate`

```text
{
  "ok": true,
  "api_version": 1,
  "op": "simulate",
  "result": {
    "runs": 20,
    "seed": 7,
    "net": {"mean": 72615.5, "std": 399541.0307243175, "min": -407955.0,
      "p05": -407955.0, "p25": -407955.0, "p50": 0.0, "p75": 501045.0,
      "p95": 592045.0, "max": 592045.0},
    "cvar": -407955.0,
    "p_loss": 0.5
  },
  "warnings": [
    {"code": "fact-unset", "count": 2},
    {"code": "mixed-node", "count": 5},
    {"code": "probability-fill", "count": 27}
  ]
}
```

Read `warnings` and `provenance` before interpreting any number. In these runs, `fact-unset` (2) means two matter-known facts used their authored priors; `mixed-node` (5) reports act-or-wait fallback modeling; and `probability-fill` (27) means missing probabilities were filled by the selected mode. Other responses can report `payoff-not-authored`, `sink`, or `authored-hole`; those also signal fallbacks or explicit unknowns. The summary judgment rates are marked in the rule source as teaching estimates. These are illustrative model results, not case predictions.

## Known limitations

The generated strict-profile snapshot [`docs/qa/strict-baseline.tsv`](../qa/strict-baseline.tsv), last changed 2026-11-01, records 635 packages; 101 have one or more in-scope strict errors. These packages have accepted strict-profile gaps, including unresolved source/cite holes and authored-completeness issues. Examples in the repository include unverified or undated sources, placeholder payoffs, and unsourced probabilities; the baseline count is not a count of legal holes, and does not mean every listed package has every example. The affected package IDs are:

`acca`, `afcca`, `ar-criminal-trial-court`, `ar-workers-compensation-commission`, `bankr-als`, `bankr-are`, `bankr-arw`, `bankr-gas`, `bankr-ias`, `bankr-nyw`, `bankr-oke`, `bankr-pr`, `bankr-ri`, `ca-criminal-trial-court`, `ca-superior-court-appellate-division`, `cand`, `cgcca`, `co-criminal-trial-court`, `co-industrial-claim-appeals-office`, `ct-compensation-review-board`, `ct-criminal-trial-court`, `dc-criminal-trial-court`, `dc-state-appellate`, `de-court-of-chancery`, `de-court-on-the-judiciary`, `de-criminal-trial-court`, `de-family-court`, `fiscr`, `ga-criminal-trial-court`, `hi-tax-appeal-court`, `id-criminal-trial-court`, `il-criminal-trial-court`, `in-criminal-trial-court`, `in-tax-court`, `ky-circuit-court`, `ky-court-of-appeals`, `ky-criminal-trial-court`, `ky-rules`, `ky-supreme-court`, `la-supreme`, `ma-appellate-tax-board`, `ma-criminal-trial-court`, `ma-department-of-industrial-accidents`, `ma-district-court-appellate-division`, `ma-superior-civil`, `me-criminal-trial-court`, `mo-circuit`, `mo-criminal-trial-court`, `mo-supreme`, `moctapp`, `moctapp-ed`, `moctapp-sd`, `moctapp-wd`, `mp-criminal-trial-court`, `mp-trial`, `ms-criminal-trial-court`, `mt-criminal-trial-court`, `mt-tax-appeal-board`, `nc-criminal-trial-court`, `nc-industrial-commission`, `nd-criminal-trial-court`, `ne-criminal-trial-court`, `ne-supreme-court`, `nh-criminal-trial-court`, `nj-criminal-trial-court`, `nj-tax-court`, `nm-criminal-trial-court`, `nmcca`, `nv-criminal-trial-court`, `ny-appellate-terms`, `ny-commission-of-appeals`, `ny-court-for-the-trial-of-impeachments`, `ny-criminal-trial-court`, `ny-saratoga-chancery-court`, `oh-court-of-claims`, `oh-criminal-trial-court`, `ok-court-of-civil-appeals`, `ok-court-on-the-judiciary`, `ok-criminal-trial-court`, `ok-district`, `ok-judicial-ethics-advisory-panel`, `ok-supreme-court`, `or-tax-court`, `pa-court-of-judicial-discipline`, `pa-criminal-trial-court`, `pr-criminal-trial-court`, `pr-supreme-court`, `ri-criminal-trial-court`, `sc-court-of-chancery`, `sc-criminal-trial-court`, `sd-criminal-trial-court`, `tx-court-of-civil-appeals`, `tx-court-of-review-tribunal`, `tx-judicial-panel-on-multidistrict-litigation`, `tx-special-court-of-review`, `ut-criminal-trial-court`, `va-criminal-trial-court`, `vi-criminal-trial-court`, `vt-criminal-trial-court`, `wawd-criminal`, `wv-criminal-trial-court`

Other known limitations are documented in the repository: there is no ongoing law tracker or automated good-law check; cite verification checks a cite against vendored source text but does not establish current precedential validity. State-holiday coverage is incomplete: where a state holiday statute is not vendored (or fetching it is blocked), federal district and bankruptcy clocks count federal holidays only, so a state holiday may not roll a due date. See [README's deferred state sources](../../README.md#deferred-state-sources) and [`sources/PROVENANCE.md`](../../sources/PROVENANCE.md).

## Reporting a problem

File an issue on [gitlab.com/davidawad/lit-lang](https://gitlab.com/davidawad/lit-lang) with `litigate --version` and the full JSON envelope (including warnings and provenance), plus the command and the relevant package or matter file. Do not include confidential client or case information. This repository snapshot does not contain a feedback issue template, so use that fallback format.
