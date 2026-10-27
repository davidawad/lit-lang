//! `litigate calibrate-replay CASE... --id ID` (DESIGN §10, RECAP replay
//! calibration): derive calibration observations from replayed, sourced
//! docket histories and emit them as a calibration set in the existing
//! format (`docs/CALIBRATION.md`): a `calibration` declaration in the
//! package it calibrates, checked there and lowered back to the set.
//!
//! A history is a matter whose events came from a docket (by hand or by
//! `match-docket --out`). It is *sourced* when it declares a `source`
//! whose title begins `Docket` and which has a `url`: that is the docket
//! its events were read from. An unsourced history contributes nothing.
//! Each matter must check clean; its replay trace, not its event names,
//! says what happened.
//!
//! An observation is one replay step that took an edge by a docketed
//! event (`docket N`): facts, `anytime` acts and events without a docket
//! number are not docket evidence and are left out. From them:
//!
//! - `edge-probability`, for an edge out of a `chance` stage only (a
//!   party's own choice is not odds): the share of the histories that
//!   left the stage by a docketed event that left it by this edge. A
//!   history still at the stage at its `as_of` is censored, not counted.
//! - `edge-duration`, for any edge: the days from the docketed event that
//!   entered the stage to the one that took the edge (`min`, the median
//!   as `mode`, `max`).
//!
//! Every estimate keeps its provenance and selection criteria in its
//! `method`: each observation's matter, docket entry, date and docket
//! URL, the counts behind the value, and the rules above. Docket matches
//! are not ground truth: an estimate is marked unreviewed, and `--out`
//! writes the set only with `--reviewed`, the attestation that every
//! history's docket matches were reviewed against the docket. Estimates
//! from fewer than `--min-n` observations are withheld, with the reason.
use super::build::lower_all;
use super::check::{check_session, checked_in};
use crate::matter::CaseCheck;
use crate::out::{Failure, Outcome, Report};
use crate::ws::{self, Session};
use litigate_base::{Diagnostic, DiagnosticCode as C, Profile, Severity, Span};
use litigate_engine::api::{
    CalibrationEntry, CalibrationSet, CalibrationSource, CalibrationTarget,
};
use litigate_engine::clock::Date;
use litigate_engine::model::Duration;
use litigate_import::Input;
use litigate_lower::Options;
use litigate_sema::case::{display_event, docket_order, Effect};
use litigate_sema::hir::{self, CaseSource, Declaration, NodeKind};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::Path;

/// What `calibrate-replay` was asked for.
#[derive(Debug, Clone, Default)]
pub struct Args {
    /// The `.case` files (or directories of them): the docket histories.
    pub cases: Vec<String>,
    /// The calibration set's id.
    pub id: String,
    /// Its title (default: one naming the histories).
    pub title: Option<String>,
    /// Fewest observations an estimate rests on.
    pub min_n: u64,
    /// The docket matches of every history were reviewed.
    pub reviewed: bool,
    /// Where to write the set, if anywhere.
    pub out: Option<String>,
    /// Library directories.
    pub lib: Vec<String>,
}

/// One docketed replay step that took an edge.
#[derive(Debug, Clone)]
struct Observation {
    /// The matter's id.
    case: String,
    /// The docket the matter cites.
    docket: CaseSource,
    /// The stage left.
    stage: String,
    /// The edge taken.
    edge: String,
    /// The event as written.
    event: String,
    /// Its date.
    date: String,
    /// Its docket number.
    entry: u64,
    /// The docketed event that entered the stage: `(date, docket number)`.
    entered: Option<(String, u64)>,
}

impl Observation {
    /// The observation as one line of an estimate's `method`.
    fn cite(&self) -> String {
        let url = self.docket.url.as_deref().unwrap_or_default();
        match &self.entered {
            Some((date, n)) => format!(
                "{} docket {n} ({date}) to docket {} ({}) {} <{url}>",
                self.case, self.entry, self.date, self.event
            ),
            None => format!(
                "{} docket {} ({}) {} <{url}>",
                self.case, self.entry, self.date, self.event
            ),
        }
    }

