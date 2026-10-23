//! A matter checked as HIR (`case::check`), the contract a client that
//! builds `Case` and `Package` values itself relies on: pins against the
//! lock and the loaded packages, values only source resolution rules out,
//! and malformed declarations, each reported rather than trusted.
mod cov_case_support;

use cov_case_support::{fixture, lock, matter, outcome, ALPHA_ONLY, STAKES};
use litigate_base::Profile;
use litigate_sema::case::{check, resolve, CaseSpans, CheckedCase, Env, NoClock};
use litigate_sema::hir::{
    Case, DeadlineWindow, Declaration, Expr, ExprKind, Money, Package, Stake, Type, Value,
};
use litigate_sema::pkg::lock::LockFile;
use std::collections::BTreeMap;

const SERVED: &str = "2010-01-04 Served(complaint, Mail) on d";

/// `events` over `alpha` after `head`, resolved to HIR with its spans.
fn resolved(head: &str, events: &[&str]) -> (Case, CaseSpans) {
    let text = matter(ALPHA_ONLY, head, events, STAKES);
    let refs: Vec<&Package> = fixture().iter().collect();
    let resolved = resolve("cases/t.case", &text, &refs, Some(&lock(fixture())));
    let case = resolved
        .case
        .unwrap_or_else(|| panic!("{:#?}", resolved.diagnostics));
    (case, resolved.spans)
}

/// `events` over `alpha`, resolved to HIR.
fn case(events: &[&str]) -> Case {
    resolved("", events).0
}

/// Checks `case` without source spans.
fn checked(case: Case, packages: &[Package], lock: Option<&LockFile>) -> CheckedCase {
    spanned(case, &CaseSpans::default(), packages, lock)
}

/// Checks `case` with `spans`.
fn spanned(
    case: Case,
    spans: &CaseSpans,
    packages: &[Package],
    lock: Option<&LockFile>,
) -> CheckedCase {
    let refs: Vec<&Package> = packages.iter().collect();
    check(
        case,
        spans,
        Env {
            packages: &refs,
            lock,
            clock: &NoClock,
            profile: Profile::Dev,
        },
    )
}

fn row(case: Case) -> Vec<String> {
    outcome(&checked(case, fixture(), Some(&lock(fixture()))))
}

#[test]
fn pins_table() {
    let full = lock(fixture());
    let mut partial = full.clone();
    partial.packages.retain(|p| p.name != "alpha");
    let mut stale = full.clone();
    let zero = "0".repeat(64);
    for p in &mut stale.packages {
        p.digest = format!("sha256:{zero}");
    }
    let mut unloaded = case(&[SERVED]);
    let mut extra = unloaded.uses[0].clone();
    extra.name = "omega".into();
    unloaded.uses.push(extra);
    let mut other_version = case(&[SERVED]);
    other_version.uses[0].version = "9.9.9".into();
    let mut other_digest = case(&[SERVED]);
    other_digest.uses[0].content_hash.clone_from(&zero);
    let mut newer = full.clone();
    for p in &mut newer.packages {
        p.version = "9.9.9".into();
    }
    let rows: Vec<(&str, Case, Option<&LockFile>)> = vec![
        (
            "a pin of a package that is not loaded",
            unloaded,
            Some(&full),
        ),
        ("no lock", case(&[SERVED]), None),
        (
            "a lock without the package",
            case(&[SERVED]),
            Some(&partial),
        ),
        (
            "a pin whose version the lock disagrees with",
            other_version.clone(),
            Some(&full),
        ),
        (
            "a pin whose digest the lock disagrees with",
            other_digest.clone(),
            Some(&full),
        ),
        (
            "a pin the lock agrees with but the loaded package's digest does not",
            other_digest,
            Some(&stale),
        ),
        (
            "a pin the lock agrees with but the loaded package's version does not",
            other_version,
            Some(&newer),
        ),
    ];
    let table: BTreeMap<&str, Vec<String>> = rows
        .into_iter()
        .map(|(name, case, lock)| (name, outcome(&checked(case, fixture(), lock))))
        .collect();
    insta::assert_debug_snapshot!(table);
}

fn literal(value: Value) -> Expr {
    Expr {
        ty: Type::Unit,
        kind: ExprKind::Literal { value },
        span: None,
    }
}

