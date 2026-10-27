# Legal change: instrument classes and worlds

Status: normative for language 1 (bead lg-fq2.1). SPEC §4.1, §5.7 and
§5.8 state the compiler's rules; this file states the legal model they
implement, the per-forum matrix they read, the examples, and what is not
done yet. Domain first: federal deadline computation (FRCP 6, FRAP 26,
and the local rules, orders and decisions that modify them), per lg-fq2.

Paradigm (SPEC §0): typed stratified Datalog with priorities over
bitemporal facts. A *layer* is a package; its *class* is the kind of
legal instrument it encodes. A class is an index on the package's type
(SPEC §5.7): it fixes which effects the package's declarations may have,
so an instrument that does something the law does not let that kind of
instrument do is a compile error, not a quiet wrong answer.

Sources: design review 2026-10-23 §2.4-§2.6, §3.2-§3.4, §6 item 7
(`research/design-review-2026-10-23.md`); the lg-fq2.1 forum research
register (every URL fetched 2026-10-23, quoted in the bead). Literature:
Governatori and Rotolo, *Changing legal systems: legal abrogations and
annulments in Defeasible Logic*, Logic Journal of the IGPL (2010),
doi:10.1093/jigpal/jzp075; Horty, *Rules and reasons in the theory of
precedent*, Legal Theory 17 (2011), doi:10.1017/S1352325211000036; Horty,
*Reasoning with dimensions and magnitudes*, Artificial Intelligence and
Law 27 (2019), doi:10.1007/s10506-019-09245-0; Prakken and Sartor,
*Modelling reasoning with precedents in a formal dialogue game*,
Artificial Intelligence and Law 6 (1998), doi:10.1023/A:1008278309945;
Merigoux, Chataing and Protzenko, *Catala: a programming language for the
law* (ICFP 2021), arXiv:2103.03198; OASIS LegalDocML, *Akoma Ntoso
Version 1.0* (2018).

## 1. What a change is

A legal change is never an edit of a file. It is an instrument (an
order, a rule, a decision) that someone with power issued on a date, and
that has effect over some scope from some date. Litigate records it as
its own package, separately versioned, beside the base it acts on:

- **Separately declared.** `kind` in `Lit.toml` names the class. A class
  other than `base` is justified by `[package.instrument]`: the vendored
  text (`source`), the `issuer`, the `date`, a `designation`; never only
  asserted (design review §2.5 item 1).
- **Effects, not edits.** Each declaration form has an effect (SPEC §5.7):
  `declare`, `prescribe`, `rank`, `version`, `override`, `interpret`,
  `recharacterize`, `invalidate`, `waive`, `grant-cap`, `set-prior`,
  `add-fact`. A class has an effect row; a forum family's matrix (§3) can
  only narrow it.
