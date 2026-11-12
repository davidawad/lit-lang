/// The layer class of a package with authority level `authority`.
#[must_use]
pub fn layer_class(authority: &str) -> LayerClass {
    match authority {
        "LocalRule" => LayerClass::LocalRule,
        "StandingOrder" => LayerClass::StandingOrder,
        "CaseOrder" => LayerClass::CaseOrder,
        _ => LayerClass::BaseRule,
    }
}

/// The layer package `pkg` is.
fn layer_of(pkg: &hir::Package) -> TimelineLayer {
    let m = &pkg.manifest;
    TimelineLayer {
        id: m.name.clone(),
        class: layer_class(&m.authority),
        authority: m.authority.clone(),
        label: m.title.clone(),
        cite: None,
        effective: TimelineEffective {
            from: Some(m.in_force.from.clone()),
            to: m.in_force.to.clone(),
        },
        package: m.name.clone(),
        enabled: true,
    }
}

/// How a status reads in a timeline, with its reason.
fn status(state: RowState, reason: Option<String>) -> TimelineStatus {
    TimelineStatus { state, reason }
}

/// A row before ids are assigned: `trigger` indexes the draft rows.
#[derive(Debug, Clone)]
struct Draft {
    key: String,
    world: usize,
    kind: RowKind,
    date: Option<String>,
    due: Option<String>,
    label: String,
    status: TimelineStatus,
    cite: Option<String>,
    source: Option<TimelineSpan>,
    layer: String,
    /// The layers that applied before `layer`, base first (a limit's
    /// displaced layers); empty for every other row.
    via: Vec<String>,
    trigger: Option<usize>,
    /// The row that replaces this one in its world's view.
    replaced_by: Option<usize>,
}

/// The constructor name of event pattern `e` (`Served` of
/// `Served(complaint)`).
fn event_name(e: &hir::Expr) -> String {
    hirx::event_parts(e).map(|(c, _)| c).unwrap_or_default()
}

/// The module-qualified id (`<module>.<id>`) of
/// declaration `id` of `pkg`: the rule or node id a row's `key` names.
fn decl_key(pkg: &hir::Package, id: &str) -> String {
    let module = pkg
        .modules
        .iter()
        .find(|m| {
            m.declarations
                .iter()
                .any(|d| hirx::decl_id(d).as_deref() == Some(id))
        })
        .map_or(pkg.manifest.name.as_str(), |m| m.name.as_str());
    format!("{module}.{id}")
}

/// The edge an election or removal event `e` names (its `elects` or
/// `removes` argument, as written), by its event type among `all`.
fn option_edge(all: &[&hir::Package], e: &hir::Expr) -> Option<String> {
    let (name, args) = hirx::event_parts(e)?;
    all.iter()
        .copied()
        .flat_map(hirx::declarations)
        .find_map(|d| match d {
            Declaration::Event(t) if t.name == name => match t.effect {
                Some(hir::EventEffect::Removes { param } | hir::EventEffect::Elects { param }) => {
                    args.get(param).cloned()
                }
                _ => None,
            },
            _ => None,
        })
}

/// The `key` of matter event `e`: `event:<Name>@<id>` when it declares
/// an id, else `event:<Name>@<date>/<docket>`, else
/// `event:<Name>@<date>/<option edge id>` for an election or removal,
/// else `event:<Name>@<date>`.
fn event_key(all: &[&hir::Package], e: &hir::Event) -> String {
    if e.id.is_some() {
        return e.key();
    }
    let declared = e
        .docket
        .map(|n| n.to_string())
        .or_else(|| option_edge(all, &e.action))
        .map(|d| format!("/{d}"))
        .unwrap_or_default();
    format!("event:{}@{}{declared}", event_name(&e.action), e.date)
}

/// A deadline declaration's display label (`answer-period: 30 days after
/// InstitutionNoticed()`, or for a designated one `answer-period: the date
/// InstitutionNoticed() designates, at least 30 days after it`); a
/// chained one names the period end it runs from (`90 days after
/// deadline(opposition-period).nominal`).
fn deadline_label(d: &hir::Deadline) -> String {
    let unit = match d.unit {
        hir::DayUnit::Calendar => "day",
        hir::DayUnit::Court => "court day",
        hir::DayUnit::Business => "business day",
        hir::DayUnit::Week => "week",
        hir::DayUnit::Month => "month",
        hir::DayUnit::Year => "year",
        hir::DayUnit::Hour => "hour",
    };
    let plural = if d.amount == 1 { "" } else { "s" };
    let direction = match d.direction {
        hir::Direction::Forward => "after",
        hir::Direction::Backward => "before",
    };
    let trigger = match &d.chain {
        Some(c) => {
            let end = match c.end {
                hir::DueEnd::Nominal => "nominal",
                hir::DueEnd::Adjusted => "adjusted",
            };
            format!("deadline({}).{end}", hirx::local(&c.deadline))
        }
        None => hirx::event_text(&d.trigger),
    };
    if d.designated.is_some() {
        return format!(
            "{}: the date {trigger} designates, at least {} {unit}{plural} {direction} it",
            d.meta.id, d.amount
        );
    }
    format!(
        "{}: {} {unit}{plural} {direction} {trigger}",
        d.meta.id, d.amount
    )
}

/// A limit's bounds as a label (`30 pages or 13000 words or 1300
/// lines`), a hole as `?`; `no length bound` without one.
fn limit_label(l: &hir::Limit) -> String {
    let bounds: Vec<String> = l
        .bounds
        .iter()
        .map(|b| {
            let max = match &b.max {
                hir::Authored::Known(n) => n.to_string(),
                hir::Authored::Hole(_) => "?".to_owned(),
            };
            format!("{max} {}", b.measure.as_str())
        })
        .collect();
    if bounds.is_empty() {
        "no length bound".to_owned()
    } else {
        bounds.join(" or ")
    }
}

/// Builds one timeline over the loaded packages.
struct Builder<'a> {
    all: &'a [&'a hir::Package],
    /// The matter's law: its packages and their dependencies (empty for
    /// a module), over which a filing limit is selected.
    law: Vec<&'a hir::Package>,
    texts: BTreeMap<&'a str, &'a str>,
    as_of: String,
    known_at: Option<String>,
    layers: Vec<TimelineLayer>,
    worlds: Vec<TimelineWorld>,
    rows: Vec<Draft>,
    /// The fork row of each world, by draft index (parallel to `worlds`).
    forks: Vec<Option<usize>>,
    /// `supersedes` links, by draft index.
    supersedes: Vec<(usize, usize)>,
    /// The docket's deadline rows nothing has triggered, by draft index,
    /// with their qualified deadline ids.
    untriggered: Vec<(usize, String)>,
    /// Each amendment history proving a row's cite (SPEC §4.3), at the
    /// row's source: an L9008 note of the timeline.
    proofs: Vec<(Option<TimelineSpan>, String)>,
}