    /// Days from entering the stage to leaving it.
    fn days(&self) -> Option<i64> {
        let (from, _) = self.entered.as_ref()?;
        let day = |d: &str| Date::parse_iso(d).ok().map(Date::epoch_day);
        Some(day(&self.date)? - day(from)?)
    }
}

/// The docket `case` cites: its first `source` titled `Docket...` with a url.
fn docket_source(case: &hir::Case) -> Option<CaseSource> {
    case.sources
        .iter()
        .find(|s| s.url.is_some() && s.title.to_lowercase().starts_with("docket"))
        .cloned()
}

/// The docketed edge-taking steps of a matter's replay.
fn observations(check: &CaseCheck, case: &hir::Case, docket: &CaseSource) -> Vec<Observation> {
    // A step names its event by date and text; the k-th step of one
    // (date, text) is the k-th such event in docket order.
    let events: Vec<&hir::Event> = docket_order(&case.events).collect();
    let mut seen: BTreeMap<(&str, &str), usize> = BTreeMap::new();
    let mut entered: BTreeMap<&str, Option<(String, u64)>> = BTreeMap::new();
    let mut out = Vec::new();
    for step in &check.replay {
        let k = seen.entry((&step.date, &step.event)).or_default();
        let number = events
            .iter()
            .filter(|e| e.date == step.date && display_event(&e.action) == step.event)
            .nth(*k)
            .and_then(|e| e.docket);
        *k += 1;
        if step.from == step.to || !matches!(step.effect, Effect::Took | Effect::Fact) {
            continue;
        }
        let at = (step.effect == Effect::Took)
            .then_some(number)
            .flatten()
            .map(|n| (step.date.clone(), n));
        if let (Some((date, entry)), Some(edge)) = (&at, &step.edge) {
            out.push(Observation {
                case: case.id.clone(),
                docket: docket.clone(),
                stage: step.from.clone(),
                edge: edge.clone(),
                event: step.event.clone(),
                date: date.clone(),
                entry: *entry,
                entered: entered.get(step.from.as_str()).cloned().flatten(),
            });
        }
        entered.insert(&step.to, at);
    }
    out
}

/// Whether `stage` (`pack::local`, or `pack@from::local` for a linked
/// instance) is a `chance` node in the loaded packages.
fn is_chance(all: &[&hir::Package], stage: &str) -> bool {
    let (pack, local) = stage.rsplit_once("::").unwrap_or(("", stage));
    let pack = pack.split('@').next().unwrap_or(pack);
    all.iter()
        .filter(|p| p.manifest.pack_id == pack)
        .flat_map(|p| crate::hirx::declarations(p))
        .any(|d| {
            matches!(d, Declaration::Node(n) if n.meta.id == local
                && matches!(n.kind, NodeKind::Chance { .. }))
        })
}

/// What an estimate rests on, for its `source`, `n` and `method`.
struct Basis<'o> {
    /// The observations behind the value.
    used: Vec<&'o Observation>,
    /// The value's arithmetic.
    arithmetic: String,
    /// The selection criteria.
    criteria: &'static str,
}

