# litigate: ideas mined from David's library

Scope: `~/Documents/agent-library/` and `~/Knowledge/`. Nothing in either was modified. Working text extractions live in `/private/tmp/claude-501/-Users-david--dotfiles/7f88988e-5ac5-4c4f-bf9c-723e63585248/scratchpad/txt/` (pandoc for epub, pdftotext for pdf) and `.../scratchpad/dijk/` (tesseract OCR of selected Dijkstra pages).

Conventions
- Path prefixes: `QA/` = `/Users/david/Knowledge/Stacks/Q Science/QA Mathematics/QA71-90 Instruments and machines/QA75-76.95 Calculating machines/QA75.5-76.95 Electronic computers. Computer science/`; `K/` = `/Users/david/Knowledge/Stacks/K Law/`; `CE/` = `/Users/david/Knowledge/General/Computational Epistemology/Papers/`; `S/` = `/Users/david/Knowledge/Stacks/`.
- "p." = printed page of the book/paper unless I write "pdf p.". Epubs have no stable pagination, so I cite chapter/section titles or numbers instead. Where a number is missing I say so.
- "Facts" (what the source says) are verified against the extracted text. "Mapping" lines are my proposal for litigate, not the author's claim.
- The library has NO material on salsa/incremental computation frameworks, tree-sitter, LSP, Datalog, or legal-rules-as-code DSLs (Catala, L4, etc). See the Gaps section.

---

## 1. Inventory

R = readable by me (y = text extracted and mined, y* = readable, mined lightly, n = not text-readable).

