/// One docket row per event of `case`, in source order (dates sort them
/// later): its layer is the package of the edge it took, else of a
/// deadline it triggered, else `primary`. With the rows, the last row
/// whose event moved the matter (took an edge): the cause of the posture.
fn events(
    b: &mut Builder,
    m: &CaseCheck,
    case: &hir::Case,
    primary: &str,
) -> (Vec<usize>, Option<usize>) {
    let spans = litigate_sema::case::resolve(&m.path, texts_of(b, &m.path), b.all, None).spans;
    let mut rows = Vec::new();
    let mut moved = None;
    for (i, e) in case.events.iter().enumerate() {
        let text = hirx::event_text(&e.action);
        let step = m
            .replay
            .iter()
            .find(|s| s.date == e.date && s.event == text);
        let taken = step.and_then(|s| {
            let (pkg, node) = b.node(&s.from)?;
            let edge = node.edges.iter().find(|x| {
                x.event
                    .as_ref()
                    .is_some_and(|p| hirx::matches(p, &e.action))
                    && target(pkg, &x.to) == s.to
            })?;
            Some((pkg, node, edge))
        });
        let layer = taken
            .map(|(p, _, _)| p.manifest.name.clone())
            .or_else(|| {
                m.counted
                    .iter()
                    .find(|s| s.trigger_event == i)
                    .and_then(|s| s.deadline.split_once("::"))
                    .and_then(|(pack, _)| b.package(pack))
                    .map(|p| p.manifest.name.clone())
            })
            .unwrap_or_else(|| primary.to_owned());
        let relation = match e.relation {
            hir::EventParty::By => "by",
            hir::EventParty::On => "on",
        };
        let row = Draft {
            key: event_key(b.all, e),
            world: 0,
            kind: RowKind::Event,
            date: Some(e.date.clone()),
            due: None,
            label: format!("{text} {relation} {}", e.party),
            status: status(RowState::None, awaited(&m.counted, i)),
            cite: e
                .authority
                .clone()
                .or_else(|| taken.and_then(|(_, _, x)| x.authority.clone())),
            source: b.source(spans.events.get(i).and_then(|s| s.whole.as_ref())),
            layer,
            via: Vec::new(),
            trigger: None,
            replaced_by: None,
        };
        // The edge's authority is in force with its node's version.
        if let Some((pkg, node, edge)) = taken.filter(|_| e.authority.is_none()) {
            let from = &node.meta.effective.from;
            b.prove(pkg, edge.authority.as_deref(), from, row.source.as_ref());
        }
        let row = b.push(row);
        b.limit_row(0, &text, Some(&e.date), row);
        if step.is_some_and(|s| s.effect == Effect::Took) {
            moved = Some(row);
        }
        rows.push(row);
    }
    (rows, moved)
}

/// One docket row per obligation instance replay created, triggered by
/// its event's row; the qualified ids of the deadlines given a row.
fn deadlines(
    b: &mut Builder,
    m: &CaseCheck,
    case: &hir::Case,
    event_rows: &[usize],
) -> Vec<String> {
    let closed = |at: Option<usize>| {
        at.map(|j| {
            let e = &case.events[j];
            (hirx::event_text(&e.action), e.date.clone())
        })
    };
    let mut seen: Vec<String> = Vec::new();
    // The row of each instance, parallel to `m.counted`.
    let mut made: Vec<Option<usize>> = vec![None; m.counted.len()];
    for (n, s) in m.counted.iter().enumerate() {
        seen.push(s.deadline.clone());
        let ended = closed(s.closed_by);
        let ended = ended.as_ref().map(|(e, d)| (e.as_str(), d.as_str()));
        let trigger = Some(event_rows[s.trigger_event]);
        let name = event_name(&case.events[s.trigger_event].action);
        let Some(due) = s.extended_to.clone().or_else(|| s.due.clone()) else {
            // The clock cannot count it: no due date, and why (an act
            // against it says so itself, L4010).
            if let Some((pkg, decl)) = b.deadline(&s.deadline) {
                let mut state = obligation_status(s.status, ended, None, &b.as_of, &name);
                if !matches!(s.status, Status::Unjudged | Status::Unknown) {
                    state.reason = Some(format!(
                        "{}: {}",
                        state.reason.unwrap_or_default(),
                        s.unknown.as_deref().unwrap_or_default()
                    ));
                }
                let state = awaiting(state, s);
                made[n] = Some(b.deadline_row(0, pkg, decl, None, state, trigger));
            }
            continue;
        };
        let Some((pkg, decl)) = b.counted_by(s) else {
            continue;
        };
        let state = awaiting(
            obligation_status(
                s.status,
                ended,
                Some(&due),
                s.judged_at.as_deref().unwrap_or(&b.as_of),
                &name,
            ),
            s,
        );
        let row = b.deadline_row(0, pkg, decl, Some(due), state, trigger);
        made[n] = Some(row);
        if let (Status::Superseded, Some(j)) = (s.status, s.closed_by) {
            b.supersedes.push((event_rows[j], row));
        }
    }
    for n in 0..m.counted.len() {
        if let Some(row) = made[n] {
            b.rows[row].replaced_by = replacement(&m.counted, n).and_then(|k| made[k]);
        }
    }
    seen
}

