//! `litigate worlds CASE --world FILE`: alternate timelines (lg-fq2.4;
//! docs/litigate/CHANGE.md §5.1). The matter is replayed in every world
//! the `.world` file spans: its own layers plus one option at each of its
//! choice points (`[choice.ID]`, and `--vary NAME`: with and without a
//! layer), at the query's `--as-of` and `--known-at` (the matter's own by
//! default). Each world's layers in force join the law the matter's
//! packages see, so an amendment, override, waiver or grant governs where
//! precedence selects it. The result lists every world with its outcome,
//! each filing (an event `by` a party) with the worlds in which it is
//! timely, late, unjudged, untimed or rejected, and the worlds grouped by
//! identical outcomes. A filing whose event files a paper kind (`event
//! E() paper k`) names, in each world, the limit precedence selects for
//! it there on its date and the layer that set it (lg-962.10.3), and
//! lists the worlds by that layer: a world without an overriding layer
//! is the base's.
//!
//! A contested question (a choice point the file declares
//! `[contested.ID]`: unsettled law at ι) lowers to a chance node whose
//! branches are its options, each with its prior's probability and the
//! worlds taking it (design review §3.4, §6 item 23). The uncontested
//! choice points stay worlds: each `timelines` entry is one combination
//! of them, with the probability of each filing's outcome there over the
//! contested draws.
//!
//! A world's own errors (two of its layers in unresolved conflict, a
//! synthetic layer in an actual world) leave its law undetermined: its
//! filings are `undetermined` there, never judged. A late filing is an
//! outcome of its world, not an error of the query: the envelope carries
//! errors only when the world file or a package fails, or the matter
//! itself (its syntax, names, pins or types, which no layer changes).
use super::filing::paper;
use crate::clock::{self, Clocks};
use crate::ics;
use crate::matter::{self, At, CaseCheck, Scheduled};
use crate::out::{Failure, Outcome, Report};
use crate::ws::{self, Session, Targets};
use litigate_base::{Diagnostic, Profile, Severity};
use litigate_engine::clock::Date;
use litigate_lower::Options;
use litigate_sema::case::Status;
use litigate_sema::change::choice::{self, Alternate, MAX_WORLDS};
use litigate_sema::change::world::{parse_world, World};
use litigate_sema::db::span::TextRange;
use litigate_sema::hir::{self, EventParty};
use litigate_sema::pkg::source::Source;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

/// What `worlds` was asked for.
#[derive(Debug, Clone, Default)]
pub struct Args {
    /// The `.case` file.
    pub case: String,
    /// The `.world` file.
    pub world: String,
    /// Package directories (or directories to search) to load besides
    /// the library.
    pub paths: Vec<String>,
    /// Layers added to every world.
    pub layers: Vec<String>,
    /// Layers each made a choice point: every world with and without it.
    pub vary: Vec<String>,
    /// Replaces the matter's `as_of`.
    pub as_of: Option<String>,
    /// Replaces the matter's `known_at`.
    pub known_at: Option<String>,
    /// Library directories.
    pub lib: Vec<String>,
    /// Each deadline also names the `cite` of its version and the
    /// `counted_by` cite of the counting version that counted it (`null`
    /// when it has none): the expect tests record them (`litigate test
    /// tests/expect`).
    pub cites: bool,
}

/// How a filing fares in one world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
enum Timeliness {
    /// It discharged every deadline it answered, in time.
    Timely,
    /// It answered a deadline after its due date (L4002).
    Late,
    /// It answered a deadline whose due date is unknown (L4010) or cannot
    /// be counted (L3009).
    Unjudged,
    /// It answered no deadline in this world.
    Untimed,
    /// Replay rejected it (not a legal move here, say).
    Rejected,
    /// The world's own law is in error: nothing is judged in it.
    Undetermined,
}

fn is_error(d: &Diagnostic) -> bool {
    d.severity == Severity::Error
}

/// The case, the world and the packages under `paths`.
fn targets(src: &dyn Source, args: &Args) -> Result<Targets, Failure> {
    let case = ws::discover_in(src, std::slice::from_ref(&args.case))?;
    let world = ws::discover_in(src, std::slice::from_ref(&args.world))?;
    let [case] = case.cases.as_slice() else {
        return Err(Failure::new(format!("{}: not a .case file", args.case)));
    };
    let [world] = world.worlds.as_slice() else {
        return Err(Failure::new(format!("{}: not a .world file", args.world)));
    };
    Ok(Targets {
        packages: ws::discover_in(src, &args.paths)?.packages,
        cases: vec![case.clone()],
        worlds: vec![world.clone()],
        files: vec![case.clone()],
    })
}

