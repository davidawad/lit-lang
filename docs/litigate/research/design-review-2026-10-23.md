# litigate: adversarial design review (one-time, pre-freeze)

**Scope**
- Snapshot: `origin/main` at `2b8a330`, re-fetched on 2026-10-23. A diff against the extracted tree shows no change under `docs/litigate`, `crates`, `lit`, `cases` or `editors`.
- Read in full: DESIGN.md, SPEC.md, the three research docs, and the `crates/litigate-*` sources.
- Corpus: `lit/{core,frcp,frap,ptab,ttab,links}` were read in depth. `cafc`, `cofc`, `itc`, `mpep` and `frcrimp` were sampled.
- Cases: `cases/*.case`, a sample of `cases/requests/*.case`, and `editors/tree-sitter-litigate`.
- Beads: lg-962 and .1 to .5; lg-fq2 and .1 to .6, including the lg-fq2.1 forum-matrix comments.
- Line numbers marked `~` are approximate. Every other `file:line` was read in the snapshot.

**Citation markers**
- `[C]`: DOI resolved through the Crossref API, with title and authors matched.
- `[F]`: URL fetched (HTTP 200) and the quoted text located in the page.
- `[B]`: taken from the lg-fq2.1 research comment, which says its own URLs were fetched. I did not re-fetch these.
- `[K]`: background knowledge, not fetched. Treat as a lead, not as evidence.

---

## 0. Verdict

1. **litigate should be declarative, and more so than it is now.** Its semantic core should be a typed, bitemporal, stratified deductive database (Datalog with priorities). On top of that sit a closed library of total counting combinators and a typed concurrent transition system (a Petri net with clocks) for procedure. It should not be ASP, a general functional language, or a dependent type theory.
2. **The type system should be ground-indexed and capability-typed. It should not be SMT-first.** Almost every invariant lg-962.3 lists concerns closed terms. Exact evaluation decides those today, so a solver adds nothing.
   - The real gains come from five places: calendar-aware period units, typed delegation capabilities with bounds, linear obligations, bitemporal indices with applicability, and provenance labels.
   - SMT (QF_LIA/LRA) is warranted only for parameterised values and delegation bounds.
3. **The two most dangerous facts about main:**
   - **Precedence is advisory.** `override`, `notwithstanding` and `subject_to` produce diagnostics, but lowering and replay never pick a winner (§5, P1).
   - **Excusal is too broad.** Any cited `Extended`/`Waived`/`Excused` event excuses any late filing (§5, P3). In the Bowles fact pattern (*Bowles v. Russell*, 551 U.S. 205 (2007) [F]), an appeal the Supreme Court held untimely would be scored timely.
4. **The lg-fq2 plan's "ordered layer stack" is the wrong algebra.** Legal precedence is a property of instruments: authority, dates, scope, and declared relations. It is not a property of the order a manifest lists files. Layers should merge by set union, which is associative, commutative and idempotent. Resolution then applies the precedence relation, and an incomparable conflict is an explicit error, or a chance fork when the law is unsettled.

---

## 1. Nature of the language

### 1.1 What litigate has to compute

| Component | Natural formalism | Decidability / cost |
|---|---|---|
| Rule base: declarations, versions, relations, delegations | Facts plus rules over a finite domain | Stratified Datalog: polynomial data complexity (Ceri, Gottlob, Tanca 1989, doi:10.1109/69.43410 [C]) |
| Which version applies at (valid time, transaction time, proceeding) | Bitemporal relational query | A selection over intervals; Jensen and Snodgrass 1999, doi:10.1109/69.755613 [C]; 1996, doi:10.1016/0306-4379(96)00017-8 [C] |
| Conflict resolution and exceptions | Defeasible logic or prioritized defaults | Propositional defeasible logic is linear (Maher 2001, doi:10.1017/S1471068401001168 [C]); Catala's default calculus (Merigoux, Chataing, Protzenko 2021, doi:10.1145/3473582 [C]) |
| Change of law: abrogation, annulment, retroactivity | Temporal defeasible logic | Governatori and Rotolo 2009, doi:10.1093/jigpal/jzp075 [C]; Governatori, Rotolo, Sartor 2005, doi:10.1145/1165485.1165490 [C] |
| Counting (FRCP 6 and analogues) | Small total functions over a calendar | Terminating by construction if the step set is closed and every loop is bounded |
| Case replay: events to posture | Event calculus (Kowalski and Sergot 1986, doi:10.1007/BF03037383 [C]); token game of a Petri net | Checking a concrete trace is linear |
| Procedure graph soundness | Workflow nets (van der Aalst 1998, doi:10.1142/S0218126698000043 [C]); timed automata (Alur and Dill 1994, doi:10.1016/0304-3975(94)90010-8 [C]) | Decidable for bounded nets and a discrete bounded horizon. Stopwatch clocks (tolling) are undecidable in general (Cassez and Larsen 2000, doi:10.1007/3-540-44618-4_12 [C]), so tolling must stay discrete and bounded |
| Game: decisions, chance, payoffs | Stochastic games (Shapley 1953, doi:10.1073/pnas.39.10.1095 [C]) | Belongs to the engine, not the language |

Every row except counting and the game is relational or deductive. Counting is a pure function of a closed vocabulary. Nothing on the list needs general recursion, higher-order functions, mutation, or user-defined algorithms.

### 1.2 Verdict and paradigm

**Declarative: yes.** The paradigm has three layers:

1. **Core:** a typed stratified Datalog with priorities, over bitemporal facts.
   - Each `.lit`/layer declaration denotes a set of *claims*: (decl-id, version, valid interval, transaction interval, applicability predicate, authority, scope, value, relations, cite).
   - Resolution is a fixed query over the union of claims.
   - Flix (Madsen, Yee, Lhoták 2016, doi:10.1145/2908080.2908096 [C]) and Datafun (Arntzenius and Krishnaswami 2016, doi:10.1145/2951913.2951948 [C]) show typed Datalog with lattices is practical. Soufflé (Jordan, Scholz, Subotić 2016, doi:10.1007/978-3-319-41540-6_23 [C]) and Doop (Bravenboer and Smaragdakis 2009, doi:10.1145/1640089.1640108 [C]) show it scales. Ascent (Sahebolamri, Gilray, Micinski 2022, doi:10.1145/3497776.3517779 [C]) gives an embedding in Rust.
2. **Values:** a first-order, non-recursive expression language, like Dhall's totality (https://dhall-lang.org/ [F]) but simpler. Counting is a closed combinator algebra (anchor, count, roll, add, chain, clerk), not user code.
3. **Procedure:** a typed 1-safe (or k-bounded) Petri net with discrete clocks.
   - A posture is a marking plus open obligations plus flags. A `.case` is a firing sequence.
   - This replaces "posture = one graph node". Petri nets model linear logic (Martí-Oliet and Meseguer 1991, doi:10.1017/S0960129500000062 [C]; Girard 1987, doi:10.1016/0304-3975(87)90045-4 [C]), which is exactly the resource discipline obligations need (§2.7).

**Not ASP as the core.** Stable-model semantics allows zero or several answer sets, and ASP is NP-complete in general. Law resolution must give one answer or an explicit error. Use clingo (Gebser et al. 2018, doi:10.1017/S1471068418000054 [C]) only as an optional differential oracle for lg-fq2.4 world enumeration.

**Not a general functional language.** Catala is the strongest evidence that the default-with-exceptions core belongs in a small calculus with formal semantics (doi:10.1145/3473582 [C]; Merigoux, Monat, Protzenko, CC 2021, doi:10.1145/3446804.3446850 [C]). Its own limit is that it has no process, events, or agents, and litigate needs all three.

### 1.3 Candidates and what to take

| Candidate | Take | Reject | Evidence |
|---|---|---|---|
| Datalog (stratified, lattices) | Semantic core for resolution, versions, delegation and replay facts; stratification gives a unique model | Untyped Datalog; unstratified negation | Ceri et al. [C]; Van Gelder, Ross, Schlipf 1991, doi:10.1145/116825.116838 [C]; Flix [C] |
| ASP (clingo) | Differential oracle for enumerating counterfactual worlds | As the core: multiple models, NP | Gebser et al. [C] |
| Catala | Default/exception calculus; literate rule text beside code; a semantics proved in a proof assistant | Its lack of process and time | Merigoux et al. [C] |
| Rego/OPA | Evidence that a Datalog-derived declarative policy language is practical, with partial evaluation | Untyped JSON documents as input | https://www.openpolicyagent.org/docs/latest/policy-language/ [F] |
| CUE | Constraints that may only narrow, an order-independent merge (unification is ACI) for the *claim set*, and "delegation as narrowing within bounds" | Unification as the conflict rule: law is not commutative at the value level, since precedence decides | https://cuelang.org/docs/concept/the-logic-of-cue/ [F] |
| Dhall | Totality without recursion; integrity hashes over the *normal form*, which is the model for a semantic digest | Imports by URL | https://dhall-lang.org/ [F] |
| Nickel | Contracts with blame at the untyped JSON boundary (`analysis`, `patch`, imported packs) | Gradual typing inside the core | https://nickel-lang.org/user-manual/contracts [F] |
| LegalRuleML | Separate time axes (enforceability, efficacy, applicability); strict/defeasible/defeater; Override vocabulary | XML | OASIS spec [F]; Palmirani et al. 2011, doi:10.1007/978-3-642-24908-2_30 [C] |
| Akoma Ntoso | FRBR work/expression identifiers as the type of a citation; active/passive modification metadata | Text-edit operations as the semantic model | AKN vocabulary [F]; Palmirani and Vitali 2011, doi:10.1007/978-94-007-1887-6_6 [C] |
| Lean 4 / Agda | Metatheory and an executable reference interpreter for differential testing | Surface language | de Moura and Ullrich 2021, doi:10.1007/978-3-030-79876-5_37 [C] |
| Liquid types, F* | Decidable refinements by predicate abstraction over a fixed qualifier set | SMT where evaluation suffices; F*'s full dependent effects | Rondon, Kawaguchi, Jhala 2008, doi:10.1145/1375581.1375602 [C]; Vazou et al. 2014, doi:10.1145/2628136.2628161 [C]; Swamy et al. 2016, doi:10.1145/2837614.2837655 [C] |
| DML-style indexed types | Indices drawn from decidable domains (authority poset, scope lattice, intervals) | Arbitrary terms as indices | Xi and Pfenning 1999, doi:10.1145/292540.292560 [C] |
| Session types / typestate | Per-party projection of legal moves (agent action sets); typestate for single-track sub-protocols | One global typestate as the posture: litigation is concurrent | Honda, Yoshida, Carbone 2008/2016, doi:10.1145/1328438.1328472, doi:10.1145/2827695 [C]; Strom and Yemini 1986, doi:10.1109/TSE.1986.6312929 [C] |
| Linear / affine types | Obligations are linear, powers are affine, waivers are explicit weakening | User-visible linearity annotations; infer them from rule kind | Girard [C]; Bernardy et al. 2017, doi:10.1145/3158093 [C] |
| Effects / capabilities | File class as an effect row; delegation as a capability with bounds; emergency declarations as capability grants | Ambient authority | Lucassen and Gifford 1988, doi:10.1145/73560.73564 [C]; Dennis and Van Horn 1966, doi:10.1145/365230.365252 [C] |
| Bitemporal databases | Valid time × transaction time, with retraction; SQL:2011 period semantics | A single `known_at` date per version (main today) | Jensen and Snodgrass [C]; Kulkarni and Michels 2012, doi:10.1145/2380776.2380786 [C]; XTDB https://xtdb.com/ [F] |
| Precedent theory | Horty's reason model for the *interpretation* effect class | Factor-based case reasoning as compile-time logic | Horty 2011, doi:10.1017/S1352325211000036 [C]; Horty 2019, doi:10.1007/s10506-019-09245-0 [C]; Horty and Bench-Capon 2012, doi:10.1007/s10506-012-9125-8 [C] |
| Argumentation | Pollock's rebut/undercut split (already adopted); Dung semantics only as an explanation format | Dung extensions as the resolution procedure: they can be multiple | Pollock 1987, doi:10.1207/s15516709cog1104_4 [C]; Dung 1995, doi:10.1016/0004-3702(94)00041-X [C] |

