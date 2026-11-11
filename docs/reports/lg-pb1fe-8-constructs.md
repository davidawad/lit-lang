# lg-pb1fe.8: low-use language constructs

Measured in this checkout on 2026-11-11 (`lit` `.lit` files only). Corpus counts distinguish actual declaration/keyword forms from prose in comments and notes. The baseline recorded for 2026-11-10 on `624243a` reported 0 exception blocks and 2 transitions; the same two measured counts are present here.

## Inventory

| Construct | Corpus uses | Implementation | Existing tests |
|---|---:|---|---|
| `default` declaration | 0 | `litigate-syntax`: grammar `crates/litigate-syntax/src/grammar/decls.rs:284`, AST `src/ast/decls.rs:188`; `litigate-sema`: parse/type build `crates/litigate-sema/src/db/parse/items.rs:167`, `src/types/build/logic.rs:26`, evaluation `src/precedence/defeasible.rs:80`; no runtime lowering (see `crates/litigate-lower/src`). | Syntax AST `crates/litigate-syntax/src/ast/tests/decls.rs:144,262,270`; sema `crates/litigate-sema/tests/precedence.rs`, `tests/derivations.rs`, `tests/cov_build_logic.rs`; verify obligation `crates/litigate-verify/tests/obligations.rs:184`. |
| `exception` declaration | 0 | Grammar `crates/litigate-syntax/src/grammar/decls.rs:299-310`, AST `src/ast/decls.rs:202`; sema construction `crates/litigate-sema/src/types/build/logic.rs:49`, evaluation `src/precedence/defeasible.rs:80-90, 201-226`; verification-only, not emitted by `litigate-lower`. The markers are `rebuts` and `undercuts`; neither appears as a declaration in `lit/`. | Syntax `crates/litigate-syntax/src/ast/tests/decls.rs:154-176`; sema `crates/litigate-sema/tests/precedence.rs`, `tests/derivations.rs`, `tests/cov_rules_nosource.rs`; verify `crates/litigate-verify/tests/obligations.rs:184-189`. |
| `transition` declaration | 2 | Grammar `crates/litigate-syntax/src/grammar/decls.rs:250`, AST `src/ast/decls.rs:154`; sema validation `crates/litigate-sema/src/temporal/transition.rs:201-429`, matter event selection in the same file `:125-198`; HIR case graph `crates/litigate-sema/src/case/graph.rs:610`. It governs replay and is not a runtime pack field. | Syntax `crates/litigate-syntax/src/ast/tests/decls.rs:114`; sema `crates/litigate-sema/tests/change.rs`, `tests/bitemporal.rs`, `tests/case.rs`, `tests/frcp_2009.rs`; verify invalid-transition obligations `crates/litigate-verify/tests/obligations.rs`. |
| `subject_to` relation | 0 | Relation grammar `crates/litigate-syntax/src/grammar/clauses.rs:174-198`; relation parsing and precedence in `crates/litigate-sema/src/precedence/`. | `crates/litigate-sema/tests/precedence.rs`, `tests/cov_build_meta.rs`; syntax coverage `crates/litigate-syntax/tests/fixtures/constructs.lit:110`. |
| `notwithstanding` relation | 49 literal occurrences (includes comments, tests, and syntax examples) | Same relation grammar; precedence implementation `crates/litigate-sema/src/precedence/`. | `crates/litigate-sema/tests/precedence.rs`, `tests/cov_build_meta.rs`, `tests/derivations.rs`; syntax fixture `crates/litigate-syntax/tests/fixtures/constructs.lit:110`. |
| `revives` relation | 1 literal occurrence | Relation grammar `crates/litigate-syntax/src/grammar/clauses.rs:174-198`; version activity in `crates/litigate-sema/src/temporal/version.rs` and `src/temporal/activity.rs`. | `crates/litigate-sema/tests/bitemporal.rs`, `tests/change.rs`; grammar/recovery coverage `crates/litigate-syntax/tests/recovery_decls.rs:24-38`. |
| `rebuts` marker | 1 literal occurrence, in parser fixture/example material; no `lit/` declaration use | `crates/litigate-syntax/src/grammar/decls.rs:305-310`; AST marker accessor `crates/litigate-syntax/src/ast/decls.rs:202-210`; semantics `crates/litigate-sema/src/types/build/logic.rs:71-90` and `src/precedence/defeasible.rs`. | Syntax `crates/litigate-syntax/src/ast/tests/decls.rs:154-160`; sema `crates/litigate-sema/tests/precedence.rs`; verify `crates/litigate-verify/tests/obligations.rs`. |
| `undercuts` marker | 0 corpus occurrences; no `lit/` declaration use | Same parser and AST sites; semantics in `crates/litigate-sema/src/types/build/logic.rs:71-90` and `src/precedence/defeasible.rs`. | Syntax `crates/litigate-syntax/src/ast/tests/decls.rs:163-176`; sema `crates/litigate-sema/tests/precedence.rs`, `tests/cov_rules_nosource.rs`; verify `crates/litigate-verify/tests/obligations.rs`. |

