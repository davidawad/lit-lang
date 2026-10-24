//! `workflow-sound` (SPEC §7): the lowered pack as a bounded workflow net.
//! Each test pins the findings of one unsound shape: a marking the flags
//! leave stuck, a cycle with no way out, an edge no marking enables, a
//! duty no act can consume, and a net past the marking bound.
mod support;

use litigate_base::Profile;
use litigate_verify::{Checked, Obligation, ObligationStatus};
use serde_json::json;
use std::fmt::Write as _;
use support::{edit, outcome, verify_one, verify_pack_edited, with, BASE};

/// The `workflow-sound` finding and diagnostics of `lit`, in `dev`.
fn sound(lit: &str) -> serde_json::Value {
    let c = verify_one(lit, Profile::Dev);
    json!(outcome(&c, Obligation::WorkflowSound))
}

fn status(c: &Checked) -> Option<ObligationStatus> {
    c.verification
        .report
        .finding(Obligation::WorkflowSound)
        .map(|f| f.status)
}

#[test]
fn the_base_program_is_sound() {
    let c = verify_one(BASE, Profile::Strict);
    assert_eq!(status(&c), Some(ObligationStatus::Passed));
}

#[test]
fn a_start_tagged_entry_is_one_root() {
    let lit = edit(
        "answer \"File an answer\"",
        "tags [entry]\n  answer \"File an answer\"",
    );
    let c = verify_one(&lit, Profile::Strict);
    let found = c.verification.report.finding(Obligation::WorkflowSound);
    assert_eq!(
        found.and_then(|f| f.evidence.as_ref()).map(|e| e.checked),
        Some(2)
    );
}

#[test]
fn a_pack_edge_naming_no_node_of_the_pack_is_not_a_transition() {
    let v = verify_pack_edited(BASE, |pack| {
        let edge = pack.edges[0].clone();
        pack.edges.push(litgraph::model::RawEdge {
            from: "ghost".into(),
            ..edge.clone()
        });
        pack.edges.push(litgraph::model::RawEdge {
            to: "ghost".into(),
            ..edge
        });
    });
    let found = v.report.finding(Obligation::WorkflowSound);
    assert_eq!(
        found.map(|f| f.status),
        Some(ObligationStatus::Passed),
        "{v:#?}"
    );
}

#[test]
fn a_node_whose_every_edge_its_flags_disable_is_stuck() {
    // `answer` requires `late`, which nothing sets: `served` enables no
    // edge, and `answer` is dead.
    insta::assert_json_snapshot!(sound(&edit(
        "within answer-period }",
        "within answer-period, requires [late] }"
    )));
}

#[test]
fn a_cycle_with_no_way_out_cannot_complete() {
    insta::assert_json_snapshot!(sound(&with(
        "state loop-a: \"A\" { tags [entry], _ \"Go\" -> loop-b }\n\
         state loop-b: \"B\" { _ \"Back\" -> loop-a }"
    )));
}

#[test]
fn an_edge_no_reached_flag_set_enables_is_dead() {
    // `served` is reached with no flags and with `{a}`, `{b}` and `{a,
    // b}`: `answer`, which forbids `a` and `b`, is enabled only with none;
    // `again`, which needs `c`, never is. `served` loses its option to
    // complete twice and is reported once.
    insta::assert_json_snapshot!(sound(&edit(
        "within answer-period }",
        "within answer-period, forbids [a, b] }\n  \
         twice \"Serve again\" -> served { sets [a] }\n  \
         thrice \"Serve a third time\" -> served { sets [b] }\n  \
         again \"Answer again\" -> answered { requires [c] }"
    )));
}

#[test]
fn a_duty_no_marking_can_act_on_cannot_be_consumed() {
    // Answering triggers `reply-period`, and the matter then ends. The
    // answer fires with and without `marked`, and is reported once.
    insta::assert_json_snapshot!(sound(
        &with(
            "rule duty reply-duty cite \"FRCP 12(a)\"\n{\n  deadline reply-period: 14.days from \
             Answered(complaint) counted_by frcp6\n}"
        )
        .replacen(
            "within answer-period }",
            "within answer-period }\n  mark \"Mark\" -> served { sets [marked] }",
            1
        )
    ));
}

#[test]
fn a_duty_some_marking_can_act_on_is_consumable() {
    // The reply is offered after the answer, and one path skips it.
    let lit = with(
        "rule duty reply-duty cite \"FRCP 12(a)\"\n{\n  deadline reply-period: 14.days from \
         Answered(complaint) counted_by frcp6\n}\n\
         state replying: \"Reply due\" { reply \"Reply\" -> answered { within reply-period }, \
         skip \"Let it pass\" -> answered, wait \"Wait\" -> replying }",
    )
    .replacen("-> answered { event", "-> replying { event", 1);
    let c = verify_one(&lit, Profile::Dev);
    assert_eq!(status(&c), Some(ObligationStatus::Passed), "{c:#?}");
}

#[test]
fn a_long_firing_sequence_shows_its_last_edges() {
    // Fourteen states from `start` to a stuck one: the note keeps the
    // last twelve edges.
    let chain = (0..14).fold(String::new(), |mut out, i| {
        let _ = writeln!(out, "state s{i}: \"S{i}\" {{ e{i} \"Go\" -> s{} }}", i + 1);
        out
    });
    let lit = edit(
        "answer \"File an answer\" -> answered",
        "answer \"File an answer\" -> s0",
    );
    let lit = with(&format!(
        "{chain}state s14: \"Stuck\" {{ stop \"Stop\" -> answered {{ requires [never] }} }}"
    ))
    .replace(BASE, &lit);
    insta::assert_json_snapshot!(sound(&lit));
}

#[test]
fn a_net_past_the_marking_bound_is_undecided() {
    // Seventeen independent flags on one node: 2^17 markings.
    let toggles = (0..17).fold(String::new(), |mut out, i| {
        let _ = write!(out, ", f{i} \"Flag {i}\" -> served {{ sets [f{i}] }}");
        out
    });
    let lit = edit(
        "within answer-period }",
        &format!("within answer-period }}{toggles}"),
    );
    let c = verify_one(&lit, Profile::Strict);
    let found = outcome(&c, Obligation::WorkflowSound);
    assert_eq!(status(&c), Some(ObligationStatus::Failed));
    insta::assert_json_snapshot!(json!(found));
}