### 1.4 What to give up (each is a current feature or escape hatch)

1. **Semantics that depend on order.**
   - SPEC.md:340-343 makes node order "part of the source" because Monte Carlo visits nodes in order. Renaming a module file then changes analysis results.
   - Transition arms are first-match (SPEC.md:503-506).
   - Same-date events take source order (SPEC.md:596-597; `temporal/transition.rs:83-87`).
   - lg-fq2's ordered layer stack would add a fourth order dependency.
   - Replace them as follows: canonical engine order by id; arms provably disjoint, like exceptions; events ordered by docket sequence; layers as a set.
2. **Untyped JSON escape hatches.**
   - `instance ... patch` JSON merge patches (`lit/links/links.lit:69-79`; `build/logic.rs:455-459` turns numbers into f64 and skips probability checks).
   - `.case analysis { ... }` passthrough (`hir/case.rs:153`).
   - Expression strings such as `"1000000 - payoff"` (`lit/links/links.lit:99`).
   - Give them up or type them. At minimum, put them behind Nickel-style contracts.
3. **Semantics by event name.** `Struck`, `Forfeited`, `Withdrawn`, `Waived`, `Extended` and `Excused` are recognized by spelling (SPEC.md:607-611; `case/graph.rs:18`; `replay/removal.rs:99`). Make them declared effect kinds.
4. **Edge references as strings.** `Forfeited("e-filingfork-protest")` (`option: String`, SPEC.md:610).
5. **Citations as free text.** `cite` swallows the rest of the line and stops at a comma (`litigate-syntax/src/lexer.rs:317`). About 20 cites in `lit/links/links.lit:117-175` are holes for that reason alone.
6. **Structured data in prose `note`s.** Examples: "basis: estimate" (`lit/ptab/01-pre-petition-to-institution.lit:154`), fee schedules (`:60`), "historically 40" (`lit/ttab/07-opposition-to-trial.lit:30`).
7. **Mixed decision/chance nodes and `p` on any edge** (SPEC.md:398-402; `lit/ptab/01-pre-petition-to-institution.lit:149`). A node is a choice or a draw, never both.
8. **The semver pretense for legal packages.** Nothing defines "breaking" (§2.12).

**Tradeoffs.**
- A closed counting algebra means a new forum quirk is a compiler change. TBMP 112 chaining from the unadjusted date and ITC's first-business-day anchor are the current examples. Mitigate by making the step set small, versioned, and declared in the IR, not in engine enums.
- Forbidding order dependence means rewriting about ten existing scenario goldens once, and changing litgraph to visit nodes by canonical order.
- Typed boundaries mean `cases/requests/*` must spell units (`params: { rate: 650 }` today).
- Each cost is one-time. The order and typing fixes are breaking later and cheap now.

---

## 2. Type system: the strongest decidable design I recommend

Design rule: **indices come from finite or decidable domains; everything closed is evaluated, not proved; a solver sees only open formulas in QF_LIA, QF_LRA and finite enums; no quantifiers; no user recursion.** Every judgment below is checked in polynomial time, except the propositional checks, which are NP but over a handful of atoms (§2.9).

Notation: `T[i]` means type `T` indexed by `i`.

### 2.1 Units, calendar periods, instants

**Design.**
- `Period[U]`, where `U ∈ {CalendarDay, CourtDay, BusinessDay, Hour, Week, Month, Year}`. These units do not convert into one another. `Date ⊕ Period[U]` exists only through a `Counting[U, Dir]`.
- Month and year arithmetic needs an explicit end-of-month policy (`clamp`, `roll`), declared by the counting rule.
- `Instant[Zone]` represents filing timestamps.
- `deadline(d)` returns `Due = { nominal: Date, adjusted: Date, ends_at: Instant[Zone], trace }`, not `Date`.
- Money gains `Rate[Money/Hour]` so that `params: { rate: 650 }` gets a unit.

**Bugs eliminated.**
- 9 months encoded as `270.days`: `lit/ptab/deadlines.lit:54`. 35 U.S.C. § 321(c) says "9 months after the date of the grant" [F]. The test at `:59-60` expects 2025-12-01. Nine calendar months from 2025-03-04 is 2025-12-04, so the test enshrines a wrong date.
- "Three months after" in 37 C.F.R. § 42.107(b) [F], encoded as 90 days at `lit/ptab/deadlines.lit:120-121`. The test expects 2025-06-02; three months gives 2025-06-04.
- The one-year 315(b) bar as 365 days (`lit/ptab/deadlines.lit:75`), and the year in FRCP 60(c)(1) as 365 days (`lit/frcp/periods.lit:126`).
- The 6-year § 2501 tolling cap as `2190.days` (`lit/cofc/01-threshold.lit:39`).
- About ten MPEP month periods (`lit/mpep/deadlines.lit:92-366`).
- FRCP 6(a)(4) "midnight in the court's time zone", and the Eastern-time rules (RCFC, Fed. Cir. R. 26 [B]), which cannot be stated today.
- TTAB chaining "from the actual due date ..., not the adjusted date" (TBMP 112 [B]). Requiring an explicit `.nominal` or `.adjusted` on every chain removes the ambiguity.

**Check.** Ground phantom-index equality, the degenerate case of Kennedy's dimension types (doi:10.1007/3-540-57880-3_23 [C]; doi:10.1007/978-3-642-17685-2_8 [C]). The units do not form a multiplicative group here, so no unit inference is needed and checking is linear.
- The lexer already rejects unknown suffixes.
- Add a lint: compare the declared unit with the unit word in the resolved source span ("months", "year"). The source text is already vendored.

**Cost.**
- Ergonomics: low. `9.months` reads like the statute.
- Error messages: excellent. "`270.days` but § 321(c) states `9 months`; replace with `9.months` (machine_applicable)".
- Breaking: yes. The lexer closed set (`lexer.rs:72`), `deadline() -> Date` (`builtins.rs:68`), and the lowered engine clock (no month step) all change.

### 2.2 Refinements: ground first, SMT second

**Observation.** Today every quantity lg-962.3 lists is a closed term:
- `p` literals, `Dist` weights (forced constant at `types/shape.rs:351`), interval endpoints, deadline amounts, and version windows.
- Exact rational evaluation already decides them, so an SMT call would only re-prove arithmetic.

**Design.**
- Refinement types `{x: B | φ}`, with φ drawn from a qualifier grammar: linear constraints over Rat, Int and Date-as-Int; enum membership; and boolean structure.
- Discharge in three tiers:
  1. **Ground evaluation** for closed terms. No solver.
  2. **Exhaustive enumeration over the Gregorian cycle** for properties quantified over dates, such as "for all trigger dates, the due date is not a closed day" or "the window intersection is non-empty".
     - The Gregorian calendar repeats every 146,097 days, and 146,097 ≡ 0 (mod 7). I checked this by computation.
     - Weekday-based and fixed-date holiday rules are therefore periodic with that period, except for finitely many `once` dates and effective-interval boundaries.
     - Enumerating one cycle plus the boundary segments decides the property for all dates. That is about 1.5e5 evaluations per rule-version pair.
     - This replaces DESIGN §7's Kani plan for date arithmetic with a complete decision procedure.
  3. **SMT** (QF_LIA/QF_LRA plus enums, no quantifiers) only for open terms:
     - parameterised values, e.g. FRCP 12(a)(1)(A): 21 days, 60 if a waiver was sent, 90 if outside the US;
     - delegation bounds;
     - "exists a world" queries.
     - Presburger and LRA are decidable. Simplex-based LRA is the practical engine (Dutertre and de Moura 2006, doi:10.1007/11817963_11 [C]). Z3 (doi:10.1007/978-3-540-78800-3_24 [C]) and cvc5 (doi:10.1007/978-3-030-99524-9_24 [C]) are both fine.

**Bugs eliminated.**
- Non-constant probability arithmetic escapes the [0,1] check. `default c when z => a + b` with a=0.7 and b=0.6 is accepted (`types/ops.rs:152-156, 201-210`).
- Delegated extensions that exceed their bound (§2.4).
- A `duration { max }` that exceeds its `within` window. This affects about 170 hand-typed triples, e.g. `lit/frcp/02-pleading-to-trial.lit:42`.

**Cost.**
- Tiers 1 and 2: none for the user.
- Tier 3: an error must render as a counterexample: "fails when service = Mail and waiver_sent = true: 63 > 60".
- Determinism: forbid timeouts and use resource limits. Cache results by query hash plus solver version, and store unsat certificates in the litigatable so LSP and wasm can check results without the solver.

### 2.3 Indexed rule types

**Design.** `Rule[a: Authority, s: Scope, v: ValidInterval, k: TxInterval, ap: Applicability]`.
- `Scope` is a lattice: Nation ⊒ Circuit ⊒ District ⊒ Court ⊒ Judge ⊒ Case, plus agency forums.
- `Applicability` is a predicate over typed proceeding attributes. The forum-specific version keys [B] are:
  - REA amendments: the proceeding's commencement date;
  - PTAB: the petition filing date (83 FR 51340);
  - ITC: the institution date (90 FR 225).

