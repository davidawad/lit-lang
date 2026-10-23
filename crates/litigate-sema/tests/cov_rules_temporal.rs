//! Temporal edge cases (SPEC §5.4): supersession and revival as `as_of`
//! sees them, overlaps and gaps the fixes cannot or can repair, and the
//! ways `governing` refuses to pick a version.
mod cov_rules_support;
mod legal;
use cov_rules_support::{demo, edit, world, Pack};
use legal::table;
use litigate_sema::hir::{Declaration, Expr, ExprKind, Type, Value};
use litigate_sema::temporal::{self, condition, governing, Key, MatterEvent, World};

fn d(id: &str, days: u32, clauses: &str) -> String {
    format!("deadline {id}: {days}.days from Served(complaint) counted_by c6 {clauses}\n")
}

fn key(name: &str) -> Key {
    Key {
        pack: "demo".into(),
        name: name.into(),
    }
}

#[test]
fn overlaps_gaps_and_relations_table() {
    let two_then_gap = d("p", 20, "effective [2009-12-01, 2012-01-01)")
        + &d("p", 21, "effective [2012-01-01, 2014-01-01)");
    insta::assert_debug_snapshot!(table(&[
        (
            "1 unversioned self-supersession documents succession",
            &(d("p", 20, "effective [2009-12-01, 2016-12-01)")
                + &d("p", 21, "effective [2016-12-01, ..) supersedes p")),
        ),
        (
            "2 versions starting together overlap with no fix",
            &(d("p", 20, "effective [2009-12-01, ..)")
                + &d("p", 21, "effective [2009-12-01, 2012-01-01)")),
        ),
        (
            "3 a later-written earlier version is the one to close",
            &(d("p", 21, "effective [2016-12-01, ..)")
                + &d("p", 20, "effective [2009-12-01, 2017-01-01)")),
        ),
        ("4 trailing gap after two versions", &two_then_gap),
        (
            "5 a third declaration's revives ends a suspension",
            &(d("p", 20, "")
                + &d("q", 9, "effective [2012-01-01, 2014-01-01) supersedes p")
                + &d("r", 7, "effective [2014-01-01, ..) revives p")),
        ),
        (
            "6 amending the first version with a foreign arm",
            &(d("p", 20, "effective [2009-12-01, 2016-12-01)")
                + &d("p", 21, "effective [2016-12-01, ..)")
                + "transition tr amends p@2009-12-01 { otherwise -> p@2016-12-01 }\n"),
        ),
    ]));
}

/// The amount of the deadline version `as_of` selects, or the L3007
/// diagnostic with its notes.
fn amount_as_of(world: &World<'_>, name: &str, date: &str) -> String {
    let at = temporal::world::nowhere("query");
    match temporal::as_of(world, &key(name), date, None, at) {
        Ok(i) => match world.decls[i].decl {
            Declaration::Deadline(x) => format!("{}.days", x.amount),
            other => format!("not a deadline: {other:?}"),
        },
        Err(e) => {
            let notes: Vec<&str> = e.notes.iter().map(|n| n.message.as_str()).collect();
            format!("{} {} [{}]", e.code.as_str(), e.message, notes.join("; "))
        }
    }
}

#[test]
fn as_of_names_overlaps_versioned_supersession_and_third_party_revival() {
    let body = d("ov", 20, "effective [2009-12-01, ..)")
        + &d("ov", 21, "effective [2012-01-01, 2014-01-01)")
        + &d("p", 20, "")
        + &d("q", 9, "effective [2012-01-01, ..) supersedes p@2009-12-01")
        + &d("p2", 20, "effective [2009-12-01, 2016-12-01)")
        + &d("p2", 21, "effective [2016-12-01, ..)")
        + &d(
            "q2",
            9,
            "effective [2012-01-01, ..) supersedes p2@2016-12-01",
        )
        + &d("p3", 20, "")
        + &d("q3", 9, "effective [2012-01-01, 2014-01-01) supersedes p3")
        + &d("r3", 7, "effective [2014-01-01, ..) revives p3")
        + &d("p4", 20, "effective [2009-12-01, 2016-12-01)")
        + &d("p4", 21, "effective [2016-12-01, ..) supersedes p4");
    let packs = demo(&body);
    let w = world(&packs);
    let got: Vec<String> = [
        ("ov", "2013-01-01"),
        ("p", "2013-01-01"),
        ("p2", "2013-01-01"),
        ("p2", "2017-01-01"),
        ("p3", "2013-01-01"),
        ("p3", "2015-01-01"),
        ("p4", "2017-01-01"),
    ]
    .iter()
    .map(|(n, at)| format!("{n} as_of {at}: {}", amount_as_of(&w, n, at)))
    .collect();
    insta::assert_debug_snapshot!(got);
}

