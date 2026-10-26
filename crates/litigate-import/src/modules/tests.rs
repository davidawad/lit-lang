use super::{sections, Section};
use litigate_engine::model::{Group, Pack, RawEdge, RawNode};

fn pack(nodes: &[(&str, Option<&str>)], edges: &[(&str, &str)]) -> Pack {
    Pack {
        schema_version: 2,
        id: "p".into(),
        title: "P".into(),
        start_node_id: nodes.first().map(|n| n.0.to_owned()).unwrap_or_default(),
        groups: vec![Group {
            id: "g".into(),
            label: "G".into(),
        }],
        nodes: nodes
            .iter()
            .map(|(id, group)| RawNode {
                id: (*id).into(),
                label: (*id).into(),
                group: group.map(Into::into),
                ..RawNode::default()
            })
            .collect(),
        edges: edges
            .iter()
            .map(|(from, to)| RawEdge {
                from: (*from).into(),
                to: (*to).into(),
                label: "e".into(),
                actor: "either".into(),
                ..RawEdge::default()
            })
            .collect(),
        ..Pack::default()
    }
}

fn names(parts: &[Section]) -> Vec<(&str, std::ops::Range<usize>)> {
    parts
        .iter()
        .map(|s| (s.name.as_str(), s.nodes.clone()))
        .collect()
}

#[test]
fn runs_of_one_group_become_modules_named_without_the_group_suffix() {
    let p = pack(
        &[
            ("a", Some("x-group")),
            ("b", Some("x-group")),
            ("c", Some("y")),
            ("d", None),
        ],
        &[("a", "b"), ("b", "c"), ("c", "d")],
    );
    assert_eq!(
        names(&sections(&p)),
        [("x", 0..2), ("y", 2..3), ("ungrouped", 3..4)]
    );
}

#[test]
fn mutually_reaching_runs_merge_with_everything_between() {
    let p = pack(
        &[
            ("a", Some("x")),
            ("b", Some("y")),
            ("c", Some("z")),
            ("d", Some("w")),
        ],
        &[("a", "b"), ("b", "c"), ("c", "a"), ("c", "d")],
    );
    assert_eq!(names(&sections(&p)), [("x-to-z", 0..3), ("w", 3..4)]);
}

#[test]
fn a_group_that_recurs_gets_numbered_modules() {
    let p = pack(
        &[("a", Some("x")), ("b", Some("y")), ("c", Some("x"))],
        &[("a", "b"), ("b", "c")],
    );
    assert_eq!(
        names(&sections(&p)),
        [("x", 0..1), ("y", 1..2), ("x-2", 2..3)]
    );
}

#[test]
fn module_imports_are_acyclic_after_merging() {
    let p = pack(
        &[
            ("a", Some("x")),
            ("b", Some("y")),
            ("c", Some("z")),
            ("d", Some("w")),
            ("e", Some("v")),
        ],
        &[("a", "c"), ("c", "a"), ("b", "e"), ("e", "d"), ("d", "b")],
    );
    let parts = sections(&p);
    let index = |node: usize| parts.iter().position(|s| s.nodes.contains(&node));
    let ids = ["a", "b", "c", "d", "e"];
    let at = |id: &str| ids.iter().position(|x| *x == id).and_then(index);
    for (from, to) in [("a", "c"), ("c", "a"), ("b", "e"), ("e", "d"), ("d", "b")] {
        assert_eq!(at(from), at(to), "{from}->{to} crosses modules on a cycle");
    }
}

#[test]
fn edges_naming_unknown_nodes_do_not_affect_sections() {
    let p = pack(
        &[("a", Some("x")), ("b", Some("y"))],
        &[("a", "b"), ("b", "missing"), ("missing", "a")],
    );
    assert_eq!(names(&sections(&p)), [("x", 0..1), ("y", 1..2)]);
}

#[test]
fn an_empty_pack_has_no_sections() {
    assert_eq!(sections(&pack(&[], &[])), []);
}

#[test]
fn a_bare_suffix_group_keeps_its_name() {
    let p = pack(&[("a", Some("-group"))], &[]);
    assert_eq!(names(&sections(&p)), [("-group", 0..1)]);
}