**Bug eliminated.**
- Today a transition fires only when "the matter's first event" precedes the amendment (SPEC.md:494-498; `temporal/transition.rs:69-73`).
- For a matter that spans proceedings (a district action plus a later IPR joined by `links`), the first event is the complaint. The PTAB 2018 claim-construction rule would then be judged against the district filing date, not the petition date.
- A judge-scoped standing order overriding a declaration in another judge's case is not rejectable today: `delegable_to` names a level, not an issuer (`precedence/delegation.rs`).

**Check.**
- Finite-poset reachability for `a`, lattice containment `s ⊑ s'`, and interval intersection: all polynomial.
- `ap` predicates are propositional over enums plus date comparisons, so they are ground per case and QF_LIA when checked generically.

**Cost.** Low. Indices default from the manifest and the enclosing rule. Rule-level `authority` is required, because `cx.rs:377` hard-wires `meta.authority = manifest.authority` although SPEC §5.5 says "its rule's or package's".

### 2.4 Authority and delegation as typed capabilities

lg-962.4 plans `Override<target>` with a proof that the overrider's authority is in `target.delegable_to`. That is not enough. Real delegations have five parts:

```
deadline notice-of-appeal: 30.days after Entered(Judgment) counted_by frap26
  characterization jurisdictional cite 28 U.S.C. § 2107(a); Bowles v. Russell, 551 U.S. 205
  delegable {
    to Court op extend bound <= max(due + 30.days, order + 14.days) when motion_within(due + 30.days)
       cause ExcusableNeglect | GoodCause                    cite FRAP 4(a)(5)
    to Court op reopen bound 14.days after order when no_notice_within(21.days) cite FRAP 4(a)(6)
  }
  lock against [LocalRule, StandingOrder, Stipulation]        cite FRAP 26(b)(1)
```

**Design.**
- `Cap[op, target, principal, bound, condition, procedure, window]`.
  - `op ∈ {extend, shorten, set, suspend, waive, compute_method, add_closed_day}`.
  - `principal` is an issuer type: Court(c), Judge(j), Clerk, Parties-by-stipulation, Board, Director, JudicialConference.
  - `bound` is a refinement.
  - `condition` is a cause standard. It lowers to a `chance … discretion` node.
  - `procedure` holds preconditions such as "motion filed before expiry".
