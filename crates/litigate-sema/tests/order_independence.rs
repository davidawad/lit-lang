//! Order independence (lg-flu.5, design review §6 item 4, P13): a
//! transition reads the history before an event in docket order, never a
//! later event of the same day; its arms are disjoint, so the one that
//! holds never depends on the order they are written in; and the
//! transitions of a world are a set, so load order picks none.
mod cov_rules_support;
mod legal;
use cov_rules_support::{demo, edit, world};
use legal::table;
use litigate_sema::hir::{Declaration, Expr, ExprKind, Type, Value};
use litigate_sema::temporal::transition::Seq;
use litigate_sema::temporal::{condition, governing, Key, MatterEvent, World};

const VERSIONS: &str =
    "deadline p: 20.days from Served(complaint) counted_by c6 effective [2009-12-01, 2016-12-01)
deadline p: 21.days from Served(complaint) counted_by c6 effective [2016-12-01, ..)
";

fn key(name: &str) -> Key {
    Key {
        pack: "demo".into(),
        name: name.into(),
    }
}

/// The `has_event` pattern of arm `arm` of the one transition.
fn pattern<'w>(w: &World<'w>, arm: usize) -> &'w Expr {
    w.decls
        .iter()
        .find_map(|d| match d.decl {
            Declaration::Transition(t) => t.arms[arm].unless.as_ref(),
            _ => None,
        })
        .and_then(|c| {
            condition::atoms(&[c])
                .into_iter()
                .find_map(condition::has_event)
        })
        .unwrap_or_else(|| panic!("arm {arm} tests has_event"))
}

fn outcome(w: &World<'_>, events: &[MatterEvent<'_>], at: usize) -> String {
    match governing(w, &key("p"), events, at, None) {
        Ok(g) => format!("p@{}", w.decls[g.version].meta.effective.from),
        Err(e) => format!("{e:?}"),
    }
}

fn unit() -> Expr {
    Expr {
        ty: Type::Unit,
        kind: ExprKind::Literal { value: Value::Unit },
        span: None,
    }
}

fn at<'e>(date: &'e str, seq: Seq, action: &'e Expr) -> MatterEvent<'e> {
    MatterEvent { date, seq, action }
}

#[test]
fn precedes_orders_dates_then_docket_numbers_and_ends_the_day_last() {
    let e = unit();
    let (d1, d2) = ("2016-12-05", "2016-12-06");
    let cases = [
        (
            at(d1, Seq::Unsequenced, &e),
            at(d2, Seq::Unsequenced, &e),
            true,
        ),
        (
            at(d2, Seq::Docket(1), &e),
            at(d1, Seq::Docket(9), &e),
            false,
        ),
        (at(d1, Seq::Docket(1), &e), at(d1, Seq::Docket(2), &e), true),
        (
            at(d1, Seq::Docket(2), &e),
            at(d1, Seq::Docket(2), &e),
            false,
        ),
        (
            at(d1, Seq::Docket(2), &e),
            at(d1, Seq::Docket(1), &e),
            false,
        ),
        (
            at(d1, Seq::Unsequenced, &e),
            at(d1, Seq::Docket(1), &e),
            false,
        ),
        (
            at(d1, Seq::Docket(1), &e),
            at(d1, Seq::Unsequenced, &e),
            false,
        ),
        (
            at(d1, Seq::Unsequenced, &e),
            at(d1, Seq::Unsequenced, &e),
            false,
        ),
        (
            at(d1, Seq::Unsequenced, &e),
            at(d1, Seq::EndOfDay, &e),
            true,
        ),
        (at(d1, Seq::Docket(7), &e), at(d1, Seq::EndOfDay, &e), true),
        (at(d1, Seq::EndOfDay, &e), at(d1, Seq::Docket(7), &e), false),
        (at(d1, Seq::EndOfDay, &e), at(d1, Seq::EndOfDay, &e), false),
    ];
    for (n, (a, b, want)) in cases.iter().enumerate() {
        assert_eq!(a.precedes(b), *want, "case {n}");
    }
    assert_eq!(MatterEvent::on(d1, &e).seq, Seq::default());
}

#[test]
fn a_later_same_day_order_does_not_govern_an_earlier_event() {
    let packs = demo(
        &(VERSIONS.to_owned()
            + "transition tr amends p@2016-12-01 { unless has_event(Ordered()) -> p@2009-12-01, otherwise -> p@2016-12-01 }\n"),
    );
    let w = world(&packs);
    let (filed, ordered) = (unit(), pattern(&w, 0));
    let day = "2016-12-05";
    // Docket 2 is filed, docket 3 is the order: the order governs itself
    // and what follows, not the filing before it.
    let docketed = [
        at("2016-11-20", Seq::Unsequenced, &filed),
        at(day, Seq::Docket(2), &filed),
        at(day, Seq::Docket(3), ordered),
    ];
    assert_eq!(outcome(&w, &docketed, 1), "p@2016-12-01");
    assert_eq!(outcome(&w, &docketed, 2), "p@2009-12-01");
    // The slice order is not the history order.
    let reversed = [docketed[2], docketed[1], docketed[0]];
    assert_eq!(outcome(&w, &reversed, 1), "p@2016-12-01");
    assert_eq!(outcome(&w, &reversed, 0), "p@2009-12-01");
    // Undocketed same-day events are unordered: neither is the other's
    // history, whichever is written first.
    let unordered = [
        at("2016-11-20", Seq::Unsequenced, &filed),
        at(day, Seq::Unsequenced, ordered),
        at(day, Seq::Unsequenced, &filed),
    ];
    assert_eq!(outcome(&w, &unordered, 2), "p@2016-12-01");
    // The end of the day sees the whole day.
    let end = [unordered[0], unordered[1], at(day, Seq::EndOfDay, &filed)];
    assert_eq!(outcome(&w, &end, 2), "p@2009-12-01");
}