/// `world` with the `--layer` names added to its layers and each `--vary`
/// name a choice point of its own (`without`, `with`); each must be a
/// loaded package not in the world already.
fn widen(
    session: &Session,
    world: &mut World,
    at: &mut BTreeMap<String, TextRange>,
    args: &Args,
) -> Result<(), Failure> {
    for (name, vary) in args
        .layers
        .iter()
        .map(|n| (n, false))
        .chain(args.vary.iter().map(|n| (n, true)))
    {
        if !session.packages.iter().any(|p| &p.name == name) {
            return Err(Failure::new(format!(
                "layer `{name}` is not a loaded package"
            )));
        }
        let taken = world.layers.contains(name)
            || world.choices.contains_key(name)
            || choice::layers(&world.choices).any(|l| l == name);
        if taken {
            return Err(Failure::new(format!(
                "layer `{name}` is in the world already"
            )));
        }
        at.entry(name.clone()).or_default();
        if vary {
            let options = [
                ("without".to_owned(), BTreeSet::new()),
                ("with".to_owned(), BTreeSet::from([name.clone()])),
            ];
            world.choices.insert(name.clone(), options.into());
        } else {
            world.layers.insert(name.clone());
        }
    }
    let worlds = world
        .choices
        .values()
        .fold(1usize, |n, options| n.saturating_mul(options.len()));
    if worlds > MAX_WORLDS {
        return Err(Failure::new(format!(
            "the choice points span {worlds} worlds, more than {MAX_WORLDS}"
        )));
    }
    Ok(())
}

/// A scheduled deadline as a world reports it; with the packages `all`
/// it is counted under, also the `cite` of the deadline's version that
/// governs it and the `counted_by` cite of the counting rule's version
/// that counted it.
fn deadline(s: &Scheduled, all: Option<&[&hir::Package]>) -> Value {
    let due = ics::due_of(s).ok().map(Date::to_iso);
    let mut out = json!({
        "deadline": s.deadline, "trigger": s.trigger, "trigger_date": s.trigger_date,
        "due": due, "status": ics::status_name(s),
    });
    if let Some(all) = all {
        out["cite"] = json!(deadline_cite(all, s));
        out["counted_by"] = json!(s.cite);
    }
    out
}

/// The `cite` of the version of `s`'s deadline that counted it (`id@from`;
/// the first version of that id when none did).
fn deadline_cite(all: &[&hir::Package], s: &Scheduled) -> Option<String> {
    let (pack, id) = s.deadline.split_once("::")?;
    let from = s.version.rsplit_once('@').map(|(_, from)| from);
    let versions = all
        .iter()
        .filter(|p| p.manifest.name == pack)
        .flat_map(|p| crate::hirx::declarations(p))
        .filter_map(|d| match d {
            hir::Declaration::Deadline(d) if d.meta.id == id => Some(d),
            _ => None,
        });
    let mut first = None;
    for d in versions {
        if from.is_none_or(|f| d.meta.effective.from == f) {
            return d.meta.cite.clone();
        }
        first.get_or_insert_with(|| d.meta.cite.clone());
    }
    first.flatten()
}

/// One filing: its source index, its key (its id, else `#index`), and
/// the line range of its event.
struct Filing<'c> {
    index: usize,
    key: String,
    event: &'c hir::Event,
    lines: (u32, u32),
}

/// Every event `by` a party in `case`, with where its line is.
fn filings<'c>(case: &'c hir::Case, spans: &litigate_sema::case::CaseSpans) -> Vec<Filing<'c>> {
    case.events
        .iter()
        .enumerate()
        .filter(|(_, e)| e.relation == EventParty::By)
        .map(|(index, event)| Filing {
            index,
            key: event.id.clone().unwrap_or_else(|| format!("#{index}")),
            event,
            lines: spans
                .events
                .get(index)
                .and_then(|s| s.whole.as_ref())
                .map_or((0, 0), |s| (s.line, s.end_line)),
        })
        .collect()
}

