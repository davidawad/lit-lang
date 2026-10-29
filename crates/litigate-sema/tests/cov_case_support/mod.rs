//! Fixture packages for the `cov_case_*` tests: a primary package `alpha`
//! whose graph has one branch per replay path, its dependency `beta` (a
//! counting rule outside the matter's `uses`), a second used package
//! `gamma`, and a links package `delta`.
#![allow(dead_code)]
use litigate_base::{Diagnostic, DiagnosticCode, Profile};
use litigate_sema::case::{check_source, CheckedCase, Clock, Env};
use litigate_sema::hir::Package;
use litigate_sema::pkg::load::{LoadedFile, LoadedPackage};
use litigate_sema::pkg::lock::LockFile;
use std::fmt::Write as _;
use std::sync::OnceLock;

fn manifest(name: &str, deps: &[&str], roles: &str) -> String {
    let mut text = format!(
        "[package]\nname = \"{name}\"\nversion = \"1.0.0\"\nlanguage = 1\npack_id = \"{name}\"\n\
         forum = \"demo\"\nauthority = \"FederalRule\"\ntitle = \"T\"\nin_force = \"2009-12-01\"\n\
         known_at = \"2025-01-01\"\n\n[package.roles]\nPlaintiff = \"self\"\nDefendant = \"opponent\"\n\
         Court = \"nature\"\n{roles}\n[sources.frcp]\ntitle = \"FRCP\"\n\
         url = \"https://www.law.cornell.edu/rules/frcp\"\nas_of = \"2024-12-01\"\n\
         cite_prefixes = [\"FRCP\"]\n"
    );
    if !deps.is_empty() {
        text.push_str("\n[dependencies]\n");
        for d in deps {
            let _ = writeln!(text, "{d} = {{ version = \"^1.0.0\", path = \"../{d}\" }}");
        }
    }
    text
}

pub fn package(name: &str, deps: &[&str], roles: &str, text: &str) -> LoadedPackage {
    LoadedPackage {
        texts: std::collections::BTreeMap::new(),
        root: format!("pkgs/{name}"),
        manifest: manifest(name, deps, roles),
        lock: None,
        files: vec![LoadedFile {
            relative: format!("{name}.lit"),
            text: text.to_owned(),
        }],
    }
}

/// Checks `loaded` and returns their HIR, panicking with the diagnostics
/// of any package that does not check.
pub fn compile(loaded: &[LoadedPackage]) -> Vec<Package> {
    let mut db = litigate_sema::LitDatabase::default();
    let ws = litigate_sema::db::inputs::create_workspace(&mut db, loaded);
    litigate_sema::check_workspace(&db, ws)
        .iter()
        .map(|c| {
            c.package.clone().unwrap_or_else(|| {
                panic!(
                    "{}",
                    serde_json::to_string_pretty(&c.diagnostics).unwrap_or_default()
                )
            })
        })
        .collect()
}

pub const BETA: &str = "module beta
start b0
counting bc cite \"FRCP 6(a)\" { exclude_trigger_day, include_last_day, holidays bcal }
calendar bcal { new-year: fixed(1, 1) effective [1870-06-28, ..) observed nearest_weekday }
terminal b0: \"B0\" { payoff 0.usd }
";

pub const ALPHA: &str = "module alpha
import beta.{ bc }
start intake

enum Document { complaint, motion }
enum Method { Personal, Mail }

event Served(document: Document, method: Method)
event Filed(document: Document, note: String)
event Answered(document: Document)
event Ruled(on: Date, granted: Bool, votes: Number, fee: Money<usd>)
event Logged(hours: Hours)
event Twin(document: Document)
event Noted(document: Document)
event Reopened(document: Document)
event Forfeited(option: EdgeRef) removes option
event Excused(document: Document) waives court-period
event Benched(document: Document)

counting ac cite \"FRCP 6(a)\" { exclude_trigger_day, include_last_day, holidays acal }
calendar acal { new-year: fixed(1, 1) effective [1870-06-28, ..) observed nearest_weekday }

deadline court-period: 5.court_days from Served(complaint) counted_by ac { waive { by Court } }
deadline business-period: 3.business_days from Filed(complaint) counted_by ac
deadline remote-period: 10.days from Filed(complaint) counted_by bc
deadline one-day: 1.days from Filed(complaint) counted_by ac
deadline future-period: 10.days from Served(complaint) counted_by ac effective [2030-01-01, ..)
deadline grace: 10.days from Reopened(complaint) counted_by ac effective [2009-12-01, 2015-01-01)
deadline grace: 20.days from Reopened(complaint) counted_by ac effective [2015-01-01, ..)
transition keep-grace amends grace@2015-01-01 cite \"FRCP 1\"
{
  unless !has_event(Noted(complaint)) && (has_event(Noted(motion)) || false) -> grace@2009-12-01
  otherwise -> grace@2015-01-01
}