#[test]
fn two_transitions_for_one_version_govern_nothing() {
    let packs = demo(
        &(VERSIONS.to_owned()
            + "transition tr amends p@2016-12-01 { otherwise -> p@2016-12-01 }\n"
            + "transition u amends p@2016-12-01 { otherwise -> p@2009-12-01 }\n"),
    );
    let w = world(&packs);
    assert_eq!(w.transitions(&key("p"), "2016-12-01").len(), 2);
    assert_eq!(w.transition(&key("p"), "2016-12-01"), None);
    assert_eq!(w.transition_for(&key("p"), "2016-12-01"), None);
    let filed = unit();
    let events = [
        MatterEvent::on("2016-11-20", &filed),
        MatterEvent::on("2016-12-05", &filed),
    ];
    assert_eq!(
        outcome(&w, &events, 1),
        "Transition(\"`tr` and `u` both amend `p@2016-12-01`\")"
    );
}

#[test]
fn arms_that_both_hold_with_different_versions_govern_nothing() {
    let both = "transition tr amends p@2016-12-01 { unless has_event(Ordered()) -> p@2009-12-01, unless has_event(Hearing(motion)) -> p@2016-12-01, otherwise -> p@2016-12-01 }\n";
    let packs = demo(&(VERSIONS.to_owned() + both));
    let w = world(&packs);
    let (ordered, hearing, filed) = (pattern(&w, 0), pattern(&w, 1), unit());
    let events = [
        MatterEvent::on("2016-11-20", &filed),
        MatterEvent::on("2016-12-01", ordered),
        MatterEvent::on("2016-12-02", hearing),
    ];
    assert_eq!(outcome(&w, &events, 1), "p@2009-12-01");
    assert_eq!(
        outcome(&w, &events, 2),
        "Transition(\"`tr`: two arms that select different versions both hold\")"
    );
    // Two holding arms that agree are no conflict.
    let mut agree = demo(&(VERSIONS.to_owned() + both));
    edit(&mut agree, |d| {
        if let Declaration::Transition(t) = d {
            t.arms[1].selects = "p@2009-12-01".into();
        }
    });
    let w = world(&agree);
    let (ordered, hearing) = (pattern(&w, 0), pattern(&w, 1));
    let events = [
        MatterEvent::on("2016-11-20", &filed),
        MatterEvent::on("2016-12-01", ordered),
        MatterEvent::on("2016-12-02", hearing),
    ];
    assert_eq!(outcome(&w, &events, 2), "p@2009-12-01");
    // A holding arm naming no version is refused.
    let mut unknown = demo(&(VERSIONS.to_owned() + both));
    edit(&mut unknown, |d| {
        if let Declaration::Transition(t) = d {
            t.arms[0].selects = "p@2001-01-01".into();
        }
    });
    let w = world(&unknown);
    let ordered = pattern(&w, 0);
    let events = [
        MatterEvent::on("2016-11-20", &filed),
        MatterEvent::on("2016-12-01", ordered),
    ];
    assert_eq!(
        outcome(&w, &events, 1),
        "Transition(\"arm selects unknown `p@2001-01-01`\")"
    );
}

#[test]
fn transition_arms_must_be_disjoint_table() {
    let arms = |body: &str| {
        format!(
            "{VERSIONS}transition tr amends p@2016-12-01 {{ {body}, otherwise -> p@2016-12-01 }}\n"
        )
    };
    let events: String = (0..17).map(|i| format!("event E{i}()\n")).collect();
    let any = |r: std::ops::Range<usize>| {
        r.map(|i| format!("has_event(E{i}())"))
            .collect::<Vec<_>>()
            .join(" || ")
    };
    let wide = format!(
        "{events}{}",
        arms(&format!(
            "unless {} -> p@2009-12-01, unless {} -> p@2016-12-01",
            any(0..9),
            any(9..17)
        ))
    );
    insta::assert_debug_snapshot!(table(&[
        (
            "1 overlapping arms selecting different versions",
            &arms("unless has_event(Ordered()) -> p@2009-12-01, unless has_event(Hearing(motion)) -> p@2016-12-01"),
        ),
        (
            "2 exclusive arms",
            &arms("unless has_event(Ordered()) -> p@2009-12-01, unless has_event(Hearing(motion)) && !has_event(Ordered()) -> p@2016-12-01"),
        ),
        (
            "3 overlapping arms selecting the same version",
            &arms("unless has_event(Ordered()) -> p@2009-12-01, unless has_event(Hearing(motion)) -> p@2009-12-01"),
        ),
        ("4 too many atoms to prove disjoint", &wide),
    ]));
}
