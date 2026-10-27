//! The timeline of a `.case` or `.lit` file (`litigate.timeline/1`,
//! docs/litigate/TIMELINE.md): events and deadlines in date order, the
//! worlds a matter's current posture forks into, and the layer each row
//! comes from.
//!
//! A matter's docket world holds its events and every obligation instance
//! replay created (a deadline the clock cannot count stays, with `due:
//! null` and the clock's reason), then the deadlines governing the options
//! of its current posture that nothing has triggered yet, then a fork row
//! at that posture. Each option of the posture is one alternate world: the
//! hypothetical event of taking it, then the deadlines governing the
//! options it leads to, which no recorded event dates (`unknown`).
//!
//! Until the change layers of lg-fq2 land, every package is one layer:
//! `base_rule` unless its authority is `LocalRule`, `StandingOrder` or
//! `CaseOrder`, and every row's provenance is that one layer.
use crate::hirx::{self, local};
use crate::matter::{CaseCheck, Scheduled};
use litigate_base::timeline::{
    ChoiceKind, EdgeKind, LayerClass, RowKind, RowState, Timeline, TimelineChoice,
    TimelineDiagnostic, TimelineEdge, TimelineEffective, TimelineLayer, TimelineRow,
    TimelineSeverity, TimelineSource, TimelineSourceKind, TimelineSpan, TimelineStatus,
    TimelineWorld, TIMELINE_SCHEMA,
};
use litigate_base::{Diagnostic, DiagnosticCode, Severity, Span};
use litigate_lower::source_map::Citations;
use litigate_sema::case::{Effect, Status};
use litigate_sema::hir::{self, DeadlineWindow, Declaration, NodeKind};
use std::collections::{BTreeMap, BTreeSet};

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
/// InstitutionNoticed() designates, at least 30 days after it`).
fn deadline_label(d: &hir::Deadline) -> String {
    let unit = match d.unit {
        hir::DayUnit::Calendar => "day",
        hir::DayUnit::Court => "court day",
        hir::DayUnit::Business => "business day",
        hir::DayUnit::Week => "week",
        hir::DayUnit::Month => "month",
        hir::DayUnit::Year => "year",
    };
    let plural = if d.amount == 1 { "" } else { "s" };
    let direction = match d.direction {
        hir::Direction::Forward => "after",
        hir::Direction::Backward => "before",
    };
    let trigger = hirx::event_text(&d.trigger);
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

/// Builds one timeline over the loaded packages.
struct Builder<'a> {
    all: &'a [&'a hir::Package],
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

