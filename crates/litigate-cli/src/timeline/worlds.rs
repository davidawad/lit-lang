/// The docket's untriggered deadlines that `world`'s assumed event
/// `taken` (the edge event `event`) triggers: each is a row of `world`
/// triggered by it, and the docket's row is re-emitted in `world` (its
/// key, a new id) `superseded`, replaced by the new row, so a diff of the
/// two worlds pairs them by key rather than showing a removal and an
/// addition. Each is already among the deadlines the world has a row
/// for, so the world's governing deadlines do not list it again.
fn triggered(b: &mut Builder, world: usize, taken: usize, event: &hir::Expr) {
    for (parent, id) in b.untriggered.clone() {
        let Some((owner, decl)) = b
            .deadline(&id)
            .filter(|(_, d)| hirx::matches(&d.trigger, event))
        else {
            continue;
        };
        let label = b.rows[taken].label.clone();
        let mut again = b.rows[parent].clone();
        again.world = world;
        again.status = status(
            RowState::Superseded,
            Some(format!("triggered in this world by `{label}`")),
        );
        let again = b.push(again);
        b.supersedes.push((taken, again));
        let reason = format!(
            "counted from {}, which this world assumes but no recorded event dates",
            hirx::event_text(&decl.trigger)
        );
        let row = b.deadline_row(
            world,
            owner,
            decl,
            None,
            status(RowState::Unknown, Some(reason)),
            Some(taken),
        );
        b.rows[again].replaced_by = Some(row);
    }
}

/// The timeline of `.lit` module `file` of `pkg`: each deadline it
/// declares that is in force at `as_of`, undated (no matter triggers it).
#[must_use]
pub fn of_lit(
    file: &str,
    pkg: &hir::Package,
    all: &[&hir::Package],
    texts: &[(String, String)],
    dates: (String, Option<String>),
    diagnostics_in: &[Diagnostic],
) -> Timeline {
    let (as_of, known_at) = dates;
    let mut b = Builder::new(all, texts, as_of, known_at);
    b.layer(pkg);
    let declared: Vec<&hir::Deadline> = hirx::selected(all, pkg, &b.as_of, b.known_at.as_deref())
        .into_iter()
        .filter_map(|d| match d {
            Declaration::Deadline(v) => Some(v),
            _ => None,
        })
        .filter(|v| v.meta.span.as_ref().is_some_and(|s| s.file.0 == file))
        .collect();
    for decl in declared {
        let reason = format!(
            "counted from {}; a rule module records no events",
            hirx::event_text(&decl.trigger)
        );
        b.deadline_row(
            0,
            pkg,
            decl,
            None,
            status(RowState::Unknown, Some(reason)),
            None,
        );
    }
    let source = TimelineSource {
        file: file.to_owned(),
        kind: TimelineSourceKind::Lit,
    };
    let diagnostics = diagnostics(&b, file, diagnostics_in);
    b.finish(source, diagnostics)
}

/// Keeps only the rows of enabled layers: a disabled layer's rows go, with
/// the worlds forking from them and their edges, but it stays listed.
pub fn apply_layers(t: &mut Timeline, disabled: &[String]) {
    for layer in &mut t.layers {
        layer.enabled = !disabled.contains(&layer.id);
    }
    let gone = |r: &TimelineRow| r.provenance.iter().any(|p| disabled.contains(p));
    let mut dropped: Vec<String> = t
        .rows
        .iter()
        .filter(|r| gone(r))
        .map(|r| r.id.clone())
        .collect();
    let mut dead_worlds: Vec<String> = Vec::new();
    for w in &t.worlds {
        let forked_from_gone = w.fork_row.as_ref().is_some_and(|f| dropped.contains(f));
        let parent_gone = w.parent.as_ref().is_some_and(|p| dead_worlds.contains(p));
        if forked_from_gone || parent_gone {
            dead_worlds.push(w.id.clone());
        }
    }
    dropped.extend(
        t.rows
            .iter()
            .filter(|r| dead_worlds.contains(&r.world))
            .map(|r| r.id.clone()),
    );
    retain(t, &dropped, &dead_worlds);
}