/// How `filing` fares in a world whose law is in error (`!lawful`) or
/// whose matter checked as `m`.
fn timeliness(lawful: bool, m: &CaseCheck, filing: &Filing<'_>) -> Timeliness {
    if !lawful {
        return Timeliness::Undetermined;
    }
    // An instance it closed whose due date cannot be counted (a version
    // not recorded by `known_at`, say) is unjudged, whatever replay made
    // of it.
    let closed: Vec<Status> = m
        .counted
        .iter()
        .filter(|s| s.closed_by == Some(filing.index))
        .map(|s| match ics::due_of(s) {
            Ok(_) => s.status,
            Err(_) => Status::Unjudged,
        })
        .collect();
    let rejected = m
        .diagnostics
        .iter()
        .any(|d| is_error(d) && (filing.lines.0..=filing.lines.1).contains(&d.span.line));
    if closed.contains(&Status::Late) {
        Timeliness::Late
    } else if closed.contains(&Status::Unjudged) {
        Timeliness::Unjudged
    } else if rejected {
        Timeliness::Rejected
    } else if closed.contains(&Status::Discharged) {
        Timeliness::Timely
    } else {
        Timeliness::Untimed
    }
}

/// The limit governing `filing`'s paper in a world whose law is `law`,
/// in error unless `lawful`: `{limit, layer}`, the layer the one that set
/// it, else `{limit, layer: null, hole}`; `None` when its event files no
/// paper kind it declares.
fn limit(
    law: &[&hir::Package],
    lawful: bool,
    f: &Filing<'_>,
    known_at: Option<&str>,
) -> Option<Value> {
    let kind = paper::declared(law, &crate::hirx::event_text(&f.event.action))?;
    let name = format!("{}::{}", kind.pack, kind.name);
    let set = if lawful {
        paper::governing(law, &kind, &f.event.date, known_at)
    } else {
        Err("the world's law is in error: no limit is selected in it".to_owned())
    };
    Some(match set {
        Ok(set) => json!({"limit": name, "cite": set.limit.meta.cite, "layer": set.layer()}),
        Err(why) => json!({"limit": name, "layer": null, "hole": why}),
    })
}

/// One world run: its result entry, how each filing fares, and each
/// filing's [`limit`].
struct Run {
    entry: Value,
    fates: Vec<Timeliness>,
    limits: Vec<Option<Value>>,
}

/// What every world run shares.
struct Query<'s> {
    session: &'s Session,
    case: &'s ws::CaseText,
    filings: Vec<Filing<'s>>,
    fragments: clock::Fragments,
    as_of: String,
    known_at: Option<String>,
    at: At<'s>,
    cites: bool,
}

impl Query<'_> {
    /// The matter replayed in world `alt`.
    fn run(&self, alt: &Alternate) -> Run {
        let in_force: BTreeSet<String> = alt
            .check
            .layers
            .iter()
            .filter(|l| l.in_force_at(&self.as_of, self.known_at.as_deref()))
            .map(|l| l.name.clone())
            .collect();
        let mut names = self.session.scope(self.case);
        names.extend(self.session.closure(in_force.iter().cloned()));
        let all = self.session.hir_of(&names);
        let layers: Vec<&hir::Package> = all
            .iter()
            .copied()
            .filter(|p| in_force.contains(&p.manifest.name))
            .collect();
        let clocks = Clocks::with_fragments(&all, &self.fragments);
        let m = matter::check_under(&all, &layers, self.case, &clocks, Profile::Dev, self.at);
        let lawful = !alt.check.diagnostics.iter().any(is_error);
        let fates: Vec<Timeliness> = self
            .filings
            .iter()
            .map(|f| timeliness(lawful, &m, f))
            .collect();
        let layers: Vec<Value> = alt
            .check
            .layers
            .iter()
            .map(|l| {
                json!({
                    "name": l.name, "version": l.version, "class": l.class,
                    "in_force": in_force.contains(&l.name),
                    "from": l.from,
                    "overruled_by": l.overruled_by,
                })
            })
            .collect();
        let limits: Vec<Option<Value>> = self
            .filings
            .iter()
            .map(|f| limit(&all, lawful, f, self.known_at.as_deref()))
            .collect();
        let diagnostics: Vec<&Diagnostic> = alt
            .check
            .diagnostics
            .iter()
            .chain(m.diagnostics.iter().filter(|d| is_error(d)))
            .collect();
        let entry = json!({
            "id": alt.id,
            "choices": alt.combination.choices,
            "layers": layers,
            "lawful": lawful,
            "filings": self.filings.iter().zip(&fates)
                .map(|(f, t)| (f.key.clone(), json!(t)))
                .collect::<serde_json::Map<_, _>>(),
            "limits": self.filings.iter().zip(&limits)
                .filter_map(|(f, l)| Some((f.key.clone(), l.clone()?)))
                .collect::<serde_json::Map<_, _>>(),
            "deadlines": m.counted.iter()
                .map(|s| deadline(s, self.cites.then_some(all.as_slice())))
                .collect::<Vec<_>>(),
            "diagnostics": diagnostics,
        });
        Run {
            entry,
            fates,
            limits,
        }
    }
}