/// The docket source of an estimate resting on `used`: the docket when
/// there is one, else every docket named in the title and the first's url
/// (the `method` gives each observation's own); the vintage the
/// observations' dates span; retrieved, the latest docket's `as_of`.
fn source(used: &[&Observation]) -> CalibrationSource {
    let mut dockets: Vec<&CaseSource> = Vec::new();
    for o in used {
        if !dockets.contains(&&o.docket) {
            dockets.push(&o.docket);
        }
    }
    let dates = used.iter().flat_map(|o| {
        o.entered
            .iter()
            .map(|(d, _)| d.as_str())
            .chain([o.date.as_str()])
    });
    let start = dates.clone().min().unwrap_or_default().to_owned();
    let end = dates.max().unwrap_or_default().to_owned();
    let title = match dockets.as_slice() {
        [one] => one.title.clone(),
        many => format!(
            "Replayed docket histories: {}",
            many.iter()
                .map(|d| d.title.as_str())
                .collect::<Vec<_>>()
                .join("; ")
        ),
    };
    CalibrationSource {
        title,
        url: dockets
            .first()
            .and_then(|d| d.url.clone())
            .unwrap_or_default(),
        vintage_end: (end != start).then_some(end),
        vintage_start: start,
        retrieved: dockets
            .iter()
            .filter(|d| !d.as_of_hole)
            .filter_map(|d| d.as_of.clone())
            .max(),
    }
}

/// The selection criteria of an `edge-probability` estimate.
const PROBABILITY: &str = "a share of the histories that left a chance stage by a docketed \
     event (`docket N`); facts, `anytime` acts and undocketed events are not counted, and a \
     history still at the stage at its `as_of` is censored";

/// The selection criteria of an `edge-duration` estimate.
const DURATION: &str = "days from the docketed event that entered the stage to the docketed \
     event that took the edge; a stage entered by a fact or an undocketed event gives no \
     duration; `mode` is the median";

/// An entry with its provenance and selection criteria in `method`.
fn entry(
    target: CalibrationTarget,
    edge: &str,
    (value, distribution): (Option<f64>, Option<Duration>),
    basis: &Basis<'_>,
    reviewed: bool,
) -> CalibrationEntry {
    let review = if reviewed {
        "docket matches reviewed"
    } else {
        "UNREVIEWED docket matches: not ground truth until reviewed against the docket"
    };
    CalibrationEntry {
        target,
        target_ref: edge.to_owned(),
        attr: None,
        value,
        distribution,
        source: source(&basis.used),
        n: basis.used.len() as u64,
        method: Some(format!(
            "Replay calibration ({review}). {}. Selection: {}. Observations: {}.",
            basis.arithmetic,
            basis.criteria,
            basis
                .used
                .iter()
                .map(|o| o.cite())
                .collect::<Vec<_>>()
                .join("; ")
        )),
    }
}

/// `x` to four decimals, as the shipped sets write rates.
fn round4(x: f64) -> f64 {
    (x * 10_000.0).round() / 10_000.0
}

/// The `edge-probability` of `edge` out of `stage`: `used` of the
/// `total` docketed exits, with `out` histories censored at the stage.
fn probability<'o>(
    (stage, edge): (&str, &str),
    used: &[&'o Observation],
    total: usize,
    out: usize,
) -> (f64, Basis<'o>) {
    #[allow(clippy::cast_precision_loss)] // observation counts are small
    let p = used.len() as f64 / total as f64;
    let k = used.len();
    let basis = Basis {
        used: used.to_vec(),
        arithmetic: format!(
            "{k} of {total} docketed exits from `{stage}` took `{edge}`: {k} / {total} = {p:.4}; \
             {out} history(ies) censored at the stage"
        ),
        criteria: PROBABILITY,
    };
    (round4(p), basis)
}

/// The `edge-duration` of the observations of `used` whose stage a
/// docketed event entered, if any.
fn duration<'o>(used: &[&'o Observation]) -> Option<(Duration, Basis<'o>)> {
    let timed: Vec<&Observation> = used
        .iter()
        .copied()
        .filter(|o| o.days().is_some())
        .collect();
    let mut days: Vec<i64> = timed.iter().filter_map(|o| o.days()).collect();
    days.sort_unstable();
    let (&min, &max) = days.first().zip(days.last())?;
    let mid = days.len() / 2;
    #[allow(clippy::cast_precision_loss)] // day counts are small
    let (median, min_f, max_f) = (
        if days.len() % 2 == 1 {
            days[mid] as f64
        } else {
            (days[mid - 1] + days[mid]) as f64 / 2.0
        },
        min as f64,
        max as f64,
    );
    let listed: Vec<String> = days.iter().map(i64::to_string).collect();
    let basis = Basis {
        used: timed,
        arithmetic: format!(
            "days {}: min {min}, median {median}, max {max}",
            listed.join(", ")
        ),
        criteria: DURATION,
    };
    let distribution = Duration {
        min: Some(min_f),
        mode: median,
        max: Some(max_f),
    };
    Some((distribution, basis))
}

