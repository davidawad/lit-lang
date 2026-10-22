# Prior art for "litigate" (.lit rules -> litigatable; .case -> litgraph + artifacts)

Researched 2026-10-22. Links listed in Sources returned HTTP 200 via curl. Items marked (K) rest on my background knowledge, not a page fetched this session. Items marked (V) are vendor/secondary claims.

## Comparison table

Legend: Y = yes natively, P = partial/by convention, N = no, ? = not verified.

| Tool | What it is | Effective-dated versions | Precedence / defaults | Procedural sequences + deadlines | Multi-party | Stochastic outcomes | Compiles to analyzable graph | Maturity | License |
|---|---|---|---|---|---|---|---|---|---|
| Catala (Inria) | Statute-to-code DSL, prioritized default logic (Lawsky) | P (dates/durations in language; law versions kept as separate source files) (K) | Y, core feature | N (no process/state model; pure functional scopes) | N | N | N (compiles to OCaml, Python, Java, C, R, etc. (K), formally verified core) | Active, 2.4k stars, pushed today; production DGFiP income tax since 2024, CNAF 2027 (self-reported) | Apache-2.0 |
| L4 (SMU CCLaw -> Legalese, l4-ide / jl4) | Typed functional DSL for rules and contracts, decision service | ? | P | P (contract-style deontic/temporal in earlier versions) (K) | P | N | Ladder diagrams, eval traces, WASM; not a game graph | Active (l4-ide 37 stars; smucclaw/dsl archived-ish, last push 2025-06) | Apache-2.0 (l4-ide) |
| Akoma Ntoso / LegalDocML | OASIS XML for legislative/judicial documents | Y for document versions (point-in-time, FRBR) (K) | N (markup only) | N | N | N | N | Standard, stable | OASIS (open) |
| LegalRuleML | OASIS XML interchange for rules | Y (TemporalCharacteristic: enforceability, efficacy, applicability) | Y (strict/defeasible/defeater, Override @over/@under) | P (deontic: obligation, permission, prohibition, violation, reparation) | P | N | N (no execution semantics defined) | Standard, little tooling | OASIS |
| Docassemble | Python/YAML guided-interview and document assembly | N | N | N (interview flow, not court procedure) | N | N | N | Active, 992 stars, pushed 2026-10-20 | MIT |
| CompuLaw / Aderant (Milana) | Commercial rules-based docketing, 2.5-3k+ jurisdictions | Y operationally (staff attorneys update rules) (V) | Y operationally (local rules, judge orders) (V) | Y: trigger event -> deadlines, holiday roll, service add-ons | P (one calendar per party) | N | N (calendar output only) | Mature, closed | Proprietary |
| LawToolBox, CourtAlert | Same category (Microsoft-integrated calendaring; CourtAlert) | ? | ? | Y | ? | N | N | Mature, closed | Proprietary |
| OpenFisca | Python microsimulation of tax/benefit law | Y, best-in-class (parameters and formulas dated) | P | N | P (entities, roles) | N (deterministic; population simulation) | N | Active, 247 stars, pushed 2026-10-20 | AGPL-3.0 |
| Blawx | Web IDE over s(CASP) for rules-as-code | N | Y (defeasible) | N | N | N | N | Low activity (last push 2024-11) | MIT |
| Logical English (Kowalski) | Controlled English syntax for Prolog/s(CASP) | P | P | N | N | N | N | Active (pushed 2026-05), 52 stars | Apache-2.0 |
| Symboleo (Ottawa) | Contract spec language, statechart semantics, model checker | N | N | Y (obligation/power lifecycles, deadlines) | Y (contract parties) | N | Statecharts + temporal-logic model checking | Academic; no public repo found | n/a |
| Stipula (Padua/Bologna) | Contract DSL, legal calculus | N | N | P (state/transition obligations) | Y | N | N | Academic prototype (Java, 4 stars) | none stated |
| Lexon | English-like contract language -> Solidity | N | N | N | P | N | N | Unclear | ? |
| Rotolo and Smith, ICAIL 2021, "Modelling legal procedures" | PDL variant + preference operator for court procedure | N | P (preference) | Y (procedure as program) | P | N | Logic, not graph engine | Paper only (full text restricted) | n/a |
| Meldman 1978, Petri net of civil procedure | Earliest procedure-as-graph | N | N | Y | P | N | Y (Petri net) | Historical | n/a |
| Priest-Klein and successors | Game-theoretic settlement/litigation models | N | N | N (reduced form) | Y (two-sided) | Y | Analytic, not procedural | Canonical literature | n/a |
| Bend (HigherOrderCO) | Parallel functional language, now Bend 2 with proofs | n/a | n/a | n/a | n/a | n/a | n/a | See below | Apache-2.0 |
| litgraph (yours) | Procedure graph + stochastic game solver | per forum packs | per-expression | Y | Y | Y | Y | yours | yours |