- Locks are negative capabilities. FRCP 6(b)(2), RCFC 6(b)(2), FRCrimP 45(b)(2) and FRAP 26(b) differ by forum [B].
- An emergency declaration is a capability grant with a ≤90-day window: FRCP 87(c)(2) unlocks 6(b)(2) items for ≤30 days [B].
- **Every** relation that changes a target's effect needs a capability: `override`, `notwithstanding`, `supersedes`, `waive`, and precedent `waiver`. Today only `override` is checked (`precedence/delegation.rs:51-52`; see P2).
- Add `characterization ∈ {jurisdictional, claim_processing}` as an index:
  - Jurisdictional deadlines admit no waiver, forfeiture or equitable tolling (*Bowles* [F]).
  - Claim-processing deadlines can be forfeited by the opponent (*Hamer v. Neighborhood Housing Services*, 583 U.S. 17 (2017): "a time limit prescribed only in a court-made rule is not jurisdictional. It is a mandatory claim-processing rule that may be waived or forfeited" [F]).
  - Most statutory bars are nonjurisdictional absent a clear statement (*Boechler v. Commissioner*, 596 U.S. 199 (2022) [F]; *Harrow v. Department of Defense*, 601 U.S. 480 (2024): § 7703(b)(1)'s 60-day deadline "is not jurisdictional" [F]).
  - The index changes replay: missing a claim-processing deadline creates an opponent `object | forfeit` move, not a hard L4002.

**Bugs eliminated.**
- *Bowles*-type over-extension, which main accepts (P3).
- `notwithstanding` from a CaseOrder beating a FederalRule with no delegation (P2).
- `delegable_to [CaseOrder]` on the cert period (`lit/frap/11-cert.lit:20`). The real extender is a Justice under 28 U.S.C. § 2101(c), up to 60 days [K].
- `lit/frcp/periods.lit:130`: a jurisdictional notice-of-appeal period marked plainly delegable.
- Precedent recharacterizes a deadline. *Harrow* reversed the Federal Circuit's jurisdictional treatment, which is a precedent effect on a *type index*. None of the four lg-fq2.1 effect classes names this; see §3.4.

**Check.**
- Capability derivation is a Datalog query: `holds(cap) ← declared(target, cap) | granted(emergency, cap) ∧ window ∋ t`.
- Bounds are tier-1 (ground per case) or tier-3 (generic) refinements.
- No amplification: a derived capability is never wider than its source. This is a Lean lemma (§3.7).

**Cost.** Medium. Each delegating declaration needs about three lines. Errors are very good, for example "extension to 2026-03-20 exceeds FRAP 4(a)(5)(C) bound max(due+30, order+14) = 2026-03-12".

### 2.5 File classes as capabilities (lg-962.5, lg-fq2.1)

**Design.** Each declaration form has an effect: `Declare`, `Version`, `Override`, `Waive`, `Interpret`, `Recharacterize`, `Invalidate`, `SetPrior`, `AddFact`, `GrantCap`. Each instrument class has an effect row. Checking is set inclusion, which is linear.

Corrections to the plan:
1. **A kind must be justified, not self-declared.**
   - `kind = "precedent"` in `Lit.toml` proves nothing. A file must cite its instrument (issuer, date, designation).
   - Binding scope is computed from the court hierarchy: a circuit binds its districts; a PTAB precedential designation binds the Board after its designation date, with a portion scope [B].
   - Otherwise a typo makes persuasive precedent binding.
2. **`hypothetical` is a modality of a world, not a file class.** The research comment says as much: "`hypothetical` and `world` have no legal source" [B].
   - As a kind it cannot express "the world without this real local rule".
   - Make `world.modality ∈ {actual, counterfactual}`. Any layer may appear in a counterfactual world. Strict and real-case modes reject counterfactual worlds. Synthetic what-if layers carry `authoritative = false`.
3. **Bindingness is relative to an addressee.**
   - Director memoranda bind PTAB judges but not parties; the MPEP, TMEP and TBMP bind no one [B].
   - Type it as `Guidance[binds: Set<Actor>]`. Its permitted effect is to constrain chance nodes whose actor is in `binds` (Board discretion probabilities) and never party duties.
4. **COFC has no local-rule layer**, and ITC has ALJ ground rules [B]. The class set is per forum family, so the kind index is `Kind[forum_family]` and the matrix is data in `lit/core`, not compiler code.

**Cost.** Low for authors (one header). The error explains why the class cannot do X and cites the delegation that would permit it.

### 2.6 Temporal and bitemporal typing

**Design.**
- Every claim carries valid time `[from, to)` and transaction time `[recorded, retracted)`.
- A correction is a new claim with the same valid window and a later `recorded`. The old claim gets `retracted`.
- Disjointness is checked per transaction-time slice, not across all claims.
- The typing context carries `(t_v, t_k, proceeding)`. A reference resolves only to claims visible in that context, so a `.case` with `known_at 2016-01-01` cannot see a 2020-recorded correction.

**Bugs eliminated.**
- Main cannot record "the corpus said X for [a,b), later corrected to Y". That is L3001, because "Disjointness ignores `known_at`" (SPEC.md:482-483; `temporal/version.rs:109-128`). The `known_at` axis is therefore cosmetic.
- Fake start dates: `[2024-12-01, ..)` on rules that have stood since 2009 (`lit/frcp/periods.lit:13-19, 73`). A case before 2024-12-01 silently has no answer deadline.
  - Fix with tx-time honesty plus an `unknown_before` marker. A query earlier than the earliest known version should be the hole "law not encoded before D", not "no version in force" (L3007).

**Check.** Interval algebra (Allen 1983, doi:10.1145/182.358434 [C]) over sorted endpoints, O(n log n).

**Cost.** Low for authors. `recorded` defaults from the commit or lock. Retraction is written only for corrections.

### 2.7 Linearity: obligations, powers, events

**Design** (Hart's kinds become substructural kinds):
- **Obligation tokens are linear.**
  - `Obl[party, act, due: Due, char]` is created by a trigger firing. It must be consumed exactly once, either by a matching act by `party` or by a `breach` transition (a liability).
  - An excusal or extension is a typed transformation `Obl → Obl'` that must hold a capability (§2.4).
- **Powers and options are affine.** They may go unused. `Struck` or `Forfeited` is explicit weakening.
- **Events have identity** (docket number or sequence), so one event cannot discharge two obligations unless a rule says so.
- **Trigger semantics are explicit**: `first | latest | each | later_of(a, b)`. Main hard-codes "most recent" (SPEC.md:426-427; `case/deadline.rs:331-334`).
  - FRCP 15(a)(3) needs `later_of(14 days after amended pleading, remaining time)`.
  - The FRAP 4(a)(3) cross-appeal needs `later_of(14 days after first notice, Rule 4(a) time)`. `lit/frap/03-docketing.lit:10` admits the combination is missing.

**Bugs eliminated.**
- One defendant's answer discharging another's deadline. Nothing ties a `within` check to the party bearing it.
- A second trigger silently restarting a period.
- Duties never acted on are never reported. Nothing evaluates open deadlines at `as_of`, so SPEC §5.5's "a duty requires its deadline to be met" is enforced only for events that occur.
- `ExtendedPeriodExpired()` asserted as an event although the rules compute it (`cases/ttab-opposition-applicant-defense.case:21-23`). With obligation tokens, expiry is derived.

**Check.**
- On a trace: counting, linear time.
- On the net: place invariants by linear algebra (rational LP, polynomial) prove "every obligation created on any path is consumed on every path to a terminal" for bounded nets.

**Cost.**
- Authors write nothing new, because linearity follows from the rule kind (`duty` = linear, `power` = affine).
- Error example: "obligation `answer(widget)` created by Served on 2025-03-05 is neither discharged nor breached at as_of 2025-06-01".

### 2.8 Procedure typing: markings, not a single posture

**Critique of lg-962.4 "Event : Posture → Posture".** A single posture index cannot represent:
- a pending motion to dismiss while discovery runs;
- several defendants on different clocks;
- a parallel IPR and district action (`links`);
- `lit/frcp/02-pleading-to-trial.lit:193-215`, where `rule12-response-window` is entered from five tracks whose real clocks differ (60/90 days on waiver, 81(c) on removal, 12(a)(4)(A) after denial), yet every out-edge says `within answer-period` (21 days). The node's own note concedes it.

**Design.**
- A posture is a marking: a multiset of typed places, plus open obligation tokens, plus flags.
- An event's type is a transition `pre-pattern ⇒ post-pattern`.
- A `.case` is well-typed iff its events form a firing sequence.
- Per-party projection, in the style of multiparty session types (Honda et al. [C]), gives the agent action sets that DESIGN §10 promises.

**Bugs eliminated.**
- The "timing-only" hatch accepts any event matching any trigger or transition pattern at any posture, terminals included (SPEC.md:598-600; `case/replay.rs:370-394`). Example: `PatentGranted() on owner` in `cases/ptab-ipr-defense.case:23-24` is never posture-checked.
- Fake chance routers such as `lit/cafc/09-decision-to-rehearing.lit:99-104`, which exist "only because the schema requires probabilities to sum to 1". Origin becomes a token colour, not a draw.

**Check.**
- Trace membership: linear.
- Workflow soundness (option to complete, proper completion, no dead transitions) for bounded nets (van der Aalst [C]).
- `graph-exits` and `graph-reachable` become guard-aware. Today they ignore `requires`/`forbids` (`litigate-verify/src/graph.rs:22-110`).

**Cost.** Medium. Authors keep writing nodes and edges. Concurrency appears only where a rule creates a parallel track (`spawn` and `join`). The lowering to the litgraph engine needs a product construction or an engine extension, and that is the real price.

### 2.9 Exhaustiveness

**Requirements.**
- Total maps over enums wherever the law is a total function:
  - `service_extension` must cover every `ServiceMethod`, with `0.days` where none applies.
  - FRCP 6(d) 2009-2016 must say `Electronic: 3.days` and the 2016 version `Electronic: 0.days`. Today `Electronic` exists in `lit/core/time.lit:20`, but `lit/core/frcp6.lit:52-58` omits it "because the engine has none" (P8).
- Transition arms must be provably disjoint and exhaustive, the same propositional check as exceptions, instead of first-match.
- Exception disjointness must use mutual-exclusion axioms between `has_event` atoms of the same enum parameter. Today atoms are independent, so `Served(Mail)` and `Served(Personal)` look co-satisfiable, which gives false L3005 (`temporal/condition.rs:155-175`).
- Stakes, facts and `Dist` (already exhaustive).
- Information sets: every decision node names what its actor observes, and nodes in one information set have identical action sets.

**Check.** Enum totality is linear. Arm and exception disjointness is SAT over at most 16 atoms today; keep a real SAT or BDD above that rather than "give up conservatively".

**Cost.** Low. Missing-arm fixes already exist (`types/shape.rs:233-254`).

### 2.10 Provenance and citation typing

**Design.**
- `Cite` is a structured value: `Cite[Work, Expression(as_of), Pinpoint]`, Akoma Ntoso FRBR style.
  - It is parsed by a cite grammar, not by line-rest lexing, so commas, `§§`, case names and URLs work.
  - A version's cite must resolve to an expression whose date lies in the version's valid window, or is explicitly historical.
  - `lit/core/frcp6.lit:9` cites a 2008 edition for a 1963 rule version, and the type should say so.
- Each resolved cite stores `sha256` of the cited span text. Today `[sources].sha256` is shape-checked and copied but never verified (`pkg/manifest.rs:385-394`; `litigate-lower/src/pack.rs:65-77`). Cites resolve against a corpus embedded in the binary regardless of `as_of`.
- Every value carries a `Basis` label in a lattice: `Sourced(cite) < Calibrated(dataset, as_of) < Expert(name) < Estimate < Hole`.
  - Labels propagate by join through every analysis, as in information-flow typing (Denning 1976, doi:10.1145/360051.360056 [C]).
  - `strict` requires no `Estimate` on any output the user asks for, or the output is labelled. Today "teaching estimate" lives in prose (`lit/frcp/02-pleading-to-trial.lit:253`).

**Check.** Cite parsing is a grammar. Label propagation is linear.

**Cost.**
- Medium: about 329 existing holes need structured cites. The win is that `cite-resolves` becomes meaningful, and agents get a quote hash to verify against.
- Breaking: yes, because the lexer's cite mode goes away.

### 2.11 Probability and decision typing

**Design.**
- Keep exact `Probability` and `Dist[E]`.
- Add `Dist[E | C]`, conditional on facts, judge or forum, and total over `C`.
- Add `Credal[lo, hi]` and `Beta(α, β)` for epistemic uncertainty. The `beliefs { observe … }` counts are Dirichlet pseudo-counts. Type them as such and declare the prior they update, instead of mutating an authored point probability.
- Node kinds: `decision` (a choice by actor), `chance discretion` (aleatory draw by a tribunal), `chance nature`, and `fact` (an unknown but fixed matter fact, drawn once per run).
  - **Derived facts must be functions, not probabilities.** `ptab::ipr-time-barred p 0.03` (`lit/ptab/01-pre-petition-to-institution.lit:167`) is computable from `ComplaintServed` and the petition date, which `cases/ptab-ipr-defense.case` already supplies.
- Mixed nodes are forbidden (§1.4).
- Money: `Money[USD]` stays integral. Add `Rate` and `Params` types for `analysis`.
- Also make probabilities whose label disagrees with their node a type error. `lit/ptab/01-pre-petition-to-institution.lit:146-160` labels `p 0.97` as "meets fee/word-count/form" while the comment calls it the not-time-barred prior.

**Check.**
- Dist totality and sum: ground, exact.
- Credal operations are interval arithmetic. Using a `Credal` where a point is required needs an explicit `lower`, `upper` or `mean` projection (a decision rule).

**Cost.** Low to medium. This is where analysis quality comes from, and it is CAN-ADD-LATER except for the node-kind split.

### 2.12 Versioning and compatibility of packages

**Problems.**
- `^1.2.3` has no defined meaning (no taxonomy anywhere in the SPEC).
- Legal time is already in-language, so package semver only versions the *encoding*.
- The lock is single-version per name and not Merkle (`pkg/lock.rs:158,172`).
- Resolution takes the first matching package in load order (`types/build/package.rs:54-61`).
- The digest is over raw bytes, so reformatting churns the lock.
- Node ids are bare kebab strings, so a rename breaks every `.case`.

**Design.**
- **Identity:** each declaration has a stable work id (`frcp/6/a/1/C` style) independent of its lowered node id, plus `formerly <id>` for renames.
- **Two digests:**
  - a *source* digest (bytes);
  - a *semantic* digest of the canonical, span-free HIR, the Dhall approach [F]. Today `content_hash` includes spans (`litigate-cli/src/cmd/build.rs:91-107`).
- **The lock is Merkle.** Each row holds the dependency digests and source-text digests.
- **Compatibility is computed, not declared.** `litigate compat A B` classifies a diff as:
  - `interface` (removed or renamed ids, changed event signatures, changed types);
  - `value` (any computed due date or probability changes on the package's own tests or on a pinned corpus of cases);
  - `none`.
  
  Encoding releases bump accordingly, after Elm's enforced semver (https://package.elm-lang.org/help/design-guidelines [F]) and cargo-semver-checks (https://github.com/obi1kenobi/cargo-semver-checks [F]).
- `litigate update` prints a per-matter deadline diff (library-ideas.md L1).
- Use PubGrub for resolution (https://pubgrub-rs.github.io/pubgrub/pubgrub/ [F]).

**Cost.** Low for authors. Breaking: lock format v2, plus `uses` requirements that mean compatibility classes.

### 2.13 What not to do

- **No general dependent types.** lg-962 already rules this out; keep it that way.
- **No user-defined recursion or higher-order functions.**
- **No gradual typing inside the core.**
- **No SMT for ground facts.**
- **No solver timeouts.**
- **No refinement syntax `{x: T | p}` in v1 surface code** unless it is needed: the lexer lacks `|`, and `n-1` lexes as one identifier (`lexer.rs:44`). Put qualifiers on declared types (`type Extension = Period[Day] where <= 30.days`) instead.

### 2.14 Summary

| Feature | Bug class removed | Decision procedure | Agent/lawyer cost | Error quality |
|---|---|---|---|---|
| Period units incl. months/years; `Due` | Month-as-days; time-zone cutoffs; chaining from the wrong date | Phantom equality | Low | Exact fix-its |
| Ground refinements | Out-of-range p, sums, intervals | Exact evaluation | None | Exact |
| Gregorian-cycle enumeration | "For all trigger dates" counting properties | Enumerate 146,097 days per segment | None | Counterexample date |
| SMT refinements (open terms only) | Parameterised values; delegation bounds | QF_LIA/LRA, no quantifiers | Low | Counterexample model |
| Rule[a, s, v, k, ap] | Wrong-scope override; wrong applicability key | Poset, lattice, intervals | Low | Names the missing index |
| Delegation capabilities + characterization | Bowles over-extension; notwithstanding bypass; waiver of jurisdictional limits | Datalog derivation + refinements | Medium | Cites the bound |
| File-class effect rows | Persuasive precedent changing rule text; standing order touching a non-delegable rule | Set inclusion | Low | Names the class and the effect |
| Bitemporal claims | Lost corrections; time-travel in replay | Interval algebra | Low | Names the hidden claim |
| Linear obligations / affine powers | Double discharge; wrong-party discharge; silent missed duties | Trace counting; P-invariants | None (inferred) | Names the token |
| Markings, not postures | Concurrency collapse; timing-only hatch | Firing-sequence check; workflow soundness | Medium | Shows the enabled set |
| Exhaustiveness (total maps, arms, info sets) | e-service gap; first-match arms | Enum totality; SAT | Low | Missing-arm fix |
| Structured cites + Basis labels | Unverifiable citations; estimates passed off as data | Grammar; label join | Medium | Quote hash mismatch |
| Computed compatibility | Silent deadline changes on update | Semantic diff | Low | Per-matter diff |

---

## 3. Semantics

### 3.1 Domains

- `Date` = ℤ, counted as days since an epoch.
- `Instant` = ℤ minutes, plus a `Zone`.
- `Calendar` = `Date → {open, closed(reason)}`, versioned.
- A *claim* is a tuple `c = (id, ver, kind, auth, scope, [v₀, v₁), [k₀, k₁), ap, val, rels, cite, origin-layer)`.
- A *layer* `L` is a finite set of claims, plus instrument metadata (class, issuer, date, cite).

### 3.2 World

- `W = (Λ, ι, μ)`.
  - `Λ` is a finite **set** of layers.
  - `ι` is the location index (forum family, court, circuit, judge). It is needed because precedent binds per circuit; the lg-fq2.3 corpus has three-way circuit splits on FRAP 4(a)(6) notice (`sources/change-corpus/rulings/README.md:61-71`).
  - `μ ∈ {actual, counterfactual}`.
- A query is `(W, t_v, t_k, π)`, where `π` is a proceeding with typed attributes.
- `as_of` and `known_at` are query coordinates, not world fields.
- This differs from lg-fq2's ".world = ordered layers + as_of + known_at". Putting dates in the world forces one file per date and invites order semantics.

### 3.3 Denotation: stratified Datalog

`⟦W⟧(t_v, t_k, π)` is the unique stratified model of these rules, written in Datalog syntax:

```
S0  claim(c)          ← c ∈ ⋃Λ
S1  visible(c)        ← claim(c), v₀(c) ≤ t_v < v₁(c), k₀(c) ≤ t_k < k₁(c), ap(c)(π), binds(scope(c), ι)
S2  cap(x, op, d, b)  ← declares_cap(d, x, op, b) | grant(e, x, op, d, b), visible(e)   ; §2.4
    licensed(c)       ← visible(c), ∀ target d of c: auth(c) ≥ auth(d) ∨ ∃ cap(issuer(c), op(c), d, b) ∧ within(val(c), b)
    unlicensed(c)     ← visible(c), ¬licensed(c)                        → diagnostic, never silent
S3  defeats(c, c')    ← licensed(c), licensed(c'), declared_win(c, c')  ; notwithstanding / override / subject_to
    defeats(c, c')    ← … canon(c, c') if comparable(auth(c), auth(c'))   ; incomparable: no canon
S4  winner(d, c)      ← claims_on(c, d), licensed(c), ¬∃ c'. defeats(c', c) ∧ licensed(c')
    conflict(d, c, c') ← two winners for d with different values      → L3003 error, or a fork under §3.4
S5  value(d) = val(c) for the unique winner(d, c)
```

- **Stratification:** `defeats` uses only S2 facts, and `winner` negates `defeats`. Cycles in declared wins are rejected statically, which is the existing precedence-acyclic check.
- **Complexity:** polynomial in the number of claims (Ceri et al. [C]). Defeasible exceptions within one authority compile to S3/S4 with priorities, as in Catala's default calculus.

**Canons that must change from main:**
- "Specificity" must be subject-matter specificity (antecedent entailment or declared `specific_to`), not window width. `precedence/conflict.rs:121` uses window width, so a one-year emergency order beats an open-ended rule of the same rank on length alone.
- Incomparable authorities must not fall through to canons (`conflict.rs:96-100`). `lit/core/authority.lit:8-11` states that intent.

### 3.4 Change-layer algebra

**Definition.** `merge(Λ) = ⋃Λ` on claim sets, keyed by content hash.

| Property | Holds? | Why |
|---|---|---|
| Associative | yes | Set union |
| Commutative | yes, and must hold | The legal effect of two instruments does not depend on which file a manifest lists first. Order *inside the law* (enactment dates, authority, `supersedes`) is data on the claims |
| Idempotent | yes | Loading a layer twice is a no-op. Edit-script layers (insert/delete) are not idempotent, so do not represent amendments as edit scripts; Akoma Ntoso's active modifications stay *metadata* about text |
| Monotone in claims | yes | `⋃` is monotone |
| Monotone in conclusions | **no** | Resolution is defeasible: adding a layer can retract a conclusion. That is correct, and the explanation must say which claim defeated which |

**Conflict resolution.** Neither a total order nor silent. The precedence relation is a strict partial order, and two undefeated incompatible claims are an explicit error. The exception is a declared *contested* question:
- When the binding scope for `ι` contains no controlling precedent and a circuit split exists, the conflict lowers to a `chance discretion` node.
- Its prior is labelled by basis. For example, a district in a circuit with no FRAP 4(a)(6) holding gets a fork over readings A, B and C.
- Legal uncertainty becomes game uncertainty, which is litgraph's purpose.

**Precedent effect classes, typed as operations on claims:**
- `interpret(d, reason-priority)`: Horty's model [C]. Adds a priority constraint between reasons.
- `recharacterize(d, attr, value)`: new. For example *Harrow* changes `characterization` of 5 U.S.C. § 7703(b)(1) to claim-processing.
- `invalidate(d, mode)`, with `mode ∈ {annul, repeal}`.
  - **Default for judicial decisions is retroactive to open cases**, citing *Harper v. Virginia Dep't of Taxation*, 509 U.S. 86 (1993) [B]. Annul versus repeal is a real choice only for rules and regulations.
  - The plan's free annul/repeal choice for precedent is wrong.
- `waive(d, bound)`: needs a capability, e.g. 37 C.F.R. § 42.5(b) for PTAB *Light & Wonder* [B].
- `evidence(node, Δprior)`: the only effect allowed to persuasive authority.
- **Overruling** = `retract` of the precedent's claims in transaction time **and** valid time, from the overruling date. Retroactivity follows the same default.
- A world before an overruling replays to the outcome decided at the time. That is lg-fq2.5's test, and it requires the bitemporal claims of §2.6.

### 3.5 Case semantics (small-step)

```
config σ = (M, O, F, H)   marking, open obligations, flags, history
⟨σ, e@t⟩ → σ'  iff  e is enabled in M under ⟦W⟧(t, t_k, π), e.by holds a matching power or obligation,
                    O' = O − {discharged by e} ∪ {created by triggers of e, Due computed under ⟦W⟧(trigger time)},
                    every o ∈ O with o.due < t and not discharged → breach(o) is forced first (liability), unless a
                    licensed capability transformed o
```

- Events are totally ordered by (date, docket sequence), not by source order.
- The counting version is selected at the **trigger** date by default, overridable by a declared transition.
- Main is inconsistent on this. Replay uses the judged event's date (`case/deadline.rs:336-347`); the CLI test path uses the trigger date and ignores supersession and transitions (`litigate-cli/src/clock.rs:74-102`); lowering uses `case.as_of` (`litigate-lower/src/lib.rs:157-168, 230`).

### 3.6 Counting semantics

A counting rule is a pipeline of steps in a closed algebra:

`anchor(kind) ; count(n, unit, short_period?) ; roll(dir, closed-set(dir)) ; base ; add(service-map) ; roll ; clerk(inaccessible) ; ends_at(zone, medium)`

The closed set depends on direction:
- FRCP 6(a)(6)(C): state holidays only for periods counted forward [F] (`notes/frcp6.txt`).
- FRAP 26(a)(6)(C) uses a different state [B].
- 6(a)(6)(B): days declared by the President or Congress.
  - Executive closure orders exist for 2018-12-05, 2018-12-24, 2019-12-24, 2020-12-24, 2024-12-24, 2025-01-09, and 2025-12-24/26 (Federal Register API query, [F]).
  - Whether a closure order is a 6(a)(6)(B) "declared holiday" is a legal question the calendar must declare per forum, with a cite.
  - `lit/core/calendar.lit` has none of them and cannot express the question.

Every step is total and bounded:
- `roll` terminates iff at least one weekday is open. That is a static check; `litgraph/src/clock/rule.rs:316` does not make it, and `interp.rs:66-89` loops forever otherwise.
- `count` is bounded by n.

### 3.7 What to prove in Lean later

1. **Uniqueness:** for an acyclic declared-win graph, S0-S5 has a unique model.
2. **Permutation invariance:** `⟦merge(Λ)⟧` is independent of any enumeration order of Λ, of module file order, and of declaration order.
3. **Checker soundness:** if `check` reports no L3003/L3004, then for every `(t_v, t_k, π)` in the checked horizon each target has exactly one licensed winner.
4. **No amplification:** every licensed claim has a capability derivation rooted in a base declaration or an emergency grant, and the derived bound is never wider.
5. **Bitemporal monotonicity:** for fixed `t_v`, the visible set changes only at recorded or retracted instants, and replay at `t_k` reads only claims with `k₀ ≤ t_k`.
6. **Counting:**
   - The result is never a closed day.
   - It is monotone in n.
   - Service days are added after the roll, per the 2005 amendment to 6(e) [K].
   - The backward/forward roll duality holds.
   - The Gregorian periodicity lemma (146,097 ≡ 0 mod 7) justifies exhaustive enumeration.
7. **Lowering correctness by translation validation:** for each built pack, the engine clock IR computes the same `Due` as the reference interpreter on the enumerated domain.
8. **Replay progress and preservation:** a well-typed configuration either steps, terminates, or reports a coded error. Linearity holds: each obligation token is consumed at most once.
9. **dist-exact after lowering:** the exact sum is 1, and the binary64 sum is within n·ulp of 1. Main never re-checks after rounding (`litigate-lower/src/num.rs`).

Lean 4 [C] can also host the reference interpreter used as a differential oracle (§4.7).

---

## 4. Compiler architecture

### 4.1 Queries

Keep salsa. It is 0.28.5 per `Cargo.toml:30`; SPEC.md:29 still says 0.26. The query design needs changes:
- **A position-independent item tree.** HIR embeds line/col `Span`s (`hir::Expr.span`), so a whitespace edit invalidates every downstream query.
  - Store an AST pointer in the item tree, and map spans in a side table, as rust-analyzer does (https://rust-analyzer.github.io/book/contributing/architecture.html [F]).
  - Ground the incrementality story in Mokhov, Mitchell, Peyton Jones 2018, doi:10.1145/3236774 [C], and Adapton, doi:10.1145/2594291.2594324 [C].
- **Granularity:**
  - per declaration for typing;
  - per (declaration, world-slice) for resolution;
  - per (case, world) for replay.
  - Today `check_module` is per file and `check_session` is not cached at all. The LSP re-reads disk and re-replays every case on each keystroke (`litigate-lsp/src/host.rs:204-227`).
- **The world is an interned key.** `syntax`, `item_tree` and `signature` stay world-independent. Resolution queries take `(decl, WorldId, t_v, t_k)`.
- **Durability:** stdlib layers HIGH, workspace layers LOW.

### 4.2 IR layers

1. CST (rowan, lossless).
2. AST.
3. ItemTree (position-free).
4. DefMap (names).
5. **Typed HIR** (all indices explicit).
6. **Claim IR.** Datalog facts. This is the single semantic form, hashed for the semantic digest.
7. **Resolved world IR** per query slice, holding winners plus their derivations.
8. **Net IR** (typed Petri net plus obligations) and **Clock IR**.
9. Engine JSON (pack, links, scenario), carrying `schema`, `world_digest` and pins.
   - Today a scenario drops pins, `as_of`, `known_at` and events (`litigate-lower/src/case.rs:259, 283-299`).
   - Holes lower to "absent" (`litigate-lower/src/edge.rs:45-52`). They must lower to an explicit `hole` marker or block the build.

### 4.3 Diagnostics

- Keep stable codes. Add tombstones for retired codes.
- Add **labelled secondary spans**. Today notes are unlabelled and cross-file notes are text-only (`db/span.rs:45-48`).
- Add **multi-edit fixes**. A `Fix` is one edit, so renames are inexpressible.
- Add a stable fingerprint, the profile and world, and the derivation (the S2-S4 proof tree) for every L3xxx error.
- **Keep `result` alongside errors.** `litigate-cli/src/out.rs:108` drops the obligations report exactly when it fails.
- Make code deserialization tolerant. Unknown codes fail hard today (`litigate-base/src/code.rs:~150-160`), so old agents cannot read a new compiler's output.
- Render one primary error per root cause. Today an unresolved match arm reports "unknown variant" and then a cascading "non-exhaustive" (`types/shape.rs:200-203, 233-254`).
- Evidence on what good compiler messages look like: Becker et al. 2019, doi:10.1145/3344429.3372508 [C]; Elm, https://elm-lang.org/news/compiler-errors-for-humans [F].

### 4.4 LSP

- Share the salsa db for everything, including verify, lower and replay.
- Debounce and cancel.
- Add a world selector: actual, any `.world`, `t_v`, `t_k`.
- Inlay hints for `Due` in `.case`, with nominal and adjusted dates.
- A code lens "overridden in world W by X (capability C)".
- Completion, rename (requires multi-edit fixes) and semantic tokens are missing today.

### 4.5 Determinism and hashing

- The semantic digest is SHA-256 over canonical claim IR.
- Use JCS (RFC 8785, https://www.rfc-editor.org/rfc/rfc8785 [F]) for every emitted JSON.
- No `elapsed_ms` in hashed envelopes. No compiler version in sidecars that are compared.
- No `serde_json` `preserve_order` dependence on source order (`Cargo.toml:20` with `case.rs:73-84`).
- The engine visits nodes in canonical id order.
- Lock v2 is Merkle and includes source-text hashes.

### 4.6 Solver boundary

- Put the solver in a `litigate-smt` crate behind a trait (lg-962.2), called only by tier 3 (§2.2).
- Queries are pure functions of a canonical SMT-LIB string (https://smtlib.cs.uiowa.edu/logics.shtml [F]). Restrict them to QF_LIA, QF_LRA and QF_UF with enums, under a deterministic resource limit and no timeouts.
- Unsat results carry checkable certificates (Farkas for LRA) stored in the litigatable. wasm and LSP verify the certificates and never run Z3.
- Cache key: query hash plus solver id plus version.
- **Ship tiers 1 and 2 first. They cover every item lg-962.3 lists.**

### 4.7 Test strategy

- **Golden:** keep the insta snapshots. Add per-construct negative tests, as lg-962 requires.
- **Differential:**
  - against a reference interpreter written independently (Lean, or a 300-line Rust interpreter over claim IR);
  - against litgraph's clock (existing);
  - against clingo for world enumeration;
  - **against real rulings**: lg-fq2.3/.5. Agreement is reported per rule version. Every disagreement is classified as a modelling error or a source ambiguity.
  - Per Yang et al. 2011, doi:10.1145/1993498.1993532 [C], differential testing finds bugs single oracles miss.
- **Exhaustive:** Gregorian-cycle enumeration for every counting-rule version (§2.2). This replaces sampled proptests on dates.
- **Property and metamorphic** (QuickCheck, doi:10.1145/351240.351266 [C]):
  - permuting files, declarations or layers leaves outputs bit-identical;
  - renaming a module file changes nothing;
  - adding an unrelated layer leaves unrelated deadlines unchanged;
  - adding an excusal never makes a jurisdictional deadline timely;
  - `parse ∘ print = id`.
- **Generators:** a generator of *well-typed* `.lit` and `.case` programs. None exists; `prop_lowering.rs` loops one fixed package.
- **Fuzzing:** extend beyond the parser and replay to manifest, lock, lowering and resolution.

---

## 5. Concrete defects on main and in the beads

**Columns**
- **Brk**: a later fix breaks source, HIR, lock or output.
- **Rung**: the elimination-ladder rung that removes the defect class. 1 = architecture or types; 2 = lint, test or CI.
- I filed nothing: the brief is read-only.

### 5.1 Semantics-critical

| ID | Where | Defect | Consequence | Brk | Rung |
|---|---|---|---|---|---|
| P1 | `litigate-sema/src/precedence/conflict.rs` (`winner`/`beats`/`conclude` have no callers outside `precedence/` and tests); `litigate-lower/src/select.rs:34-47` | Precedence is diagnostics-only. Lowering and replay take every `active()` version, so a local rule that `override`s an FRCP deadline lowers both and nothing chooses | The language's central promise is not executed | yes (outputs change) | 1 |
| P2 | `precedence/delegation.rs:51-52`; `temporal/activity.rs:88-128`; SPEC.md:518-526 | Only `override` needs delegation. `notwithstanding`, `supersedes` and `revives` from a lower authority win with no `delegable_to` | A CaseOrder can displace a FederalRule | yes | 1 |
| P3 | `case/deadline.rs:177-186`; SPEC.md:602-605 | Any cited `Waived`/`Extended`/`Excused` event between trigger and filing excuses any late event, whatever the deadline, extent or authority | The *Bowles* pattern [F] is scored timely; a 3-day extension excuses a filing 3 months late | yes | 1 |
| P4 | `case/replay.rs:370-394`; SPEC.md:598-600 | Timing-only events are accepted at any posture, terminals included, if they match any trigger or transition pattern in any used package | A `.case` is not actually a typed path | yes | 1 |
| P5 | `precedence/conflict.rs:96-100` vs `lit/core/authority.lit:8-11` | Incomparable authorities fall through to the canons | A Regulation vs court-rule collision is decided by window length | yes | 1 |
| P6 | `precedence/conflict.rs:108-122` | "Specificity" is window width (invalid dates count as widest) | A short emergency order beats an open-ended rule of equal rank on length alone | yes | 1 |
| P7 | SPEC.md:340-343 | Node order is semantically observable (Monte Carlo order) | Renaming a file changes analysis results | yes | 1 |
| P8 | `lit/core/frcp6.lit:52-66`; `lit/core/frap26.lit:47,67`; `litgraph/src/clock/types.rs:44-61` | No 2016 version of FRCP 6(d) or FRAP 26(c); electronic service is omitted "because the engine has none" | E-served deadlines 2001-2016 lack the +3 days. A closed engine enum dictated the law | yes | 1 (total service map) |
| P9 | `temporal/version.rs:109-128`; SPEC.md:482-483 | Disjointness ignores `known_at` | A bitemporal correction is impossible, so DESIGN §4.4's bitemporal claim is unmet | yes | 1 |
| P10 | `case/deadline.rs:331-334`; SPEC.md:426-427 | Trigger is always "most recent matching event"; no deadline instances; open duties never evaluated at `as_of` | Re-triggering resets periods; missed-but-never-acted duties are never reported | yes | 1 |
| P11 | `litigate-lower/src/lib.rs:157-168, 230`; `litigate-cli/src/clock.rs:74-102`; `case/deadline.rs:336-347`; `litigate-cli/src/cmd/check.rs:~84` | Three code paths select counting versions differently, and `known_at` is dropped by `check` and by case lowering | Tests, replay, lowering and the embedded report can describe different law | yes | 1 (one query) |
| P12 | `temporal/world.rs:224` | An exception's `@D` target selector is dropped | An exception to one version applies to all versions | no | 2 |
| P13 | `temporal/transition.rs:83-87` | `has_event` history includes later same-day events | A same-day order retroactively changes the governing version of an earlier event | yes | 1 (docket order) |
| P14 | `precedence/defeasible.rs:109-115` | An undercutter with a concluding sub-exception becomes a rebutter (reviewer's reading; verify against SPEC wording) | Wrong conclusion in nested exceptions | no | 2 |

### 5.2 Legal modelling in the corpus

| ID | Where | Defect |
|---|---|---|
| L1 | `lit/ptab/deadlines.lit:54,59-60,75,120-127,144,162,200-201`; `lit/mpep/deadlines.lit:92-366`; `lit/frcp/periods.lit:126`; `lit/cofc/01-threshold.lit:39` | Months and years encoded as days. Tests assert the wrong dates: PGR 2025-12-01 vs 2025-12-04 [F]; POPR 2025-06-02 vs 2025-06-04 [F] |
| L2 | `lit/ptab/deadlines.lit:27-36` | PTAB counted by `rcfc6`, a COFC rule, while `lit/mpep/deadlines.lit:14` has a real `uspto-1-7`. Two encodings of 37 C.F.R. § 1.7 |
| L3 | `lit/ttab/ttab.lit:19-23` | TTAB counted by `frcp6`, which adds 3 days for Mail. Current 37 C.F.R. § 2.119(c) has no added time for mail [F], and TBMP 113.05 says no extra days [B] |
| L4 | `lit/frap/11-cert.lit:20`; `lit/cafc/11-cert.lit:81` | Cert period counted by `frap26` and `delegable_to [CaseOrder]`. Sup. Ct. R. 30 and a Justice's extension under § 2101(c) govern [K] |
| L5 | `lit/frcp/periods.lit:13-19,73,78,130`; `lit/frap/*` uniformly `[2009-12-01, ..)`; `lit/frcp/periods.lit:62-63` (4(m) 90 days from 2017, but the change was 2015-12-01 [K]) | Fake or late start dates. Cases before them have no deadline, and nothing flags it |
| L6 | `lit/ttab/ttab.lit:146-155`; `lit/ttab/Lit.toml` `in_force = "2025-01-18"` | The answer period exists only from 2026-06-01, so a 2025 TTAB case has no answer deadline |
| L7 | `lit/frap/03-docketing.lit:10,17`; `lit/frcp/periods.lit:134` | Cross-appeal lacks "whichever later" (FRAP 4(a)(3)) |
| L8 | `lit/frcp/periods.lit:130` | Notice of appeal described as jurisdictional but plainly `delegable_to [CaseOrder]`, with no bound. Cited to FRAP 4(a)(1)(A) rather than 28 U.S.C. § 2107(a) [F] |
| L9 | `lit/frcp/02-pleading-to-trial.lit:193-215` | One node gives five tracks with different clocks a single 21-day window |
| L10 | `lit/ptab/deadlines.lit:90-97` | § 42.106(b) correction period kept at 14 days although the rule says one month; the author knows (citation hole) |
| L11 | `lit/core/calendar.lit` | No state holidays (6(a)(6)(C), forward-only) and no declared closures (6(a)(6)(B)); see §3.6 |
| L12 | `lit/core/itc210.lit:21` | A single version from 1976 although the file calls the pre-2024 text UNVERIFIED |
| L13 | `lit/ptab/01-pre-petition-to-institution.lit:146-167` | A derivable fact (315(b) time bar) modelled as `p 0.03`; the label and comment disagree on what `p 0.97` means |
| L14 | `lit/core/frcp6.lit:38-48` | The pre-2005 versions use the post-2005 "add after expiry" order; contested before Dec 1, 2005 [K] |
| L15 | `packs/clock/us-federal-holidays.json` `observed: nearest-weekday` | The observed-Friday treatment under 6(a)(6)(A) needs validation against the lg-fq2.3 rulings. Reviewer uncertain; the direction of risk is a later due date |

### 5.3 Type checker and sema

| ID | Where | Defect | Brk |
|---|---|---|---|
| T1 | `types/ops.rs:152-156, 201-210` | Non-constant Probability `+ - * /` is unchecked (SPEC §6 says checked) | no |
| T2 | `types/rational.rs:64-66, 76, 101, 110` | `ad*bd`, `an*bn`, `an*bd` are unchecked i128 products of u64-range values. They overflow above 2^127: a panic in debug, silent wrap in release (no `overflow-checks` in the release profile) | no |
| T3 | `resolve/sig.rs:299-320`; `types/ty.rs:123`; `build/logic.rs:267` | A forward alias reference resolves to `None` with no diagnostic. Events with such parameter types are dropped from HIR and their arguments go unchecked | no |
| T4 | `build/node.rs:201-205` | A chance node with a self edge skips the exact-sum check (self edge `p = None` reads as a hole). Non-chance nodes with `p` edges get no sum check | no |
| T5 | `resolve/sig.rs:77`; `build/logic.rs:180` vs SPEC §5.2 | Aliases are transparent, though the SPEC says nominal | yes |
| T6 | `build/logic.rs:56, 91-94` | Exceptions to exceptions skip conclusion type checking | no |
| T7 | `resolve/graph.rs:107-114` | Module lookup takes the first package in load order and then checks visibility | yes |
| T8 | `resolve/scope.rs:444-460` | `enums_covering` ignores import lists | no |
| T9 | `cx.rs:364` | Relations accept any target kind (nodes, calendars) | yes |
| T10 | `cx.rs:377` | Authority is always the manifest's; no rule-level authority | yes |
| T11 | `builtins.rs:68` | `deadline(d)` is typed `Date` though there may be no trigger (a totality gap; there is no Option type) | yes |
| T12 | `types/expr.rs:147-150`; `package.rs:356` | A hole checks against any type, including `when` conditions, and still produces package HIR | no |

### 5.4 Verification, lowering, packages, diagnostics

| ID | Where | Defect | Brk |
|---|---|---|---|
| V1 | `litigate-verify/src/deadline.rs:80-84` | Budget exhaustion returns "satisfiable" (unsound) | no |
| V2 | `litigate-verify/src/deadline.rs:20-122` | `deadline-satisfiable` never intersects date windows. It checks version-interval overlap and direction clashes only | no |
| V3 | `litigate-verify/src/sema.rs:7-58` | Obligations "pass" by absence of diagnostics. There is no positive evidence object | yes |
| V4 | `litigate-verify/src/graph.rs:22-110` | Reachability and exits ignore `requires`/`forbids` | no |
| LW1 | `litigate-lower/src/edge.rs:45-52`; `pack.rs:140-143` | Holes lower to absent fields; dev output is a different model | yes |
| LW2 | `litigate-lower/src/case.rs:259, 283-299` | Scenario JSON lacks pins, worlds, events and a schema version | yes |
| LW3 | `litigate-lower/src/engine_clock.rs:~118-129` vs `edge.rs:94` vs `litigate-cli/src/clock.rs:171-176` | Business days become court days in replay, an error in edge lowering, and are rejected by the CLI | no |
| LW4 | `litigate-lower/src/clock.rs:157-181` | Cite prefix match without a token boundary: "FRCP 6" matches "FRCP 60" | no |
| LW5 | `litigate-cli/src/cmd/build.rs:55,160` | `unwrap_or_default` can write an empty output file | no |
| LW6 | `litgraph/src/clock/rule.rs:316`; `interp.rs:66-89` | A rule that closes all weekdays loops forever; `count` is unbounded | no |
| K1 | `pkg/manifest.rs:385-394`; `litigate-lower/src/pack.rs:65-77`; `litigate-verify/src/cite.rs:85-139` | Source `sha256` is never verified; cites resolve against a corpus embedded in the binary, ignoring `as_of` | yes |
| K2 | `pkg/lock.rs:8-29, 158, 172`; `types/build/package.rs:54-61` | Digest omits L0 text and dependency digests; one version per name; first-match resolution; wrong code L1006 | yes |
| K3 | SPEC §4.1-4.2 (absence) | No definition of a breaking change for legal packages | yes |
| D1 | `litigate-cli/src/out.rs:108` | `result` dropped on error; the obligations report is lost when it matters | yes |
| D2 | `litigate-base/src/code.rs:~150-160` | Unknown codes fail deserialization | yes |
| D3 | `litigate-base/src/diagnostic.rs:70-87` | No labelled secondary spans, no multi-edit fixes, no fingerprint, no profile/world tag | yes |
| D4 | `litigate-lsp/src/host.rs:204-227`; `lib.rs:93-132` | Each keystroke reloads disk and re-replays every case under a mutex; no `known_at` or world setting | no |

### 5.5 Syntax

| ID | Where | Defect | Brk |
|---|---|---|---|
| S1 | `litigate-syntax/src/lexer.rs:317` | Cite mode ends at `,` `{` `}` `//`, so case names, `§§ a, b` and URLs are impossible. `cite` swallows the rest of the line | yes |
| S2 | `lexer.rs:72` | Closed unit set: no weeks, months, years, minutes, time of day or zone | yes |
| S3 | `parse.rs:19,109` | `FileKind` is closed and `parse` takes no edition. Adding `.world` or an edition breaks the API. No in-file edition marker | yes |
| S4 | `lexer.rs:44` | `n-1` is one identifier, a trap for any future predicate syntax | yes |
| S5 | `grammar/expr.rs:99-110` | Postfix `.field` chain is uncapped, unlike the binary-chain and depth caps | no |
| S6 | `parser.rs:127` | Recovery anchors only at column 1, so recovery depends on indentation | no |
| S7 | `editors/tree-sitter-litigate` (conformance test checks acceptance only) | Tree-sitter accepts `2026-02-30`, `1__0` and NBSP whitespace; Rust rejects them. No rejection-parity test | no |
| S8 | `parse.rs:122-133` | Dedup by start offset hides distinct-code errors at the same position | no |

### 5.6 Defects in the plan (beads)

- **lg-962.2 / .3 (SMT before need).**
  - All five refinements .3 lists (Probability range, Interval from<to, n≥0, Dist sum, version disjointness) are closed and decided today without a solver: exact rationals, string-ordered dates, interval sweep.
  - Making Z3 a hard dependency adds a native build, wasm exclusion, and nondeterminism risk, for no new rejected programs.
  - Reorder: tiers 1 and 2 (§2.2) under .3, and the solver (.2) only when parameterised values or delegation bounds land.
  - .3 should also own T1, T2 and T4.
- **lg-962.4.**
  - `delegable_to` as an authority set cannot express op, bound, principal, condition, procedure, lock or emergency grant (§2.4). It is the same weakness that lets P3 through.
  - `Event : Posture → Posture` cannot express concurrency (§2.8).
  - `Rule<authority, scope, effective>` lacks transaction time and applicability (§2.3, §2.6).
  - Also add `characterization` (jurisdictional or claim-processing).
- **lg-962.5.**
  - The kind is self-asserted. It must be justified by an instrument cite and computed binding scope.
  - `hypothetical` belongs on worlds, not files.
  - Binding is relative to an addressee (§2.5).
- **lg-962.1 audit.** Add a column, "selected at runtime (lower/replay) vs diagnostic only". Without it, the audit will miss P1.
- **lg-fq2.1.**
  - The ordered layer stack is the wrong algebra (§3.4).
  - The effect classes lack `recharacterize`.
  - The precedent annul/repeal choice conflicts with *Harper*'s retroactivity default (already noted in the bead's own research [B]; make it the rule, not an option).
  - Precedent binding needs the location index `ι` (circuit). Without it the lg-fq2.3 circuit splits cannot "differ by circuit" (lg-fq2.5).
- **lg-fq2.4.** "Exhaustive outcome listing across change combinations" enumerates 2^|Λ| subsets, most of them legally meaningless (a world with the 2009 FRCP amendment but not its transition).
  - Enumerate only declared choice points: contested questions, discretionary orders, unknown facts, and counterfactual toggles the user names.
  - Evaluate each world with the same Datalog engine.
  - Use clingo only as the differential oracle.
- **lg-fq2 overall.** Its own rule "model the actual legal situation accurately; never bend tools to fit data" is already violated by P8 and L2-L4. Those are the deadline domain lg-fq2 targets first, so fix them inside lg-fq2.6, not after it.

---

## 6. Prioritized list

**MUST-BEFORE-FREEZE** means doing it later breaks `.lit`/`.case` source, the HIR or lock format, generated JSON, or the meaning of existing programs.

| # | Item | Tag | Beads |
|---|---|---|---|
| 1 | Make precedence select: lowering and replay use S4 winners; incomparable → error; specificity = subject matter; every displacing relation needs a capability (P1, P2, P5, P6) | MUST-BEFORE-FREEZE | NEW (precedes lg-962.4, lg-fq2.1) |
| 2 | Delegation as typed capabilities {principal, op, bound, condition, procedure} plus locks, emergency grants, and the `characterization` index; replaces `delegable_to [Level]` | MUST-BEFORE-FREEZE | lg-962.4 (rewrite), lg-fq2.1 |
| 3 | Linear obligation instances (bearer, trigger policy `first/latest/later_of`, discharge, breach); typed excusal and extension transforms bounded by capabilities; delete the timing-only hatch and name-magic events (P3, P4, P10; name-magic events, §1.4 item 3) | MUST-BEFORE-FREEZE | lg-962.4 + NEW |
| 4 | Order independence: canonical engine node order, set-merged layers, disjoint transition arms, events ordered by docket sequence (P7, P13) | MUST-BEFORE-FREEZE | NEW; lg-fq2.1 (drop the ordered stack) |
| 5 | Period units (weeks, months, years with EOM policy), `Instant[Zone]`, and `deadline() → Due{nominal, adjusted, ends_at}`; fix L1 data | MUST-BEFORE-FREEZE | NEW; lg-fq2.6 (data) |
| 6 | Bitemporal claims: tx interval with retraction, disjointness per tx slice, applicability keyed by a per-forum proceeding attribute, `unknown_before` (P9, L5, L6) | MUST-BEFORE-FREEZE | lg-962.4 (time-indexed), lg-fq2.1 |
| 7 | World = set of layers + location index ι + modality; `as_of`/`known_at` as query coordinates; `hypothetical` as modality, not kind; kinds justified by instrument cite; guidance binds an addressee; precedent effects incl. `recharacterize`, Harper default | MUST-BEFORE-FREEZE | lg-fq2.1, lg-962.5 |
| 8 | Structured `Cite` type (FRBR expression, pinpoint, span-text hash); remove lexer cite mode; verify source hashes per `as_of` (S1, K1) | MUST-BEFORE-FREEZE | NEW |
| 9 | Stable declaration identity + `formerly`; semantic digest; Merkle lock v2 incl. L0 text; computed compatibility classes replacing semver meaning; PubGrub (K2, K3) | MUST-BEFORE-FREEZE | NEW |
| 10 | Total `service_extension` maps over `ServiceMethod` incl. `Electronic`; 2016 versions of FRCP 6(d) and FRAP 26(c); direction-dependent closed sets; declared-closure calendar class; nominal/adjusted chaining; no cross-forum borrowing (P8, L2-L4, L11) | MUST-BEFORE-FREEZE | lg-fq2 (counting per forum), lg-fq2.6 |
| 11 | Node kinds: no mixed nodes, no `p` on decision edges; `fact` nodes computed by guards when derivable; information sets (L13) | MUST-BEFORE-FREEZE | NEW |
| 12 | Typed boundaries: type or remove `patch` JSON, `analysis` JSON and expression strings; `EdgeRef` instead of `option: String`; declared event effect kinds | MUST-BEFORE-FREEZE | NEW |
| 13 | Markings, not single postures: concurrent tracks (`spawn`/`join`) and a typed firing-sequence check for `.case` | MUST-BEFORE-FREEZE | lg-962.4 (rewrite "posture-indexed events") |
| 14 | One version-selection query used by test, replay, lower, check and LSP; `known_at` honoured everywhere (P11) | MUST-BEFORE-FREEZE | NEW |
| 15 | Edition marker in source plus `FileKind` as non-exhaustive/extensible; reserve keywords `world`, `layer`, `precedent`, `overrule`, `delegable`, `lock`, `grant`, `characterization` now (S3) | MUST-BEFORE-FREEZE | NEW |
| 16 | Diagnostic/envelope schema v2: result alongside errors, labelled spans, multi-edit fixes, fingerprint, profile/world tag, tolerant codes, derivation trees for L3xxx (D1-D3) | MUST-BEFORE-FREEZE | NEW |
| 17 | Lowered artifacts carry schema, pins, world digest and explicit holes; JCS output (LW1, LW2) | MUST-BEFORE-FREEZE | NEW (needs a litgraph schema bump) |
| 18 | Ground refinement tier + Gregorian-cycle enumeration; fix T1, T2, T4 | CAN-ADD-LATER (bugs fix now) | lg-962.3 (re-scoped) |
| 19 | SMT tier for open terms with certificates, deterministic resource limits, cache | CAN-ADD-LATER | lg-962.2 |
| 20 | Indexed `Rule[a, s, v, k, ap]` as surface sugar over items 2 and 6 | CAN-ADD-LATER (once 2/6 land) | lg-962.4 |
| 21 | File-class effect rows checked by set inclusion | CAN-ADD-LATER (once 7 lands) | lg-962.5 |
| 22 | `Basis` labels and propagation; `Credal`/`Beta` priors; conditional `Dist`; `Rate`/`Params` money types | CAN-ADD-LATER | NEW |
| 23 | Contested-question forks (unsettled law → chance node, per ι) | CAN-ADD-LATER | lg-fq2.4 |
| 24 | World enumeration over declared choice points; clingo as oracle | CAN-ADD-LATER | lg-fq2.4 |
| 25 | Position-independent item tree; per-declaration and per-world queries; LSP on salsa end to end, with debounce, world picker, rename, completion | CAN-ADD-LATER | NEW |
| 26 | Verify fixes: V1, V2 (real window intersection), V4 (guard-aware), positive evidence objects (V3 is breaking: include it in 16) | CAN-ADD-LATER (V3 with 16) | lg-962.1 |
| 27 | Workflow-soundness and P-invariant checks on the net | CAN-ADD-LATER | NEW |
| 28 | Reference interpreter (Lean or minimal Rust) for differential testing; metamorphic tests; well-typed program generators; fuzz manifest, lock and lowering | CAN-ADD-LATER (start now) | lg-fq2.5 + NEW |
| 29 | Lean metatheory (§3.7) | CAN-ADD-LATER | lg-litigate-epic-tuq.38 |
| 30 | Remaining type/sema bugs T3, T5-T12; syntax S4-S8; LW3-LW6 | CAN-ADD-LATER, except T5/T7/T9-T11, which break: fold them into 2, 6, 9, 15 | NEW |

---

## Sources

**DOIs resolved via Crossref [C]**
- 10.1145/1375581.1375602
- 10.1145/2628136.2628161
- 10.1145/3473582
- 10.1145/3446804.3446850
- 10.1093/jigpal/jzp075
- 10.1145/1165485.1165490
- 10.1017/S1352325211000036
- 10.1007/s10506-019-09245-0
- 10.1007/s10506-012-9125-8
- 10.1023/A:1008278309945 (Prakken and Sartor; listed by lg-fq2.1, not used above)
- 10.1109/69.755613
- 10.1016/0306-4379(96)00017-8
- 10.1145/2380776.2380786
- 10.1109/69.43410
- 10.1145/116825.116838
- 10.1007/978-3-319-41540-6_23
- 10.1145/2908080.2908096
- 10.1145/2951913.2951948
- 10.1145/1640089.1640108
- 10.1145/3497776.3517779
- 10.1017/S1471068418000054
- 10.1145/371316.371517 (Antoniou et al.; background on defeasible logic)
- 10.1017/S1471068401001168
- 10.1207/s15516709cog1104_4
- 10.1016/0004-3702(94)00041-X
- 10.1016/0004-3702(80)90014-4 (Reiter; background)
- 10.1145/1328438.1328472
- 10.1145/2827695
- 10.1109/TSE.1986.6312929
- 10.1016/0304-3975(87)90045-4
- 10.1017/S0960129500000062
- 10.1145/3158093
- 10.1007/3-540-57880-3_23 and 10.1007/978-3-642-17685-2_8 (Kennedy, units of measure: the basis for §2.1's phantom-unit design)
- 10.1145/73560.73564
- 10.1145/365230.365252
- 10.1145/360051.360056
- 10.1145/292540.292560
- 10.1145/2837614.2837655
- 10.1007/978-3-540-78800-3_24
- 10.1007/978-3-030-99524-9_24
- 10.1007/11817963_11
- 10.1016/0304-3975(94)90010-8
- 10.1007/3-540-44618-4_12
- 10.1142/S0218126698000043
- 10.1007/BF03037383
- 10.1145/182.358434
- 10.1073/pnas.39.10.1095
- 10.1145/2594291.2594324
- 10.1145/3236774
- 10.1007/978-3-030-79876-5_37
- 10.1145/1993498.1993532
- 10.1145/351240.351266
- 10.1145/3344429.3372508
- 10.1007/978-3-642-24908-2_30
- 10.1007/978-94-007-1887-6_6

**URLs fetched [F]**
- Case law, with the quoted holdings located in the text:
  - *Hamer*: https://www.law.cornell.edu/supremecourt/text/16-658
  - *Harrow*: https://www.law.cornell.edu/supremecourt/text/23-21
  - *Boechler*: https://www.law.cornell.edu/supremecourt/text/20-1472
  - *Bowles*: https://www.law.cornell.edu/supct/html/06-5306.ZO.html
- Statutes and rules, with the cited text located:
  - 35 U.S.C. § 321(c): https://www.law.cornell.edu/uscode/text/35/321
  - 37 C.F.R. § 42.107: https://www.law.cornell.edu/cfr/text/37/42.107
  - 37 C.F.R. § 2.119: https://www.law.cornell.edu/cfr/text/37/2.119
  - FRAP 26: https://www.law.cornell.edu/rules/frap/rule_26
  - FRAP 4, 28 U.S.C. § 2107, 37 C.F.R. § 1.7, 35 U.S.C. §§ 21 and 315: law.cornell.edu (HTTP 200)
- Executive closure orders: Federal Register API query "Providing for the Closing of Executive Departments" (federalregister.gov, 2018-2025 results)
- Languages and tools:
  - https://www.openpolicyagent.org/docs/latest/policy-language/
  - https://cuelang.org/docs/concept/the-logic-of-cue/
  - https://dhall-lang.org/
  - https://nickel-lang.org/user-manual/contracts
  - LegalRuleML and AKN OASIS specs (docs.oasis-open.org)
  - https://github.com/salsa-rs/salsa
  - rust-analyzer architecture
  - https://pubgrub-rs.github.io/pubgrub/pubgrub/
  - https://github.com/obi1kenobi/cargo-semver-checks
  - https://package.elm-lang.org/help/design-guidelines
  - https://elm-lang.org/news/compiler-errors-for-humans
  - https://www.rfc-editor.org/rfc/rfc8785
  - https://smtlib.cs.uiowa.edu/logics.shtml
  - https://xtdb.com/
  - https://github.com/potassco/clingo
  - https://arxiv.org/abs/2103.03198
  - https://catala-lang.org/

**[B], from the lg-fq2.1 research comment**
- TBMP 112 and 113.05.
- FRCP 87, FRCrimP 62, FRAP 2(b).
- 6(b)(2) lock lists per forum.
- 83 FR 51340, 90 FR 225.
- *Harper*, *Loper Bright*.
- PTAB SOP 2 and *Light & Wonder*.
- COFC and ITC layer facts.

**[K], unverified leads**
- § 2101(c) extension by a Justice, and Sup. Ct. R. 30.
- The FRCP 4(m) change effective 2015-12-01.
- The 2005 amendment to 6(e).
