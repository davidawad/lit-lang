//! Fact guards in replay (SPEC §5.1, §5.6): the `when` guards of a fact
//! node's edges derive the fact from the matter's history, before an
//! event and at `as_of`; a `facts` entry the history contradicts is
//! L4003; an unknown atom leaves the fact to the matter.
mod cov_case_support;

use cov_case_support::{check, compile, lock, matter, outcome, package, At};
use litigate_base::Profile;
use litigate_sema::case::{CaseSpans, Env, NoClock};
use litigate_sema::hir::{Declaration, Expr, ExprKind, Package, Type, Value};
use std::collections::BTreeMap;
use std::sync::OnceLock;

const OMEGA: &str = "module omega
start intake

event Served()
event Filed()
event Fixed()
event Noted()
event Benched()
event Waited()

counting oc cite \"FRCP 6(a)\" { exclude_trigger_day, include_last_day, holidays ocal }
calendar ocal { new-year: fixed(1, 1) effective [1870-06-28, ..) observed nearest_weekday }

deadline bar: 30.days from Served() counted_by oc
deadline never: 30.days from Benched() counted_by oc
deadline fixer: 10.days from Fixed() counted_by oc

state intake: \"Intake\" by Plaintiff
{
  file \"File\" -> check { event Filed() }
  ops \"Ops\" -> lt { event Noted() }
  unknown \"Unknown\" -> left { event Waited() }
  old \"Old\" -> old { event Benched() }
}

chance check: \"Timely?\" by Court
{
  tags [fact]
  timely \"Timely\" -> heard p 1/2 { when !has_event(Fixed()) && date_of(Filed()) <= deadline(bar) }
  late \"Late\" -> barred p 1/2 { when !has_event(Fixed()) && !(date_of(Filed()) <= deadline(bar)) }
}

state heard: \"Heard\" by Plaintiff
{
  go \"Go\" -> won { event Noted() }
}

chance lt: \"Lt\" by Court
{
  tags [fact]
  y \"Y\" -> le p 1/2 { when (date_of(Noted()) < 2010-03-01) }
  n \"N\" -> barred p 1/2 { when !(date_of(Noted()) < 2010-03-01) }
}

chance le: \"Le\" by Court
{
  tags [fact]
  y \"Y\" -> gt p 1/2 { when (date_of(Noted()) <= 2010-02-01) }
  n \"N\" -> barred p 1/2 { when !(date_of(Noted()) <= 2010-02-01) }
}

chance gt: \"Gt\" by Court
{
  tags [fact]
  y \"Y\" -> ge p 1/2 { when (date_of(Noted()) > 2010-01-01) }
  n \"N\" -> barred p 1/2 { when !(date_of(Noted()) > 2010-01-01) }
}

chance ge: \"Ge\" by Court
{
  tags [fact]
  y \"Y\" -> ne p 1/2 { when (date_of(Noted()) >= 2010-02-01) }
  n \"N\" -> barred p 1/2 { when !(date_of(Noted()) >= 2010-02-01) }
}

chance ne: \"Ne\" by Court
{
  tags [fact]
  y \"Y\" -> eq p 1/2 { when (date_of(Noted()) != 2010-01-01) }
  n \"N\" -> barred p 1/2 { when !(date_of(Noted()) != 2010-01-01) }
}

chance eq: \"Eq\" by Court
{
  tags [fact]
  y \"Y\" -> tail p 1/2 { when (date_of(Noted()) == 2010-02-01) }
  n \"N\" -> barred p 1/2 { when !(date_of(Noted()) == 2010-02-01) }
}

state tail: \"Tail\" by Plaintiff
{
  go \"Go\" -> won { event Filed() }
}

chance left: \"Left unknown\" by Court
{
  tags [fact]
  y \"Y\" -> right p 1/2 { when (date_of(Benched()) < 2010-03-01) }
  n \"N\" -> barred p 1/2 { when !(date_of(Benched()) < 2010-03-01) }
}

chance right: \"Right unknown\" by Court
{
  tags [fact]
  y \"Y\" -> won p 1/2 { when (2010-03-01 < deadline(never)) }
  n \"N\" -> barred p 1/2 { when !(2010-03-01 < deadline(never)) }
}

chance old: \"Old\" by Court effective [2009-12-01, 2011-01-01)
{
  tags [fact]
  y \"Y\" -> won p 1/2 { when has_event(Benched()) }
  n \"N\" -> barred p 1/2 { when !has_event(Benched()) }
}

state retired: \"Retired\" by Court effective [2011-01-01, ..) supersedes old
{
  y \"Y\" -> won
}

terminal won: \"Won\" { payoff 0.usd }
terminal barred: \"Barred\" { payoff 0.usd }
";

const STAKES: &str = "stakes { won: 0.usd, barred: 0.usd }";

fn omega() -> &'static [Package] {
    static P: OnceLock<Vec<Package>> = OnceLock::new();
    P.get_or_init(|| compile(&[package("omega", &[], "", OMEGA)]))
}