state intake: \"Intake\" by Plaintiff
{
  serve \"Serve\" -> respond { event Served(complaint) }
  file \"File\" -> filed { event Filed(complaint) }
  rule \"Rule\" -> answered { event Ruled() }
  log \"Log\" -> answered { event Logged() }
  twin-a \"Twin A\" -> answered { event Twin(complaint) }
  twin-b \"Twin B\" -> defaulted { event Twin(complaint) }
  probe \"Probe\" -> f1 { event Noted(motion) }
  to-bench \"To bench\" -> bench { event Benched(complaint) }
  to-lone \"To lone\" -> lone { event Benched(motion) }
  archive \"Archive\" -> new-stage { event Answered(motion) }
  stage \"Stage\" -> staged { event Reopened(motion) }
}

decision respond: \"Respond\" by Defendant
{
  answer \"Answer\" -> answered { event Answered(complaint), within court-period }
  late \"Late\" -> defaulted { event Twin(motion), within future-period }
  reopen \"Reopen\" -> answered { event Reopened(motion), within grace }
  default \"Default\" -> defaulted
}

state filed: \"Filed\"
{
  reply \"Reply\" -> answered { event Answered(complaint), within business-period }
  remote \"Remote\" -> defaulted { event Twin(complaint), within remote-period }
  quick \"Quick\" -> answered { event Reopened(complaint), within one-day }
  _ \"Withdraw\" -> defaulted { event Noted(motion) }
}

chance f1: \"F1\" by Court
{
  tags [fact]
  to-f2 \"To F2\" -> f2 p 1/2
  out \"Out\" -> answered p 1/2
}

chance f2: \"F2\" by Court
{
  tags [fact]
  to-f1 \"To F1\" -> f1 p 1/2
  out \"Out\" -> defaulted p 1/2
}

decision bench: \"Bench\" by Court
{
  allow \"Grant\" -> answered
  deny \"Deny\" -> defaulted
}

decision lone: \"Lone\" by Court
{
  only \"Only\" -> answered { event Benched(motion) }
}

state new-stage: \"New\" by Plaintiff effective [2030-01-01, ..)
{
  go \"Go\" -> answered { event Answered(motion) }
}

state staged: \"Staged\" by Plaintiff effective [2009-12-01, 2010-06-01)
{
  go \"Go\" -> answered { event Answered(motion) }
}
state staged: \"Staged\" by Plaintiff effective [2010-06-01, ..) supersedes staged@2009-12-01
{
  go \"Go\" -> answered { event Answered(motion) }
}
transition keep-staged amends staged@2010-06-01 cite \"FRCP 1\"
{
  unless has_event(Noted(complaint)) -> staged@2009-12-01
  otherwise -> staged@2010-06-01
}

terminal answered: \"Answered\" { payoff 0.usd }
terminal defaulted: \"Defaulted\" outcome [loss]
";

pub const GAMMA: &str = "module gamma
start g0
enum Kind { a, b }
event Pinged(kind: Kind)
event Struck(kind: Kind)
state g0: \"G0\" by Plaintiff
{
  ping \"Ping\" -> g1 { event Pinged(a) }
  strike \"Strike\" -> g1 { event Struck(a) }
}
terminal g1: \"G1\" { payoff 0.usd }
";

pub const DELTA: &str = "module delta
import alpha.{ Noted, Benched, ac }
deadline link-period: 3.days from Benched(motion) counted_by ac
link hop \"Hop\": alpha::bench -> gamma::g0 by Court { event Noted(complaint) }
link copy \"Copy\": alpha::lone -> alpha@copy::intake by Court { event Noted(complaint) }
link wait \"Wait\": alpha::lone -> alpha::defaulted by Court { event Noted(motion), within link-period }
instance alpha@copy { pack alpha }
";

/// `alpha`, `beta`, `gamma`, `delta` in load order.
pub fn fixture() -> &'static [Package] {
    static P: OnceLock<Vec<Package>> = OnceLock::new();
    P.get_or_init(|| {
        compile(&[
            package("beta", &[], "", BETA),
            package("alpha", &["beta"], "either = \"nature\"\n", ALPHA),
            package("gamma", &[], "", GAMMA),
            package("delta", &["alpha", "gamma"], "", DELTA),
        ])
    })
}