| # | Title | Author | Path | R | Used for |
|---|---|---|---|---|---|
| 1 | Engineering a Compiler (2nd ed) | Cooper, Torczon | `QA/QA76 General works, treatises, and textbooks/Engineering a Compiler - Cooper Torczon/Engineering a Compiler - Cooper Torczon.epub` | y | IR design, scopes, type systems, attribute grammars, error recovery |
| 2 | Modern Compiler Implementation in C | Appel | `QA/QA76 General works, treatises, and textbooks/Modern Compiler Implementation in C - Appel/Modern Compiler Implementation in C - Appel.epub` | y (OCR'd text, no printed page markers except where noted) | persistent environments, interned symbols |
| 3 | Crafting a Compiler | Fischer, Cytron, LeBlanc | `QA/QA76 General works, treatises, and textbooks/Crafting a Compiler - Fischer Cytron LeBlanc/Crafting a Compiler - Fischer Cytron LeBlanc.pdf` (+ .epub) | y | import/export, error repair, IDE, reachability, AST/visitor |
| 4 | Practical Foundations for Programming Languages (abbreviated, 2nd ed) | Harper | `QA/QA76 General works, treatises, and textbooks/Practical Foundations for Programming Languages - Harper (abbrev)/...pdf` (+ .epub) | y, but ABBREVIATED: only Ch.1-11 and a few later chapters (Ch.28-29, 34, 40) have bodies; the TOC lists chapters (behavioral typing Ch.25, modularity Ch.42) that are NOT in the file | judgments, statics, safety, sum types |
| 5 | Structure and Interpretation of Computer Programs | Abelson, Sussman | `QA/QA76.73.A-76.73.Z Individual languages, A-Z/QA76.73.S34 Scheme/Structure and Interpretation of Computer Programs - Abelson Sussman Sussman.epub` | y | analyze-then-execute, data-directed dispatch, logic programming |
| 6 | A Discipline of Programming | Dijkstra | `QA/QA76.6 General works/Edsger W. Dijkstra - A Discipline of Programming-Prentice Hall (1976)/Edsger W. Dijkstra - A Discipline of Programming-Prentice Hall (1976).pdf` | n by pdftotext (scanned, 232 pp, no text layer). I OCR'd with tesseract only pdf pp. 4, 15-50, 205-220 (printed pp. ~ vii-viii, 12-47, 190-205). Rest unread | guarded commands, wp, spec vs implementation |
| 7 | Bitemporal Data: Theory and Practice | Johnston | `QA/QA76 General works, treatises, and textbooks/Bitemporal Data Theory and Practice - Johnston/Bitemporal Data Theory and Practice - Johnston.pdf` | y | state/assertion/inscription time |
| 8 | Developing Time-Oriented Database Applications in SQL | Snodgrass | `QA/QA76 General works, treatises, and textbooks/Developing Time-Oriented Database Applications in SQL - Snodgrass/...pdf` | y | valid vs transaction time, periods, sequenced/current/nonsequenced |
| 9 | Domain Modeling Made Functional | Wlaschin | `QA/QA76.75-76.765 Computer software/QA76.758 Software engineering/Domain Modeling Made Functional - Wlaschin/Domain Modeling Made Functional - Wlaschin.epub` | y | typing documents/states/errors, bounded contexts |
| 10 | A Philosophy of Software Design, 2nd ed | Ousterhout | `QA/QA76.75-76.765 Computer software/QA76.758 Software engineering/A Philosophy of Software Design, 2nd Edition - Ousterhout.epub` | y* | deep modules, errors out of existence |
| 11 | Rust for Rustaceans | Gjengset | `QA/QA76.73.A-76.73.Z Individual languages, A-Z/QA76.73.R87 Rust/Rust for Rustaceans - Gjengset/Rust for Rustaceans - Gjengset.epub` | y | typestate, error design, semver, testing, macros |
| 12 | Programming Rust, 2nd ed | Blandy, Orendorff, Tindall | `QA/QA76.7 General works/Programming Rust - Blandy/Programming Rust - Blandy.epub` | y | modules/imports, Cargo.lock, doc-tests |
| 13 | The Rust Programming Language | Klabnik, Nichols | `QA/QA76 General works, treatises, and textbooks/The Rust Programming Language - Klabnik Nichols/...epub` | y* | modules, workspaces, test organization (not cited below beyond a pointer) |
| 14 | Rust in Action | McNamara | `QA/QA76.73.A-76.73.Z Individual languages, A-Z/QA76.73.R87 Rust/Rust in Action - McNamara/...epub` | not opened | - |
| 15 | A Logic for Default Reasoning (AI 13, 1980) | Reiter | `S/Q Science/Q1-390 Science (General)/Q300-390 Cybernetics/Q339 Default reasoning/reiter-1980-logic-default-reasoning.pdf` | y | defaults and exceptions |
| 16 | Defeasible Reasoning (Cognitive Science 11, 1987) | Pollock | `S/B Philosophy. Psychology. Religion/BC Logic/BC171-199 Special topics/BC199.A-199.Z Other special topics, A-Z/BC199.D38 Defeasible reasoning/pollock-1987-defeasible-reasoning.pdf` | y | rebutting vs undercutting defeaters |
| 17 | The ASPIC+ framework for structured argumentation: a tutorial (Argument & Computation 5(1), 2014) | Modgil, Prakken | `S/B Philosophy. Psychology. Religion/BC Logic/BC171-199 Special topics/BC177 Reasoning, argumentation, etc./modgil-2014-aspic-framework-structured-argumentation-tutorial.pdf` | y | strict/defeasible rules, preference orderings |
| 18 | Answer-set programming encodings for argumentation frameworks (A&C 1(2), 2010) | Egly, Gaggl, Woltran | `QA/QA76.63 Logic programming/egly-2010-answer-set-programming-encodings-argumentation-frameworks.pdf` | y | fixed-program + input-facts architecture |
| 19 | Understanding ProbLog as Probabilistic Argumentation (2023) | Toni et al. | `QA/QA76.63 Logic programming/toni-2023-understanding-problog-probabilistic-argumentation.pdf` | y* (abstract only) | probabilistic layer, low priority |
| 20 | ProbLog: probabilistic Prolog (2007) | De Raedt et al. | same dir, `de-raedt-2007-...pdf` | not opened | - |
| 21 | Reading Law: The Interpretation of Legal Texts | Scalia, Garner | `K/K201-487 Jurisprudence. Philosophy and theory of law/K288-296 Interpretation and construction of law. Lacunae in law/K290 English/Reading Law - Scalia & Garner/Reading Law - Scalia & Garner.epub` (second copy under `Reading Law The Interpretation of Legal Texts - Scalia Garner/`) | y | canons = conflict-resolution semantics |
| 22 | Jurisprudence: Text and Readings on the Philosophy of Law, 4th ed (includes Hart, The Concept of Law, "Primary and Secondary Rules") | Christie, Martin, MacLeod (ed.) | `K/K201-487 Jurisprudence. Philosophy and theory of law/K237-264 The concept of law/K237 General (Table K8)/Christie, Martin, and MacLeod's Jurisprudence, Text and Readings on the Philosophy of Law, 4th - Christie.epub` | y | rules of recognition/change/adjudication |
| 23 | Federal Rules of Civil Procedure (2024 ed) | - | `K/KF United States/KF8700-9075 Courts. Procedure/KF8810-9075 Civil procedure/KF8820 General/federal-rules-civil-procedure-2024/federal-rules-civil-procedure-2024.pdf` | y | Rules 6, 83, 86 text |
| 24 | Examples & Explanations: Civil Procedure | Glannon | `K/KF United States/KF8700-9075 Courts. Procedure/KF8810-9075 Civil procedure/KF8841 Compends/Examples & Explanations for Civil Procedur - Joseph W. Glannon/...epub` | y | worked deadline example, running "Schulansky" case |
| 25 | Other civ-pro corpus (not mined): Yeazell FRCP; Moore's Federal Practice; Manual for Complex Litigation 4th; Sedona Principles 3d; Friedenthal et al. casebook; Cross/Abramson/Deason; CALI FRCP + Pleading; Subrin FRCP | various | same `KF8810-9075 Civil procedure/` tree | present, not mined | candidate pack-sourcing and fixture corpora |
| 26 | Legal idioms: a framework for evidential reasoning (A&C 4(1), 2013) | Lagnado, Fenton, Neil | `CE/lagnado-2013-legal-idioms-framework-evidential-reasoning.pdf` | y* | reusable idioms |
| 27 | A General Structure for Legal Arguments About Evidence Using Bayesian Networks (Cog Sci 37, 2013) | Fenton, Neil, Lagnado | `S/K Law/KF United States/KF8700-9075 Courts. Procedure/KF8810-9075 Civil procedure/KF8931-8935 General (Table KF3)/fenton-2013-general-structure-legal-arguments-evidence-bayesian-networks.pdf` | y* | idioms/templates |
| 28 | The Principles of Judicial Proof | Wigmore | `CE/wigmore-1913-principles-judicial-proof.pdf` | y (5 MB text, not mined) | evidence charting, out of scope |
| 29 | Argument Mining: A Survey (2020) | Lawrence, Reed | `S/P Language and literature/P1-1091 Philology. Linguistics/P98-98.5 Computational linguistics. Natural language processing/P98 General works/lawrence-2020-argument-mining-survey.pdf` | y* | typed markup of legal argument |
| 30 | Argumentative LLMs for Explainable and Contestable Claim Verification (2024) | Freedman et al. | same dir, `freedman-2024-...pdf` | y* | contestable explanations |
| 31 | HeidelTime: High Quality Rule-based Extraction and Normalization of Temporal Expressions (SemEval-2010) | Strotgen, Gertz | same dir, `HeidelTime- High Quality Rule-Based Extraction and Normalization of Temporal Expressions.pdf` | y | docket-text date matching |
| 32 | Identification of Rhetorical Roles of Sentences in Indian Legal Judgments (2019) | Bhattacharya et al. | `S/Z Bibliography. Library science. Information resources (General)/ZA Information resources (General)/ZA3075 General works/Identification of Rhetorical Roles of Sentences in Indian Legal Judgments.pdf` | y* | NLP of legal text, low priority |
| 33 | Span-level citations in ALCE and Elicit | David's own note (cites Gao et al. 2023) | `/Users/david/Documents/agent-library/Research/Computational Epistemology/alce-and-elicit-span-citations/README.md` | y | provenance/citation of outputs |
| 34 | A Simple, Fast Dominance Algorithm | Cooper, Harvey, Kennedy | `QA/QA76.9.A-76.9.Z Other topics, A-Z/QA76.9.A43 Algorithms/cooper-2006-simple-fast-dominance-algorithm.pdf` | y* | engine side; litgraph already has dominators |
| 35 | Syntax highlighting as an influencing factor when reading and comprehending source code (J. Eye Movement Res. 9(1), 2016) | Beelders, du Plessis | `QA/QA76.9.H85 Human-computer interaction/Beelders-du-Plessis-2016-Syntax-Highlighting-Eye-Tracking.pdf` | y | editor tooling evidence |
| 36 | The impact of syntax colouring on program comprehension (2015) | Sarkar | `QA/QA76.9.H85 Human-computer interaction/Sarkar-2015-Impact-of-Syntax-Colouring-on-Program-Comprehension.pdf` | y | editor tooling evidence |
| 37 | KBSpec: LLM-driven Formal Specification Generation with Evolving Domain Knowledge Base (arXiv 2606.21339) | Wang, Sun | `QA/QA76.75-76.765 Computer software/QA76.758 Software engineering/KBSpec LLM-driven Formal Specification Generation with Evolving .../*.md` | y* (abstract/intro) | LLM-assisted authoring loop |
| 38 | Modern Type Theory (lectures 1-2 only, KU Leuven student notes) | Vandenbergh | `S/B Philosophy. Psychology. Religion/BC Logic/BC171-199 Special topics/BC199.A-199.Z Other special topics, A-Z/BC199.T9 Type theory/Mathematics Modern Type Theory Ridan Vandenbergh (Course)/` | y but thin (intro + Pi-types stub) | not used |
| 39 | On the Impact of Programming Languages on Code Quality: A Reproduction Study (2019) | Berger et al. | `QA/QA76.75-76.765 Computer software/QA76.758 Software engineering/Berger-et-al-2019-On-the-Impact-of-Programming-Languages-on-Code-Quality.pdf` | y* | only a caution: language-choice effects on defects are tiny (abstract) |
| 40 | Other CS books present, not mined: Clean Code (Martin); Accelerate (Forsgren et al.); Functional Programming in Scala 2e; Algorithm Design (Kleinberg/Tardos); Algorithms (Sedgewick/Wayne); Algorithm Design Manual (Skiena); Linux Programming Interface | various | `QA/QA76 General works, treatises, and textbooks/` | present | no direct litigate fit found |

Searches that returned nothing relevant: filename searches (find -iname, bounded to ~/Knowledge) for tree-sitter, salsa, datalog, prolog (only ProbLog papers), catala, interpreter, parser, grammar, ontology, formal, petri, automata, coq, haskell, lisp. `~/Documents/agent-library/` contains only the ALCE note.

---

## 2. Ideas grouped by design decision

### A. Precedence, defaults, exceptions (the heart of `.lit`)

A1. Two orderings, not one: hierarchy vs specificity.
- Facts: Reading Law §28 (General/Specific Canon): the specific provision is treated as an exception; it suspends the general rule only for the cases it covers, the general rule "continues to govern all other cases". The same section says the canon "applies only to provisions that are of the same level of legal hierarchy"; a specific statute cannot override a general constitutional rule. §27 (Harmonious-Reading) says to reconcile before applying conflict rules; §29 (Irreconcilability): truly irreconcilable provisions at the same level of generality, simultaneously adopted, neither takes effect.
- Mapping: resolve conflicts in two stages: tier (statute > FRCP > local > standing order > case order) decides validity/consistency; within a tier, specificity decides. A same-tier, same-specificity, same-effective-date clash is a compile error (§29), not a silent last-wins. Compile exceptions as scoped suspension ("applies except where X"), never as deletion.
- Source: Scalia and Garner, Reading Law, Part Two "Contextual Canons", §§27-29 (epub, no pages).

A2. Reserve explicit precedence keywords.
- Facts: Reading Law §13 (Subordinating/Superordinating Canon): "subject to" means the main clause does not derogate from the referenced provision; "notwithstanding"/"despite" means the main clause prevails over it; "subject to" must not completely contradict the thing it modifies. Catch-all "notwithstanding anything to the contrary" is common drafting.
- Mapping: first-class `subject_to <rule-id>` and `notwithstanding <rule-id>` modifiers in `.lit`, resolved at compile time into edges in a precedence graph; lint on catch-all "notwithstanding" with no actual referent.
- Source: Reading Law §13.

A3. Implied repeal and revival.
- Facts: §55 (Presumption Against Implied Repeal): repeals by implication are "very much disfavored", but flatly contradictory later provisions repeal earlier ones. §56 (Repeal-of-Repealer Canon): repeal or expiration of a repealing statute "does not reinstate the original statute"; modern federal statute says a repealed repealer does not revive the former act unless expressly provided; Bentham: revival is as easy to state directly as repeal.
- Mapping: a rule version ends only by an explicit `supersedes`/`repeals` or by an `effective_to`; an expiring or repealed amendment does NOT resurrect its predecessor unless `revives <id>` is written. Warn when two same-tier rules overlap without an explicit supersession.
- Source: Reading Law §§55-57.

A4. Hart's secondary rules as the architecture of a forum module.
- Facts: Hart (excerpted in Christie, Ch.6 "Analytical Positivism", Hart "Primary and Secondary Rules", reprinted pp. 722-727): rule of recognition identifies valid rules; "where more than one of such general characteristics are treated as identifying criteria, provision may be made for their possible conflict by their arrangement in an order of superiority" (p.725). Rules of change empower bodies to introduce and eliminate rules, and define procedure for doing so (legislative enactment and repeal are understood this way). Power-conferring rules (making wills, contracts) are distinct from duty-imposing rules (pp.725-726). Rules of adjudication authorize determinations that a rule was broken (p.726-727).
- Mapping: a `.lit` for a forum declares its `recognition` clause (what sources count and their order), its `change` clause (how it is amended, effective-dating), and its `adjudication` clause (who can rule). Give rules a kind: `duty` (must file X within N days) vs `power` (may move to extend; court may extend under FRCP 6(b)). Power rules are what produce case-order overlays.
- Source: Christie et al., Jurisprudence, 4th ed., Ch.6, Hart excerpt; pp. 725-727 (page numbers appear inline in the epub text).

A5. Statutory text already contains the default/exception pattern.
- Facts: FRCP 6(a) applies "in computing any time period specified in these rules, in any local rule or court order, or in any statute that does not specify a method of computing time". Rule 6(a)(4): "Unless a different time is set by a statute, local rule, or court order, the last day ends: (A) for electronic filing, at midnight in the court's time zone". Rule 6(b)(2): a court "must not extend the time" under Rules 50(b), 50(d), 52(b), 59(b), (d), (e), 60(b). Rule 83(a)(1): a local rule "must be consistent with - but not duplicate - federal statutes and rules". Rule 83(a)(2): a local rule "imposing a requirement of form must not be enforced in a way that causes a party to lose any right because of a nonwillful failure to comply". Rule 83(b): sanctions for a requirement not in law/rules/local rules only if the violator "has been furnished in the particular case with actual notice of the requirement."
- Mapping: (i) defaults declare their scope ("applies to local rules and orders that do not set a method"); (ii) deadlines carry `extendable: bool` with the enumerated exceptions as data; (iii) a standing order's obligations compile with a precondition `notice_given(case, order)`, a fact the `.case` must supply or the linter must flag as unknown; (iv) form-requirement lints get severity `warning`, not `bar`, per 83(a)(2); (v) duplication of a higher-tier rule inside a lower tier is a lint ("local rule duplicates FRCP 5(d)") per 83(a)(1).
- Source: Federal Rules of Civil Procedure 2024, Rules 6, 83 (pdf above; Rule 6 starts on its p.12-13, Rule 83 on p.100-101 of the document).

A6. Which default logic to adopt (and what to avoid).
- Facts: Reiter §1.1.1 "Defaults and exceptions" (p.82): "most birds fly except penguins..." representing exceptions explicitly in first-order logic is awkward. Defaults can yield multiple extensions (p.87, closed-world example `p v q` gives two complete extensions) and some default theories have no extension (Example 2.6, p.91). §3 "Normal Default Theories" (p.94) restricts to a class where extensions are guaranteed to exist and proof stays local to the defaults used. Closed world assumption as a default (§1.1.3, p.84).
- Mapping: constrain `.lit` to the normal-default fragment: every default is `A : B / B` with exceptions written as explicit `unless` clauses naming rule ids. That guarantees an extension exists, makes evaluation local (you do not need every rule to answer one question), and turns "multiple extensions" into a detectable ambiguity (two same-priority defaults with incompatible conclusions) reported at compile time. "Absence of a docket entry" is a closed-world default: state it as such.
- Source: Reiter, "A Logic for Default Reasoning", Artificial Intelligence 13 (1980) 81-132, §§1.1.1, 1.1.3, 2, 3.

A7. Two kinds of exception: rebut vs undercut.
- Facts: Pollock §2: prima facie reason (2.2); rebutting defeater (2.4) gives a reason for the opposite conclusion; undercutting defeaters "attack the connection between the reason and the conclusion rather than attacking the conclusion itself" (journal pp. ~485-486; text says rebutting defeaters "form the basis for most current AI work" but undercutting ones are "equally important").
- Mapping: two syntactic forms in `.lit`: `overrides R` (rebut: a different conclusion for the same question) and `unless <condition>` / `does_not_apply_when` (undercut: the rule's applicability is attacked). They behave differently in explanations and in conflict detection.
- Source: Pollock, "Defeasible Reasoning", Cognitive Science 11 (1987) 481-518, §2-3.

A8. Use ASPIC+ as the semantic backbone for precedence.
- Facts: Modgil and Prakken (Prakken is in a law faculty): arguments are built from strict rules (premises guarantee conclusion) and defeasible rules (create only a presumption); conflicts are resolved by explicit preferences (abstract, p.31); Definition 3.12 (successful rebuttal, undermining, defeat, p.40); Definition 3.21 last-link principle compares arguments by the last defeasible rules used (p.43); undercutters name the rule they attack, which requires a naming convention for rules (p.50); rationality postulates (subargument closure, consistency) are properties a well-formed instantiation should satisfy (pp.32, 40).
- Mapping: (i) every `.lit` rule has a stable id so exceptions can name it; (ii) classify rules strict (arithmetic like Rule 6(a)) vs defeasible (anything an order can vary); (iii) precedence is an explicit preorder over rule ids, built from tier + `subject_to`/`notwithstanding` + specificity; use last-link when a derivation chain mixes tiers; (iv) property-test consistency and closure on the compiled library.
- Source: Modgil and Prakken, "The ASPIC+ framework for structured argumentation: a tutorial", Argument & Computation 5(1), 2014, 31-62, Defs 3.12, 3.21.

A9. Architecture: fixed rule program plus input facts.
- Facts: Egly et al.: "encodings are formulated as fixed queries, such that the input is the only part depending on the actual AF" (abstract, p.147), for admissible, preferred, stable, complete, grounded semantics, plus preference-based and value-based AFs.
- Mapping: the litigatable is the fixed program; the `.case` contributes only facts. If conflict analysis ever outgrows hand-written Rust, an ASP backend (clingo-style) over a lowered fact base is a documented way to compute extensions; keep the lowering to facts clean so that backend stays possible.
- Source: Egly, Gaggl, Woltran, "Answer-set programming encodings for argumentation frameworks", A&C 1(2), 2010, 147-177.

A10. Static conflict and gap detection from guarded commands.
- Facts: Dijkstra ch.5 "The Semantic Characterization of a Programming Language", p.34: for `if B1 -> SL1 [] B2 -> SL2 ... fi`: overlapping true guards give nondeterminacy ("left undefined which ... will then be selected"; none if guards exclude each other); if no guard is true, "activation ... will lead to abortion". Definition of wp for `if` requires at least one guard true and every true guard's branch to establish the post-condition (p.34).
- Mapping: two compile-time lints on any decision point in the lowered graph: overlap (two rules both applicable, no precedence edge between them) and non-exhaustiveness (a state where no rule is applicable and the state is not terminal). This is the same pair as Reading Law §28/§29 from the logic side.
- Source: Dijkstra, A Discipline of Programming (1976), pp. 32-34 (OCR). Note: scanned; I read only the pages listed in the inventory.

A11. Negation and closed-world in any logic layer.
- Facts: SICP §4.4 introduces a query language with assertions, rules, pattern matching and unification; §4.4.3 asks "Is logic programming mathematical logic?" (the `not` special form is a filter, not logical negation).
- Mapping: if litigate embeds a query/rule language, specify `not` as negation-as-failure over the closed fact base of the `.case` and say so; do not let authors think it proves absence in the world.
- Source: Abelson and Sussman, SICP, §4.4 and §4.4.3.

### B. Time, effective dating, versioning

B1. Two time axes minimum: when the law is in force, and when we believed it.
- Facts: Snodgrass §1.1 (p.4): user-defined, valid, and transaction time are orthogonal; valid time = when a fact was true in the modeled reality; transaction time = when it was stored. §2.3 Bitemporal Tables (pp. 20-21): reconstruct "the history of the LOT table as best known on March 15" versus "as best known on April 1", and ask "When were steerings scheduled (as opposed to being recorded after the fact)?".
- Mapping: compile takes `as_of_law = D1` (which rule versions govern events on D1) and optionally `as_known_at = D2` (which versions our library contained on D2). This answers "what would we have told the client on D2?" and supports malpractice-style audits and reproducible old deadline calendars.
- Source: Snodgrass, Developing Time-Oriented Database Applications in SQL, §1.1 p.4, §2.1-2.3 pp.12-22.

B2. Three dimensions, and future-effective rules.
- Facts: Johnston Preface p.xxviii: state time (when things are so), speech-act time (when someone asserts a statement), inscription time (when a row physically exists); he argues for keeping transaction time and adding assertion time as a separate dimension. Ch.14 "Future Assertion Time" (pp. 225-240) argues rows cannot be added with assertion time in the past (p.228) but can be in the future, and discusses a "time travel paradox" and locking (pp. 233-237).
- Mapping: record for each rule version: `effective_from/to` (state time), `adopted/promulgated` date (speech-act time), and `ingested_at` plus content hash (inscription time). Amendments announced before they take effect (FRCP changes effective Dec. 1) are future-effective versions that are already in the library.
- Source: Johnston, Bitemporal Data: Theory and Practice (2014), Preface p.xxviii, Ch.14.

B3. Transition rules belong to the version, not the engine.
- Facts: FRCP 86(a): amendments "take effect at the time specified by the Supreme Court" and govern (1) proceedings in an action commenced after the effective date and (2) proceedings after that date in an action then pending "unless (A) the Supreme Court specifies otherwise; or (B) the court determines that applying them in a particular action would be infeasible or work an injustice."
- Mapping: each rule version carries `applies_to: new_actions | pending_actions | both` and an `unless` escape that a case order can trigger. Default for rules amended under the Rules Enabling Act: pending actions, with the infeasibility/injustice exception modeled as a case-level override.
- Source: FRCP 2024, Rule 86(a).

B4. Periods are a first-class type; use half-open intervals.
- Facts: Snodgrass §12.4 and ch.4: SQL-92 lacks a period type ("left to the application developer to simulate", p.4); period predicates in Fig. 4.1 (p.92) are before, meets, overlaps, equals, starts, finishes, during and their inverses; he uses the closed-open representation throughout (p.92, p.119). §5.4 "Handling Now" (p.119) discusses the sentinel for current data, and §12.5.1 (p.408) defines "temporal upward compatibility": constraints on a snapshot table must still hold on the current time slice once valid time is added. Johnston Ch.7 (p.99-115) develops the Allen relationships.
- Mapping: `Period`, `Instant`, `Duration`, with explicit granularity (day vs timestamp-with-court-timezone; Snodgrass p.12 notes "temporal granularity ... is a day"). Rule validity as `[from, to)`; "until changed" as an explicit `open` rather than 9999-12-31. Upward compatibility: a `.lit` with no dates must behave as always-in-force, so adding dates later cannot silently change old results. Use the Allen set for stay/tolling/response-window reasoning.
- Source: Snodgrass pp. 4, 12, 89-108, 119, 408; Johnston Ch.7.

B5. Three query modes.
- Facts: Snodgrass p.4: statements are current ("now"), sequenced ("at each instant of time"), or nonsequenced ("ignoring time"); the trichotomy applies to queries, modifications, views, and integrity constraints. Ch.6 covers current, prior-state, sequenced, and nonsequenced queries (pp. 143-158).
- Mapping: CLI/engine entry points: `litigate calendar --now`, `--timeline` (every instant the rule set changes, e.g. when a pending amendment would alter this matter's deadlines), and `--ignore-time` (lint across all versions at once).
- Source: Snodgrass §1.1, Ch.6.

B6. Persistent (immutable) environments make multi-date compiles cheap.
- Facts: Appel §5.1 "Efficient Functional Symbol Tables" (pp. 107-108): a functional environment lets you compute `sigma' = sigma + {a -> tau}` while `sigma` remains usable; binary search trees share structure so creating the new table costs O(log n); "an example of a persistent data structure". Interned symbols (p.108): convert strings to symbol objects so equality is a pointer/integer comparison.
- Mapping: the rule environment at date D is a persistent map; the environment at D+1 shares structure with D. Intern rule ids, forum ids, and submission-type names. This is also the data structure a salsa-style cache wants as keys.
- Source: Appel, Modern Compiler Implementation in C, §5.1, pp. 107-108.

### C. Deadline arithmetic

C1. FRCP 6(a) is an executable specification; port it exactly.
- Facts (Rule 6(a)(1)-(6), 2024 text): for day-or-longer periods, exclude the trigger day, count every day including weekends and legal holidays, include the last day, and if it is a Saturday, Sunday or legal holiday roll to the next day that is not one; for hours, start counting immediately, count every hour, and if the period would end on a weekend or holiday continue to the same time on the next non-weekend/holiday day; clerk's-office inaccessibility extends; "last day" ends at midnight in the court's time zone for e-filing, or when the clerk's office is scheduled to close otherwise; "next day" counts forward for periods measured after an event and backward for periods measured before; legal holidays are the statutory federal holidays (incl. Juneteenth), presidential/congressional declarations, and, only for periods measured after an event, state holidays where the district court sits. Rule 6(d) adds time after certain service.
- Mapping: a stdlib module `frcp::time` with `count(trigger, n, unit, direction)`, a `holiday_calendar(court, direction)`, and `last_day_ends(court, medium)`. The direction-dependent holiday set (6(a)(6)(C)) and backward counting (6(a)(5)) are the two cases naive implementations get wrong.
- Source: FRCP 2024, Rule 6(a), (b), (c), (d).

C2. A worked cross-rule example that can seed golden tests.
- Facts: Glannon, Examples & Explanations: Civil Procedure, Ch.33 (Ronan's Answer and Counterclaim), cover memo: removal on Wednesday Dec. 28, 2016; answer due seven days after removal under FRCP 81(c)(2)(C); day of removal excluded under 6(a)(1)(A); weekend days and the holiday count; so the answer is due Wednesday Jan. 4, 2017. Same memo: third-party complaint without leave within 14 days of serving the answer (FRCP 14(a)(1)).
- Mapping: first golden fixture; shows the cross-rule pattern (81(c) supplies the period, 6(a) computes it, 14(a)(1) chains from the answer-service date). Chapters 30-35 of that book form a running case (Schulansky) that can seed an end-to-end `.case`.
- Source: Glannon, Examples & Explanations: Civil Procedure, Ch.30-35, Ch.33 memo (epub, no pages).

C3. Derived deadlines are attributes; guard against cycles.
- Facts: Cooper §4.3 (The Attribute-Grammar Framework): attributes are defined by functional rules, no implied evaluation order; evaluation strategies are dynamic (fire when operands ready), oblivious (fixed passes), or rule-based (static analysis builds an order) (§4.3.1); circularity makes evaluation fail and can be handled by avoidance (restrict to synthesized and constant attributes, which cannot create cycles) or detection (§4.3.2); §4.3.4 warns that nonlocal information flow is awkward.
- Mapping: model "deadline = f(trigger event dates, rule parameters)" as an attribute dependency graph over the case's event graph; use synthesized-only (upstream to downstream) attributes by default so cycles are impossible, and give any back-reference an explicit `fixpoint` annotation. Evaluate in topological order and cache.
- Source: Cooper and Torczon, Engineering a Compiler, 2nd ed., §4.3.1, §4.3.2, §4.3.4.

C4. Backward reasoning for "latest safe date".
- Facts: Dijkstra defines semantics by weakest preconditions: `wp(S, R)` is the set of initial states guaranteeing S terminates in a state satisfying R (ch.5 pp. 25-27, OCR).
- Mapping (analogy only): "latest date I can file X and still be able to meet Y" is a backward propagation from post-conditions (a fixed hearing date) over the same graph. Treat as a feature idea, not a claim by Dijkstra.
- Source: Dijkstra, ch.5 pp. 25-34.

### D. Module and import system

D1. Explicit export/import; resolve imports at compile time.
- Facts: Crafting a Compiler §8.4.4 (p.296): export rules make selected local names visible outside a scope; compilation units "advertise their offerings"; importing units must name what they import to avoid a polluted global namespace; the compiler examines import directives for "potentially external references" and checks validity as far as possible at compile time. §8.4.5 (p.297): forward references are "a promise that a complete definition will eventually be provided", processed in two passes; altered search rules (Pascal `with`).
- Mapping: `.lit` files export named rules, types, calendars; importers list what they use (`use frcp::{rule6, rule12}`); a local rule may forward-reference a rule from a later section in the same file (two-pass resolve). Imports are checked when the litigatable is built, not when a case runs.
- Source: Fischer, Cytron, LeBlanc, Crafting a Compiler, §8.4.4-8.4.5, pp. 296-297.

D2. Do not auto-inherit; separate name resolution from authority.
- Facts: Blandy ch.8 "Paths and Imports": modules do not automatically inherit names from parents; `use` creates aliases; "modules aren't the same thing as files". Cooper §6.3: lexical scoping, name refers to the lexically closest declaration; scopes nest and inherit.
- Mapping: a local-rule file does not silently see FRCP names; it writes `extends frcp@version` (or `use`), which also records the pinned version. Name resolution (which "Rule 16 scheduling order" does this mean?) is a different relation from authority/precedence (A1); keep two separate mechanisms so a nearer scope cannot accidentally outrank a higher tier.
- Source: Blandy et al., Programming Rust, 2nd ed., ch.8; Cooper §6.3.

D3. Bounded contexts for forums.
- Facts: Wlaschin ch.1 "Creating a Solution Using Bounded Contexts": a bounded context has a clear responsibility and corresponds to a software component; "design for autonomy"; a context map shows relationships; ch.3 treats contracts between contexts (upstream has more influence on message format, a translator/anti-corruption layer sits between).
- Mapping: one module per forum or judge is a bounded context with its own vocabulary ("motion", "brief" mean different things in PTAB vs district court); the litgraph pack JSON is the contract between the compiler context and the engine context.
- Source: Wlaschin, Domain Modeling Made Functional, ch.1 (section "Creating a Solution Using Bounded Contexts"), ch.3.

### E. Type system (documents, parties, deadlines, submissions)

E1. Model states and documents as distinct types; make illegal states unrepresentable.
- Facts: Wlaschin ch.6 "Integrity and Consistency in the Domain": "make illegal states unrepresentable ... capture business rules in the type system"; Unvalidated vs Validated address types with a private constructor; ch.7 "Modeling an Order as a Set of States" and "State Machines": replace a flag-bearing record with one type per state and transitions as functions; ch.5 "Modeling Simple Values" (wrapper types) and "Modeling with Choice Types".
- Mapping: `Complaint`, `ServedComplaint`, `Answer` are different types; `File<T>` can only be called where the state allows (a motion for default judgment requires a `ServedComplaint` with an expired answer period). Party roles are choice types (Plaintiff | Defendant | Intervenor | ThirdParty ...). Use newtypes for `CaseNumber`, `DocketEntryNo`, `Days`.
- Source: Wlaschin, chs. 5-7.

E2. In the Rust implementation, typestate for case stage.
- Facts: Gjengset ch.3 "Obvious", Listing 3-2: `Rocket<Stage = Grounded>` with `PhantomData<Stage>` so methods exist only in valid stages; sealed traits restrict implementors; "Constrained" section on semver-safe API evolution.
- Mapping: `Case<Stage>` in the engine API so impossible transitions fail to compile; seal the `Stage` trait.
- Source: Gjengset, Rust for Rustaceans, ch.3 (Obvious, Constrained).

E3. Exhaustive sums for submission types; the typing judgment carries the case as context.
- Facts: Harper ch.11 (11.2-11.3, pp. 91-93): finite sums with `case e {i.x_i -> e_i}`, safety theorem; Booleans as sums; ch.3 hypothetical/general judgments (Gamma |- e : tau).
- Mapping: `SubmissionType` is a closed sum; every `match` over it must be exhaustive (error when a new forum adds a variant a rule forgot); typing judgment `Gamma |- rule : Obligation` where Gamma = the case facts (parties, dates, notice given).
- Source: Harper, PFPL (abbrev), chs. 3, 4, 11.

E4. What the type system can and cannot promise.
- Facts: Cooper §4.2.1: types serve safety, expressiveness, and runtime efficiency; the set of ill-formed programs is not computable, so a checker is conservative. Harper §6.1-6.3 (pp. 51-54): preservation and progress together = safety ("well-typed expressions do not get stuck"); some failures are ruled out statically (unchecked errors), others are modeled as an explicit `err` outcome (checked errors, rules 6.1a-c) and progress is restated "either e err, or e val, or steps" (Thm 6.5).
- Mapping: decide the boundary: statically rejected (malformed rule, unresolved import, type mismatch, unreachable step) vs graph outcomes (missed deadline, sanction, default judgment are terminals, not compile errors). Property-test progress (every non-terminal lowered node has an enabled transition) and preservation (a transition preserves the case's well-typedness).
- Source: Cooper §4.2.1; Harper §§6.1-6.3.

E5. Data-directed registry for submission-type handlers.
- Facts: SICP §2.4.2 (tagged data) and §2.4.3 (data-directed programming and additivity): dispatch on (operation, type tag) through a table so new types are added without modifying generic code.
- Mapping: page limit, format lint, and service-requirement handlers register against `(forum, submission_type)`; adding a forum adds rows.
- Source: SICP §2.4.2-2.4.3.

### F. Lowering to the IR (litgraph pack)

F1. IR design axes and what to keep beside the IR.
- Facts: Cooper §5.1.1: classify IRs along structural organization, level of abstraction, naming discipline; §5.1 Overview: the IR is the definitive form of the program, must be expressive enough to record facts "that have no representation in source code", tables and sets that record additional information are "part of the IR", and designers must give humans ways "to examine the IR easily and directly". Many compilers use multiple IRs in one compilation.
- Mapping: staged lowering `.lit/.case source -> CST -> AST -> typed HIR (resolved names, versions pinned) -> litgraph pack`. Keep side tables for derived facts and provenance (source span, rule id, version, computed deadline) as part of the IR. litgraph's JSON is human-inspectable by design; keep that.
- Source: Cooper, §5.1, §5.1.1.

F2. One IR, many backends.
- Facts: Crafting a Compiler §1.8.3 (p.23): retargetable compilers localize target-dependent code, are harder to write, but let development cost be shared; §10.1 (pp. 391-395) middle-end between front end and back ends.
- Mapping: calendars (ics), docket-matching rules, filing linters, and agent action sets are separate backends reading the same typed HIR/pack; keep target-specific logic out of the HIR.
- Source: Fischer et al., §1.8.3, Ch.10.

F3. Analyze once, execute many times.
- Facts: SICP §4.1.7 "Separating Syntactic Analysis from Execution": interleaving analysis with execution re-analyzes expressions on each evaluation; separating them avoids repeated work.
- Mapping: matches litgraph's compile-then-simulate shape; keep litgraph packs pure data so `simulate` with 50,000 runs reuses one compiled graph.
- Source: SICP §4.1.7.

F4. Keep a concrete syntax tree for tooling.
- Facts: Cooper §5.2.1: parse trees are large; an abstract syntax tree abstracts away nodes that "serve no real purpose in the rest of the compiler". Crafting §7.4.1 (p.250): concrete vs abstract trees; §7.7 (pp. 264-270) visitor patterns.
- Mapping: the formatter and any language server need the concrete tree with comments and whitespace; the compiler can use the AST. In Rust, prefer enums plus exhaustive `match` over the visitor pattern.
- Source: Cooper §5.2.1; Fischer et al. §7.4.1, §7.7.

F5. Specification before implementation.
- Facts: Dijkstra ch.26 "On Manuals and Implementations", p.202: a discrepancy between abstract and physical machine should be read as the physical machine not working according to specification, not as an inaccuracy in the manual; semantic definitions "are not based upon any model of computation".
- Mapping: the `.lit` semantics document (what Rule 6(a) means) is authoritative; when the litgraph engine disagrees with the source text, the engine has the bug. Write the semantics of each construct in terms of its meaning, not "what the engine does".
- Source: Dijkstra, p.202 (OCR).

F6. Serialization is a separate layer.
- Facts: Wlaschin ch.11 "Serialization" (Designing for Serialization): complex domain types are poorly suited to serializers, so convert to a Data Transfer Object type and serialize that; deserialization into the DTO should succeed unless data is corrupt, and domain validation happens in the DTO-to-domain conversion inside the bounded context.
- Mapping: the pack JSON (PACK_SCHEMA) is the DTO; the Rust domain types are richer. Version the DTO schema independently.
- Source: Wlaschin ch.11.

### G. Static checks (feeds `litgraph lint` and compile errors)

G1. Reachability-style analysis: conservative, but errors must be certain.
- Facts: Crafting §9.1.1 (p.345): reachability analysis is conservative; "it is undecidable whether a given statement is ever executed", so not every unreachable statement is found, but every statement flagged unreachable "will definitely be erroneous".
- Mapping: policy for lints: an `error` must have no false positives (dead procedural step, unresolvable import, contradictory same-tier rules); anything heuristic is `warning`. Applies to litgraph's existing dead-end lint.
- Source: Fischer et al., §9.1.1.

G2. Obligations-without-owners and rules-without-sources.
- Facts: Dijkstra p.34 (see A10); Reiter Example 2.6 (a default theory can have no extension).
- Mapping: compile-time checks: every obligation has a responsible party type; every state has an outgoing rule or is terminal; every rule has a `source` citation (litgraph's existing "unsourced pack" lint, moved earlier into the compiler).

### H. Parsing, error messages, IDE feedback

H1. Report many errors per run; recover at synchronization points.
- Facts: Cooper §3.5.1: a parser "should find as many syntax errors as possible in each compilation", using synchronizing words (e.g. semicolon) and error productions; recursive descent discards input to the sync token. Crafting §5.9 (pp. 168-173): error recovery vs error repair (repair modifies the parsed or unparsed portion).
- Mapping: sync tokens for `.lit`: end of rule block, `rule`/`version`/`import` keywords. Never stop at the first error. Repair suggestions (insert missing `end`, correct a misspelled rule id by edit distance) are a later step.
- Source: Cooper §3.5.1; Fischer et al., §5.9.

H2. Domain errors as types; define errors out of existence.
- Facts: Wlaschin ch.10 "Modeling Domain Errors in Types": errors modeled "just like everything else in the domain", as a choice type with a case per kind. Ousterhout ch.10 "Define Errors Out Of Existence" and ch.4 "Modules Should Be Deep".
- Mapping: compile diagnostics are a closed enum (UnresolvedImport, ConflictingRules{a,b,tier}, NonExhaustiveMatch, UnsatisfiedPrecondition ...) with machine-readable fields for LSP and agents. Where the law itself defines the edge case (weekend last day), implement the rule (roll forward) so the user never sees an error.
- Source: Wlaschin ch.10; Ousterhout chs. 4, 10.

H3. IDE needs.
- Facts: Crafting §1.9 (p.24): IDEs need "immediate feedback concerning syntax and semantic problems as the code is entered"; focus on source with derived files managed out of the user's view; programs built incrementally.
- Mapping: design the compiler as a library with a query API (diagnostics, hover, go-to-definition, find-references) from day one, and make `.case` re-checks cheap.
- Source: Fischer et al., §1.9.

H4. Editor highlighting: modest, mixed evidence; cheap anyway.
- Facts: Sarkar (2015): in a within-subjects eye-tracking study with 10 subjects, syntax highlighting significantly improved task completion time, with the effect weakening as programming experience increases. Beelders and du Plessis (2016): fixations and regressions were higher for black-and-white code but not significantly so; students subjectively preferred colour.
- Mapping: lawyer-users are novices at code, which is the group where highlighting helped most; ship a tree-sitter or TextMate grammar and semantic tokens (rule, version, party, deadline) but do not spend design budget beyond that.
- Source: Sarkar, 2015 (abstract); Beelders and du Plessis, J. Eye Movement Research 9(1), 2016 (abstract). Small samples; treat as weak.

### I. Incremental compilation and caching

I1. Library support is thin; the usable pieces.
- Facts: Appel pp.107-108 (persistent structures, interned symbols); Cooper §5.5: a compiler "must either record this information in the IR or re-derive it on demand" and "when the compiler writes the IR to disk, it may be cheaper to recompute facts than to write them"; Crafting §1.9 (incremental builds).
- Mapping: key a memo table by (interned rule id, version hash, case-fact hash); store only content-addressed, versioned outputs on disk; recompute cheap derived facts (day counts). The library has nothing on salsa/query-based compilers; see Gaps.

### J. Testing

J1. Property-based and fuzz tests for the parser and for date arithmetic.
- Facts: Gjengset ch.6 "Testing": sections on mocking, linting, test generation, fuzzing ("Fuzzers ... require little setup: point the fuzzer at a function that takes a fuzzable input"; Listing 6-7 fuzzes a URL parser), property-based testing.
- Mapping: fuzz `parse(.lit)` and `parse(.case)` (no panics, recoverable errors); property-test `count()` (monotone in n, never lands on a weekend/holiday, forward then backward invariants given the Rule 6(a)(5) direction rule).
- Source: Gjengset ch.6.

J2. Doc-tests for rule documentation.
- Facts: Blandy ch.8 "Tests and Documentation": unit tests, integration tests, and "Doc-Tests" (examples in documentation are compiled and run).
- Mapping: every rule's doc comment holds a worked example (`given: served 2025-12-23; expect: answer due 2026-01-13`) that `litigate test` executes; docs cannot drift from behavior. Seed with Glannon's example (C2).
- Source: Blandy ch.8.

J3. Proof-shaped tests.
- Facts: Harper §§6.1-6.2 (preservation and progress); Dijkstra p.34.
- Mapping: preservation and progress as property tests over generated cases (see E4).

J4. Golden outputs.
- The library has no source on snapshot/golden testing. FRCP Rule 6(a) text and Glannon's worked examples are the best in-library oracle material. Gap.

### K. Formatter, language server, tooling

- Formatter: needs CST with trivia (F4). No library text on formatters or LSP; see Gaps.
- Diagnostics shape: H2. IDE obligations: H3.

### L. Package management, lockfiles, pinning rule versions

L1. Lockfile semantics.
- Facts: Blandy ch.8 "Specifying Dependencies / Versions / Cargo.lock": the first build writes `Cargo.lock` recording the exact version of every crate used; later builds reuse it; Cargo upgrades only when you change `Cargo.toml` or run `cargo update`, which stays within the compatibility range declared. Gjengset ch.5 "Versioning": crates follow semantic versioning, which separates breaking changes, additions, and non-breaking fixes; ch.3 "Constrained": plan for what breaking means for dependents.
- Mapping: `.case` declares the compatible range of each forum module (`frcp ^2024`); `litigate lock` writes `.case.lock` with exact (module, version, content hash, as_of_law, as_known_at) so a deadline calendar is reproducible years later; `litigate update` is the only thing that moves it, and prints a diff of changed deadlines. Rule-pack semver: any change that alters a computed date for an existing fact pattern is breaking.
- Source: Blandy ch.8; Gjengset chs.3, 5.

L2. Workspaces and features (caution).
- Facts: Gjengset ch.5: Cargo takes the union of features requested by dependents, so mutually exclusive features are hard to support.
- Mapping: do not model judge standing orders as optional "features" of a district module; model them as separate versioned modules layered by `extends`, to avoid union semantics.
- Source: Gjengset ch.5 (Workspaces, feature sections).

### M. Other outputs: docket-matching NLP, linters, agent actions, provenance

M1. Rule-based temporal tagging pattern for docket text.
- Facts: HeidelTime (SemEval-2010, pp.321-324): rule-based; rules combine regex patterns with resource lists (e.g. `reMonth`) and normalization functions; relative expressions are resolved against the document creation time; separate precision-optimized and recall-optimized rule sets; the authors chose rules over ML because normalization "can then be supervised in a much easier way" and rules allow "modular extensions".
- Mapping: generate docket-entry matchers from each submission type's name plus aliases (resources) plus dated-phrase patterns ("responses due 14 days after service"), anchored to the docket entry's filed date; ship two modes (high precision for auto-actions, high recall for human review).
- Source: Strotgen and Gertz, HeidelTime, §2 System Architecture (pp.321-323).

M2. Legal-text structure.
- Facts: Bhattacharya et al. (2019): seven rhetorical roles of sentences in Indian Supreme Court judgments; neural models beat feature-based CRF baselines; roles are partly subjective across annotators. Lawrence and Reed survey (pdf p.18) cites a typed markup for argument patterns in US decisions (Walker et al. 2014).
- Mapping: only relevant if litigate later classifies order text (e.g. extracting obligations from a judge's standing order into `.lit`). Treat as low priority; note inter-annotator disagreement implies human review of any extracted rule.
- Source: Bhattacharya et al. 2019, abstract; Lawrence and Reed 2020, survey section on legal argument.

M3. Every output cites its source span.
- Facts: David's own note on span-level citations (ALCE: citation support is a separate audit dimension from fluency and correctness; the paper reports incomplete citation support about half the time on ELI5).
- Mapping: every calendar entry, lint, and agent action carries `(module, version, rule id, source file:line, text hash)` so a reviewer can open the exact rule text.
- Source: `/Users/david/Documents/agent-library/Research/Computational Epistemology/alce-and-elicit-span-citations/README.md`.

M4. LLM-assisted authoring loop.
- Facts: KBSpec (arXiv 2606.21339): LLMs fail on low-resource specification languages; augmenting with official documentation plus verifier feedback from failed attempts improves verification pass rates by 10-25% (abstract) with no model tuning.
- Mapping: if an LLM drafts `.lit` from rule text, feed compiler diagnostics (H2, machine-readable) back as the verifier signal, and keep a growing library of accepted rule-to-`.lit` examples. Preprint; unreplicated.
- Source: Wang and Sun, KBSpec, abstract and intro.

M5. Contestable explanations.
- Facts: Freedman et al. (2024): argumentative LLMs produce structured arguments that a human can contest by changing a claim's strength, with measurable effect on the output.
- Mapping: the agent action set can expose "why this deadline" as a derivation tree over rule ids that a lawyer can contest by selecting a different rule version or asserting an exception. Low priority.
- Source: Freedman et al., 2024, intro.

### N. Rust implementation craft

- Macros: Gjengset ch.7 (declarative and procedural macros); Blandy ch.21 (macros). Use `macro_rules!` only for builders and tests; keep the DSL external (`.lit`), not an embedded macro DSL, because lawyers author it.
- Errors: Gjengset ch.4 "Enumeration" vs "Opaque Errors": library crates expose enumerated errors; binaries can use opaque. Matches H2.
- Sealing and API stability: Gjengset ch.3 "Constrained" (above, E2, L1).
- Evidence that language choice changes defect rates is weak (Berger et al. 2019: only four languages show a statistically significant association with defects and the effect is "exceedingly small"); do not justify Rust by defect rates; justify by typestate and exhaustive matching (E2, E3).

---

## 3. Prioritized top 10

1. Two-stage conflict resolution plus explicit `subject_to` / `notwithstanding` / `supersedes` / `revives` (A1-A3; Reading Law §§13, 27-29, 55-57). Highest leverage on the core semantics, and the library contains the best treatment of exactly this.
2. Bitemporal rule versions: `as_of_law` and `as_known_at`, future-effective versions, half-open periods, transition clauses (B1-B4; Snodgrass §§1.1, 2.3, 4, 12.5.1; Johnston Preface, Ch.14; FRCP 86(a)).
3. Normal-default fragment with named rules and two exception forms (A6-A8; Reiter §3, Pollock §2, ASPIC+ Defs 3.12/3.21). Gives defaults-with-exceptions a decidable, explainable semantics.
4. FRCP 6(a) as a tested stdlib with direction-dependent counting and holiday sets, plus Glannon's Dec. 28, 2016 to Jan. 4, 2017 example as the first golden fixture (C1-C2).
5. Compile-time lints from guarded-command logic: overlap, non-exhaustiveness, reachability with a no-false-positive rule for errors, attribute cycle checks (A10, G1, C3; Dijkstra p.34, Fischer §9.1.1, Cooper §4.3.2).
6. IR discipline: staged lowering, side tables for provenance, one IR with several backends, spec authoritative over engine (F1, F2, F5, M3; Cooper §5.1, Fischer §1.8.3, Dijkstra p.202, ALCE note).
7. Types: closed sums for submission types, states as types, power vs duty rule kinds, typestate `Case<Stage>`, first-class Period (E1-E3, A4; Wlaschin chs.5-7, Hart p.725, Gjengset ch.3, Snodgrass ch.4).
8. Module system: explicit imports/exports checked at build time, no implicit inheritance, forward references, bounded contexts (D1-D3; Fischer §8.4.4-8.4.5, Blandy ch.8, Wlaschin ch.1).
9. Lockfile plus semver for rule packs, with a "deadline diff" on update (L1; Blandy ch.8, Gjengset ch.5), supported by persistent environments and interned ids (B6; Appel pp.107-108).
10. Diagnostics and tooling: multi-error recovery, closed diagnostic enum with machine-readable fields, CST with trivia, query API for LSP, doc-tests for rules (H1-H3, J2; Cooper §3.5.1, Fischer §5.9/§1.9, Wlaschin ch.10, Blandy ch.8).

---

## 4. Gaps and unreadable material

- Not in the library (searched by filename, bounded): salsa or query-based incremental compilation; tree-sitter; Language Server Protocol; Datalog; Catala, L4, LegalRuleML, Akoma Ntoso, or any "rules as code" literature; formal-semantics tools (K framework, TLA+, Alloy); snapshot/golden-testing guides; package-resolution theory (PubGrub). If you want these, they have to be acquired; I have not looked any up.
- Dijkstra, A Discipline of Programming: scanned image PDF; no text layer. OCR covered only the pages listed in the inventory; the rest is unread.
- Harper PFPL: the "abbrev" file lacks the Behavioral Typing (Ch.25), Modularity (Ch.42-45), and Equality chapters despite listing them in its table of contents; I cited only chapters actually present (3, 4, 6, 11).
- Appel and the Cooper epubs have no printed page numbers in the converted text; Appel pp.107-108 were located from inline page headers in the OCR'd text, Cooper cites are by section.
- Reading Law, Glannon, Wlaschin, Gjengset, Blandy, Ousterhout, SICP, Christie are epubs: cited by section/chapter (Christie by the printed page numbers that appear inline in the text).
- Not mined though present and probably useful for pack authoring: Moore's Federal Practice, Manual for Complex Litigation 4th, Sedona Principles 3d, Yeazell/Subrin FRCP, CALI exercises, Wigmore (evidence charting, out of scope).
- Evidence quality notes: the syntax-highlighting studies are tiny samples; KBSpec is a 2026 arXiv preprint; Berger et al. is a reproduction study whose point is that the original effects were mostly not supported.