/// The outcome of a matter over `omega`, then the facts it lowers.
fn run(events: &[&str], tail: &str) -> Vec<String> {
    let text = matter("omega ^1.0.0", "", events, &format!("{tail}\n{STAKES}"));
    let checked = check(
        &text,
        omega(),
        Some(&lock(omega())),
        &At("2010-02-03"),
        Profile::Dev,
    );
    let mut out = outcome(&checked);
    out.extend(
        checked
            .case
            .iter()
            .flat_map(|c| c.facts.iter())
            .map(|(n, e)| format!("fact {n} = {e}")),
    );
    out
}

const SERVED: &str = "2010-01-04 Served() on d";
const ON_TIME: &str = "2010-01-20 Filed() by p";
const LATE: &str = "2010-03-01 Filed() by p";

#[test]
fn guards_table() {
    let rows: BTreeMap<&str, Vec<String>> = [
        (
            "as_of: the history derives the fact at the replayed posture",
            run(&[SERVED, ON_TIME], ""),
        ),
        (
            "as_of: a late filing derives the bar",
            run(&[SERVED, LATE], ""),
        ),
        (
            "before an event: the derived fact is followed",
            run(&[SERVED, ON_TIME, "2010-01-25 Noted() by p"], ""),
        ),
        (
            "agree: a stated fact the history derives is accepted",
            run(&[SERVED, ON_TIME], "facts { check: timely }"),
        ),
        (
            "contradict: a stated fact the history refutes is L4003",
            run(&[SERVED, ON_TIME], "facts { check: late }"),
        ),
        (
            "contradict before an event: L4003, and the derived fact is followed",
            run(
                &[SERVED, ON_TIME, "2010-01-25 Noted() by p"],
                "facts { check: late }",
            ),
        ),
        (
            "underivable: no guard holds, the stated fact resolves it",
            run(
                &[SERVED, "2010-01-10 Fixed() on p", ON_TIME],
                "facts { check: late }",
            ),
        ),
        (
            "underivable: no deadline instance, nothing derived",
            run(&[ON_TIME], ""),
        ),
        (
            "every comparison: a chain of facts followed before the event",
            run(&["2010-02-01 Noted() by p", "2010-02-02 Filed() by p"], ""),
        ),
        (
            "unknown left operand: no event matches `date_of`",
            run(&["2010-02-01 Waited() by p"], ""),
        ),
        (
            "unknown right operand: no instance of the deadline",
            run(
                &["2010-02-01 Waited() by p", "2010-02-02 Fixed() on p"],
                "facts { left: y }",
            ),
        ),
        (
            "not in force at as_of: nothing derived there",
            run(&["2010-02-01 Benched() by p"], ""),
        ),
    ]
    .into_iter()
    .collect();
    insta::assert_debug_snapshot!(rows);
}

/// A guard a client builds as HIR is evaluated as written: an atom that
/// is neither `has_event` nor a date comparison, or a date term that is
/// not a literal, `date_of` or `deadline(ref)`, is unknown.
#[test]
fn guards_built_as_hir_with_foreign_atoms_are_unknown() {
    fn ex(ty: Type, kind: ExprKind) -> Expr {
        Expr {
            ty,
            kind,
            span: None,
        }
    }
    let number = ex(
        Type::Number,
        ExprKind::Literal {
            value: Value::Number(litigate_sema::hir::Rational::integer(1)),
        },
    );
    let foreign = [
        ex(
            Type::Boolean,
            ExprKind::Ref {
                target: "flag".into(),
            },
        ),
        ex(
            Type::Boolean,
            ExprKind::Binary {
                op: litigate_sema::hir::BinaryOp::Lt,
                lhs: Box::new(number.clone()),
                rhs: Box::new(number.clone()),
            },
        ),
        ex(
            Type::Boolean,
            ExprKind::Binary {
                op: litigate_sema::hir::BinaryOp::Lt,
                lhs: Box::new(ex(
                    Type::Date,
                    ExprKind::Call {
                        callee: "deadline".into(),
                        args: vec![number.clone()],
                    },
                )),
                rhs: Box::new(number),
            },
        ),
    ];
    let mut rows = Vec::new();
    for guard in foreign {
        let mut packages = omega().to_vec();
        for d in packages
            .iter_mut()
            .flat_map(|p| p.modules.iter_mut())
            .flat_map(|m| m.declarations.iter_mut())
        {
            if let Declaration::Node(n) = d {
                if n.meta.id == "check" {
                    n.edges[0].guard = Some(guard.clone());
                    n.edges[1].guard = None;
                }
            }
        }
        let text = matter("omega ^1.0.0", "", &[SERVED, ON_TIME], STAKES);
        let refs: Vec<&Package> = packages.iter().collect();
        let pins = lock(&packages);
        let resolved = litigate_sema::case::resolve("cases/t.case", &text, &refs, Some(&pins));
        let case = resolved
            .case
            .unwrap_or_else(|| panic!("{:#?}", resolved.diagnostics));
        let checked = litigate_sema::case::check(
            case,
            &CaseSpans::default(),
            Env {
                packages: &refs,
                lock: Some(&pins),
                clock: &NoClock,
                profile: Profile::Dev,
            },
        );
        rows.push(checked.case.map(|c| c.facts).unwrap_or_default());
    }
    assert_eq!(rows, vec![BTreeMap::new(); 3]);
}
