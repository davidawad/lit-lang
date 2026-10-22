# Legal process/workflow DSLs: prior art and feasibility for "litigate"

Researched 2026-10-22. Supplements litigate-prior-art.md (not repeated: Catala, L4, LegalRuleML, Docassemble, CompuLaw, OpenFisca, Symboleo, Stipula, Rotolo-Smith, Meldman, Bend). Every URL in Sources returned HTTP 200 via curl. (K) = background knowledge, not fetched this session. (S) = search-result snippet or abstract only; I did not read the full paper. Search tooling was weak: several targeted queries (RaC court procedure, GitHub FRCP engines, ICAIL/JURIX procedure papers) returned nothing, so absence claims are low-to-moderate confidence.

## Comparison table

Axes: Proc = models a sequence of procedural steps; DL = deadline/time computation; Ver = effective-dated rule versions; Prec = precedence/defaults; Disc = discretion ("court may"); Art = emits runnable artifacts.

| System | Category | Proc | DL | Ver | Prec | Disc | Art | Status | Lesson for litigate |
|---|---|---|---|---|---|---|---|---|---|
| OASIS ECF 4.01 / 5.0 | E-filing message standard (NIEM-conformant in 5.0) | Message exchange (assemble, review, serve), not case procedure | Court schedule/calendar via WS-Calendar in 5.0; no deadline rules | N | N | N | XML schemas, web service profiles | 5.0 Committee Spec 2019; Tyler co-editor | Court Policy MDE is policy as flags and code lists (accepted case types, sizes, sealed, fees), explicitly NOT procedural logic. Natural output target for a "filing linter", not a source of rules |
| NIEM Justice domain | Data vocabulary | N | N | N | N | N | Schemas | Active (K) | Reuse for entity types (party, case, document); my curl to niem.gov domain failed so only reached via ECF page |
| Tyler Odyssey File & Serve / eFileCA, File & ServeXpress | Commercial e-filing + clerk review | Submit, clerk review, accept/reject | Not documented | ? | ? | Clerk judgement | Closed | Mature | Public docs show only the review step with published rejection-reason lists per court (e.g. Vermont). No documented rules engine. The rejection-code lists are free labeled data for a docket/linter corpus |
| Accord Project: Concerto (models), Cicero (templates), Ergo (logic) | Smart legal contract stack | Contract state machine in Ergo (K) | Duration/time types (K) | N | N | N | JS/templates | Ergo repo ARCHIVED Sep 7 2023 (read-only; "early development"); Cicero repo still public | Concerto-style typed model language is the closest design analog for `.lit` types. Ergo (own language, Coq/OCaml compiler) was archived: custom-language cost is real |
| BPMN 2.0 / DMN / CMMN (OMG) | General process/decision/case standards | BPMN yes; CMMN for event-driven case plans | Timer events; DMN FEEL has date arithmetic (K) | N | DMN hit policies (K) | Human tasks | Engines (Camunda etc.) | Mature | Judicial BPMN models exist (Italian 2024, Serbian civil procedure) but are diagrams for specific national procedures, not rule-parameterized. Authors report difficulty showing court pathways and time limits (S) |
| Temporal | Durable-execution workflow engine | Code-as-workflow, signals, timers, versioning APIs | Durable timers | Workflow versioning for in-flight runs (K) | N | Signals for human decisions | Runtime | Very active | Right runtime model for LIVE matters (long-running case with timers and external events, replay-safe). Wrong for compile-time analysis |
| YAWL, workflow patterns (van der Aalst) | Petri-net-based workflow language | Yes, formal | Timers (K) | N | N | N | Engine | Academic/mature | Soundness checking (no deadlock, always terminates) on compiled graph. Litigation graphs with appeals/remands are cyclic, so use relaxed soundness |
| SCXML (W3C) / statecharts | State machine standard | Hierarchical, parallel regions | Delayed events | N | N | N | Interpreters | Stable | Hierarchical states fit "case contains motion contains response window" nesting |
| Session types (Honda-Vasconcelos-Kubo ESOP 1998; ETAPS 2019 Test of Time) | Types for protocols between roles | Yes, per-party projections | N | N | N | Choice = branch | Type-checkers | Mature theory | Multiparty session types give: global procedure, projected per-party "legal moves", checked for deadlock freedom. Fits `.case` -> agent action set (per-party endpoint type). Gotcha: published systems had soundness bugs that took years to fix (S) |
| Gordon Pleadings Game; Prakken adjudication dialogues | AI & Law dialogue-game models of procedure | Yes (turn-taking, burden of proof) | N | N | Defeasible logic | Partly (burden/standard) | Prototype | Academic, 1990s-2008 | Prakken's model was tested against a real Dutch civil case (Springer 2008, S). Best prior art for modeling who may move when, but was never deployed |
| Governatori/Rotolo/Palmirani/Sartor temporal defeasible logic | Formal logic for norm change | N | N | YES: persistence, retroactivity, abrogation, two timelines (version-internal vs system evolution) | YES | N | Implemented reasoner (Rubino et al. 2010) (S) | Academic | Directly the theory for effective-dated rule versions. Key distinction litigate needs: when a rule was in force vs when it applies to a matter that straddles an amendment |
| Dignum-Kuiper; Broersen et al., deadlines in deontic+temporal logic | Logic | Obligation with deadline | Dense/continuous time (S) | N | N | N | N | 1990s-2000s | Theory of deadline obligations; not a calendar engine |
| "Time as Structure" (AACL-IJCNLP 2026 SRW, arXiv 2608.15270) | LLM + temporal dependency graph + calendar engine | N | YES | partly | N | N | Calendar engine | Aug 2026 | Hybrid evidence: engine matched judges' dates in 6 of 7 UK EAT timeliness rulings. On contracts, errors were mostly picking the wrong triggering event, not arithmetic. Same failure point as docket matching |
| Digital.NSW Rules as Code | Government RaC program | N | N | N | N | Deliberately excluded judgement rules | Rules engine | 2019+ | Scope statement: prescriptive rules first, judgement later. Same boundary litigate faces for "court may" |
| FRCP/courtrules calculators (terms.law, courtrules.net, deadlinecalculator.org, LawToolBox) | Web calculators, docketing | Trigger -> date | YES, Rule 6 mechanics | Operationally | ? | N | Dates | Commercial/marketing | No open rule-representation found |
| Stanford CodeX / Genesereth computational law | Research center | ? | ? | ? | ? | ? | ? | Active | Could not find any CodeX project modeling court procedure (SEARCH: only courses, prize to Kowalski-Sadri-Sergot 2021). Treat as none found |
| Logical English (Kowalski) | Controlled English over Prolog | N | N | N | Prolog | N | Prolog | Active | Surface syntax inspiration only; no procedure work found |