/// The estimates `observations` support: `(entries, withheld)`.
fn estimates(
    all: &[&hir::Package],
    observations: &[Observation],
    censored: &BTreeMap<String, Vec<String>>,
    args: &Args,
) -> (Vec<CalibrationEntry>, Vec<Value>) {
    let mut by_stage: BTreeMap<&str, BTreeMap<&str, Vec<&Observation>>> = BTreeMap::new();
    for o in observations {
        by_stage
            .entry(&o.stage)
            .or_default()
            .entry(&o.edge)
            .or_default()
            .push(o);
    }
    let mut entries = Vec::new();
    let mut withheld = Vec::new();
    let mut keep = |e: CalibrationEntry, withheld: &mut Vec<Value>| {
        if e.n >= args.min_n {
            entries.push(e);
        } else {
            withheld.push(json!({
                "target": e.target, "ref": e.target_ref, "n": e.n,
                "reason": format!("{} observation(s), fewer than --min-n {}", e.n, args.min_n),
            }));
        }
    };
    for (stage, edges) in &by_stage {
        let total: usize = edges.values().map(Vec::len).sum();
        let chance = is_chance(all, stage);
        let out = censored.get(*stage).map_or(0, Vec::len);
        for (edge, used) in edges {
            if chance {
                let (p, basis) = probability((stage, edge), used, total, out);
                let target = CalibrationTarget::EdgeProbability;
                let e = entry(target, edge, (Some(p), None), &basis, args.reviewed);
                keep(e, &mut withheld);
            } else {
                withheld.push(json!({
                    "target": CalibrationTarget::EdgeProbability, "ref": edge, "n": used.len(),
                    "reason": format!("`{stage}` is not a chance stage: a party's choice is not odds"),
                }));
            }
            if let Some((d, basis)) = duration(used) {
                let target = CalibrationTarget::EdgeDuration;
                let e = entry(target, edge, (None, Some(d)), &basis, args.reviewed);
                keep(e, &mut withheld);
            }
        }
    }
    (entries, withheld)
}

/// What the histories hold: each one's report, the observations of the
/// sourced ones, and the matters still at each stage at their `as_of`.
#[derive(Default)]
struct Gathered {
    histories: Vec<Value>,
    observed: Vec<Observation>,
    censored: BTreeMap<String, Vec<String>>,
}

/// The observations of the checked matters `cases`.
fn gather(cases: &[CaseCheck]) -> Gathered {
    let mut g = Gathered::default();
    for check in cases {
        let Some(case) = &check.case else { continue };
        let Some(docket) = docket_source(case) else {
            g.histories.push(json!({
                "path": check.path, "case": case.id, "docket": null, "observations": 0,
                "excluded": "no `source \"Docket ...\"` with a url: not a sourced docket history",
            }));
            continue;
        };
        let found = observations(check, case, &docket);
        for stage in &check.marking {
            g.censored
                .entry(stage.clone())
                .or_default()
                .push(case.id.clone());
        }
        g.histories.push(json!({
            "path": check.path, "case": case.id, "as_of": case.as_of,
            "docket": {"title": docket.title, "url": docket.url, "as_of": docket.as_of},
            "observations": found.len(), "marking": check.marking,
        }));
        g.observed.extend(found);
    }
    g
}