## Per-item notes

### 1. Catala
- Semantics: prioritized default logic. Each rule is a default with exceptions that override it; conflicts are resolved by explicit priority. Literate: code interleaved with statute text.
- Compiles to a scope-language intermediate, then to backends (OCaml, Python, etc. (K)); can emit lawyer-readable PDF. Compiler core partly formally verified in F*.
- Deployments (self-reported by lead Denis Merigoux): DGFiP income-tax contribution in production since 2024; CNAF family benefits greenlit for 2027; ASP (CAP subsidies) in study in 2026. Original paper evaluated US IRC section 121 (about 15% formalized, 350 lines) and French family benefits (1,500 lines, found a bug in the official implementation). CUTECat (arXiv 2410.18212) adds concolic test generation.
- Limits for litigate: calculation-of-an-outcome model. No process/state, no sequence of events, no agents, no uncertainty. Deadlines would be date arithmetic inside a scope, but there is no notion of "what moves are available after this filing". Law versions are handled by source files, not by a first-class effective-date resolver (K).
- Verdict: best model for the default-with-exception core of `.lit`. Could be a backend target or a design template, not a substitute.

### 2. L4
- SMU Centre for Computational Law (CCLAW, S$15M NRF grant) with Legalese (Meng Weng Wong). Haskell `smucclaw/dsl` (82 stars, last push 2025-06) is the older line; `l4-ide` says "continued by Legalese". Current l4-ide README: typed functional DSL, `@export` REST decision service, MCP server, GraphViz ladder diagrams and eval traces, golden tests, OpenAPI/JSON schema, jl4-mlir to WASM, jurisdiction/currency libraries, Apache-2.0.
- Not verified: whether current L4 has first-class effective dating, deontic/temporal constructs, or procedure. The README fetched did not show them.
- Verdict: closest in spirit to `.lit` (typed, imports, tooling, LSP). No procedure graph, no stochastic layer. Watch it for design; do not depend on it.

### 3. Akoma Ntoso and LegalRuleML
- Akoma Ntoso (OASIS LegalDocML): XML vocabulary for legislative and judicial documents, with FRBR-style versioning for point-in-time text (K). Useful as the source-text anchor/citation format for rules (cite a rule to a versioned AKN URI). Not a rules language.
- LegalRuleML (OASIS): richest vocabulary for what `.lit` needs semantically: temporal characteristics (enforceability, efficacy, applicability), strict/defeasible/defeater strengths, Override priority, deontic modalities, jurisdiction, authority. But it is an interchange schema with no execution semantics and thin tooling. Steal the ontology, not the XML.

### 4. Docassemble
- Interview engine (YAML + Python + Mako), assembles documents. MIT, 992 stars, very active. It encodes forms, not procedure. Useful as a downstream consumer of `.case` output (client intake feeding a `.case`), not a competitor.

### 5. Docketing engines
- CompuLaw (Aderant, successor Milana SaaS), LawToolBox, CourtAlert. Public descriptions are marketing only: "rules library across 2,500-3,000+ jurisdictions", maintained by in-house attorneys, trigger-event -> dependent deadlines, then holiday adjustment (V). I found no public rule-representation format, no precedence-resolution description, and no open spec. The "patented date engine" claim comes from a reseller, unverified. A 2005 article calls CompuLaw a rule-based expert system (V).
- They answer "when is it due" for a trigger. They do not model choice, cost, probability, or settlement. Moat is rule-data curation and a malpractice-grade accuracy guarantee, not technology.
- Small open-source: agg2-lab/court-deadline-calculator (2 stars, MIT), FRCP-style counting. Toy.