Bottom line of table: no existing system is a typed, versioned, precedence-aware DSL for forum procedure that compiles to a party-indexed graph. The pieces exist in separate fields: ECF/NIEM (data and filing messages), temporal defeasible logic (versions and precedence), session types/YAWL (protocol graphs), Temporal (live execution), docketing vendors (rule data), Prakken-style dialogue games (who may move).

## Per-item notes (new material only)

### ECF 4.01 / 5.0 and NIEM
- ECF 5.0 splits into message-design elements: Court Policy, Filing Assembly, Filing Review, Court Record, and others. Court Policy returns run-time policy (code lists, court public key, court schedule with operating days and judge schedules) and development-time policy (e-filing rules unlikely to change, such as signature and service profiles).
- Policy is flags: accepts URLs in lieu of documents, accepts documents requiring fees, sealed documents, multiple lead documents, max attachment sizes. Court-specific extensions via SchemaExtension and CodeListExtension. Code lists: CaseTypeCode, CaseCategoryCode, DocumentTypeCode, CourtEventTypeCode, FeeExceptionReasonCode. Fee calculation is an operation (GetFeesCalculation), not a schedule. I saw no deadline fields (I read only the first half of the 204k-char spec).
- Implication: ECF gives the vocabulary for `.lit` submission types and a place to export "what this forum accepts". It confirms the industry never standardized procedural rules as data. The gap litigate targets is real, and also hints at why: courts only standardize what a clerk can mechanically check.