impl<'a> Builder<'a> {
    fn new(
        all: &'a [&'a hir::Package],
        texts: &'a [(String, String)],
        as_of: String,
        known_at: Option<String>,
    ) -> Self {
        Self {
            all,
            texts: texts
                .iter()
                .map(|(f, t)| (f.as_str(), t.as_str()))
                .collect(),
            as_of,
            known_at,
            layers: Vec::new(),
            worlds: vec![TimelineWorld {
                id: "w0".to_owned(),
                label: "docket".to_owned(),
                parent: None,
                fork_row: None,
                choice: None,
            }],
            rows: Vec::new(),
            forks: vec![None],
            supersedes: Vec::new(),
            untriggered: Vec::new(),
            proofs: Vec::new(),
        }
    }

    /// Records each vendored amendment history proving that `cite` (of
    /// `pkg`, cited by a version in force from `from`) was that version's
    /// text, at `source`.
    fn prove(
        &mut self,
        pkg: &hir::Package,
        cite: Option<&str>,
        from: &str,
        source: Option<&TimelineSpan>,
    ) {
        let Some(cite) = cite else {
            return;
        };
        let citations = Citations::of_package(pkg, Some(crate::verdict::corpus()));
        for proof in citations.proofs(cite, from) {
            // A row repeated in another world repeats its proof: once.
            let at = (source.cloned(), proof);
            self.proofs.retain(|p| *p != at);
            self.proofs.push(at);
        }
    }

    /// `span` as a timeline span (byte range from the loaded text).
    fn span(&self, span: &Span) -> TimelineSpan {
        let text = texts_of(self, &span.file.0);
        let range = crate::span::byte_range(text, span);
        TimelineSpan {
            file: span.file.0.clone(),
            start: range.start,
            end: range.end,
            line: span.line,
            column: span.col,
        }
    }

    fn source(&self, span: Option<&Span>) -> Option<TimelineSpan> {
        span.map(|s| self.span(s))
    }

    /// The layer id of `pkg`, listing it on first use.
    fn layer(&mut self, pkg: &hir::Package) -> String {
        if !self.layers.iter().any(|l| l.id == pkg.manifest.name) {
            self.layers.push(layer_of(pkg));
        }
        pkg.manifest.name.clone()
    }

    fn push(&mut self, row: Draft) -> usize {
        self.rows.push(row);
        self.rows.len() - 1
    }

    /// Pushes `row`, whose cite (of `pkg`) a version in force from `from`
    /// makes, recording what proves it.
    fn push_cited(&mut self, row: Draft, pkg: &hir::Package, from: &str) -> usize {
        self.prove(pkg, row.cite.as_deref(), from, row.source.as_ref());
        self.push(row)
    }

    /// An L9008 note for each recorded proof.
    fn proof_notes(&self) -> impl Iterator<Item = TimelineDiagnostic> + '_ {
        self.proofs.iter().map(|(at, proof)| TimelineDiagnostic {
            code: DiagnosticCode::CiteProvedByHistory.as_str().to_owned(),
            severity: TimelineSeverity::Note,
            message: proof.clone(),
            source: at.clone(),
        })
    }

    /// The loaded package with pack id `pack`.
    fn package(&self, pack: &str) -> Option<&'a hir::Package> {
        self.all
            .iter()
            .copied()
            .find(|p| p.manifest.pack_id == pack)
    }

    /// The node version in force named by qualified `id`, with its package.
    fn node(&self, id: &str) -> Option<(&'a hir::Package, &'a hir::Node)> {
        let (pack, name) = id.split_once("::")?;
        let pkg = self.package(pack)?;
        hirx::nodes_at(self.all, pkg, &self.as_of, self.known_at.as_deref())
            .into_iter()
            .find(|n| n.meta.id == name)
            .map(|n| (pkg, n))
    }

    /// The deadline version selected at the timeline's dates for qualified
    /// `id`, with its package.
    fn deadline(&self, id: &str) -> Option<(&'a hir::Package, &'a hir::Deadline)> {
        let (pack, name) = id.split_once("::")?;
        let pkg = self.package(pack)?;
        hirx::selected(self.all, pkg, &self.as_of, self.known_at.as_deref())
            .into_iter()
            .find_map(|d| match d {
                Declaration::Deadline(v) if v.meta.id == name => Some((pkg, v)),
                _ => None,
            })
    }

    /// The deadline version (`id@from`) that counted `s`, with its package.
    fn counted_by(&self, s: &Scheduled) -> Option<(&'a hir::Package, &'a hir::Deadline)> {
        self.all.iter().copied().find_map(|p| {
            hirx::declarations(p).find_map(|d| match d {
                Declaration::Deadline(v)
                    if format!("{}@{}", v.meta.id, v.meta.effective.from) == s.version =>
                {
                    Some((p, v))
                }
                _ => None,
            })
        })
    }

    /// A deadline row of `world`, from declaration `decl` of `pkg`.
    #[allow(clippy::too_many_arguments)] // one row's every field
    fn deadline_row(
        &mut self,
        world: usize,
        pkg: &hir::Package,
        decl: &hir::Deadline,
        due: Option<String>,
        state: TimelineStatus,
        trigger: Option<usize>,
    ) -> usize {
        let layer = self.layer(pkg);
        // An instance is keyed by its trigger row; a deadline nothing has
        // triggered, by the event that would.
        let cause = trigger.map_or_else(
            || format!("event:{}", event_name(&decl.trigger)),
            |t| self.rows[t].key.clone(),
        );
        // The cite and the version citing it: the deadline's, else its
        // counting rule's.
        let cited = Some(&decl.meta)
            .filter(|m| m.cite.is_some())
            .or_else(|| hirx::rule_of(pkg, &decl.meta.id).map(|r| &r.meta));
        let row = Draft {
            key: format!("{}@{cause}", decl_key(pkg, &decl.meta.id)),
            world,
            kind: RowKind::Deadline,
            date: None,
            due,
            label: deadline_label(decl),
            status: state,
            cite: cited.and_then(|m| m.cite.clone()),
            source: self.source(decl.meta.span.as_ref()),
            layer,
            trigger,
            replaced_by: None,
        };
        // No cite (no `cited`) has nothing to prove.
        let from = cited.map_or("", |m| m.effective.from.as_str());
        self.push_cited(row, pkg, from)
    }

    /// Unknown rows in `world` for the deadlines governing `node`'s
    /// options that have no row yet among `seen` (qualified ids), each
    /// triggered by `trigger`.
    fn governing(
        &mut self,
        world: usize,
        pkg: &hir::Package,
        node: &hir::Node,
        seen: &mut Vec<String>,
        trigger: Option<usize>,
    ) {
        for edge in &node.edges {
            let Some(DeadlineWindow::Declared { deadline }) =
                edge.deadline.as_ref().map(|d| &d.window)
            else {
                continue;
            };
            let id = format!("{}::{}", pkg.manifest.pack_id, local(deadline));
            if seen.contains(&id) {
                continue;
            }
            seen.push(id.clone());
            if let Some((owner, decl)) = self.deadline(&id) {
                let reason = format!(
                    "counted from {}, which no recorded event dates as of {}",
                    hirx::event_text(&decl.trigger),
                    self.as_of
                );
                let row = self.deadline_row(
                    world,
                    owner,
                    decl,
                    None,
                    status(RowState::Unknown, Some(reason)),
                    trigger,
                );
                if trigger.is_none() {
                    self.untriggered.push((row, id));
                }
            }
        }
    }

    /// The timeline.
    fn finish(self, source: TimelineSource, mut diagnostics: Vec<TimelineDiagnostic>) -> Timeline {
        diagnostics.extend(self.proof_notes());
        // Each world's rows in date order (undated last), creation order
        // breaking ties; worlds in order.
        let mut order: Vec<usize> = (0..self.rows.len()).collect();
        order.sort_by_key(|&i| {
            let r = &self.rows[i];
            let date = r.date.clone().or_else(|| r.due.clone());
            (r.world, date.is_none(), date, i)
        });
        let mut ids = vec![String::new(); self.rows.len()];
        for (n, &i) in order.iter().enumerate() {
            ids[i] = format!("r{}", n + 1);
        }
        let world_id = |w: usize| self.worlds[w].id.clone();
        let mut rows = Vec::new();
        let mut edges = Vec::new();
        let mut last: BTreeMap<usize, usize> = BTreeMap::new();
        for &i in &order {
            let r = &self.rows[i];
            let class = self
                .layers
                .iter()
                .find(|l| l.id == r.layer)
                .map_or(LayerClass::BaseRule, |l| l.class);
            let link = |from: usize, kind| TimelineEdge {
                from: ids[from].clone(),
                to: ids[i].clone(),
                world: world_id(r.world),
                kind,
            };
            if let Some(&prev) = last.get(&r.world).filter(|&&p| r.trigger != Some(p)) {
                edges.push(link(prev, EdgeKind::Sequence));
            }
            if let Some(t) = r.trigger {
                let kind = if self.rows[t].world == r.world {
                    EdgeKind::Triggers
                } else {
                    EdgeKind::Forks
                };
                edges.push(link(t, kind));
            }
            for &(from, _) in self.supersedes.iter().filter(|&&(_, to)| to == i) {
                edges.push(link(from, EdgeKind::Supersedes));
            }
            last.insert(r.world, i);
            rows.push(TimelineRow {
                id: ids[i].clone(),
                key: r.key.clone(),
                world: world_id(r.world),
                kind: r.kind,
                date: r.date.clone(),
                due: r.due.clone(),
                label: r.label.clone(),
                status: r.status.clone(),
                cite: r.cite.clone(),
                source: r.source.clone(),
                layer: class,
                provenance: vec![r.layer.clone()],
                trigger: r.trigger.map(|t| ids[t].clone()),
                replaced_by: r.replaced_by.map(|x| self.rows[x].key.clone()),
            });
        }
        let worlds = self
            .worlds
            .into_iter()
            .zip(&self.forks)
            .map(|(mut w, f)| {
                w.fork_row = f.map(|f| ids[f].clone());
                w
            })
            .collect();
        Timeline {
            schema: TIMELINE_SCHEMA.to_owned(),
            source,
            as_of: self.as_of,
            known_at: self.known_at,
            layers: self.layers,
            worlds,
            rows,
            edges,
            diagnostics,
        }
    }
}

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
            trigger: None,
            replaced_by: None,
        };
        // The edge's authority is in force with its node's version.
        if let Some((pkg, node, edge)) = taken.filter(|_| e.authority.is_none()) {
            let from = &node.meta.effective.from;
            b.prove(pkg, edge.authority.as_deref(), from, row.source.as_ref());
        }
        let row = b.push(row);
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
            trigger: Some(fork_at),
            replaced_by: None,
        };
        let taken = b.push_cited(taken, pkg, &node.meta.effective.from);
        if let Some(event) = &edge.event {
            triggered(b, world, taken, event);
        }
        if let Some((next_pkg, next)) = b.node(&target(pkg, &edge.to)) {
            let mut seen = seen.to_vec();
            b.governing(world, next_pkg, next, &mut seen, Some(taken));
        }
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matter;
    use crate::testkit::{self, ACME};
    use crate::ws::Session;
    use litigate_base::DiagnosticCode;
    use serde_json::json;

    const PACK: &str = "frcp-rule12-example";

    #[test]
    fn layer_classes_follow_the_package_authority() {
        assert_eq!(layer_class("LocalRule"), LayerClass::LocalRule);
        assert_eq!(layer_class("StandingOrder"), LayerClass::StandingOrder);
        assert_eq!(layer_class("CaseOrder"), LayerClass::CaseOrder);
        assert_eq!(layer_class("Regulation"), LayerClass::BaseRule);
    }

    #[test]
    fn every_obligation_end_reads_as_a_status() {
        let closed = Some(("Answered(complaint)", "2025-03-20"));
        let read = |s, closed, due| {
            let t = obligation_status(s, closed, due, "2025-06-01", "Served");
            (t.state, t.reason)
        };
        let cases = [
            (Status::Open, RowState::Open, None),
            (
                Status::Discharged,
                RowState::Met,
                Some("discharged by Answered(complaint) on 2025-03-20"),
            ),
            (
                Status::Late,
                RowState::Late,
                Some("acted by Answered(complaint) on 2025-03-20, after the due date"),
            ),
            (
                Status::Unjudged,
                RowState::Unknown,
                Some("timeliness unknown: Served due date not recorded; acted by Answered(complaint) on 2025-03-20"),
            ),
            (
                Status::Unknown,
                RowState::Unknown,
                Some("Served due date not recorded"),
            ),
            (
                Status::Breached,
                RowState::Breached,
                Some("past due at 2025-06-01 with no act by its bearer"),
            ),
            (
                Status::Lapsed,
                RowState::Lapsed,
                Some("past due at 2025-06-01 with no act"),
            ),
            (
                Status::Waived,
                RowState::Superseded,
                Some("waived by Answered(complaint) on 2025-03-20"),
            ),
            (
                Status::Superseded,
                RowState::Superseded,
                Some("restarted by Answered(complaint) on 2025-03-20 (a later trigger)"),
            ),
            (
                Status::Merged,
                RowState::Superseded,
                Some("absorbed by a `later_of` instance"),
            ),
        ];
        for (status, state, reason) in cases {
            assert_eq!(
                read(status, closed, Some("2025-03-26")),
                (state, reason.map(str::to_owned)),
                "{status:?}"
            );
        }
        assert_eq!(
            read(Status::Discharged, None, None),
            (RowState::Met, Some("discharged".to_owned()))
        );
        assert_eq!(
            read(Status::Open, None, None),
            (
                RowState::Unknown,
                Some("open at 2025-06-01 with no due date".to_owned())
            )
        );
    }

    #[test]
    fn layers_are_enabled_named_or_disabled() {
        let ids = vec!["a".to_owned(), "b".to_owned(), "c".to_owned()];
        let spec = |items: &[&str]| {
            disabled_layers(
                &ids,
                &items.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>(),
            )
        };
        assert_eq!(spec(&[]), Ok(vec![]));
        assert_eq!(spec(&["-b"]), Ok(vec!["b".to_owned()]));
        assert_eq!(spec(&["a", "+c"]), Ok(vec!["b".to_owned()]));
        assert_eq!(
            spec(&["a", "-a"]),
            Ok(vec!["a".to_owned(), "b".to_owned(), "c".to_owned()])
        );
        assert_eq!(
            spec(&["-d"]),
            Err("no layer `d`; layers: a, b, c".to_owned())
        );
    }

    /// The frozen example document.
    fn example() -> Timeline {
        serde_json::from_str(include_str!(
            "../../../docs/litigate/examples/timeline-ttab-opposition.json"
        ))
        .unwrap()
    }

    fn row_ids(t: &Timeline) -> Vec<&str> {
        t.rows.iter().map(|r| r.id.as_str()).collect()
    }

    fn world_ids(t: &Timeline) -> Vec<&str> {
        t.worlds.iter().map(|w| w.id.as_str()).collect()
    }

    /// The example with a world `w3` nested under `w1` that names no fork
    /// row, and a docket row after the fork.
    fn nested() -> Timeline {
        let mut t = example();
        let mut w3 = t.worlds[1].clone();
        w3.id = "w3".to_owned();
        w3.parent = Some("w1".to_owned());
        w3.fork_row = None;
        t.worlds.push(w3);
        let mut late = t.rows[0].clone();
        late.id = "r10".to_owned();
        t.rows.push(late);
        t
    }

    #[test]
    fn a_disabled_layer_drops_its_rows_and_the_worlds_forking_from_them() {
        let mut t = nested();
        // r7 triggered by the reset (r6) loses its trigger with it.
        t.rows[6].trigger = Some("r6".to_owned());
        apply_layers(&mut t, &["order-reset-1".to_owned()]);
        assert_eq!(
            row_ids(&t),
            ["r1", "r2", "r3", "r4", "r5", "r7", "r9", "r10"]
        );
        assert_eq!(t.rows[5].trigger, None);
        // r7's replacement (r8) went with its layer, so r7 names none.
        assert_eq!(t.rows[5].replaced_by, None);
        assert!(!t.layers[1].enabled);
        assert!(t.layers[0].enabled);
        let ends: Vec<&str> = t
            .edges
            .iter()
            .flat_map(|e| [e.from.as_str(), e.to.as_str()])
            .collect();
        assert!(!ends.contains(&"r6"));
        let mut t = nested();
        apply_layers(&mut t, &["ttab".to_owned()]);
        assert_eq!(world_ids(&t), ["w0"]);
        assert!(t.rows.is_empty());
        assert!(t.edges.is_empty());
    }

    #[test]
    fn a_world_keeps_its_ancestors_up_to_its_fork() {
        let mut t = nested();
        assert_eq!(apply_world(&mut t, "w1"), Ok(()));
        assert_eq!(world_ids(&t), ["w0", "w1"]);
        assert_eq!(
            row_ids(&t),
            ["r1", "r2", "r3", "r4", "r5", "r6", "r7", "r8"]
        );
        assert_eq!(t.rows[6].replaced_by.as_ref(), Some(&t.rows[7].key));
        assert!(t.edges.iter().all(|e| e.world != "w2"));
        // A world naming no fork row cuts nothing from its parent.
        let mut t = nested();
        assert_eq!(apply_world(&mut t, "w3"), Ok(()));
        assert_eq!(world_ids(&t), ["w0", "w1", "w3"]);
        assert_eq!(t.rows.len(), 8);
        let mut t = nested();
        assert_eq!(
            apply_world(&mut t, "w7"),
            Err("no world `w7`; worlds: w0, w1, w2, w3".to_owned())
        );
    }

    /// The worked example (plus `extra`) with matter `text`, checked: its
    /// session, which `f` reads with the matter.
    fn with_matter(extra: &str, text: &str, f: impl FnOnce(&Session, matter::CaseCheck)) {
        let session = testkit::example_locked(extra, text, None);
        let all = session.hir();
        let c = matter::check(
            &session,
            &session.cases[0],
            &crate::clock::Clocks::new(&all),
            litigate_base::Profile::Dev,
        );
        f(&session, c);
    }

    /// The timeline of `m` in `session`, as JSON.
    fn build(session: &Session, m: &matter::CaseCheck, case: &hir::Case) -> serde_json::Value {
        let all = session.hir();
        let texts = session.sources();
        serde_json::to_value(of_case(m, case, &all, &texts, &m.diagnostics)).unwrap()
    }

    fn rows(t: &serde_json::Value) -> Vec<(String, String, String)> {
        t["rows"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| {
                (
                    r["world"].as_str().unwrap().to_owned(),
                    r["kind"].as_str().unwrap().to_owned(),
                    r["status"]["state"].as_str().unwrap().to_owned(),
                )
            })
            .collect()
    }

    #[test]
    fn the_lattice_is_the_package_s_its_dependencies_and_core() {
        let session = testkit::example_with("");
        let all = session.hir();
        let mut pkg = all[0].clone();
        assert!(lattice(&all, &pkg).contains("FederalRule"));
        // A dependency that is not loaded adds nothing; one met twice is
        // read once.
        pkg.manifest
            .dependencies
            .insert("nowhere".to_owned(), "^1".to_owned());
        pkg.manifest
            .dependencies
            .insert("core".to_owned(), "^1".to_owned());
        assert_eq!(lattice(&all, &pkg), lattice(&all, all[0]));
        with_matter("", ACME, |session, m| {
            let all = session.hir();
            let texts = session.sources();
            let mut t = of_case(&m, m.case.as_ref().unwrap(), &all, &texts, &[]);
            assert_eq!(authority_errors(&t, &all), Vec::<Diagnostic>::new());
            let mut bad = t.layers[0].clone();
            bad.authority = "Constitution".to_owned();
            let mut ghost = bad.clone();
            ghost.package = "ghost".to_owned();
            t.layers.extend([bad, ghost]);
            let errors = authority_errors(&t, &all);
            assert_eq!(errors.len(), 1, "{errors:#?}");
            assert_eq!(errors[0].code, DiagnosticCode::InvalidManifest);
            assert_eq!(errors[0].span.file.0, "lit/examples/Lit.toml");
            assert!(errors[0]
                .message
                .starts_with("layer `frcp-rule12-example`: authority `Constitution` is not in the authority lattice package `frcp-rule12-example` declares (known: "));
        });
    }

    #[test]
    fn rows_sharing_a_key_in_one_world_are_an_error_naming_both() {
        with_matter("", ACME, |session, m| {
            let all = session.hir();
            let texts = session.sources();
            let mut t = of_case(&m, m.case.as_ref().unwrap(), &all, &texts, &[]);
            assert_eq!(check_keys(&mut t), Vec::<Diagnostic>::new());
            let before = t.diagnostics.len();
            // The same key in another world is no collision.
            let mut elsewhere = t.rows[0].clone();
            elsewhere.id = "x1".to_owned();
            elsewhere.world = "w1".to_owned();
            let mut twin = t.rows[0].clone();
            twin.id = "x2".to_owned();
            let mut first = t.rows[1].clone();
            first.source = None;
            first.id = "x3".to_owned();
            first.key = "k".to_owned();
            let mut second = first.clone();
            second.id = "x4".to_owned();
            t.rows.extend([elsewhere, twin, first, second]);
            let errors = check_keys(&mut t);
            let messages: Vec<&str> = errors.iter().map(|d| d.message.as_str()).collect();
            assert_eq!(
                messages,
                [
                    "rows `r1` (Served(complaint, Personal) on widget (line 20)) and `x2` (Served(complaint, Personal) on widget (line 20)) of world `w0` share the key `event:Served@served-1`: nothing in their cause tells them apart; give the events docket numbers",
                    "rows `x3` (MovedToDismiss(complaint) by widget) and `x4` (MovedToDismiss(complaint) by widget) of world `w0` share the key `k`: nothing in their cause tells them apart; give the events docket numbers",
                ]
            );
            assert!(errors
                .iter()
                .all(|d| d.code == DiagnosticCode::DuplicateName));
            assert_eq!(errors[0].span.line, 20);
            assert_eq!(errors[1].span.file.0, "cases/examples/acme.case");
            assert_eq!(errors[1].span.line, 1);
            assert_eq!(t.diagnostics.len(), before + 2);
            assert_eq!(t.diagnostics[before].code, "L1001");
        });
    }

    #[test]
    fn only_an_election_or_removal_names_an_option_edge() {
        with_matter("", ACME, |session, m| {
            let case = m.case.as_ref().unwrap();
            let all = session.hir();
            let served = &case.events[0].action;
            let moved = &case.events[1].action;
            assert_eq!(option_edge(&all, served), None);
            let mut pkg = all[0].clone();
            let declare = |pkg: &mut hir::Package, effect| {
                for module in &mut pkg.modules {
                    for d in &mut module.declarations {
                        if let Declaration::Event(t) = d {
                            t.effect.clone_from(&effect);
                        }
                    }
                }
            };
            declare(&mut pkg, Some(hir::EventEffect::Elects { param: 0 }));
            let elects = [&pkg];
            assert_eq!(option_edge(&elects, served).as_deref(), Some("complaint"));
            assert_eq!(option_edge(&elects, moved).as_deref(), Some("complaint"));
            declare(&mut pkg, Some(hir::EventEffect::Removes { param: 9 }));
            assert_eq!(option_edge(&[&pkg], served), None);
            declare(
                &mut pkg,
                Some(hir::EventEffect::Waives {
                    deadline: "answer-period".to_owned(),
                }),
            );
            assert_eq!(option_edge(&[&pkg], served), None);
            let mut literal = served.clone();
            literal.kind = hir::ExprKind::Literal {
                value: hir::Value::Edge("e".to_owned()),
            };
            assert_eq!(option_edge(&[&pkg], &literal), None);
            assert_eq!(hirx::event_text(&literal), "");
        });
    }

    #[test]
    fn the_worked_example_forks_at_the_motion_ruling() {
        with_matter("", ACME, |session, m| {
            let t = build(session, &m, m.case.as_ref().unwrap());
            assert_eq!(t["worlds"][1]["choice"]["kind"], "chance");
            assert_eq!(t["worlds"][1]["choice"]["probability"], json!(null));
            assert_eq!(t["rows"][2]["status"]["state"], "met");
            assert_eq!(t["edges"][0]["kind"], "sequence");
        });
    }

    #[test]
    fn instances_without_a_version_or_declaration_are_left_out() {
        with_matter("", ACME, |session, mut m| {
            let case = m.case.clone().unwrap();
            let mut superseded = m.counted[0].clone();
            superseded.status = Status::Superseded;
            superseded.closed_by = Some(1);
            let mut unclosed = superseded.clone();
            unclosed.closed_by = None;
            m.counted[0].version = "nope@2000-01-01".to_owned();
            m.counted.extend([superseded, unclosed]);
            let mut uncounted = m.counted[0].clone();
            uncounted.deadline = format!("{PACK}::answer-period");
            uncounted.due = None;
            uncounted.unknown = Some("no counting rule".to_owned());
            uncounted.status = Status::Discharged;
            uncounted.closed_by = Some(1);
            let mut undeclared = uncounted.clone();
            undeclared.deadline = format!("{PACK}::nope");
            undeclared.status = Status::Open;
            undeclared.closed_by = None;
            let mut unjudged = uncounted.clone();
            unjudged.status = Status::Unjudged;
            m.counted.extend([uncounted, undeclared, unjudged]);
            let t = build(session, &m, &case);
            let deadlines: Vec<&serde_json::Value> = t["rows"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|r| r["kind"] == "deadline")
                .collect();
            assert_eq!(deadlines.len(), 4, "{t:#}");
            assert_eq!(
                deadlines[2]["status"],
                json!({"state": "met", "reason": "discharged by MovedToDismiss(complaint) on 2025-03-20: no counting rule"})
            );
            // An act the engine cannot judge says why itself.
            assert_eq!(
                deadlines[3]["status"],
                json!({"state": "unknown", "reason": "timeliness unknown: Served due date not recorded; acted by MovedToDismiss(complaint) on 2025-03-20"})
            );
            let supersedes = t["edges"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|e| e["kind"] == "supersedes")
                .count();
            assert_eq!(supersedes, 1);
        });
    }

    #[test]
    fn events_fall_back_to_the_primary_layer() {
        with_matter("", ACME, |session, mut m| {
            let mut case = m.case.clone().unwrap();
            // No replay step for the first event, an unknown stage for the second.
            m.replay.remove(0);
            m.replay[0].from = format!("{PACK}::nope");
            m.counted.clear();
            let t = build(session, &m, &case);
            assert_eq!(t["rows"][0]["provenance"], json!([PACK]));
            // No used package: no layer, and no posture to fork at.
            m.packages.clear();
            case.start = None;
            let t = build(session, &m, &case);
            assert_eq!(t["rows"][0]["provenance"], json!([""]));
            assert_eq!(t["worlds"].as_array().unwrap().len(), 1);
            case.start = Some(format!("{PACK}::nope"));
            let t = build(session, &m, &case);
            assert_eq!(t["worlds"].as_array().unwrap().len(), 1);
        });
    }

    #[test]
    fn a_matter_at_the_start_forks_with_probabilities() {
        let text = ACME
            .replace(
                "  served-1: 2025-03-05 Served(complaint, Personal) on widget\n",
                "",
            )
            .replace(
                "  moved-to-dismiss-1: 2025-03-20 MovedToDismiss(complaint) by widget\n",
                "",
            )
            .replace("facts { time-barred: not-barred }\n", "");
        with_matter("", &text, |session, m| {
            let case = m.case.clone().unwrap();
            assert_eq!(case.start, None);
            let t = build(session, &m, &case);
            assert_eq!(t["worlds"][1]["choice"]["probability"], json!(0.1));
            assert_eq!(t["worlds"][2]["choice"]["probability"], json!(0.9));
        });
    }

    #[test]
    fn a_decision_forks_into_elections_with_unknown_governing_deadlines() {
        let text = ACME
            .replace(
                "  served-1: 2025-03-05 Served(complaint, Personal) on widget\n",
                "",
            )
            .replace(
                "  moved-to-dismiss-1: 2025-03-20 MovedToDismiss(complaint) by widget\n",
                "",
            )
            .replace(
                "perspective widget",
                "start frcp-rule12-example::respond\nperspective widget",
            );
        with_matter("", &text, |session, m| {
            let mut case = m.case.clone().unwrap();
            assert_eq!(case.start.as_deref(), Some("frcp-rule12-example::respond"));
            let t = build(session, &m, &case);
            let keys: Vec<&str> = t["rows"]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| r["key"].as_str().unwrap())
                .collect();
            assert_eq!(
                keys,
                [
                    "frcp.rule12.answer-period@event:Served",
                    "fork:frcp.rule12.respond@start",
                    "event:Answered@fork:frcp.rule12.respond@start",
                    "event:MovedToDismiss@fork:frcp.rule12.respond@start",
                    "event:default@fork:frcp.rule12.respond@start",
                ]
            );
            assert_eq!(
                rows(&t),
                [
                    ("w0", "deadline", "unknown"),
                    ("w0", "fork", "unknown"),
                    ("w1", "event", "none"),
                    ("w2", "event", "none"),
                    ("w3", "event", "none"),
                ]
                .map(|(a, b, c)| (a.to_owned(), b.to_owned(), c.to_owned()))
            );
            assert_eq!(t["worlds"][1]["choice"]["kind"], "election");
            assert_eq!(
                t["rows"][0]["status"]["reason"],
                "counted from Served(complaint), which no recorded event dates as of 2025-06-01"
            );
            // A removed option forks no world.
            case.removals.push(hir::Removal {
                edge: format!("{PACK}::default"),
                date: "2025-04-01".to_owned(),
                event: 0,
                authority: "FRCP 12".to_owned(),
            });
            let t = build(session, &m, &case);
            assert_eq!(t["worlds"].as_array().unwrap().len(), 3);
        });
    }

    #[test]
    fn forks_read_the_actor_and_skip_what_does_not_branch() {
        with_matter("", ACME, |session, m| {
            let case = m.case.clone().unwrap();
            let all = session.hir();
            let texts = session.sources();
            let mut b = Builder::new(&all, &texts, "2025-06-01".to_owned(), None);
            assert!(b.node("nope").is_none());
            assert!(b.node("nope::x").is_none());
            assert!(b.node(&format!("{PACK}::nope")).is_none());
            assert!(b.deadline("nope").is_none());
            assert!(b.deadline("nope::x").is_none());
            assert!(b.deadline(&format!("{PACK}::nope")).is_none());
            let (pkg, terminal) = b.node(&format!("{PACK}::dismissed")).unwrap();
            fork(&mut b, &case, pkg, terminal, &[], None);
            assert!(b.rows.is_empty());
            let (_, respond) = b.node(&format!("{PACK}::respond")).unwrap();
            let mut state = respond.clone();
            state.kind = NodeKind::State;
            state.edges[0].actor = "Court".to_owned();
            state.edges[1].to = "nope::x".to_owned();
            // An edge back to the node itself is not an option.
            state.edges[2].to = "respond".to_owned();
            fork(&mut b, &case, pkg, &state, &[], None);
            let kinds: Vec<ChoiceKind> = b.worlds[1..]
                .iter()
                .map(|w| w.choice.as_ref().unwrap().kind)
                .collect();
            assert_eq!(
                kinds,
                [ChoiceKind::HypotheticalDecision, ChoiceKind::Election]
            );
            // A deadline that does not resolve has no row.
            let mut odd = respond.clone();
            odd.edges[0].deadline = Some(hir::EdgeDeadline {
                window: DeadlineWindow::Declared {
                    deadline: "nope".to_owned(),
                },
                ..respond.edges[0].deadline.clone().unwrap()
            });
            let before = b.rows.len();
            b.governing(
                0,
                pkg,
                &odd,
                &mut vec![format!("{PACK}::answer-period")],
                None,
            );
            assert_eq!(b.rows.len(), before);
        });
    }

    #[test]
    fn keys_derive_from_causes() {
        with_matter("", ACME, |session, m| {
            let case = m.case.clone().unwrap();
            let t = build(session, &m, &case);
            let keys: Vec<&str> = t["rows"]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| r["key"].as_str().unwrap())
                .collect();
            assert_eq!(
                keys,
                [
                "event:Served@served-1",
                "event:MovedToDismiss@moved-to-dismiss-1",
                "frcp.rule12.answer-period@event:Served@served-1",
                "fork:frcp.rule12.mtd-ruling@event:MovedToDismiss@moved-to-dismiss-1",
                "event:granted@fork:frcp.rule12.mtd-ruling@event:MovedToDismiss@moved-to-dismiss-1",
                "event:denied@fork:frcp.rule12.mtd-ruling@event:MovedToDismiss@moved-to-dismiss-1",
                ]
            );
            // A named option without an event uses its stable edge ID.
            let all = session.hir();
            let texts = session.sources();
            let mut b = Builder::new(&all, &texts, "2025-06-01".to_owned(), None);
            let (pkg, answered) = b.node(&format!("{PACK}::answered")).unwrap();
            fork(&mut b, &case, pkg, answered, &[], None);
            assert_eq!(
                b.rows[1].key,
                "event:edge-answered-judgment-entered@fork:frcp.rule12.answered@start"
            );
            let mut unnamed = answered.clone();
            unnamed.edges[0].event = None;
            unnamed.edges[0].id = None;
            let mut b = Builder::new(&all, &texts, "2025-06-01".to_owned(), None);
            fork(&mut b, &case, pkg, &unnamed, &[], None);
            assert_eq!(
                b.rows[1].key,
                "event:judgment-entered@fork:frcp.rule12.answered@start"
            );
            let mut served = case.events[0].clone();
            served.id = None;
            served.docket = Some(2);
            assert_eq!(event_key(&all, &served), "event:Served@2025-03-05/2");
            served.docket = None;
            assert_eq!(event_key(&all, &served), "event:Served@2025-03-05");
            let mut electing = all[0].clone();
            for module in &mut electing.modules {
                for declaration in &mut module.declarations {
                    if let Declaration::Event(event) = declaration {
                        if event.name == "Served" {
                            event.effect = Some(hir::EventEffect::Elects { param: 0 });
                        }
                    }
                }
            }
            assert_eq!(
                event_key(&[&electing], &served),
                "event:Served@2025-03-05/complaint"
            );
            // An explicit ID overrides the case event's own key.
            served.id = Some("service".to_owned());
            assert_eq!(event_key(&all, &served), "event:Served@service");
            assert_eq!(decl_key(all[0], "nope"), "frcp-rule12-example.nope");
        });
    }

    #[test]
    fn a_replacement_is_the_restart_or_the_absorbing_instance() {
        with_matter("", ACME, |_, m| {
            let at = |deadline: &str, trigger, status, closed_by, floor: Option<&str>| {
                let mut s = m.counted[0].clone();
                s.deadline = deadline.to_owned();
                s.trigger_event = trigger;
                s.status = status;
                s.closed_by = closed_by;
                s.floor = floor.map(str::to_owned);
                s
            };
            let counted = [
                at("p::a", 0, Status::Merged, Some(1), None),
                // Triggered by the absorbing event, but absorbs nothing:
                // the same deadline, then one with no floor.
                at("p::a", 1, Status::Open, None, None),
                at("p::c", 1, Status::Open, None, None),
                at("p::b", 1, Status::Open, None, Some("2025-07-02")),
                at("p::a", 2, Status::Superseded, Some(3), None),
                at("p::c", 3, Status::Open, None, None),
                at("p::a", 3, Status::Open, None, None),
                at("p::a", 4, Status::Discharged, Some(5), None),
            ];
            assert_eq!(replacement(&counted, 0), Some(3));
            assert_eq!(replacement(&counted, 1), None);
            assert_eq!(replacement(&counted, 4), Some(6));
            assert_eq!(replacement(&counted, 7), None);
        });
    }

    #[test]
    fn deadline_labels_read_like_the_source() {
        let session = testkit::example_with("");
        let all = session.hir();
        let decl = hirx::deadlines(all[0], "answer-period").next().unwrap();
        assert_eq!(
            deadline_label(decl),
            "answer-period: 21 days after Served(complaint)"
        );
        let mut d = decl.clone();
        d.amount = 1;
        d.direction = hir::Direction::Backward;
        let units = [
            (hir::DayUnit::Calendar, "day"),
            (hir::DayUnit::Court, "court day"),
            (hir::DayUnit::Business, "business day"),
            (hir::DayUnit::Week, "week"),
            (hir::DayUnit::Month, "month"),
            (hir::DayUnit::Year, "year"),
        ];
        for (unit, name) in units {
            d.unit = unit;
            assert_eq!(
                deadline_label(&d),
                format!("answer-period: 1 {name} before Served(complaint)")
            );
        }
        d.designated = Some(hir::Designated {
            event: "Served".to_owned(),
            param: 0,
            name: "due".to_owned(),
            interpretation: None,
        });
        assert_eq!(
            deadline_label(&d),
            "answer-period: the date Served(complaint) designates, at least 1 year before it"
        );
    }

    #[test]
    fn diagnostics_keep_errors_and_the_file_s_own_warnings() {
        let session = testkit::example_with("");
        let all = session.hir();
        let texts = session.sources();
        let b = Builder::new(&all, &texts, "2025-06-01".to_owned(), None);
        let at = |file: &str, severity| {
            let mut d = Diagnostic::error(
                DiagnosticCode::InvalidStart,
                Span {
                    file: file.into(),
                    line: 1,
                    col: 1,
                    end_line: 1,
                    end_col: 7,
                },
                format!("{severity:?}"),
            );
            d.severity = severity;
            d
        };
        let given = [
            at("other.lit", Severity::Error),
            at("other.lit", Severity::Warning),
            at("lit/examples/rule12.lit", Severity::Warning),
            at("lit/examples/rule12.lit", Severity::Info),
            at("lit/examples/rule12.lit", Severity::Hint),
        ];
        let kept: Vec<(TimelineSeverity, usize)> =
            diagnostics(&b, "lit/examples/rule12.lit", &given)
                .into_iter()
                .map(|d| (d.severity, d.source.unwrap().end))
                .collect();
        assert_eq!(
            kept,
            [
                (TimelineSeverity::Note, 6),
                (TimelineSeverity::Note, 6),
                (TimelineSeverity::Warning, 6),
                (TimelineSeverity::Error, 0),
            ]
        );
    }
}
