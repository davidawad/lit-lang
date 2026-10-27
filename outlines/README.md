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
    outlines/<circuit>/<state>/state-civil/              state trial court of general jurisdiction, civil
    outlines/<circuit>/<state>/state-appellate/          state intermediate appellate and supreme court, civil appeals
    outlines/federal-circuit/<court>/, outlines/national/<court>/, outlines/military/<court>/,
    outlines/supreme-court-of-the-united-states/          one package per court

Each directory holds `Lit.toml` plus `outline.lit`. Copy
`outlines/third-circuit/new-jersey/state-civil/` as the template.

A bankruptcy-court outline differs: it depends on `lit/frbp` (and `lit/core`),
imports the FRBP periods from it with their cites bound to the vendored FRBP
text, and declares only the court's local bankruptcy rule layer as `?(...)`
cites and holes. It carries a `Lit.lock` (`litigate lock DIR`). Copy
`outlines/third-circuit/new-jersey/bankruptcy-district/` as the template.

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
