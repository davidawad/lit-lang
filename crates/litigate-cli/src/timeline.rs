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
use litigate_base::{Diagnostic, Severity, Span};
use litigate_sema::case::Status;
use litigate_sema::hir::{self, DeadlineWindow, Declaration, NodeKind};
use std::collections::BTreeMap;

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
}

/// A deadline declaration's display label (`answer-period: 30 days after
/// InstitutionNoticed()`).
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
    format!(
        "{}: {} {unit}{plural} {direction} {}",
        d.meta.id,
        d.amount,
        hirx::event_text(&d.trigger)
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
        let row =
            Draft {
                world,
                kind: RowKind::Deadline,
                date: None,
                due,
                label: deadline_label(decl),
                status: state,
                cite: decl.meta.cite.clone().or_else(|| {
                    hirx::rule_of(pkg, &decl.meta.id).and_then(|r| r.meta.cite.clone())
                }),
                source: self.source(decl.meta.span.as_ref()),
                layer,
                trigger,
            };
        self.push(row)
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
                self.deadline_row(
                    world,
                    owner,
                    decl,
                    None,
                    status(RowState::Unknown, Some(reason)),
                    trigger,
                );
            }
        }
    }

    /// The timeline.
    fn finish(self, source: TimelineSource, diagnostics: Vec<TimelineDiagnostic>) -> Timeline {
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

/// How an obligation instance's end reads as a row status.
fn obligation_status(
    ended: Status,
    closed: Option<(&str, &str)>,
    due: Option<&str>,
    as_of: &str,
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
                "timeliness unknown: designated due date not recorded; acted{by}"
            )),
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
    let event_rows = events(&mut b, m, case, &primary.unwrap_or_default());
    let mut seen = deadlines(&mut b, m, case, &event_rows);
    let start = case.start.clone().or_else(|| {
        used.first()
            .map(|p| format!("{}::{}", p.manifest.pack_id, p.start))
    });
    if let Some((pkg, node)) = start.as_deref().and_then(|s| b.node(s)) {
        b.governing(0, pkg, node, &mut seen, None);
        fork(&mut b, case, pkg, node, &seen, event_rows.last().copied());
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
/// deadline it triggered, else `primary`.
fn events(b: &mut Builder, m: &CaseCheck, case: &hir::Case, primary: &str) -> Vec<usize> {
    let spans = litigate_sema::case::resolve(&m.path, texts_of(b, &m.path), b.all, None).spans;
    let mut rows = Vec::new();
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
            Some((pkg, edge))
        });
        let layer = taken
            .map(|(p, _)| p.manifest.name.clone())
            .or_else(|| {
                m.schedule
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
            world: 0,
            kind: RowKind::Event,
            date: Some(e.date.clone()),
            due: None,
            label: format!("{text} {relation} {}", e.party),
            status: status(RowState::None, None),
            cite: e
                .authority
                .clone()
                .or_else(|| taken.and_then(|(_, x)| x.authority.clone())),
            source: b.source(spans.events.get(i).and_then(|s| s.whole.as_ref())),
            layer,
            trigger: None,
        };
        rows.push(b.push(row));
    }
    rows
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
    for s in &m.schedule {
        seen.push(s.deadline.clone());
        let ended = closed(s.closed_by);
        let ended = ended.as_ref().map(|(e, d)| (e.as_str(), d.as_str()));
        let trigger = Some(event_rows[s.trigger_event]);
        let Some(due) = s.extended_to.clone().or_else(|| s.due.clone()) else {
            // The clock cannot count it: no due date, and why.
            if let Some((pkg, decl)) = b.deadline(&s.deadline) {
                let mut state = obligation_status(s.status, ended, None, &b.as_of);
                state.reason = Some(format!(
                    "{}: {}",
                    state.reason.unwrap_or_default(),
                    s.unknown.as_deref().unwrap_or_default()
                ));
                b.deadline_row(0, pkg, decl, None, state, trigger);
            }
            continue;
        };
        let Some((pkg, decl)) = b.counted_by(s) else {
            continue;
        };
        let state = obligation_status(s.status, ended, Some(&due), &b.as_of);
        let row = b.deadline_row(0, pkg, decl, Some(due), state, trigger);
        if let (Status::Superseded, Some(j)) = (s.status, s.closed_by) {
            b.supersedes.push((event_rows[j], row));
        }
    }
    seen
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

/// The fork at the matter's posture `node` (of `pkg`): a fork row in the
/// docket, and one world per option still open to the matter.
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
    let fork_row = Draft {
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
    };
    let fork_at = b.push(fork_row);
    for edge in options {
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
        let world = b.worlds.len();
        b.worlds.push(TimelineWorld {
            id: format!("w{world}"),
            label: edge.label.clone(),
            parent: Some("w0".to_owned()),
            fork_row: None,
            choice: Some(TimelineChoice {
                kind,
                label: edge.label.clone(),
                cite: edge.authority.clone(),
                probability,
            }),
        });
        b.forks.push(Some(fork_at));
        let taken = Draft {
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
        };
        let taken = b.push(taken);
        if let Some((next_pkg, next)) = b.node(&target(pkg, &edge.to)) {
            let mut seen = seen.to_vec();
            b.governing(world, next_pkg, next, &mut seen, Some(taken));
        }
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
/// and triggers naming a dropped row.
fn retain(t: &mut Timeline, dropped: &[String], dead: &[String]) {
    t.worlds.retain(|w| !dead.contains(&w.id));
    t.rows.retain(|r| !dropped.contains(&r.id));
    for r in &mut t.rows {
        if r.trigger.as_ref().is_some_and(|x| dropped.contains(x)) {
            r.trigger = None;
        }
    }
    t.edges.retain(|e| {
        !dead.contains(&e.world) && !dropped.contains(&e.from) && !dropped.contains(&e.to)
    });
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

    #[test]
    fn layer_classes_follow_the_package_authority() {
        assert_eq!(layer_class("LocalRule"), LayerClass::LocalRule);
        assert_eq!(layer_class("StandingOrder"), LayerClass::StandingOrder);
        assert_eq!(layer_class("CaseOrder"), LayerClass::CaseOrder);
        assert_eq!(layer_class("Regulation"), LayerClass::BaseRule);
    }
}