impl Query<'_> {
    /// Each filing with the worlds of each outcome, and the worlds grouped
    /// by identical outcomes, from `runs` of the worlds `ids`.
    fn outcomes(&self, runs: &[Run], ids: &[&str]) -> (Vec<Value>, Vec<Value>) {
        let per_filing = self
            .filings
            .iter()
            .enumerate()
            .map(|(i, f)| {
                let mut by: BTreeMap<Timeliness, Vec<&str>> = BTreeMap::new();
                // The worlds by the layer that set the filing's limit
                // (`hole` where none is selected).
                let mut set_by: BTreeMap<String, Vec<&str>> = BTreeMap::new();
                let mut kind = None;
                for (run, id) in runs.iter().zip(ids) {
                    by.entry(run.fates[i]).or_default().push(id);
                    if let Some(l) = &run.limits[i] {
                        kind = l["limit"].as_str();
                        let package = l["layer"]["package"].as_str().unwrap_or("hole");
                        set_by.entry(package.to_owned()).or_default().push(id);
                    }
                }
                let mut out = json!({
                    "key": f.key, "date": f.event.date, "party": f.event.party,
                    "event": crate::hirx::event_text(&f.event.action),
                    "worlds": by.iter()
                        .map(|(t, w)| (json!(t).as_str().unwrap_or_default().to_owned(), json!(w)))
                        .collect::<serde_json::Map<_, _>>(),
                });
                if let Some(kind) = kind {
                    out["limit"] = json!({"limit": kind, "set_by": set_by});
                }
                out
            })
            .collect();
        let mut classes: BTreeMap<&[Timeliness], Vec<&str>> = BTreeMap::new();
        for (run, id) in runs.iter().zip(ids) {
            classes.entry(run.fates.as_slice()).or_default().push(id);
        }
        let outcomes = classes
            .iter()
            .map(|(fates, worlds)| {
                json!({
                    "worlds": worlds,
                    "filings": self.filings.iter().zip(fates.iter())
                        .map(|(f, t)| (f.key.clone(), json!(t)))
                        .collect::<serde_json::Map<_, _>>(),
                })
            })
            .collect();
        (per_filing, outcomes)
    }

    /// Each world of the uncontested choice points of `world` with its
    /// worlds' probabilities, and each filing's probability of each
    /// outcome there, the contested questions' draws weighed, from `runs`
    /// of the worlds `ids`.
    fn timelines(&self, world: &World, runs: &[Run], ids: &[&str]) -> Vec<Value> {
        choice::timelines(world)
            .iter()
            .map(|t| {
                let fates = |i: usize| {
                    let mut by: BTreeMap<Timeliness, f64> = BTreeMap::new();
                    for (id, p) in &t.worlds {
                        if let Some(run) = ids.iter().position(|w| w == id).map(|k| &runs[k]) {
                            *by.entry(run.fates[i]).or_default() += p;
                        }
                    }
                    by.iter()
                        .map(|(t, p)| (json!(t).as_str().unwrap_or_default().to_owned(), json!(p)))
                        .collect::<serde_json::Map<_, _>>()
                };
                json!({
                    "choices": t.choices,
                    "worlds": t.worlds.iter()
                        .map(|(id, p)| json!({"id": id, "p": p}))
                        .collect::<Vec<_>>(),
                    "filings": self.filings.iter().enumerate()
                        .map(|(i, f)| (f.key.clone(), Value::Object(fates(i))))
                        .collect::<serde_json::Map<_, _>>(),
                })
            })
            .collect()
    }
}