/// The instance that replaced instance `n` of `counted`: for a period
/// restarted by a later trigger, the instance of the same deadline that
/// trigger created; for one a `later_of` instance absorbed, that instance
/// (created by the event that closed it, with the absorbed date as its
/// floor). `None` for any other end.
fn replacement(counted: &[Scheduled], n: usize) -> Option<usize> {
    let s = &counted[n];
    let by = s.closed_by?;
    let restarted = match s.status {
        Status::Superseded => true,
        Status::Merged => false,
        _ => return None,
    };
    counted.iter().position(|o| {
        o.trigger_event == by
            && if restarted {
                o.deadline == s.deadline
            } else {
                o.deadline != s.deadline && o.floor.is_some()
            }
    })
}

/// The loaded text of `path`.
fn texts_of<'t>(b: &Builder<'t>, path: &str) -> &'t str {
    b.texts.get(path).copied().unwrap_or("")
}

/// Qualified id of edge target `to` written in `pkg`.
fn target(pkg: &hir::Package, to: &str) -> String {
    if to.contains("::") {
        to.to_owned()
    } else {
        format!("{}::{to}", pkg.manifest.pack_id)
    }
}

/// The choice of taking `edge` of `node` (of `pkg`): a draw at a chance
/// node, else a party's election or a tribunal's decision.
fn choice(pkg: &hir::Package, node: &hir::Node, edge: &hir::Edge) -> TimelineChoice {
    let kind = match node.kind {
        NodeKind::Chance { .. } => ChoiceKind::Chance,
        NodeKind::Decision => ChoiceKind::Election,
        _ => match pkg.manifest.roles.get(&edge.actor) {
            Some(hir::ActorRole::Nature) => ChoiceKind::HypotheticalDecision,
            _ => ChoiceKind::Election,
        },
    };
    let probability = match &edge.probability {
        Some(hir::Authored::Known(p)) => litigate_lower::num::rational(*p),
        _ => None,
    };
    TimelineChoice {
        kind,
        label: edge.label.clone(),
        cite: edge.authority.clone(),
        probability,
    }
}

/// The fork at the matter's posture `node` (of `pkg`): a fork row in the
/// docket, triggered by `trigger` (the event that moved the matter there,
/// else none: `@start`), and one world per option still open to it.
fn fork(
    b: &mut Builder,
    case: &hir::Case,
    pkg: &hir::Package,
    node: &hir::Node,
    seen: &[String],
    trigger: Option<usize>,
) {
    let here = format!("{}::{}", pkg.manifest.pack_id, node.meta.id);
    let options: Vec<&hir::Edge> = node
        .edges
        .iter()
        .filter(|e| target(pkg, &e.to) != here)
        .filter(|e| {
            let id =
                e.id.as_ref()
                    .map(|x| format!("{}::{x}", pkg.manifest.pack_id));
            !case.removals.iter().any(|r| Some(&r.edge) == id.as_ref())
        })
        .collect();
    if options.is_empty() {
        return;
    }
    let layer = b.layer(pkg);
    let cause = trigger.map_or_else(|| "start".to_owned(), |t| b.rows[t].key.clone());
    let fork_key = format!("fork:{}@{cause}", decl_key(pkg, &node.meta.id));
    let fork_row = Draft {
        key: fork_key.clone(),
        world: 0,
        kind: RowKind::Fork,
        date: None,
        due: None,
        label: node.meta.label.clone(),
        status: status(
            RowState::Unknown,
            Some(format!("no outcome of `{here}` recorded as of {}", b.as_of)),
        ),
        cite: node.meta.cite.clone(),
        source: b.source(node.meta.span.as_ref()),
        layer: layer.clone(),
        via: Vec::new(),
        trigger,
        replaced_by: None,
    };
    let fork_at = b.push_cited(fork_row, pkg, &node.meta.effective.from);
    for edge in options {
        let world = b.worlds.len();
        b.worlds.push(TimelineWorld {
            id: format!("w{world}"),
            label: edge.label.clone(),
            parent: Some("w0".to_owned()),
            fork_row: None,
            choice: Some(choice(pkg, node, edge)),
        });
        b.forks.push(Some(fork_at));
        // The assumed event: the edge's event, else the edge, else where
        // it leads.
        let name = edge
            .event
            .as_ref()
            .map(event_name)
            .or_else(|| edge.id.clone())
            .unwrap_or_else(|| local(&edge.to).to_owned());
        let taken = Draft {
            key: format!("event:{name}@{fork_key}"),
            world,
            kind: RowKind::Event,
            date: None,
            due: None,
            label: edge.label.clone(),
            status: status(RowState::None, None),
            cite: edge.authority.clone(),
            source: b.source(edge.span.as_ref()),
            layer: layer.clone(),
            via: Vec::new(),
            trigger: Some(fork_at),
            replaced_by: None,
        };
        let taken = b.push_cited(taken, pkg, &node.meta.effective.from);
        if let Some(event) = &edge.event {
            b.limit_row(world, &hirx::event_text(event), None, taken);
            triggered(b, world, taken, event);
        }
        if let Some((next_pkg, next)) = b.node(&target(pkg, &edge.to)) {
            let mut seen = seen.to_vec();
            b.governing(world, next_pkg, next, &mut seen, Some(taken));
        }
    }
}
