
/// How an obligation instance's end reads as a row status; `trigger` is
/// the name of the event that created it.
fn obligation_status(
    ended: Status,
    closed: Option<(&str, &str)>,
    due: Option<&str>,
    as_of: &str,
    trigger: &str,
) -> TimelineStatus {
    let by = closed.map_or_else(String::new, |(event, date)| {
        format!(" by {event} on {date}")
    });
    match ended {
        Status::Open => status_open(due, as_of),
        Status::Discharged => status(RowState::Met, Some(format!("discharged{by}"))),
        Status::Late => status(
            RowState::Late,
            Some(format!("acted{by}, after the due date")),
        ),
        // L4010: an act against a due date the trigger left unknown is
        // neither timely nor late; the frozen schema says so as unknown.
        Status::Unjudged => status(
            RowState::Unknown,
            Some(format!(
                "timeliness unknown: {trigger} due date not recorded; acted{by}"
            )),
        ),
        // A designated date the trigger left unknown, with no act against
        // it: unknown, never open or lapsed (lg-flu.46).
        Status::Unknown => status(
            RowState::Unknown,
            Some(format!("{trigger} due date not recorded")),
        ),
        Status::Breached => status(
            RowState::Breached,
            Some(format!("past due at {as_of} with no act by its bearer")),
        ),
        Status::Lapsed => status(
            RowState::Lapsed,
            Some(format!("past due at {as_of} with no act")),
        ),
        Status::Waived => status(RowState::Superseded, Some(format!("waived{by}"))),
        Status::Superseded => status(
            RowState::Superseded,
            Some(format!("restarted{by} (a later trigger)")),
        ),
        Status::Merged => status(
            RowState::Superseded,
            Some("absorbed by a `later_of` instance".to_owned()),
        ),
    }
}

/// `state` of instance `s`'s row, saying so when a filed request awaits
/// the order a grant of its deadline licenses (its pending move).
fn awaiting(mut state: TimelineStatus, s: &Scheduled) -> TimelineStatus {
    if let Some(p) = &s.pending {
        let pending = format!("filed, awaiting order: {} on {}", p.request, p.date);
        state.reason = Some(match state.reason {
            Some(r) => format!("{r}; {pending}"),
            None => pending,
        });
    }
    state
}

/// The reason of event `i`'s row when it is a request some instance of
/// `counted` awaits an order on: the deadlines it is the pending move of.
fn awaited(counted: &[Scheduled], i: usize) -> Option<String> {
    let mut of: Vec<&str> = counted
        .iter()
        .filter(|s| s.pending.as_ref().is_some_and(|p| p.event == i))
        .map(|s| s.deadline.as_str())
        .collect();
    of.sort_unstable();
    of.dedup();
    (!of.is_empty()).then(|| format!("pending move of {}: filed, awaiting order", of.join(", ")))
}

/// An open instance: `open`, or `unknown` with why when it has no date.
fn status_open(due: Option<&str>, as_of: &str) -> TimelineStatus {
    match due {
        Some(_) => status(RowState::Open, None),
        None => status(
            RowState::Unknown,
            Some(format!("open at {as_of} with no due date")),
        ),
    }
}

/// The diagnostics a timeline of `file` reports: every error, and the
/// warnings in `file` itself.
fn diagnostics(b: &Builder, file: &str, all: &[Diagnostic]) -> Vec<TimelineDiagnostic> {
    crate::out::sorted(all.to_vec())
        .iter()
        .filter(|d| d.severity == Severity::Error || d.span.file.0 == file)
        .map(|d| TimelineDiagnostic {
            code: d.code.as_str().to_owned(),
            severity: match d.severity {
                Severity::Error => TimelineSeverity::Error,
                Severity::Warning => TimelineSeverity::Warning,
                Severity::Info | Severity::Hint => TimelineSeverity::Note,
            },
            message: d.message.clone(),
            source: Some(b.span(&d.span)),
        })
        .collect()
}

/// The timeline of matter `m`, checked at `as_of` / `known_at` (its own
/// unless replaced), over the loaded packages `all` and source `texts`.
#[must_use]
pub fn of_case(
    m: &CaseCheck,
    case: &hir::Case,
    all: &[&hir::Package],
    texts: &[(String, String)],
    diagnostics_in: &[Diagnostic],
) -> Timeline {
    let mut b = Builder::new(all, texts, case.as_of.clone(), case.known_at.clone());
    let used: Vec<&hir::Package> = m
        .packages
        .iter()
        .filter_map(|name| all.iter().copied().find(|p| &p.manifest.name == name))
        .collect();
    for pkg in &used {
        b.layer(pkg);
    }
    b.law = litigate_sema::temporal::world::World::closure_packages(&used, all);
    let primary = used.first().map(|p| p.manifest.name.clone());
    let (event_rows, moved) = events(&mut b, m, case, &primary.unwrap_or_default());
    let mut seen = deadlines(&mut b, m, case, &event_rows);
    let start = case.start.clone().or_else(|| {
        used.first()
            .map(|p| format!("{}::{}", p.manifest.pack_id, p.start))
    });
    if let Some((pkg, node)) = start.as_deref().and_then(|s| b.node(s)) {
        b.governing(0, pkg, node, &mut seen, None);
        fork(&mut b, case, pkg, node, &seen, moved);
    }
    let source = TimelineSource {
        file: m.path.clone(),
        kind: TimelineSourceKind::Case,
    };
    let diagnostics = diagnostics(&b, &m.path, diagnostics_in);
    b.finish(source, diagnostics)
}