/// Keeps only world `id` and its ancestors, with each ancestor's rows up
/// to the fork row its child branches at.
///
/// # Errors
/// When no world is `id`, naming the worlds there are.
pub fn apply_world(t: &mut Timeline, id: &str) -> Result<(), String> {
    let mut path = Vec::new();
    let mut at = Some(id.to_owned());
    while let Some(w) = at.take() {
        let world = t.worlds.iter().find(|x| x.id == w).ok_or_else(|| {
            let ids: Vec<&str> = t.worlds.iter().map(|x| x.id.as_str()).collect();
            format!("no world `{w}`; worlds: {}", ids.join(", "))
        })?;
        at.clone_from(&world.parent);
        path.push((world.id.clone(), world.fork_row.clone()));
    }
    // Each ancestor ends at the fork row of the world below it on the path.
    let mut cut: BTreeMap<String, String> = BTreeMap::new();
    for pair in path.windows(2) {
        if let Some(fork) = &pair[0].1 {
            cut.insert(pair[1].0.clone(), fork.clone());
        }
    }
    let keep: Vec<String> = path.iter().map(|(w, _)| w.clone()).collect();
    let dead: Vec<String> = t
        .worlds
        .iter()
        .filter(|w| !keep.contains(&w.id))
        .map(|w| w.id.clone())
        .collect();
    let mut dropped = Vec::new();
    let mut past: Vec<String> = Vec::new();
    for r in &t.rows {
        if dead.contains(&r.world) || past.contains(&r.world) {
            dropped.push(r.id.clone());
        }
        if cut.get(&r.world) == Some(&r.id) {
            past.push(r.world.clone());
        }
    }
    retain(t, &dropped, &dead);
    Ok(())
}

/// Removes rows `dropped` and worlds `dead`, the edges touching either,
/// triggers naming a dropped row, and a `replaced_by` naming a key no
/// longer in its row's world (a replacing row is always in the world of
/// the row it replaces).
fn retain(t: &mut Timeline, dropped: &[String], dead: &[String]) {
    t.worlds.retain(|w| !dead.contains(&w.id));
    t.rows.retain(|r| !dropped.contains(&r.id));
    let kept: BTreeSet<(String, String)> = t
        .rows
        .iter()
        .map(|r| (r.world.clone(), r.key.clone()))
        .collect();
    for r in &mut t.rows {
        if r.trigger.as_ref().is_some_and(|x| dropped.contains(x)) {
            r.trigger = None;
        }
        if let Some(key) = r.replaced_by.take() {
            if kept.contains(&(r.world.clone(), key.clone())) {
                r.replaced_by = Some(key);
            }
        }
    }
    t.edges.retain(|e| {
        !dead.contains(&e.world) && !dropped.contains(&e.from) && !dropped.contains(&e.to)
    });
    // A proof is evidence for a row's cite: it goes with the last row at
    // its source.
    let proof = DiagnosticCode::CiteProvedByHistory.as_str();
    let rows = &t.rows;
    t.diagnostics
        .retain(|d| d.code != proof || rows.iter().any(|r| r.source == d.source));
}

/// An error (L1001) for each row whose `key` an earlier row of its world
/// already has, naming both, added to `t`'s diagnostics too. Keys derive
/// from cause, never position, so two such rows have nothing to tell them
/// apart until events declare ids.
pub fn check_keys(t: &mut Timeline) -> Vec<Diagnostic> {
    let mut found = Vec::new();
    for (n, r) in t.rows.iter().enumerate() {
        let Some(first) = t.rows[..n]
            .iter()
            .find(|p| p.world == r.world && p.key == r.key)
        else {
            continue;
        };
        let at = r.source.clone().unwrap_or_else(|| TimelineSpan {
            file: t.source.file.clone(),
            start: 0,
            end: 0,
            line: 1,
            column: 1,
        });
        let line = |x: &TimelineRow| {
            x.source
                .as_ref()
                .map_or_else(String::new, |s| format!(" (line {})", s.line))
        };
        let message = format!(
            "rows `{}` ({}{}) and `{}` ({}{}) of world `{}` share the key `{}`: nothing in their cause tells them apart; give the events docket numbers",
            first.id,
            first.label,
            line(first),
            r.id,
            r.label,
            line(r),
            r.world,
            r.key
        );
        let span = Span {
            file: at.file.as_str().into(),
            line: at.line,
            col: at.column,
            end_line: at.line,
            end_col: at.column,
        };
        found.push((
            Diagnostic::error(DiagnosticCode::DuplicateName, span, message.clone()),
            TimelineDiagnostic {
                code: DiagnosticCode::DuplicateName.as_str().to_owned(),
                severity: TimelineSeverity::Error,
                message,
                source: Some(at),
            },
        ));
    }
    found
        .into_iter()
        .map(|(d, td)| {
            t.diagnostics.push(td);
            d
        })
        .collect()
}