/// A report of `diagnostics` only: the query could not run.
fn failed(session: &Session, extra: (String, String), diagnostics: Vec<Diagnostic>) -> Report {
    Report {
        provenance: crate::prov::session(&session.packages, None, None),
        sources: session.sources().into_iter().chain([extra]).collect(),
        diagnostics,
        ..Report::default()
    }
}

/// The errors that stop the query: of a package in `scope`, of the world
/// file's own world, and the matter's own syntax, resolution, pins and
/// types (L0xxx-L2xxx), the same in every world since layers change only
/// which law governs.
fn errors(c: &crate::Checked, scope: &BTreeSet<String>) -> Vec<Diagnostic> {
    let packages = c
        .verdicts
        .iter()
        .filter(|(name, _)| scope.contains(name))
        .flat_map(|(_, v)| v.diagnostics.iter());
    let matter = c.cases[0]
        .diagnostics
        .iter()
        .filter(|d| d.code.as_str().as_bytes()[1] <= b'2');
    packages
        .chain(&c.worlds[0].diagnostics)
        .chain(matter)
        .filter(|d| is_error(d))
        .cloned()
        .collect()
}

/// `litigate worlds`.
pub fn worlds(src: &dyn Source, args: &Args) -> Outcome {
    for date in [&args.as_of, &args.known_at].into_iter().flatten() {
        Date::parse_iso(date).map_err(|e| Failure::new(format!("{date}: {e}")))?;
    }
    let targets = targets(src, args)?;
    let extra: Vec<String> = args.layers.iter().chain(&args.vary).cloned().collect();
    let session = Session::load_layers_in(src, &targets, &args.lib, &extra)?;
    let c = crate::check_session(session, Profile::Dev, &Options::default());
    let session = &c.session;
    let file = session.worlds[0].clone();
    let source = (file.path.clone(), file.text.clone());
    let base_case = &c.cases[0];
    let (mut world, mut at) = parse_world(&file.text).unwrap_or_default();
    widen(session, &mut world, &mut at, args)?;
    let mut scope = session.scope(&session.cases[0]);
    let layer_names = world
        .layers
        .iter()
        .chain(choice::layers(&world.choices))
        .cloned();
    scope.extend(session.closure(layer_names));
    let Some(case) = base_case.case.as_ref() else {
        return Ok(failed(session, source, base_case.diagnostics.clone()));
    };
    let errors = errors(&c, &scope);
    if !errors.is_empty() {
        return Ok(failed(session, source, errors));
    }
    let all = session.hir_of(&scope);
    let spans =
        litigate_sema::case::resolve(&base_case.path, &session.cases[0].text, &all, None).spans;
    let query = Query {
        session,
        case: &session.cases[0],
        filings: filings(case, &spans),
        fragments: clock::fragments(&all),
        as_of: args.as_of.clone().unwrap_or_else(|| case.as_of.clone()),
        known_at: args.known_at.clone().or_else(|| case.known_at.clone()),
        at: At {
            as_of: args.as_of.as_deref(),
            known_at: args.known_at.as_deref(),
        },
        cites: args.cites,
    };
    let alternates = choice::enumerate(
        &session.db,
        session.ws,
        &file.path,
        &file.text,
        &world,
        &at,
        Profile::Dev,
    );
    let runs: Vec<Run> = alternates.iter().map(|a| query.run(a)).collect();
    let ids: Vec<&str> = alternates.iter().map(|a| a.id.as_str()).collect();
    let (per_filing, outcomes) = query.outcomes(&runs, &ids);
    let timelines = query.timelines(&world, &runs, &ids);
    let result = json!({
        "case": case.id,
        "world": file.path,
        "modality": world.modality,
        "location": world.location,
        "choices": world.choices,
        "as_of": query.as_of,
        "known_at": query.known_at,
        "worlds": runs.into_iter().map(|r| r.entry).collect::<Vec<_>>(),
        "filings": per_filing,
        "outcomes": outcomes,
        "chance": choice::chances(&world),
        "timelines": timelines,
    });
    Ok(Report {
        result,
        provenance: crate::prov::session(
            &session.packages,
            Some(&query.as_of),
            query.known_at.as_deref(),
        ),
        sources: session.sources().into_iter().chain([source]).collect(),
        ..Report::default()
    })
}
