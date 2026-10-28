# Jurisdiction outlines (unverified)

Skeleton litigate packages for every federal and state court in formbook's
`courts/` tree. Each is an OUTLINE: the structure, cites and key decisions,
with every cite marked unverified (`cite ?("...")`) and every value a hole
(`?`), until a later pass verifies it against vendored primary text and
promotes the package into `lit/`.

## Layout

The layout mirrors formbook's `courts/<circuit>/<jurisdiction>/`.

    outlines/<circuit>/court-of-appeals/                 federal court of appeals (FRAP + circuit rules/IOPs)
    outlines/<circuit>/<state>/federal-<district>/       each U.S. district court in the state (FRCP + local rules)
    outlines/<circuit>/<state>/bankruptcy-<district>/    each U.S. bankruptcy court (uses lit/frbp; local bankruptcy rules only)
    outlines/<circuit>/bankruptcy-appellate-panel/       each circuit's BAP (1st, 6th, 9th, 10th; the 8th is lit/bap8)
    outlines/<circuit>/<state>/state-rules/              the state's base library: the rule layer its court packages share
    outlines/<circuit>/<state>/state-<court>/            one package per formbook state court, <court> its formbook path
                                                         under state/ ('/' as '-'): supreme-court, court-of-appeals and each
                                                         district of it, court-of-criminal-appeals, tax-court, court-of-chancery,
                                                         workers' compensation and tax boards, judicial-discipline courts, ...
    outlines/<circuit>/<state>/state-trial-court/        the trial court of general jurisdiction, civil, where formbook lists none
    outlines/<circuit>/<state>/state-criminal/           state trial courts of general criminal jurisdiction (every state, DC and territory)
    outlines/<circuit>/<state>/federal-<district>-criminal/  each U.S. district court's local criminal rules (uses lit/frcrimp)
    outlines/federal-circuit/<court>/, outlines/national/<court>/, outlines/military/<court>/,
    outlines/supreme-court-of-the-united-states/          one package per court

Each directory holds `Lit.toml` plus `outline.lit`. Copy
`outlines/third-circuit/delaware/state-trial-court/` as the template.

A state court package (`state-<court>`) depends on its state's base library
(`state-rules`, or `lit/<st>-rules` once the state has promoted packages),
imports the trigger events it declares (`ActionCommenced`, `JudgmentEntered`,
`NoticeOfAppealFiled`, `RecordFiled`, `DecisionFiled`, `ReviewPetitionFiled`,
`MandateIssued`) and binds its edges to them, so rule text the state's courts
share is declared once, in the base. It carries a `Lit.lock` (`litigate lock
DIR`). tests/fixtures/formbook-forums.json maps each formbook state court to
its package (lg-u561.5).

A bankruptcy-court outline differs: it depends on `lit/frbp` (and `lit/core`),
imports the FRBP periods from it with their cites bound to the vendored FRBP
text, and declares only the court's local bankruptcy rule layer as `?(...)`
cites and holes. It carries a `Lit.lock` (`litigate lock DIR`). Copy
`outlines/third-circuit/new-jersey/bankruptcy-district/` as the template.

A federal criminal outline (`federal-<district>-criminal`) differs the same
way: it depends on `lit/frcrimp`, imports the FRCrimP, Speedy Trial Act and
FRAP 4(b) periods from `frcrimp.deadlines`, and declares only the district's
local criminal rules and Speedy Trial Act plan (18 U.S.C. § 3165) as `?(...)`
cites and holes. It carries a `Lit.lock`. Copy
`outlines/eighth-circuit/arkansas/federal-eastern-district-criminal/` as the
template; `lit/njd-criminal` shows the promoted form.

## Rules for every outline

1. Public law only. Never use client material.
2. Never invent a number. Every period, fee, page or word limit is `?`, unless
   you fetched the official rule text in this run. If you did, still mark the
   cite `?("...")` and put the source URL and the retrieval date in a `note`.
3. Cites use the court's official citation form: the rule number for state and
   local rules, and U.S.C./C.F.R. sections. All of them are `?("...")`.
4. Cover the litigation's main path: commencement, service, response/answer
   (Rule 12-equivalent motions), default, discovery, dispositive motions,
   trial, post-trial motions, judgment, and appeal/notice of appeal. Appellate
   outlines cover notice, record, briefing, argument, decision, rehearing,
   mandate and further review.
5. Key decisions: name a few leading cases a practitioner in that court would
   cite for each stage (pleading standard, summary judgment, appealability,
   timeliness), in `note` text or `cite ?("...")`. They are unverified, so
   mark each "verify".
6. Every package has at least two terminals, with `payoff ?`, and every
   non-terminal node has an outgoing edge. Each must pass `litigate check DIR`
   with no errors. L9002 hole warnings are expected and fine.
7. Lit.toml: unique `name` and `pack_id`, `outline-<state>-<kind>` or
   `outline-<court>`; an `authority` of `FederalRule`, `StateRule` or
   `LocalRule`; `in_force`, `known_at` and `unknown_before` set to the date of
   the edition named, or "2025-01-01" if unknown; and `[package.roles]` as in
   the template.