### 6. Academic and ecosystem
- OECD "Cracking the Code" (2020) frames Rules as Code; NZ Better Rules and France cited as early cases. Principles (transparency, appealability) worth borrowing: every output must trace to a rule citation. All RaC work targets benefits/tax/eligibility, not adversarial procedure.
- OpenFisca: effective-dated parameters and formulas are its core strength (steal this). AGPL.
- Blawx (Jason Morris, s(CASP)): defeasible rules with explanations; stalled since 2024-11.
- Logical English (Kowalski, Davila, Sartor): readable surface syntax over Prolog/s(CASP); the model for a lawyer-readable `.lit` surface.
- Symboleo (Parvizimosaed et al., SoSyM 2022): obligation/power lifecycle as statecharts with model checking. The best formal treatment of "obligation with deadline and breach" lifecycle.
- Stipula (Crafa, Laneve, Sartor): legal calculus, liquidity analysis.
- Lexon: English to Solidity, little relevance.
- Stanford CodeX: did not retrieve a loadable page (403); no CodeX project found that models procedure. Treat as unverified.
- Priest-Klein (1984) and Klerman-Lee line: settlement/selection games; litgraph's `settle` already sits in this literature. Rule 68, fee shifting, and two-sided incomplete information are the extensions to cite.

### 7. Procedure as graph / executable model
- Meldman 1978 (Petri-net representation of civil procedure, IDEA vol. 19(2)): earliest I found; no abstract.
- Buchanan and Fennell 1981: federal criminal case processing as parallel state diagrams in a docket system (secondary description).
- Rotolo and Smith, ICAIL 2021 (DOI 10.1145/3462757.3466089): PDL with preferences. Closest recent academic work. Full text restricted.
- Univie "A Concept for Modelling Civil Procedures" (eprints 2147): surfaced in search; URL did not load, not cited.
- Valk and van der Aalst: Petri-net workflow of Dutch justice case lifecycle.
- GitHub keyword searches found nothing relevant. I found no prior system that (a) compiles court rules into a procedure graph and (b) attaches costs and probabilities to solve it as a stochastic game. That combination appears novel, with moderate confidence (searches were not exhaustive on ICAIL/JURIX proceedings; a manual proceedings sweep is still owed).

### 8. Bend (HigherOrderCO)
- Repo moved to bendlang/bend and is now "Bend 2: a fast language that blocks AI mistakes via proof". 23.9k stars, Apache-2.0, releases v2.0.35 to v2.0.38 between 2026-10-03 and 2026-10-20 (very active, weekly).
- Bend 2 is a new language; Bend 1 and HVM code does not carry over. README limitations: all types annotated, no inference; no type classes/traits/macros; affine values; termination checking; no if-then-else; numbers only Nat/U32/F32 (no U64/I64/F64); strings are linked lists; small base library; no JSON, regex, HTTP, TLS; targets C, Metal, CUDA, JavaScript; one C file per program; "compiler is young and has blind spots".
- Suitability for litigate's compiler: poor today. A compiler needs rich strings, parsing, maps, JSON, files, error reporting, and a package ecosystem; Bend 2 has few of these, lacks decimal/64-bit dates and money types, and its payoff (GPU parallelism) is irrelevant to parsing and type-checking rules. Its proof features are interesting for verifying a rule-engine kernel but cost heavy annotation. Recommendation: write the compiler in Rust (your language default, and litgraph is Rust); consider Bend only as an experiment for the Monte Carlo/solver kernels, which is the one place parallelism pays.

## What litigate should steal
1. Catala: default-with-exception as the core semantics, literate source with the rule text beside the code, scope/priority discipline, and a formally testable compiler core.
2. OpenFisca: effective-dated parameters and formulas as first-class; "as-of date" as a required compile input.
3. LegalRuleML vocabulary: separate enforceability/efficacy/applicability dates, strict/defeasible/defeater, authority and jurisdiction as typed fields (do not adopt XML).
4. Akoma Ntoso-style versioned URIs as rule citations, so every compiled rule traces to a dated source text (OECD appealability principle).
5. Symboleo: obligation lifecycle (created, active, fulfilled, violated, expired) and model-checking properties on the compiled machine (liveness: every filed motion eventually resolves; safety: no deadline unreachable).
6. L4 / Logical English: LSP, golden tests, explanation traces, readable surface syntax, MCP/decision-service exposure.
7. Docketing engines: trigger event -> dependent deadlines, service add-ons, short-period counting, holiday calendars, and the operational discipline of an attorney-reviewed rule-change feed. This is the data moat.
8. Priest-Klein / Rotolo-Smith: cite as theory anchors; Rotolo-Smith preference operator maps to move-ordering.