#[test]
fn values_source_rules_out_table() {
    let mut no_start = case(&[]);
    no_start.start = Some("alpha::nowhere".into());
    let mut known_at = case(&[SERVED]);
    known_at.known_at = Some("2010-02-30".into());
    let mut event_date = case(&[SERVED]);
    event_date.events[0].date = "2010-02-30".into();
    let mut unsorted = case(&[SERVED, "2010-01-05 Benched(complaint) by p"]);
    unsorted.events.reverse();
    let mut start = case(&[SERVED]);
    start.start = Some("alpha::intake".into());
    let mut stake = case(&[]);
    stake.stakes.insert(
        "alpha::court-period".into(),
        Stake::Value(Money {
            minor_units: 0,
            currency: "usd".into(),
        }),
    );
    let mut unspanned = case(&["2010-01-04 Forfeited(\"zzzz\") by d cite FRCP 12(h)"]);
    if let ExprKind::Call { args, .. } = &mut unspanned.events[0].action.kind {
        args[0].span = None;
    }
    let mut not_string = case(&["2010-01-04 Forfeited(\"twin-a\") by d cite FRCP 12(h)"]);
    if let ExprKind::Call { args, .. } = &mut not_string.events[0].action.kind {
        args[0] = literal(Value::Boolean(true));
    }
    let not_call = |action: Expr| {
        let mut c = case(&[SERVED]);
        c.events[0].action = action;
        c
    };
    let unit = not_call(literal(Value::Unit));
    let reference = not_call(Expr {
        ty: Type::Unit,
        kind: ExprKind::Ref {
            target: "alpha::intake".into(),
        },
        span: None,
    });
    let list = not_call(Expr {
        ty: Type::Unit,
        kind: ExprKind::List { items: Vec::new() },
        span: None,
    });
    let rows: Vec<(&str, Case)> = vec![
        ("start: no such node", no_start),
        ("known_at: not a valid date", known_at),
        ("events: a date on no calendar day", event_date),
        ("events: out of order, with no source to sort", unsorted),
        ("start: disagrees, with no source to remove", start),
        ("stakes: a deadline is not a terminal", stake),
        ("removal: an option with no span of its own", unspanned),
        (
            "removal: an option that is not a string removes nothing",
            not_string,
        ),
        ("events: a unit action takes no edge", unit),
        ("events: a reference action takes no edge", reference),
        ("events: a list action takes no edge", list),
    ];
    let table: BTreeMap<&str, Vec<String>> = rows
        .into_iter()
        .map(|(name, case)| (name, row(case)))
        .collect();
    insta::assert_debug_snapshot!(table);
}

/// The fixture with `edit` applied to every declaration of `alpha`.
fn edited(edit: impl Fn(&mut Declaration)) -> Vec<Package> {
    let mut packages = fixture().to_vec();
    for p in packages.iter_mut().filter(|p| p.manifest.name == "alpha") {
        p.modules
            .iter_mut()
            .flat_map(|m| m.declarations.iter_mut())
            .for_each(&edit);
    }
    packages
}

#[test]
fn malformed_declarations_table() {
    let within_a_node = edited(|d| {
        if let Declaration::Node(n) = d {
            for e in &mut n.edges {
                if let Some(deadline) = e.deadline.as_mut() {
                    deadline.window = DeadlineWindow::Declared {
                        deadline: "intake".into(),
                    };
                }
            }
        }
    });
    let no_otherwise = edited(|d| {
        if let Declaration::Transition(t) = d {
            t.arms.retain(|a| a.unless.is_some());
        }
    });
    let lock = lock(fixture());
    let table: BTreeMap<&str, Vec<String>> = [
        (
            "within: names a node, not a deadline",
            checked(
                case(&[SERVED, "2010-01-05 Answered(complaint) by d"]),
                &within_a_node,
                Some(&lock),
            ),
        ),
        ("transition: no arm holds and there is no otherwise", {
            let (case, spans) = resolved(
                "",
                &[
                    "2010-01-04 Reopened(motion) by p",
                    "2010-07-01 Answered(motion) by p",
                ],
            );
            spanned(case, &spans, &no_otherwise, Some(&lock))
        }),
    ]
    .iter()
    .map(|(name, c)| (*name, outcome(c)))
    .collect();
    insta::assert_debug_snapshot!(table);
}

/// With the source's spans, findings about values the source could not
/// have held point at where the source wrote them.
#[test]
fn source_spans_locate_hir_findings_table() {
    let lock = lock(fixture());
    let (mut no_start, start_spans) = resolved("start respond", &[]);
    no_start.start = Some("alpha::nowhere".into());
    let (mut bad_date, date_spans) = resolved("", &[SERVED]);
    bad_date.events[0].date = "2010-02-30".into();
    let (mut unspanned, removal_spans) =
        resolved("", &["2010-01-04 Forfeited(\"zzzz\") by d cite FRCP 12(h)"]);
    if let ExprKind::Call { args, .. } = &mut unspanned.events[0].action.kind {
        args[0].span = None;
    }
    let (mut undeclared, undeclared_spans) = resolved(
        "",
        &["2010-01-04 Forfeited(\"twin-a\") by d cite FRCP 12(h)"],
    );
    if let ExprKind::Call { callee, .. } = &mut undeclared.events[0].action.kind {
        "Withdrawn".clone_into(callee);
    }
    let rows = [
        ("start: no such node", no_start, start_spans),
        ("events: a date on no calendar day", bad_date, date_spans),
        (
            "removal: an option with no span of its own",
            unspanned,
            removal_spans,
        ),
        (
            "removal: a removing name no used package declares is illegal",
            undeclared,
            undeclared_spans,
        ),
    ];
    let table: BTreeMap<&str, Vec<String>> = rows
        .into_iter()
        .map(|(name, case, spans)| {
            (
                name,
                outcome(&spanned(case, &spans, fixture(), Some(&lock))),
            )
        })
        .collect();
    insta::assert_debug_snapshot!(table);
}