const AMENDED: &str = "deadline p: 20.days from Served(complaint) counted_by c6 effective [2009-12-01, 2016-12-01)
deadline p: 21.days from Served(complaint) counted_by c6 effective [2016-12-01, ..)
deadline q: 1.days from Served(complaint) counted_by c6 effective [2009-12-01, ..)
deadline q: 2.days from Served(complaint) counted_by c6 effective [2016-12-01, ..)
deadline r: 1.days from Served(complaint) counted_by c6 effective [2009-12-01, 2016-12-01)
deadline r: 2.days from Served(complaint) counted_by c6 effective [2016-12-01, ..)
transition foreign amends p@2016-12-01 { unless 1 < 2 -> p@2009-12-01, otherwise -> p@2016-12-01 }
transition elsewhere amends r@2016-12-01 { unless has_event(Ordered()) -> q@2009-12-01, otherwise -> r@2016-12-01 }
";

/// The `has_event` pattern of the first arm of transition `id`.
fn pattern<'w>(w: &World<'w>, id: &str) -> Option<&'w Expr> {
    w.decls.iter().find_map(|d| match d.decl {
        Declaration::Transition(t) if t.meta.id == id => {
            t.arms[0].unless.as_ref().and_then(condition::has_event)
        }
        _ => None,
    })
}

fn outcome(w: &World<'_>, name: &str, events: &[MatterEvent<'_>], at: usize) -> String {
    match governing(w, &key(name), events, at, None) {
        Ok(g) => format!(
            "version {} via {:?}",
            w.decls[g.version].meta.effective.from, g.via
        ),
        Err(e) => format!("{e:?}"),
    }
}

#[test]
fn governing_refuses_unknown_events_inactive_dates_and_bad_transitions() {
    let packs = demo(AMENDED);
    let w = world(&packs);
    let filed = Expr {
        ty: Type::Unit,
        kind: ExprKind::Literal { value: Value::Unit },
        span: None,
    };
    let ordered = pattern(&w, "elsewhere").expect("transition with a has_event arm");
    let ev = |date, action| MatterEvent { date, action };
    let pending = [ev("2016-11-20", &filed), ev("2016-12-05", ordered)];
    let early = [ev("2009-01-01", &filed)];
    insta::assert_debug_snapshot!([
        outcome(&w, "p", &pending, 5),
        outcome(&w, "p", &early, 0),
        outcome(&w, "p", &pending, 1),
        outcome(&w, "r", &pending, 1),
        outcome(&w, "r", &pending[..1], 0),
        outcome(&w, "q", &pending, 1),
    ]);
}

#[test]
fn a_transition_without_otherwise_selects_nothing_when_no_arm_holds() {
    let mut packs = demo(AMENDED);
    edit(&mut packs, |d| {
        if let Declaration::Transition(t) = d {
            t.arms.truncate(1);
        }
    });
    let w = world(&packs);
    let filed = Expr {
        ty: Type::Unit,
        kind: ExprKind::Literal { value: Value::Unit },
        span: None,
    };
    let events = [
        MatterEvent {
            date: "2016-11-20",
            action: &filed,
        },
        MatterEvent {
            date: "2016-12-05",
            action: &filed,
        },
    ];
    assert_eq!(
        outcome(&w, "r", &events, 1),
        "Transition(\"`elsewhere` has no `otherwise` arm\")"
    );
}

/// Transitions whose amendment names no version, or a version that does
/// not exist, are not registered and draw no transition diagnostics (name
/// resolution reports them, L3008 / L1002).
#[test]
fn transitions_without_a_resolvable_amendment_are_skipped() {
    let base = d("p", 20, "effective [2009-12-01, 2016-12-01)")
        + &d("p", 21, "effective [2016-12-01, ..)")
        + "transition tr amends p@2016-12-01 { unless has_event(Ordered()) -> p@2016-12-01, otherwise -> p@2016-12-01 }\n";
    for amendment in ["p", "p@2010-01-01"] {
        let mut packs: Vec<Pack> = demo(&base);
        edit(&mut packs, |d| {
            if let Declaration::Transition(t) = d {
                amendment.clone_into(&mut t.amendment);
            }
        });
        let w = world(&packs);
        assert_eq!(w.transition_for(&key("p"), "2016-12-01"), None);
        let registered = temporal::transition::transition_for(&w, &key("p"), "2010-01-01");
        assert_eq!(registered.is_some(), amendment == "p@2010-01-01");
        let found = litigate_sema::precedence::check(&w, &temporal::world::NoSource, 0);
        assert!(found.is_empty(), "{amendment}: {found:?}");
    }
}

#[test]
fn targets_parse_qualified_names_and_only_valid_version_dates() {
    let w = World::default();
    let show = |text: &str| {
        let t = w.target("demo", text);
        format!("{text} -> {}::{} @{:?}", t.key.pack, t.key.name, t.version)
    };
    insta::assert_debug_snapshot!([
        show("p"),
        show("p@2016-12-01"),
        show("frcp::p@2016-12-01"),
        show("p@2016-02-30"),
        show("p@soon"),
    ]);
}