/// The authority names of the lattice package `pkg` sees (SPEC §5.5): the
/// `authority` chains it, its dependencies (transitively) and `core`
/// declare.
fn lattice<'p>(all: &[&'p hir::Package], pkg: &hir::Package) -> BTreeSet<&'p str> {
    let mut todo = vec![pkg.manifest.name.clone(), "core".to_owned()];
    let mut visited: BTreeSet<String> = BTreeSet::new();
    let mut names = BTreeSet::new();
    while let Some(name) = todo.pop() {
        if !visited.insert(name.clone()) {
            continue;
        }
        let Some(p) = all.iter().copied().find(|p| p.manifest.name == name) else {
            continue;
        };
        todo.extend(p.manifest.dependencies.keys().cloned());
        for d in hirx::declarations(p) {
            if let Declaration::Authority(a) = d {
                names.insert(a.name.as_str());
                names.extend(a.outranks.iter().map(String::as_str));
            }
        }
    }
    names
}

/// An error (L1004) for each layer of `t` whose authority is not in the
/// lattice its package sees: such a name is never passed through.
#[must_use]
pub fn authority_errors(t: &Timeline, all: &[&hir::Package]) -> Vec<Diagnostic> {
    t.layers
        .iter()
        .filter_map(|l| {
            let pkg = all.iter().copied().find(|p| p.manifest.name == l.package)?;
            let names = lattice(all, pkg);
            if names.contains(l.authority.as_str()) {
                return None;
            }
            Some(Diagnostic::error(
                DiagnosticCode::InvalidManifest,
                crate::inline_tests::nowhere(pkg),
                format!(
                    "layer `{}`: authority `{}` is not in the authority lattice package `{}` declares (known: {})",
                    l.id,
                    l.authority,
                    pkg.manifest.name,
                    names.into_iter().collect::<Vec<_>>().join(", ")
                ),
            ))
        })
        .collect()
}

/// Ids of `t`'s layers.
#[must_use]
pub fn layer_ids(t: &Timeline) -> Vec<String> {
    t.layers.iter().map(|l| l.id.clone()).collect()
}

/// The layers `spec` disables among `ids`: `ID` or `+ID` enables (and, if
/// any is given, disables every layer not named), `-ID` disables.
///
/// # Errors
/// An id that is not a layer, naming the layers there are.
pub fn disabled_layers(ids: &[String], spec: &[String]) -> Result<Vec<String>, String> {
    let mut enabled: Vec<&str> = Vec::new();
    let mut off: Vec<&str> = Vec::new();
    for item in spec {
        let (on, id) = match item.strip_prefix('-') {
            Some(id) => (false, id),
            None => (true, item.strip_prefix('+').unwrap_or(item)),
        };
        if !ids.iter().any(|x| x == id) {
            return Err(format!("no layer `{id}`; layers: {}", ids.join(", ")));
        }
        if on {
            enabled.push(id);
        } else {
            off.push(id);
        }
    }
    Ok(ids
        .iter()
        .filter(|id| {
            off.contains(&id.as_str()) || (!enabled.is_empty() && !enabled.contains(&id.as_str()))
        })
        .cloned()
        .collect())
}