- **Three operations, as the bead asks.** A change either (a) *overrides a
  rule function*: a new value for a declaration (`override`,
  `notwithstanding`, `supersedes`; a precedent's waiver), which always
  needs a capability (SPEC §5.5, L3004); (b) *adds a priority between
  reasons*, Horty's model: an `exception` defeating another package's
  default (`interpret`), so the rule's text stands and only which reason
  wins changes; or (c) *adds facts only*: matter facts, a prior of a
  chance node (`set-prior`, `add-fact`), or declarations of its own.
- **Never rule text.** Guidance, persuasive precedent, emergency
  declarations, overrulings and `.case` files never touch another
  package's rule text: their rows lack `override`, `version` and
  `interpret`. A case order and a standing order touch only what a rule
  delegates to them.
- **Dates are data.** Every claim carries its valid time (`effective`),
  its transaction time (`known_at`, `retracted`), and its applicability
  (`commenced`): SPEC §5.4. `as_of` and `known_at` are coordinates of a
  query, never of a layer or a world (§5).

## 2. The classes

Each class below gives: scope; how `as_of` and `known_at` apply; its
authority (a rung of `lit/core/authority.lit`); the rule for pending
cases; what it does; its effect row (the forum-independent ceiling,
`litigate_sema::hir::Kind::row`). Section 3 gives the cells per forum.

### 2.1 `base`

- **What:** primary law as enacted or promulgated: a statute, a court's
  rules of procedure, an agency's rules of practice, a forum's procedure
  graph. The only class that declares rules from nothing and orders
  authority levels.
- **Scope:** its jurisdiction (the package's `forum`).
- **Dates:** each declaration's `effective` window and `known_at`; the
  package's `in_force`, `unknown_before` (SPEC §4.1).
- **Authority:** the package's `authority` (Statute, FederalRule,
  Regulation; COFC's RCFC are base at FederalRule, 28 U.S.C. § 2503(b)).
- **Pending cases:** a version applies to the proceedings its `commenced`
  window admits.
- **Row:** every effect.

### 2.2 `amendment`

- **What:** a dated new version, insertion or repeal of base
  declarations. It produces the next version of the base, never a
  parallel world: its declaration `supersedes` the base's from its
  effective date.
- **Scope:** the base's.
- **Dates:** three, as the instruments state them: the order's date
  (`[package.instrument] date`), the effective date (`effective`), and
  applicability (`commenced`). REA amendments: order by May 1, effective
  no earlier than December 1 (28 U.S.C. § 2074(a)).
- **Authority:** the base's rung (the same rulemaker).
- **Pending cases:** the instrument's own clause. The Supreme Court's
  orders say "shall govern in all proceedings thereafter commenced and,
  insofar as just and practicable, all proceedings then pending" (order
  of March 26, 2009); 28 U.S.C. § 2074(a) lets the pending-case court keep
  "the former rule" where application "would not be feasible or would
  work injustice". Agencies key applicability to a proceeding attribute:
  PTAB to the petition filing date (83 FR 51340), the ITC to institution
  (90 FR 225). Local rules: whatever the prescribing court orders
  (28 U.S.C. § 2071(b)).
- **Row:** every effect but `rank`.

### 2.3 `local-rule`

- **What:** a court's rules under 28 U.S.C. § 2071 (FRCP 83(a), FRCrimP
  57(a), FRAP 47(a), Fed. Cir. R. 47). It prescribes what the national
  rules leave open, overrides only a declaration delegated to `LocalRule`,
  and may delegate down. It never changes how FRCP 6 counts (2009
  Committee Note to Rule 6(a); S.D.N.Y./E.D.N.Y. withdrew Local Civil
  Rule 6.4).
- **Scope:** one court. **Authority:** `LocalRule`.
- **Pending cases:** as the court orders (§ 2071(b); N.D. Cal. Civ. L.R.
  1-3 keeps the old rule where fewer than ten days remain).
- **Row:** declare, prescribe, override, interpret, waive, grant-cap,
  set-prior, add-fact.

### 2.4 `general-order`

- **What:** a court-wide order: a general order or a chief judge's
  standing order (S.D.N.Y. M10-468; COFC General Order 2025-02, which
  treats a closed day as a legal holiday for RCFC 6). Distinct from a
  local rule (no notice and comment) and from a judge's standing order
  (court-scoped, not judge-scoped).
- **Scope:** one court; usually temporary. **Authority:** the
  `StandingOrder` rung: like a judge's directive it regulates practice
  consistently with the rules (FRCP 83(b), RCFC 83(b)).
- **Pending cases:** effective on entry.
- **Row:** declare, prescribe, override, waive. It states no matter facts
  and no priors.

### 2.5 `standing-order`

- **What:** a judge's standing order (FRCP 83(b), FRCrimP 57(b), RCFC
  83(b)): gap-filling for that judge's cases, or what a local rule
  delegates (N.D. Cal. Civ. L.R. 5-1(d)(7) leaves chambers copies to "a
  standing order of the assigned Judge").
- **Scope:** one judge's cases. **Authority:** `StandingOrder`.
- **Pending cases:** no sanction without actual notice (FRCP 83(b)).
- **Row:** as a local rule's. At a court of appeals FRAP 47(a)(1)
  requires a generally applicable direction to be "in a local rule rather
  than an internal operating procedure or standing order": the
  `circuit-appellate` and `federal-circuit` cells narrow it to `declare`.

### 2.6 `case-order`

- **What:** an order in one case: a scheduling order (FRCP 16(b)), an
  extension (FRCP 6(b), FRAP 26(b)), a PTAB order under 37 C.F.R.
  § 42.5(c), an ITC ALJ's ground rules (issued per investigation, below
  later orders). A date it designates is the order's, never a counted
  period (`designated p`, SPEC §5.3). It replaces the free-text cited-order
  excuse with a typed instrument.