Measured commands and outputs:

```text
rg -n '^\s*(default|exception|transition)\b' lit --glob '*.lit' | wc -l
3
rg -n '^\s*(default|exception|transition)\b' lit --glob '*.lit'
lit/examples/rule12.lit:77:  default "Let the deadline pass" -> default-risk { sets [defaulted], valence bad }
lit/core/frcp6.lit:256:transition frcp6-2009 amends frcp6@2009-12-01 cite "FRCP 86(a)(2)(B)"
lit/core/frcp6.lit:275:transition frcp6-2016 amends frcp6@2016-12-01 cite "FRCP 86(a)(2)(B)"
rg -o 'subject_to|notwithstanding|revives|rebuts|undercuts' lit --glob '*.lit' | count each keyword
subject_to 0; notwithstanding 49; revives 1; rebuts 1; undercuts 0
```

(The `default` at `lit/examples/rule12.lit:77` is a case choice edge, not a `default` rule declaration. The 49 relation-word hits and marker hits count all literal occurrences, not just active declarations.)

## Adoption assessment and removal case

No forum rule was changed. The candidate rules do not provide three safe, semantics-preserving ports for either construct:

1. **FRCP 6(b)(2)'s no-extension list:** current rules encode this as a typed `lock against [...]` on each relevant deadline (`lit/frcp/periods.lit:417-446`). An exception would require a default value to rebut/undercut plus applicable history conditions. These deadlines have distinct triggers, extension capabilities, and other consequences; replacing the locks is not a direct representation preserving the lowered clocks or extension behavior. This is not an adoption candidate for `default`/`exception`.
2. **FRCP 12(a)(2)-(3) United States answer periods:** the answer periods are distinct event-triggered deadlines from FRCP 12(a)(1), with different periods and triggers. A default/exception over one value would erase the current independently addressable deadlines; keeping both retains the present encoding rather than expressing it more directly. No safe lowering-preserving port identified.
3. **FRAP 4(a)(1)(B):** the 60-day period is separately modeled from the 30-day period in the current appeal modules and tests (for example `lit/frap/appeal-periods.lit:92` and `:199`). A default/exception would require a fact-conditioned single value and its source fact, while the current deadline model retains both rule authorities as separate deadlines. The language's defeasible evaluator does not lower into runtime deadline selection. No safe port identified.
4. **FRCP 86(a):** both currently authored transitions (`lit/core/frcp6.lit:256,275`) already use `transition`. They exhaust the named amendment examples; no three additional transition candidates were identified in the current corpus. The transition construct changes matter replay's governing rule version and has focused tests, so removing it would change behavior.

The proof criterion is lowered-value equality. `default`/`exception` declarations are excluded from `litigate-lower` output and are enforced as sema/verification constructs; thus a port that changes their logic without a runtime consumer cannot demonstrate the requested lowered-value result. Transition declarations participate in replay, so duplicating or rewriting their two existing declarations without a further legal case would be cosmetic and would not establish three additional use cases.

**Separate removal plan, pending David's ruling (no removal performed):**

- `default`/`exception`, including the `rebuts`/`undercuts` distinction: remove the declaration productions and AST nodes in `docs/litigate/SPEC.md` §5.2, §5.5 and grammar §3.3; DESIGN §4.5; `crates/litigate-syntax/src/grammar/decls.rs`, AST declaration/node/item surfaces, syntax-kind/vocab/keywords and tree-sitter grammar/generated parser; sema HIR, parse/build/type/precedence/defeasible and verification obligation/evidence paths; corresponding syntax fixtures/snapshots, sema precedence/derivation/logic tests, verify obligation tests, and related docs/examples. This is a separate implementation bead and requires deleting tests only as part of deleting the feature, never to mask a failure.
- `transition`: do not recommend removal based on this inventory. Its two occurrences encode FRCP 86(a)'s two amendment windows and drive replay; removal needs a separate semantics design and ruling, and would require removing SPEC §5.4 and DESIGN §4.4 plus the parser/AST/sema temporal replay implementation and tests listed in the inventory.
- Relation words (`subject_to`, `notwithstanding`, `revives`) and both exception markers should be evaluated with their owning constructs; do not delete them independently from the parser while their semantics remain specified.

DECIDED: document the no-port evidence and non-destructive removal plan -- candidates are separate deadlines or the already-authored FRCP 86(a) transitions, and forcing them into defeasible declarations would fail the lowered-output and directness tests.