/// The set of `entries` derived from the sourced histories `sourced`.
fn set_of(args: &Args, sourced: &[&str], entries: Vec<CalibrationEntry>) -> CalibrationSet {
    CalibrationSet {
        id: args.id.clone(),
        title: args
            .title
            .clone()
            .unwrap_or_else(|| format!("Replay calibration: {}", sourced.join(", "))),
        description: Some(format!(
            "Derived by `litigate calibrate-replay` from the replayed docket histories {} \
             ({}). Selection: only replay steps taken by docketed events of matters that \
             check clean and cite their docket; edge-probability for chance stages only; \
             at least {} observation(s) per estimate. Each entry's `method` lists its \
             observations and arithmetic.",
            sourced.join(", "),
            if args.reviewed {
                "docket matches reviewed"
            } else {
                "UNREVIEWED docket matches"
            },
            args.min_n
        )),
        entries,
    }
}

/// Where a derived set's declaration is said to come from.
const DERIVED: &str = "litigate calibrate-replay";

/// An L6002 at the derived set.
fn derived(message: String) -> Vec<Diagnostic> {
    let at = Span {
        file: DERIVED.into(),
        line: 1,
        col: 1,
        end_line: 1,
        end_col: 1,
    };
    vec![Diagnostic::error(C::SemanticDiff, at, message)]
}

/// `set` as a `calibration` declaration in the package it calibrates:
/// `(path, text)`, the path beside that package's modules. The text is
/// checked in its package and must lower back to `set`, so what `--out`
/// writes builds to the very estimates reported.
fn declaration(
    session: &Session,
    set: &CalibrationSet,
    lib: &[String],
) -> Result<(String, String), Vec<Diagnostic>> {
    let json = serde_json::to_string_pretty(set).map_err(|e| derived(e.to_string()))?;
    // Pack id to package directory name, and that name to its root.
    let mut dirs = BTreeMap::new();
    let mut roots = BTreeMap::new();
    for pkg in &session.packages {
        let Some(package) = &pkg.check.package else {
            continue;
        };
        let dir = Path::new(&pkg.root)
            .file_name()
            .map(|d| d.to_string_lossy().into_owned())
            .unwrap_or_default();
        dirs.insert(package.manifest.pack_id.clone(), dir.clone());
        roots.insert(dir, pkg.root.clone());
    }
    let imported = litigate_import::import_into(&[Input::new(DERIVED, json.clone())], &dirs)?;
    let [file] = imported.files.as_slice() else {
        return Err(derived("a calibration set imports to one file".to_owned()));
    };
    let (dir, name) = file
        .path
        .strip_prefix("lit/")
        .and_then(|r| r.split_once('/'))
        .unwrap_or_default();
    let Some(root) = roots.get(dir) else {
        return Err(derived(format!(
            "no loaded package `{dir}` to declare the set in"
        )));
    };
    let path = format!("{root}/{name}");
    lowers_back(session, (root, &path, &file.text), (&set.id, &json), lib)?;
    Ok((path, file.text.clone()))
}