### E-filing rule engines (Tyler, File & ServeXpress)
- Nothing public documents a rules engine. What is public is the observable behavior: filer submits via a service provider, clerk reviews, accept or reject with a reason (Vermont publishes its rejection reasons; California's Tyler eFileCA deck lists wrong court, mismatched case number, illegible scan). Review is human. A "filing linter" would automate the pre-submission half of what clerks check by hand.

### Accord Project
- Ergo (own language, compiler in Coq/OCaml via Q*cert) repo was archived 2023-09-07. Cicero (template runtime) and Concerto (data model language) remain. Pattern: the typed model layer survived, the bespoke logic language did not. Argument for making `.lit` a thin typed layer over an existing logic/expression core rather than a full new programming language.

### Rules-as-code experience with discretion
- NSW Digital: scope excluded judgement-based rules initially. A drafter's critique (Law School Policy Review, 2025) argues "may" and "reasonable" mark discretion and cannot simply be coded; coded rules risk rigidity where unforeseen cases need fairness.

### Temporal logic of norm change
- Governatori et al. (S): separate persistence of a conclusion after a rule modification from blocking; retroactive modifications need two timelines. Maps to litigate: an effective-dated `.lit` needs (a) the rule text version, (b) the date the rule applies to a matter. FRCP amendments say "apply to all proceedings commenced after, and insofar as just and practicable to pending proceedings" (K on the usual transition language), which is itself a discretionary clause.

## Feasibility

### Hardest technical problems (ranked)
1. Discretion ("court may", "for good cause", "unless the court orders otherwise"). In the current FRCP text: 154 occurrences of "court may", 29 of "good cause", 56 of "on motion"/"for good cause", 48 of "unless the court orders ...", "unless otherwise stipulated", or "local rule" (my grep of the Dec 2025 uscourts PDF; phrase matches, not parsed semantics). Do not encode as logic. Model as typed choice nodes: `discretion<Ruling>` with a declared input set (what the judge may consider), an outcome set, and a prior hook (the litgraph probability). The language should record that a branch is discretionary and who decides, never pretend to evaluate it. This is the same line NSW RaC drew.
2. Override layering (statute > FRCP > local rule > judge standing order > case-specific order, plus stipulations). Default-with-exception handles the shape (Catala-style). Hard part: FRCP often delegates ("unless the court orders otherwise", Rule 29 stipulations, Rule 83 local rules must not conflict) so the override direction is itself conditional. Needs a typed `delegation` construct (which source may override which parameter, within what bounds), not just numeric priority. Conflicts between sources are inputs the compiler should flag (diagnostics), not silently resolve; real conflicts need human resolution, and courts themselves disagree.
3. Time and versioning. Two timelines (see above): rule version in force at trigger date vs at deadline date vs at today. Amendments take effect Dec 1 each year (K); matters filed earlier follow transition language. Standing orders change mid-case. Need `as_of` plus `trigger_date` plus case-specific orders as dated overlays. Complexity is in matter-straddling-amendment cases, a small fraction of matters but the ones that cause malpractice.
4. Deadline computation (Rule 6 and analogs). Core algorithm is simple; edge cases are the product: (a) day-count vs hour-count vs "next-day" rule; (b) backward-counted deadlines (e.g. "at least 14 days before the hearing") where weekend/holiday rolls to the PRECEDING day, not the following; (c) "last day" definition incl. clerk-office-inaccessible days, 11:59pm vs close of business by local rule; (d) Rule 6(d) +3 days for mail/clerk/other means, but NOT electronic service since 2016 amendments (supported by search snippets); (e) which holidays (state holidays for state courts, court-declared closures, "legal holiday" lists change each year, Juneteenth added 2021 (K)); (f) statutory vs rule-based periods (Rule 6 governs statutes silent on counting); (g) jurisdictional periods that cannot be extended (Fed. R. App. P. 4(a), Rule 6(b)(2) bars extending Rules 50(b), 52(b), 59(b), (d), (e), 60(b)) (K); (h) state variants: California counts court days for motions (S); Minnesota/Utah 7-14-21-28 normalization (S); (i) suspended periods (answer clock tolled by Rule 12(a)(4) motion; then 14 days post-order) (S). Good news: deterministic, testable against vendor outputs, and the "Time as Structure" paper reports the arithmetic is rarely the error; the wrong-trigger-event choice is.
5. Trigger identification from dockets. The `.case` compiler is only as good as event classification. Docket text is unstructured and clerk-idiosyncratic. The 2026 deadline paper's finding (errors in extraction/trigger choice, not calculation) is the evidence this is the true bottleneck. Mitigation: deadlines are computed only from typed events; docket-matching proposes events with confidence and a human confirms; every computed date carries a citation chain.
6. Maintenance (not technical but fatal). Rule data decay rate: FRCP amended nearly yearly; 94 federal districts each with local rules and many judges with standing orders (K, counts approximate); states add 50 more rule sets. The vendors (CompuLaw etc.) employ attorneys full-time on this. A language does not solve the data problem; it only makes the data cheaper to verify and diff.
7. Verification. Compile-time checks that are realistic: type-checking submission types, every deadline reachable and has an anchor, no override cycles, termination of the happy path. Unrealistic: proving a forum's rules are "complete" or that the encoding matches the text. Use golden tests per rule (rule citation + worked examples + expected dates), as Catala/L4 do.

### Realistic size of encoding FRCP fully
- Text: the current rules PDF is ~63,000 words including front matter, Supplemental Rules, forms, and amendment history (wc on pdftotext output; 87 numbered rules plus supplemental rules A-G and the social-security supplemental rules). Rough breakdown is my estimate, not measured:
  - Deadline/timing core (Rules 4, 5, 6, 12, 14-16, 26-37, 45, 50, 52, 54, 56, 58-60, 62, 65, 68, 72, 73): about 25 rules, 90 or so explicit day-period phrases (grep: 93). Encodable almost completely in a few hundred lines. Weeks of work including golden tests.
  - Procedure graph (pleading, joinder, discovery, summary judgment, trial, post-trial, appeal handoff): about 40 rules; 1-3 person-months to get a reviewed graph for one forum type, assuming the types and engine exist.
  - Substantive/judgement-heavy rules (Rules 11, 19, 23, 24, 26(b) proportionality, 37 sanctions, 54(d) fees, 65 injunction standard): 20+ rules where encoding is a typed choice node plus text citation. Cheap to represent, valueless to "evaluate".
  - Full fidelity (all rules, exceptions, cross-references, forms) plus a full committee-note layer: 6-12 person-months by one careful person, mostly review. Low-confidence estimate.
- The cost explosion is not FRCP; it is the long tail: local rules and standing orders for the forums you actually practice in. Start with 1-2 forums (e.g. the districts David files in) rather than breadth. Also FRE, FRAP, bankruptcy rules, agency rules (each agency, e.g. ALJ proceedings, differ wildly).
- Per-agency/administrative proceedings are less formalized than court rules and more discretionary, but are easier markets because incumbents (CompuLaw) cover them poorly (K, inference).

### What failed before, and why (with evidence quality)
- Gordon Pleadings Game / Prakken dialogue models: right idea (procedure as formal dialogue), never deployed; modeled "normative first principles" or one reconstructed case, not the corpus of real rules. Lesson: encoding real, messy rules is what nobody funded. (S)
- Ergo (Accord): custom language, archived 2023 (verified on repo page). Likely causes, my inference: small team, contracts are less standardized than hoped, ongoing language maintenance cost.
- BPMN court models: produce diagrams per country, don't parameterize by forum or version; authors note difficulty with time limits. (S)
- L4 predecessor line stalled; l4-ide is the continuation (prior report).
- Commercial docketing vendors "succeeded" by not using a language: attorney-curated rule tables (K/V). The moat is staff, not a compiler.
- Lack of evidence: I found no documented postmortem of any court-procedure DSL. Absence of postmortems itself suggests few serious attempts, or that failures were quiet.
- Rules as code programs broadly succeeded only where rules were prescriptive (tax, benefits). Litigation has the opposite property: strategic actors plus discretionary decisions.

### Risks
1. Scope creep into "evaluating law". Keep the language as procedure + deadlines + document constraints; discretion remains a typed hole.
2. Liability: a wrong deadline = missed filing = malpractice. Needs "advisory, verify against rule text" framing, cited provenance per date, and tests against an independent calculator. Vendors sell accuracy guarantees; you cannot.
3. Data decay (see #6).
4. Language-maintenance cost (Ergo). Mitigate: small core, borrow expression syntax, generate tooling, make JSON/YAML the interchange so the surface syntax is replaceable.
5. Docket NLP error compounding into wrong triggers (the 2026 paper's finding).
6. Court access: dockets via PACER/CourtListener have fees/coverage gaps (K).

### Design recommendations specific to the proposal
- Keep `.lit` small and declarative: types (from ECF/NIEM names where possible), `event`, `deadline(anchor, period, count_rule, roll_rule)`, `move(party, preconditions, effects)`, `override(source -> target, bounds)`, `discretion<T>`, `version(effective_from, effective_to, transition_rule)`. Compile to litgraph JSON; do NOT embed a general-purpose language.
- Treat counting rules (Rule 6 style) as a small closed set of named, tested count modes (forward/backward, court-days, hours, roll-forward/roll-back, service add-ons) rather than user-programmable arithmetic. This is where DMN-FEEL-style generality hurts.
- Use per-party endpoint projection (session-type style) to derive agent action sets; use Temporal-style durable execution only for the runtime that tracks live matters.
- Golden tests with rule citation plus worked examples is the main correctness tool. Compare outputs against an independent calculator in CI.
- Do the minimal viable vertical first: FRCP Rule 6 + 12 + 26/34/36 response clocks + one district's local rules + one judge's standing order, `.case` -> calendar only. Add graph and NLP later.
- Consider whether `.lit` can be a thin typed layer (Concerto-like) over a Catala-style default core instead of a standalone language.

## Gaps
- I did not retrieve the full text of any ICAIL/JURIX procedure paper (ACM DL and MDPI 403; ICAIL 2025 site unreachable). A manual proceedings sweep for "procedure", "deadline", "BPMN", "Petri" 2015-2026 is still owed. I may be missing 2015-2026 papers specifically on procedure.
- No evidence found for CodeX, Logical English, or Catala work on court procedure. Catala date/duration handling from memory only (the book fetch returned empty).
- Tyler/File & ServeXpress internals not documented publicly.
- Counts of FRCP phrases are plain grep on the PDF text; the file includes committee-history prose, so counts are upper bounds.

## Sources (all returned HTTP 200 by curl)
- OASIS ECF committee: https://www.oasis-open.org/committees/legalxml-courtfiling/
- ECF 4.01 spec: https://docs.oasis-open.org/legalxml-courtfiling/specs/ecf/v4.01/ecf-v4.01-spec/ecf-v4.01-spec.html
- ECF 5.0 spec: https://docs.oasis-open.org/legalxml-courtfiling/ecf/v5.0/ecf-v5.0.html
- Accord Project docs: https://docs.accordproject.org
- Cicero: https://github.com/accordproject/cicero
- Concerto: https://github.com/accordproject/concerto
- Ergo (archived 2023-09-07): https://github.com/accordproject/ergo
- BPMN 2.0: https://www.omg.org/spec/BPMN/2.0/
- DMN: https://www.omg.org/spec/DMN/
- CMMN: https://www.omg.org/spec/CMMN/
- Camunda DMN: https://camunda.com/dmn/
- Temporal: https://temporal.io and https://docs.temporal.io and https://github.com/temporalio/temporal
- SCXML: https://www.w3.org/TR/scxml/
- YAWL: https://github.com/yawlfoundation/yawl
- Session types, ETAPS 2019 Test of Time: https://etaps.org/2019/test-of-time-award
- Vasconcelos, sessions overview: https://www.dcs.gla.ac.uk/~simon/BehaviouralTypes/Vasconcelos.html
- Time as Structure (arXiv 2608.15270): https://arxiv.org/abs/2608.15270
- Governatori et al., Norm Modifications in Defeasible Logic: https://cris.unibo.it/handle/11585/12991
- Governatori et al., Back to the Future (TDL variants): https://cadmus.eui.eu/entities/publication/c172f4e9-0e4d-5899-99cf-dc5976c22208/full
- Dignum-Kuiper deadlines (Utrecht): https://dspace.library.uu.nl/handle/1874/20733
- Prakken-style dialogue model on a Dutch case (Springer): https://link.springer.com/article/10.1007/s10506-008-9069-1
- Ontology Framework for Judgment Modelling (Springer chapter): https://link.springer.com/chapter/10.1007/978-3-642-35731-2_8
- LN2FR 2022 workshop proceedings: https://arxiv.org/pdf/2305.12203
- Italian judicial BPMN (Politecnico di Torino record): https://iris.polito.it/handle/11583/2994910
- Serbian civil procedure BPMN/BPEL: https://infom.fon.bg.ac.rs/index.php/infom/article/download/1747/1719
- Rotolo-Smith record: https://cris.unibo.it/handle/11585/845532
- Meldman Petri net bibliography: https://www.informatik.uni-hamburg.de/TGI/pnbib/m/meldman_j_a1.html
- NSW Rules as Code: https://www.digital.nsw.gov.au/article/an-australian-first-nsw-translating-rules-as-code-to-make-compliance-easy
- Critique of coding "may": https://lawschoolpolicyreview.com/2025/02/05/issues-with-the-use-and-coding-of-may-in-commonwealth-legislation-a-legislative-drafters-perspective/
- Genesereth (Stanford CS): https://www.cs.stanford.edu/people/michael-genesereth
- Logical English: https://github.com/LogicalContractsOrg/LogicalEnglish
- Catala: https://catala-lang.org and https://github.com/catalalang/catala
- FRCP (uscourts PDF, Dec 2025): https://www.uscourts.gov/sites/default/files/document/federal-rules-of-civil-procedure.pdf
- FRCP landing: https://www.uscourts.gov/forms-rules/current-rules-practice-procedure
- FRCP Rule 6 (Cornell): https://www.law.cornell.edu/rules/frcp/rule_6
- Rule 6 explainer: https://www.courtrules.net/federal/civil-procedure/rule-6
- Deadline-date pitfalls: https://www.litigationbythenumbers.com/getting-date-right.html
- Terms.law calculator: https://terms.law/Calcs/court-filing-deadline-calculator/
- Court Rules vendor listing: https://www.legaltechnologyhub.com/vendors/court-rules/
- Tyler Enterprise Justice: https://www.tylertech.com/products/enterprise-justice
- File & ServeXpress: https://www.fileandservexpress.com
- Vermont Odyssey File & Serve (rejection reasons): https://www.vtcourts.gov/about-vermont-judiciary/electronic-access/electronic-filing
- Maine eCourts presentation: https://courts.maine.gov/ecourts/ecourts-in-la-presentation.pdf
- Federal e-filing (CM/ECF): https://www.uscourts.gov/court-records/electronic-filing-cmecf
- CourtListener API: https://www.courtlistener.com/help/api/rest/
- CourtListener repo: https://github.com/freelawproject/courtlistener
- Juriscraper: https://github.com/freelawproject/juriscraper
- Eyecite: https://github.com/freelawproject/eyecite
- JURIX: https://jurix.nl