## Does anything make litigate redundant?
No. Each component exists somewhere: defaults (Catala), effective dating (OpenFisca), obligations lifecycle (Symboleo), deadlines (CompuLaw), settlement games (law and economics). Nothing found combines (rules with precedence and dates) -> (procedure graph) -> (cost/probability stochastic game). Real redundancy risk is narrower: Catala + a thin Rust or Python library could express the rule layer without a new language. The honest question is whether `.lit` needs to be a new surface language or could be a Catala dialect/target plus litgraph packs. Reasons for a new language: procedure is event/state driven, not scope-evaluation; judge standing orders layer over rules by forum and date; the output is a graph, not a value. Reasons against: parser, LSP, docs, and credibility cost, and you already have working pack YAML/JSON in litgraph.

## Closest competitor
Two, by axis. Language/semantics: Catala (default logic, literate, deployed in government, Apache-2.0) with L4 as the nearer peer on tooling. Product/market: CompuLaw/Milana for the docketing-from-rules use case (incumbent, closed, no analysis). Closest on thesis (procedure as formal executable model): Rotolo and Smith (ICAIL 2021) and Meldman's 1978 Petri net, both academic and not engines.

## Gaps and caveats
- Catala backend list, Catala temporal handling, AKN versioning, L4 current deontic/temporal features, CourtAlert ownership: from memory or not confirmed on fetched pages.
- Not loadable by curl (403 bot block or 000) so not cited: aderant.com, lawtoolbox.com, law.stanford.edu/codex, ACM DL, SSRN, oecd.org, univie eprints, symboleo.org.
- ICAIL/JURIX proceedings were not searched directly; recommend a manual sweep of "procedure", "deadline", "Petri", "PDL" in ICAIL 2019-2025 and JURIX.

## Sources (all returned HTTP 200)
- Catala site: https://catala-lang.org
- Catala repo: https://github.com/CatalaLang/catala
- Catala book: https://book.catala-lang.org
- Catala paper: https://arxiv.org/abs/2103.03198
- CUTECat: https://arxiv.org/abs/2410.18212
- Merigoux (deployments, self-reported): https://merigoux.ovh/
- L4 docs: https://l4-documentation.readthedocs.io/en/latest/
- L4 repo (l4-ide): https://github.com/smucclaw/l4-ide
- Older SMU DSL repo: https://github.com/smucclaw/dsl
- Legalese: https://legalese.com
- Akoma Ntoso: https://www.akomantoso.org
- AKN vocabulary: https://docs.oasis-open.org/legaldocml/akn-core/v1.0/akn-core-v1.0-part1-vocabulary.html
- LegalRuleML spec: https://docs.oasis-open.org/legalruleml/legalruleml-core-spec/v1.0/legalruleml-core-spec-v1.0.html
- LegalRuleML committee: https://www.oasis-open.org/committees/legalruleml/
- Docassemble: https://docassemble.org and https://github.com/jhpyle/docassemble
- CompuLaw: https://www.compulaw.com/
- Deadlines.com on calculation: https://deadlines.com/faqs/how-are-deadlines-calculated/
- Open-source deadline toy: https://github.com/agg2-lab/court-deadline-calculator
- OpenFisca: https://openfisca.org/doc/ and https://github.com/openfisca/openfisca-core
- Blawx: https://github.com/Lexpedite/blawx
- Logical English: https://github.com/LogicalContractsOrg/LogicalEnglish and https://ceur-ws.org/Vol-3193/paper5GDE.pdf
- Symboleo (SoSyM 2022): https://www.site.uottawa.ca/~luigi/papers/22_SoSym.pdf
- Stipula: https://github.com/stipula-language/stipula and https://arxiv.org/abs/2110.11069
- Lexon: https://lexon.org/
- OECD Cracking the Code (OPSI): https://oecd-opsi.org/publications/cracking-the-code/
- Rotolo and Smith (Bologna record): https://cris.unibo.it/handle/11585/845532
- Meldman Petri-net bibliography: https://www.informatik.uni-hamburg.de/TGI/pnbib/m/meldman_j_a1.html
- Springer chapter surfaced in Catala search: https://link.springer.com/chapter/10.1007/978-3-031-91121-7_2
- Bend: https://github.com/bendlang/bend and https://bend-lang.com