/// Checks the declaration `text` at `path` in the package at `root`, as
/// loaded, and lowers it: errors unless it lowers to the set `json`.
fn lowers_back(
    session: &Session,
    (root, path, text): (&str, &str, &str),
    (id, json): (&str, &str),
    lib: &[String],
) -> Result<(), Vec<Diagnostic>> {
    let files = session
        .packages
        .iter()
        .flat_map(|p| p.files.iter().cloned())
        .chain([(path.to_owned(), text.to_owned())]);
    let tree = crate::memory_tree(files);
    let mut libs: Vec<String> = session
        .packages
        .iter()
        .filter_map(|p| Path::new(&p.root).parent())
        .map(|d| d.to_string_lossy().into_owned())
        .chain(lib.iter().cloned())
        .collect();
    libs.sort();
    libs.dedup();
    let options = Options::default();
    let checked = checked_in(&tree, &[root.to_owned()], Profile::Dev, &options, &libs)
        .map_err(|f| derived(f.0))?;
    // The package's `Lit.lock` is not in the tree: a stale lock says nothing.
    let errors: Vec<Diagnostic> = checked
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error && d.code != C::StaleLock)
        .cloned()
        .collect();
    if !errors.is_empty() {
        return Err(errors);
    }
    let built = lower_all(&checked, &options);
    let lowered = format!("packs/calibration/{id}.json");
    let Some((_, text)) = built.files.iter().find(|(n, _)| *n == lowered) else {
        return Err(derived(format!(
            "the declaration does not lower to `{lowered}`"
        )));
    };
    match litigate_lower::semantic_diff_json(text, json) {
        Ok((_, changes)) if changes.is_empty() => Ok(()),
        Ok((_, changes)) => Err(derived(format!(
            "the declaration lowers to a set with {} difference(s): {}",
            changes.len(),
            changes
                .iter()
                .map(|c| c.pointer.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ))),
        Err(message) => Err(derived(message)),
    }
}

/// An observation as the result lists it.
fn observation(o: &Observation) -> Value {
    json!({
        "case": o.case, "stage": o.stage, "edge": o.edge, "event": o.event,
        "date": o.date, "docket": o.entry, "docket_url": o.docket.url,
        "entered": o.entered.as_ref().map(|(d, n)| json!({"date": d, "docket": n})),
        "days": o.days(),
    })
}

/// `litigate calibrate-replay`.
pub fn calibrate_replay(args: &Args) -> Outcome {
    if args.out.is_some() && !args.reviewed {
        return Err(Failure::new(
            "--out writes a calibration set only with --reviewed: docket matches are not \
             ground truth until a person has reviewed them against the docket",
        ));
    }
    let targets = ws::discover(&args.cases)?;
    if targets.cases.is_empty() {
        return Err(Failure::new("no .case file among the histories"));
    }
    let session = Session::load(&targets, &args.lib)?;
    let c = check_session(session, Profile::Dev, &Options::default());
    let mut report = Report {
        provenance: c.provenance(None, None),
        sources: c.session.sources(),
        diagnostics: c.diagnostics.clone(),
        ..Report::default()
    };
    // Every history must check clean: replay of a matter with errors says
    // nothing about what its docket did.
    if c.cases
        .iter()
        .any(|k| k.case.is_none() || k.diagnostics.iter().any(|d| d.severity == Severity::Error))
        || report.has_errors()
    {
        return Ok(report);
    }
    let all = c.session.hir();
    let g = gather(&c.cases);
    let (entries, withheld) = estimates(&all, &g.observed, &g.censored, args);
    let sourced: Vec<&str> = g
        .histories
        .iter()
        .filter(|h| h.get("excluded").is_none())
        .filter_map(|h| h["case"].as_str())
        .collect();
    let set = set_of(args, &sourced, entries);
    let declaration = if set.entries.is_empty() {
        None
    } else {
        match declaration(&c.session, &set, &args.lib) {
            Ok(declaration) => Some(declaration),
            Err(diagnostics) => {
                report.diagnostics.extend(diagnostics);
                return Ok(report);
            }
        }
    };
    let written = declaration.as_ref().and(args.out.as_ref());
    if let (Some(out), Some((_, text))) = (written, &declaration) {
        std::fs::write(out, text).map_err(|e| Failure::new(format!("{out}: {e}")))?;
    }
    report.result = json!({
        "id": set.id, "reviewed": args.reviewed, "min_n": args.min_n, "written": written,
        "histories": g.histories,
        "observations": g.observed.iter().map(observation).collect::<Vec<_>>(),
        "withheld": withheld, "set": set,
        "declaration": declaration.map(|(path, text)| json!({"path": path, "text": text})),
    });
    Ok(report)
}