/// Checks `text` as `cases/t.case` against `packages`.
pub fn check(
    text: &str,
    packages: &[Package],
    lock: Option<&LockFile>,
    clock: &dyn Clock,
    profile: Profile,
) -> CheckedCase {
    let refs: Vec<&Package> = packages.iter().collect();
    let mut checked = check_source(
        "cases/t.case",
        text,
        Env {
            packages: &refs,
            lock,
            clock,
            profile,
        },
    );
    // An event without an id (L9006, info) is on nearly every fixture
    // line; its own tests read it from `check_source`.
    checked
        .diagnostics
        .retain(|d| d.code != DiagnosticCode::UndeclaredEventId);
    checked
}

/// A clock that puts every due date on one day.
pub struct At(pub &'static str);

impl Clock for At {
    fn due(&self, _: &litigate_sema::case::DueRequest) -> Result<String, String> {
        Ok(self.0.to_owned())
    }

    fn nominal(&self, r: &litigate_sema::case::DueRequest) -> Result<String, String> {
        self.due(r)
    }
}

/// A matter over the fixture: `uses`, then `head` (items between `summary`
/// and `parties`), parties `p` (Plaintiff), `d` (Defendant), `c` (Court),
/// `events` (one per line), then `tail` (items after `events`).
pub fn matter(uses: &str, head: &str, events: &[&str], tail: &str) -> String {
    let events = events.iter().fold(String::new(), |mut s, e| {
        let _ = writeln!(s, "  {e}");
        s
    });
    format!(
        "case t\nuses {uses}\nas_of 2011-06-01\nsummary \"s\"\n{head}\n\
         parties {{\n  p: Plaintiff Corporation\n  d: Defendant Corporation\n  c: Court Tribunal\n}}\n\
         events {{\n{events}}}\n{tail}\n"
    )
}

/// The default `uses`: `alpha` alone.
pub const ALPHA_ONLY: &str = "alpha ^1.0.0";

/// Stakes for every terminal of `alpha`.
pub const STAKES: &str = "stakes { answered: 0.usd, defaulted: 0.usd }";

/// Diagnostics (with notes and fixes), then `ok: start <posture>` when
/// the matter checks, then the replay trace.
pub fn outcome(checked: &CheckedCase) -> Vec<String> {
    let mut out = render(&checked.diagnostics);
    if checked.ok() {
        let start = checked.case.as_ref().and_then(|c| c.start.clone());
        out.push(format!("ok: start {}", start.unwrap_or_default()));
    }
    out.extend(checked.trace.iter().map(|s| {
        format!(
            "trace {:?} {} {} -> {}{}",
            s.effect,
            s.event,
            s.from,
            s.to,
            s.edge
                .as_ref()
                .map(|e| format!(" via {e}"))
                .unwrap_or_default()
        )
    }));
    out
}

/// Checks a matter against the fixture and its lock with `clock`.
pub fn run(text: &str, clock: &dyn Clock, profile: Profile) -> Vec<String> {
    let lock = lock(fixture());
    outcome(&check(text, fixture(), Some(&lock), clock, profile))
}

/// A `Lit.lock` pinning `packages` as loaded, by source digest only (no
/// semantic digest: a row's content is its source digest).
pub fn lock(packages: &[Package]) -> LockFile {
    LockFile::new(
        packages
            .iter()
            .map(|p| litigate_sema::pkg::lock::LockedPackage {
                name: p.manifest.name.clone(),
                version: p.manifest.version.clone(),
                source: format!("sha256:{}", p.digest),
                semantic: litigate_sema::pkg::lock::UNCHECKED.to_owned(),
                dependencies: Vec::new(),
                language: 1,
                ..Default::default()
            })
            .collect(),
    )
}

/// One line per diagnostic, notes and fixes indented below it.
pub fn render(diagnostics: &[Diagnostic]) -> Vec<String> {
    diagnostics
        .iter()
        .map(|d| {
            let mut s = format!(
                "{}:{} {} {}",
                d.span.line,
                d.span.col,
                d.code.as_str(),
                d.message
            );
            for l in &d.labels {
                let _ = write!(
                    s,
                    "\n    note @{}:{}: {}",
                    l.span.line, l.span.col, l.message
                );
            }
            for n in &d.notes {
                let _ = write!(s, "\n    note: {}", n.message);
            }
            for (f, e) in d
                .fixes
                .iter()
                .flat_map(|f| f.edits.iter().map(move |e| (f, e)))
            {
                let _ = write!(
                    s,
                    "\n    fix @{}:{}-{}:{} {:?}: {:?}",
                    e.span.line,
                    e.span.col,
                    e.span.end_line,
                    e.span.end_col,
                    f.applicability,
                    e.replacement
                );
            }
            s
        })
        .collect()
}