- **Scope:** one case. **Authority:** `CaseOrder`.
- **Pending cases:** the case it is entered in.
- **Row:** declare, prescribe, override, interpret, waive, set-prior,
  add-fact; never `grant-cap` (an order delegates nothing), never
  `version`.

### 2.7 `emergency-declaration`

- **What:** a Judicial Conference emergency declaration (FRCP 87, FRCrimP
  62, FRAP 2(b)). It grants capabilities for a window and does nothing
  else: under a Civil Rules emergency a court "may, by order, apply Rule
  6(b)(1)(A) to extend for a period of no more than 30 days after entry of
  the order the time to act under Rules 50(b) and (d), 52(b), 59(b), (d),
  and (e), and 60(b)" (FRCP 87(c)(2)(A)), the periods 6(b)(2) otherwise
  locks.
- **Scope:** the courts it designates. **Dates:** the window is the grant's
  `effective`, bounded by the family's cell (`window_days`: 90 under FRCP
  87(b)(1)(C) and FRCrimP 62(b)(1)(C)); a longer or open-ended grant is
  L2011. No declaration has issued (research register), so the example is
  a synthetic, non-authoritative layer (§5).
- **Row:** declare, grant-cap.

### 2.8 `precedent`

- **What:** a judicial (or Director) decision. Its *scope* is computed,
  never declared (design review §2.5 item 1): binding where its `court`
  is among the courts whose precedent binds in the family (§3; `@circuit`
  is a world's circuit), and only if it was `designated` precedential;
  persuasive everywhere else. A district decision binds no one (Camreta v.
  Greene, 563 U.S. 692 n.7 (2011)); a Federal Circuit nonprecedential
  disposition is not binding (Fed. Cir. R. 32.1(d)); a PTAB decision
  binds from its designation date (SOP 2), which may follow the decision.
- **Effects of a binding precedent:**
  - `interpret`: a priority between reasons, Horty-style: an `exception`
    to another package's default. The text stands. Or a reading of a
    deadline's trigger or counting (lg-i1qw): a deadline that `interprets`
    another package's deadline, its period the text's (Benavides v.
    Bureau of Prisons, 79 F.3d 1211 (D.C. Cir. 1996): FRAP 4(a)(6)(A)'s
    window opens on written notice from the clerk or a party), written
    `deadline r: 7.days after E() counted_by c effective [from, to)
    interprets d cite "..."` (SPEC §5.7). Where it binds, the matter's
    instances of `d` run from `E` under `c`.
  - `recharacterize`: a deadline's `characterization` (Harrow v.
    Department of Defense, 601 U.S. 480 (2024): § 7703(b)(1) "is not
    jurisdictional"), written `recharacterize r: d characterization
    claim_processing effective [from, ..) cite "..."` (SPEC §5.7).
  - `invalidate`: of a rule or a doctrine (Bowles v. Russell, 551 U.S.
    205 (2007), of the "unique circumstances" exception), written
    `invalidate i: d effective [from, ..) cite "..."`.
  - `waive`: a prospective override of a declaration delegated to the
    waiving power (Light & Wonder, IPR2025-01072, Paper 30: "going
    forward, the Office is waiving the time period under 37 C.F.R.
    § 42.75(c)(1) ... from fourteen days to thirty days", under
    § 42.5(b)). In language 1 a precedent's `override`/`notwithstanding`
    *is* its waiver: the classifier reads it so, and the capability check
    (L3004) needs the target's delegation to the precedent's rung. A
    one-proceeding waiver without precedential effect is a case order.
  - `set-prior`: evidence, moving a chance node's belief.
- **Persuasive precedent** may only declare and move a prior (L2010
  otherwise).
- **Pending cases:** judicial decisions apply to every case still open on
  direct review (Harper v. Virginia Dep't of Taxation, 509 U.S. 86
  (1993); Griffith v. Kentucky, 479 U.S. 314 (1987)); annul versus repeal
  is a choice only for rules and regulations, never for a judicial
  decision (design review §3.4).
- **Authority:** language 1's lattice has no rung for decisions: a
  precedent takes the rung of the power it exercises (a PTAB waiver under
  § 42.5 the Board's orders', `CaseOrder`).

### 2.9 `guidance`

- **What:** non-binding or addressee-binding material: Committee Notes,
  the MPEP, TMEP, TBMP and Trial Practice Guide, Practice Notes, IOPs,
  USPTO Director memoranda. Bindingness is relative to an addressee
  (design review §2.5 item 3): `binds` names the actors it binds (Director
  memoranda bind PTAB judges, not parties), and it may set the prior of a
  draw only by one of them. It never prescribes and never imposes a duty.
- **Row:** declare, set-prior (the bound actors' draws only).

### 2.10 `overruling`

- **What:** a decision withdrawing a precedent (`overrules = [...]`).
  The overruling court must be the precedent's own (en banc; Fed. Cir.
  IOP 13; 28 U.S.C. § 46(c)) or one whose precedent binds it (Rodriguez
  de Quijas, 490 U.S. 477, 484 (1989)); otherwise L2010. PTAB
  de-designation (SOP 2 § III) is the agency form.
- **Dates:** the precedent's claims are withdrawn from the overruling's
  date, retroactively to open cases (Harper), never reopening final
  judgments (Loper Bright Enterprises v. Raimondo (2024): prior holdings
  "are still subject to statutory stare decisis").
- **Row:** declare, invalidate: `overrules` withdraws whole precedents,
  and an `invalidate` declaration one of their claims (Bowles invalidates
  Thompson's "unique circumstances" default, §4).

### 2.11 `.case` and `hypothetical`

A `.case` states facts and events only and never modifies rules (SPEC
§5.6). `hypothetical` is not a class: no forum contains a hypothetical
(research register §6 item 10). It is the *modality* of a world (§5); a
synthetic what-if layer of any class carries `authoritative = false`.

## 3. The matrix by forum family

The matrix is data, in `lit/core/Lit.toml` (`[family.*]`), not compiler
code (design review §2.5 item 4). A family lists its package `forums`,
its `precedent` courts, its `circuits`, and one cell per class that
exists there with the instrument that makes it exist (`cite`), a real
example (`example`, every URL fetched), an optional narrowed `row`, and
for an emergency declaration its `window_days`. A class with no cell does
not exist in the family: a package of that class there is L2011. Rank is
the rung of `lit/core/authority.lit` (1 = highest in that family's chain).

| Class | district-civil (frcp) | district-criminal (frcrimp) | circuit-appellate (frap) | federal-circuit (cafc) | cofc | ptab | ttab | itc |
|---|---|---|---|---|---|---|---|---|
| base | rank 1-2: statutes, FRCP (28 U.S.C. §§ 2072, 2074) | 1-2: FRCrimP, 18 U.S.C. § 3161 | 1-2: FRAP, 28 U.S.C. § 2107 | 1-2: FRAP, 28 U.S.C. § 1295 | 1-2: RCFC (§ 2503(b)), § 2501 | 1-2: 35 U.S.C. § 316(a); 37 C.F.R. pts. 1, 42 | 1-2: 15 U.S.C. § 1123; 37 C.F.R. pt. 2 | 1-2: 19 U.S.C. § 1337; 19 C.F.R. pts. 201, 210 |
| amendment | base rung; § 2074(a) | § 2074(a) | § 2074(a); FRAP 47(a)(1) | Fed. Cir. R. 47(a)(2)-(3) | RCFC 83(a) | notice and comment (83 FR 51340) | 81 FR 69950 | 90 FR 225 |
| local-rule | 3: § 2071; FRCP 83(a) | 3: FRCrimP 57(a) | 3: FRAP 47(a) | 3: Fed. Cir. R. 47 | none (RCFC are base) | none | none | none |
| general-order | 4: FRCP 83(b) (S.D.N.Y. M10-468) | 4: FRCrimP 57(b) | 4, `declare` only (FRAP 47(a)(1)) | 4, `declare prescribe` (Fed. Cir. R. 26(a)(1) closures) | 4: RCFC 83(b) (GO 2025-02) | none | none | none |
| standing-order | 4: FRCP 83(b) | 4: FRCrimP 57(b) | 4, `declare` only (FRAP 47(a)(1)) | 4, `declare` only (R. 47(c)) | 4: RCFC 83(b) | none | none | none |
| case-order | 5: FRCP 6(b), 16(b) | 5: FRCrimP 45(b)(1) | 5: FRAP 2(a), 26(b) | 5: Fed. Cir. R. 26(b) | 5: RCFC 6(b), 6.1 | 3: 37 C.F.R. § 42.5 | 3: §§ 2.120, 2.121 | 3: §§ 201.14(b), 210.51 (ALJ ground rules) |
| emergency-declaration | FRCP 87, 90 days | FRCrimP 62, 90 days | FRAP 2(b) | FRAP 2(b) | none | none | none | none |
| guidance | Committee Notes | Committee Notes | handbooks | Practice Notes, IOPs | Rules Committee Notes | SOP 2 Director directives (bind judges), TPG | TBMP, TMEP | 337 FAQ |
| precedent | binds: scotus, the circuit | scotus, the circuit | scotus, the circuit | scotus, cafc, ccpa, ct-cl (South Corp.) | scotus, cafc, ct-cl | scotus, cafc, ccpa, ptab (SOP 2) | scotus, cafc, ccpa, ttab (TBMP 101.03) | scotus, cafc (no designation scheme) |
| overruling | SCOTUS, en banc | SCOTUS, en banc | en banc (§ 46(c)) | en banc (IOP 13) | CAFC, SCOTUS | de-designation (SOP 2 § III) | CAFC, SCOTUS | CAFC |

Each cell's example URL is in `lit/core/Lit.toml`. Cells the research
could not verify carry no `example` (circuit standing orders and case
orders, COFC judge standing orders, CAFC overruling) and are listed in
the register's §8.

Counting is per forum, never one FRCP 6 function: FRCP 6, FRCrimP 45,
FRAP 26, RCFC 6, 37 C.F.R. § 1.7 (PTAB, via § 42.1(a)), § 2.196 (TTAB)
and 19 C.F.R. § 201.14 (ITC) are distinct counting rules of `lit/core`,
each amending order a version of its rule (CHANGE-REPORT.md lists them). The never-extendable sets differ by forum
(FRCP 6(b)(2), RCFC 6(b)(2) without 50(b), FRCrimP 45(b)(2) Rule 35
only, FRAP 26(b)) and are `extend { never }` locks an emergency `grant`
can `unlock`.

## 4. Examples: one file per class (deadline domain)

`docs/litigate/examples/change/` holds one package per class, each over
vendored instrument text (`sources/change-*.txt`, sources/PROVENANCE.md),
and the worlds that layer them (`worlds/`). `litigate check
docs/litigate/examples/change` checks them all (test
`crates/litigate-cli/tests/change_examples.rs`).

| Class | Package | Instrument | Does |
|---|---|---|---|
| base | `base/` | FRCP 12(a)(1)(A)(i), 56(b), 59(e) (2007) | declares the three periods; 56(b) delegated to `LocalRule` and `CaseOrder`; 59(e) locked |
| amendment | `amendment/` | Order of March 26, 2009 | 59(e) 10 days → 28, `supersedes` from Dec. 1, 2009 |
| local-rule | `local-rule/` | S.D.N.Y./E.D.N.Y. Local Civil Rule 6.1(a) | 7-day opposition period, delegated to standing and case orders |
| general-order | `general-order/` | COFC General Order 2025-02 | Nov. 28, 2025 closed for RCFC 6 |
| standing-order | `standing-order/` | Judge Breyer's General Standing Order | courtesy copy by the next court day |
| case-order | `case-order/` | Cancino-Perez, D. Colo., ECF 42 § 9.a (per ECF 45) | a designated June 20, 2025 amend deadline |
| emergency-declaration | `emergency-declaration/` | FRCP 87 (synthetic, `authoritative = false`) | unlocks 59(e) for 90 days, extensions ≤ 30 days |
| guidance | `guidance/` | Fed. Cir. Practice Note to Rule 26 | fixes the court's draw on waiving a late notice (p 0) |
| precedent | `precedent/` over `ptab-base/` | Light & Wonder, Paper 30 (designated June 22, 2026) | waives § 42.75(c)(1): 14 days → 30 |
| overruling | `overruling/` over `overruled/` | Bowles v. Russell (2007) overruling Thompson v. INS (1964) | withdraws Thompson from June 14, 2007, and invalidates its "unique circumstances" default |

`docs/litigate/change/` holds the deadline domain's own change files
(lg-fq2.6): the FRCP and FRAP periods the collected rulings count as base
layers, the orders of 2005 and 2009 as amendment layers on them, the
actual world at each circuit, and each ruling as a matter replayed under
it (`crates/litigate-cli/tests/change_files.rs`). CHANGE-REPORT.md lists
every change file of the domain, the agreement rates and the open
failures.

`docs/litigate/examples/timelines/` holds a procedure over the base
periods (`post-judgment/`: a Rule 59(e) motion `within` the base's
period), a synthetic matter, and the world whose alternate timelines
§5.1 runs it in.

## 5. Worlds

A world is `W = (Λ, ι, μ)` (design review §3.2): a finite **set** of
layers Λ, a location index ι, and a modality μ. It is a `.world` file
(TOML):

```toml
modality = "actual"                     # or "counterfactual"; default actual
layers = ["change-frcp-base", "change-frcp-2009-amendment", "change-sdny-local-civil-rules"]

[location]
family = "district-civil"               # a [family.*] id of core
court = "d-sdny"                        # optional
circuit = "ca2"                         # required where the family's precedent names @circuit
judge = "..."                           # optional
```

- **A set, not a stack.** Λ merges as a set: associative, commutative,
  idempotent (design review §3.4). Order and repetition in `layers` mean
  nothing; loading a layer twice is a no-op. Precedence comes from the
  instruments (authority, dates, scope, declared relations), never from
  the file. Each layer brings its dependencies.
- **Query coordinates.** `as_of` and `known_at` are not in a world: a
  world holds for every date, and a query picks one (`--as-of`,
  `--known-at`; the timeline's and LSP's `WorldId`). An alternate
  timeline is just another world file.
- **Location.** ι decides which precedent binds: in a world at `ca2` a
  `ca1` precedent is persuasive, its binding effects are not in force
  (L9009, info), and the layer is reported `in_force: false`; at `ca1`
  it binds. This is how the lg-fq2.3 circuit splits differ by circuit.
- **Overruling.** An overruling in Λ withdraws the precedents it names
  from its date (`overruled_by`; with several, the earliest); an
  overruling by a court that does not bind the precedent's court is
  L2010.
- **Modality.** An actual world admits only authoritative layers (L1014
  for `authoritative = false`); a counterfactual world admits any layer
  and is never checked under the strict profile (L1014). Real-case and
  strict checks therefore never see a what-if.
- **Conflicts.** The world reports the temporal and precedence
  diagnostics its layers have together but none has alone: two local
  rules overriding the same delegated period differently are an
  unresolved conflict (L3003) only in a world holding both. A precedent
  that does not bind at ι has no claim in force there and conflicts with
  nothing. An incomparable conflict is an error; where the law is
  unsettled at ι the readings are options of a choice point (§5.1), one
  world each.
- **Malformed:** an unknown key, modality, layer, family or circuit is
  L1013.

`litigate check PATH...` finds `.world` files with packages and matters;
a world's layers resolve among the loaded packages, else in the library
(`--lib`, default `lit`). The result lists each world with its layers'
classes (a precedent's at the location) and whether each is in force.

### 5.1 Alternate timelines (lg-fq2.4)

A world file may declare **choice points**: `[choice.ID]` tables whose
keys name options and whose values are sets of layers. The file spans one
alternate world per combination, one option taken at every choice point:
its own `layers` plus the options'. A change toggled is a choice between
leaving an instrument out and adding it; a contested question (unsettled
law at ι, a circuit split) is a choice among its readings, one layer per
reading. Each layer belongs to one choice point, so each world is exactly
one combination (L1013 otherwise, as for a choice with one option, two
options with the same layers, or more than 4096 worlds).

```toml
# docs/litigate/examples/timelines/rule-59e.world
modality = "counterfactual"
layers = ["change-frcp-base"]

[location]
family = "district-civil"
court = "d-sdny"
circuit = "ca2"

[choice.rule-59e-2009]
without = []
with = ["change-frcp-2009-amendment"]
```

`litigate worlds CASE --world FILE [--path PATH] [--layer NAME] [--vary
NAME] [--as-of DATE] [--known-at DATE]` (the MCP tool `litigate_worlds`;
in wasm, `litigate(["worlds", ...], files)`) replays the matter in every
world:

- **The law of a world.** Its layers in force join the law the matter's
  packages see, with their dependencies, exactly as a dependency's
  declarations do: an amendment's `supersedes`, a local rule's or case
  order's `override`, a precedent's waiver or exception, an emergency
  `grant` take part in selection (SPEC §5.4, §5.5). A deadline an
  amendment supersedes is counted under the amendment's version (the base
  instance, never a second one). A layer's own new deadlines are replayed
  only in a matter that `uses` the layer.
- **In force at the query.** A precedent that does not bind at ι with
  binding effects is not in force. An overruled precedent is in force
  until the overruling's date: a matter judged before it (`as_of`), or at
  a `known_at` before it, still has it; from then it is withdrawn for
  every case still open (Harper v. Virginia Dep't of Taxation, 509 U.S.
  86 (1993)). Versions are selected at `known_at` (the matter's own,
  else the latest corpus): a version recorded later does not exist yet.
- **Per filing.** Each event `by` a party is, per world, `timely`,
  `late` (L4002), `unjudged` (its due date is unknown or cannot be
  counted at `known_at`), `rejected` (another replay error at its event),
  `untimed` (it answered no deadline there) or `undetermined` (the
  world's own law is in error: a synthetic layer in an actual world, an
  unresolved conflict). The result lists every world (choices, layers,
  deadlines with due dates, its errors), every filing with the worlds of
  each outcome, and the worlds grouped by identical outcome.
- **Errors.** A late filing is an outcome of its world. The query fails
  (exit 2) only for an invalid world file, a package in scope with an
  error, or a matter error no layer can change (syntax, names, pins,
  types: L0xxx-L2xxx).

The example matter (`docs/litigate/examples/timelines/rule-59e.case`,
synthetic facts) has judgment entered March 1, 2010 and a Rule 59(e)
motion filed March 25:

| World | Rule 59(e) | Due | Motion |
|---|---|---|---|
| `w1` (`with`) | 28 days, the 2009 amendment | 2010-03-29 | timely |
| `w2` (`without`) | 10 days, the 2007 text (counted under FRCP 6 as amended in 2009) | 2010-03-11 | late |

At `--known-at 2009-01-01`, before the order of March 26, 2009, the corpus
holds neither the amendment nor a FRCP 6 version in force in 2010 (the
2005 version ends December 1, 2009): the motion is `unjudged` in both.

**Contested questions** (lg-fq2.8; design review §3.4, §6 item 23). A
choice point whose options are readings of unsettled law at ι is drawn
by the tribunal, not chosen by anyone, so the world file declares it
contested, with a basis-labelled prior (Dirichlet pseudo-counts, 1 each
by default; a `source` for any basis but `estimate`):

```toml
[contested.rule-59e-2009]
basis = "estimate"
prior = { with = 3, without = 1 }
```

`worlds` lowers it to a chance node (`chance`): one branch per option,
with its probability `α/Σα` and the worlds taking it. Uncontested choice
points (a change toggled, `--vary`) stay worlds: each `timelines` entry is
one combination of their options, with its worlds weighted by the
contested draws and each filing's probability of each outcome. With the
prior above and `--vary change-frcp87-what-if`, the motion is timely with
probability 1 in the timeline with the what-if and 3/4 (late 1/4) in the
one without. A contested option holding a precedent that binds at ι is
L1013: controlling precedent settles the question there.

**Why no solver.** The worlds are a finite product of declared options,
enumerated exhaustively, and each world's outcome is the stratified model
the checker already computes (design review §3.3). An answer-set solver
(clingo) would have to re-encode replay, counting and precedence to
compute the same product; that re-encoding is worth having only as a
differential oracle (design review §4.7, §6 item 24), and an external
solver cannot run in the wasm build or the LSP (§4.6). Grouping worlds by
outcome is a pass over the results.

## 6. Akoma Ntoso

The change-file shape follows Akoma Ntoso's lifecycle and modification
metadata, kept as metadata about instruments, never as edit scripts
(edit scripts are not idempotent; design review §3.4):

| Akoma Ntoso | Litigate |
|---|---|
| `lifecycle/eventRef` (generation, amendment, repeal) with `date`, `source` | `[package.instrument]` `date`, `source`, `issuer`, `designation`; `kind` |
| `temporalData/timeInterval` (`start`, `end`) | `effective [from, to)` |
| `activeModifications/textualMod type="substitution"` | `supersedes` (amendment) |
| `textualMod type="repeal"` | an amendment version ending a declaration, or `invalidate` |
| `forceMod` (entry into force, postponement) | `effective`, `commenced` |
| `efficacyMod` (inapplication, suspension) | `override`/`notwithstanding` under a capability; a precedent's waiver; an emergency `grant` |
| `meaningMod` (interpretation) | `interpret` (an exception between reasons) and `recharacterize` |
| `scopeMod` | `commenced` applicability; a world's location |
| `passiveModifications` | computed: a declaration's displacers are found by resolution, never listed on the target |
| FRBR work / expression / manifestation | package name / version and `effective` / the vendored text and its digest |

## 7. Decisions and limits

- `general-order` sits on the `StandingOrder` rung; the class records
  court scope, the rung rank. A distinct `GeneralOrder` rung would change
  the lattice every package shares.
- A precedent's waiver is written `override` (language 1 has no `waive`
  relation); its reading of a deadline's trigger or counting is written
  `interprets` (lg-i1qw) and, unlike the forms below, is applied when a
  world's layers merge: the circuit split `frap-4a6-notice` replays per
  circuit (VALIDATION.md §4). `recharacterize` and `invalidate` are
  declarations naming another package's deadline or claim (lg-fq2.7);
  each is checked against
  the class rows and family cells, and is not yet applied when a world's
  layers merge (lg-fq2.4). No vendored decision recharacterizes a corpus
  deadline, so no example does.
- Binding scope uses the `designated` date as a fact of the instrument;
  gating a binding precedent's claims by that date at a query is the
  claims' own `effective` window.
- A matter runs under a world through `litigate worlds` (§5.1), not by
  naming a world in the `.case` (no `.case` syntax changed). A contested
  question is a choice point the world declares `[contested.ID]`; it
  lowers to a chance node of the `worlds` result with a basis-labelled
  prior (§5.1), not yet to a node of the matter's engine scenario, and
  its prior comes from the world file (the matter's `beliefs` cannot name
  a choice point). The golden tests against the 2009 Committee Note
  examples and the collected rulings, their agreement rates and every
  disagreement are in VALIDATION.md (lg-fq2.5).
- A modification layer that declares no node has no `start` (lg-fq2.7);
  `lit/core`, a base library, still declares its placeholder.
- The COFC general-order example records its closed day; no counting
  rule reads a layer's calendar, so the closure is not counted
  (CHANGE-REPORT.md, open failure 1). A counting rule is versioned only
  within its package, so the orders amending FRCP 6 and FRAP 26 are
  versions in `lit/core`, not amendment layers (open failure 2).
- No corpus package changed class: each `lit/` package is a forum's base
  law (the CAFC package encodes FRAP with the Federal Circuit Rules
  interleaved, a procedure graph at the `LocalRule` rung).
