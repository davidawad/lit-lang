//! `litigate check`: every package and matter, every obligation.
use crate::cli::Dates;
use crate::matter::CaseCheck;
use crate::out::{Failure, Outcome, Report};
use crate::queries::{case_check, package_verdict, Build};
use crate::verdict::Verdict;
use crate::ws::{self, Session};
use litigate_base::{
    Applicability, Diagnostic, DiagnosticCode, FileId, Fix, Note, Profile, Severity, Span,
};
use litigate_lower::Options;
use litigate_lower::{EngineClock, Lowerer};
use litigate_sema::change::world::{check_world, WorldCheck};
use litigate_sema::pkg::source::{Disk, Source};
use litigate_verify::{Finding, Obligation};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

/// A completed check.
pub struct Checked {
    /// What was loaded.
    pub session: Session,
    /// Per package (load order), with its name.
    pub verdicts: Vec<(String, Verdict)>,
    /// Per matter.
    pub cases: Vec<CaseCheck>,
    /// Per world (docs/litigate/CHANGE.md §5).
    pub worlds: Vec<WorldCheck>,
    /// Every diagnostic.
    pub diagnostics: Vec<Diagnostic>,
    /// Obligation findings.
    pub findings: Vec<Finding>,
    /// The profile.
    pub profile: Profile,
    /// Every package and matter was checked: false when
    /// [`check_session_with`] was cancelled part way, and `verdicts` and
    /// `cases` hold only those checked before.
    pub complete: bool,
}

/// Loads and checks `paths`.
///
/// # Errors
/// Usage or I/O failures.
pub fn checked(
    paths: &[String],
    profile: Profile,
    options: &Options,
    lib: &[String],
) -> Result<Checked, Failure> {
    checked_in(&Disk, paths, profile, options, lib)
}

/// [`checked`] from `src`.
///
/// # Errors
/// Usage or I/O failures.
pub fn checked_in(
    src: &dyn Source,
    paths: &[String],
    profile: Profile,
    options: &Options,
    lib: &[String],
) -> Result<Checked, Failure> {
    let targets = ws::discover_in(src, paths)?;
    Ok(check_session(
        Session::load_in(src, &targets, lib)?,
        profile,
        options,
    ))
}

/// Loads and checks `targets` from `src`.
///
/// # Errors
/// I/O failures.
pub fn checked_targets(
    src: &dyn Source,
    targets: &ws::Targets,
    profile: Profile,
    options: &Options,
    lib: &[String],
) -> Result<Checked, Failure> {
    Ok(check_session(
        Session::load_in(src, targets, lib)?,
        profile,
        options,
    ))
}

/// The engine clock matters replay under: every counting rule of `all`
/// that lowers. A counting version that does not lower is left out, and
/// reported by its package's verification; every other rule still counts.
#[must_use]
pub fn engine(all: &[&litigate_sema::hir::Package]) -> EngineClock {
    EngineClock::lowered_partial(&Lowerer::new(all), all).0
}

/// Checks everything loaded in `session` at the build dates `options`
/// (each defaults to the package's `known_at`; a matter selects at its own
/// `as_of` and `known_at`), through the session database's queries
/// ([`crate::queries`]): a session checked again after an edit re-runs only
/// what the edit reaches.
#[must_use]
pub fn check_session(session: Session, profile: Profile, options: &Options) -> Checked {
    check_session_with(session, profile, options, &|| false)
}

/// Checks a session, stopping between packages and matters when cancelled.
#[must_use]
pub fn check_session_with(
    mut session: Session,
    profile: Profile,
    options: &Options,
    cancelled: &dyn Fn() -> bool,
) -> Checked {
    let inputs = session.case_inputs();
    let build = Build::of(&session.db, profile, options);
    let verdicts: Vec<(String, Verdict)> = session
        .packages
        .iter()
        .take_while(|_| !cancelled())
        .map(|p| {
            let verdict = if let Some(cache) = session.cache.as_ref() {
                crate::queries::package_verdict_with_cache(
                    &session.db,
                    session.ws,
                    p.input,
                    build,
                    Some(cache),
                )
            } else {
                package_verdict(&session.db, session.ws, p.input, build).clone()
            };
            (p.name.clone(), verdict)
        })
        .collect();
    // A matter is checked at its own dates.
    let cases: Vec<CaseCheck> = inputs
        .iter()
        .take_while(|_| !cancelled())
        .map(|c| {
            let mut check = case_check(&session.db, session.ws, *c, build).clone();
            crate::matter::upgrade_lock(&session, &mut check);
            if fixture(&session, &check.path) {
                outcomes(&mut check.diagnostics);
            }
            check
        })
        .collect();
    let worlds: Vec<WorldCheck> = session
        .worlds
        .iter()
        .take_while(|_| !cancelled())
        .map(|w| check_world(&session.db, session.ws, &w.path, &w.text, profile))
        .collect();
    let complete = verdicts.len() == session.packages.len()
        && cases.len() == inputs.len()
        && worlds.len() == session.worlds.len();
    let mut diagnostics: Vec<Diagnostic> = verdicts
        .iter()
        .flat_map(|(_, v)| v.diagnostics.iter().cloned())
        .chain(cases.iter().flat_map(|c| c.diagnostics.iter().cloned()))
        .chain(worlds.iter().flat_map(|w| w.diagnostics.iter().cloned()))
        .collect();
    if profile == Profile::Strict {
        dependency_holes(&session, &mut diagnostics);
    }
    source_pin_diagnostics(&session, &mut diagnostics);
    super::vendor::retarget(&session.packages, &mut diagnostics);
    let parts: Vec<&[Finding]> = verdicts
        .iter()
        .map(|(_, v)| v.findings.as_slice())
        .chain(cases.iter().map(|c| c.findings.as_slice()))
        .collect();
    let findings = crate::report::merge(&parts);
    Checked {
        session,
        verdicts,
        cases,
        worlds,
        diagnostics,
        findings,
        profile,
        complete,
    }
}

/// Check every pinned vendored path against the bytes loaded from disk and
/// the corpus this binary will use. Hashes are memoized by path for this run.
fn source_pin_diagnostics(session: &Session, diagnostics: &mut Vec<Diagnostic>) {
    use std::collections::BTreeMap;

    let embedded = litigate_engine::cite::SourceCorpus::embedded_ref();
    let override_active = std::env::var_os("LITGRAPH_SOURCES").is_some();
    let mut disk_hashes: BTreeMap<String, Option<String>> = BTreeMap::new();
    let mut affected_paths = BTreeSet::new();
    for loaded in &session.packages {
        let Some(package) = loaded.check.package.as_ref() else {
            continue;
        };
        let input = loaded.input;
        for binding in &package.manifest.sources {
            let (Some(path), Some(pinned)) =
                (binding.path.as_deref(), binding.path_sha256.as_deref())
            else {
                continue;
            };
            if !path.starts_with("sources/") {
                continue;
            }
            let actual = disk_hashes
                .entry(path.to_owned())
                .or_insert_with(|| input.texts(&session.db).get(path).cloned().flatten())
                .clone();
            let Some(actual) = actual else { continue };
            let embedded_hash = embedded.digest(path).map(str::to_owned);
            let stale_pin = pinned != actual;
            let skew = !override_active && embedded_hash.as_deref() != Some(actual.as_str());
            if !stale_pin && !skew {
                continue;
            }
            affected_paths.insert(path.to_owned());
            let manifest_path = input.manifest_path(&session.db);
            let manifest_text = input.manifest_text(&session.db);
            let Some((span, pin_span)) =
                source_pin_spans(&manifest_path, manifest_text, &binding.id, pinned)
            else {
                continue;
            };
            let skew_text = if skew {
                format!(
                    "; embedded corpus has {} ({}), and that is what cite checks use; rebuild `litigate` from this tree or set `LITGRAPH_SOURCES` to this tree's `sources/`",
                    embedded_hash.as_deref().unwrap_or("no file"),
                    if embedded_hash.is_some() { "sha256" } else { "no matching path" }
                )
            } else {
                String::new()
            };
            let message = if stale_pin {
                format!(
                    "package `{}` source `{}` path `{path}` has path_sha256 {pinned}, but the file on disk hashes to {actual}{skew_text}",
                    package.manifest.name, binding.id
                )
            } else {
                format!(
                    "package `{}` source `{}` path `{path}` matches path_sha256 {pinned} on disk, but the embedded corpus has {}{skew_text}",
                    package.manifest.name,
                    binding.id,
                    embedded_hash.as_deref().unwrap_or("no matching file")
                )
            };
            let mut diagnostic = Diagnostic::error(DiagnosticCode::UnboundCite, span, message);
            if stale_pin {
                diagnostic.fixes.push(Fix::single(
                    pin_span,
                    actual,
                    Applicability::MachineApplicable,
                ));
            }
            diagnostics.push(diagnostic);
        }
    }
    // One source-level diagnostic explains a bad pin or binary/tree skew;
    // per-citation L1010s for the same path add no information.
    diagnostics.retain(|diagnostic| {
        diagnostic.code != DiagnosticCode::UnboundCite
            || diagnostic.message.starts_with("package `")
            || !affected_paths
                .iter()
                .any(|path| diagnostic.message.contains(path))
    });
}

/// Locate a source binding's `path_sha256` token in its manifest. The parser
/// already validates this TOML; this only supplies a stable edit range.
fn source_pin_spans(
    manifest_path: &str,
    manifest: &str,
    source_id: &str,
    pinned: &str,
) -> Option<(Span, Span)> {
    let header = format!("[sources.{source_id}]");
    let start = manifest.find(&header)?;
    let tail = &manifest[start..];
    let pin_at = tail.find(pinned)? + start;
    let line_start = manifest[..pin_at].rfind('\n').map_or(0, |i| i + 1);
    let line_end = manifest[pin_at..]
        .find('\n')
        .map_or(manifest.len(), |i| pin_at + i);
    let line = u32::try_from(
        manifest[..line_start]
            .bytes()
            .filter(|b| *b == b'\n')
            .count(),
    )
    .ok()?
    .checked_add(1)?;
    let col = u32::try_from(manifest[line_start..pin_at].chars().count())
        .ok()?
        .checked_add(1)?;
    let end_col = col.checked_add(u32::try_from(pinned.chars().count()).ok()?)?;
    let line_end_col = u32::try_from(manifest[line_start..line_end].chars().count())
        .ok()?
        .checked_add(1)?;
    let line_span = Span {
        file: FileId(manifest_path.to_owned()),
        line,
        col: 1,
        end_line: line,
        end_col: line_end_col,
    };
    let pin_span = Span {
        file: FileId(manifest_path.to_owned()),
        line,
        col,
        end_line: line,
        end_col,
    };
    Some((line_span, pin_span))
}

/// Keep strict-profile holes in dependency packages fatal only when a
/// declaration is reachable from a package named by the user. HIR's
/// resolved references are fully qualified strings; walking the serialized
/// HIR fields includes references nested in expressions, rules, and package
/// declarations without treating labels or prose as references.
#[allow(clippy::too_many_lines)] // Keep the dependency-hole closure in one auditable pass.
fn dependency_holes(session: &Session, diagnostics: &mut Vec<Diagnostic>) {
    use litigate_sema::hir::{Declaration as D, Package};

    fn meta(d: &D) -> Option<&litigate_sema::hir::Metadata> {
        match d {
            D::Node(x) => Some(&x.meta),
            D::Deadline(x) => Some(&x.meta),
            D::Counting(x) => Some(&x.meta),
            D::Calendar(x) => Some(&x.meta),
            D::Rule(x) => Some(&x.meta),
            D::Default(x) => Some(&x.meta),
            D::Exception(x) => Some(&x.meta),
            D::Transition(x) => Some(&x.meta),
            D::Grant(x) => Some(&x.meta),
            D::Limit(x) => Some(&x.meta),
            D::Recharacterize(x) => Some(&x.meta),
            D::Invalidate(x) => Some(&x.meta),
            D::Calibration(x) => Some(&x.meta),
            D::TypeAlias(_)
            | D::Enum(_)
            | D::Event(_)
            | D::Test(_)
            | D::Authority(_)
            | D::Link(_)
            | D::Instance(_) => None,
        }
    }
    fn declarations(p: &Package) -> Vec<&D> {
        p.modules
            .iter()
            .flat_map(|m| m.declarations.iter())
            .collect()
    }
    // Tests describe expectations about rules; their references are not
    // dependencies of those rules and must never seed the closure.
    fn dependency_id(d: &D) -> Option<&str> {
        match d {
            D::Test(_) => None,
            D::Node(x) => Some(&x.meta.id),
            D::Deadline(x) => Some(&x.meta.id),
            D::Counting(x) => Some(&x.meta.id),
            D::Calendar(x) => Some(&x.meta.id),
            D::Rule(x) => Some(&x.meta.id),
            D::Default(x) => Some(&x.meta.id),
            D::Exception(x) => Some(&x.meta.id),
            D::Transition(x) => Some(&x.meta.id),
            D::Grant(x) => Some(&x.meta.id),
            D::Limit(x) => Some(&x.meta.id),
            D::Recharacterize(x) => Some(&x.meta.id),
            D::Invalidate(x) => Some(&x.meta.id),
            D::Calibration(x) => Some(&x.meta.id),
            D::TypeAlias(x) => Some(&x.name),
            D::Enum(x) => Some(&x.name),
            D::Event(x) => Some(&x.name),
            D::Authority(x) => Some(&x.name),
            D::Link(x) => x.edge.id.as_deref(),
            D::Instance(x) => Some(&x.name),
        }
    }
    fn string_leaves(value: &serde_json::Value, out: &mut BTreeSet<String>) {
        match value {
            serde_json::Value::String(text) => {
                out.insert(text.clone());
            }
            serde_json::Value::Array(items) => {
                for item in items {
                    string_leaves(item, out);
                }
            }
            serde_json::Value::Object(fields) => {
                for item in fields.values() {
                    string_leaves(item, out);
                }
            }
            _ => {}
        }
    }
    let packages: BTreeMap<&str, &Package> = session
        .packages
        .iter()
        .filter_map(|p| p.check.package.as_ref().map(|h| (p.name.as_str(), h)))
        .collect();
    let roots: Vec<&Package> = session
        .packages
        .iter()
        .filter(|p| p.requested)
        .filter_map(|p| p.check.package.as_ref())
        .collect();
    // Serialize declarations once. Versions sharing a qualified id contribute
    // the union of their references to the transitive walk.
    let mut declaration_refs: BTreeMap<String, (String, BTreeSet<String>)> = BTreeMap::new();
    let mut local_ids: BTreeMap<(String, String), String> = BTreeMap::new();
    for (package_name, package) in &packages {
        for declaration in declarations(package) {
            let Some(id) = dependency_id(declaration) else {
                continue;
            };
            let key = format!("{}::{}", package.manifest.pack_id, id);
            let mut refs = BTreeSet::new();
            if let Ok(value) = serde_json::to_value(declaration) {
                string_leaves(&value, &mut refs);
            }
            local_ids.insert(((*package_name).to_owned(), id.to_owned()), key.clone());
            declaration_refs
                .entry(key)
                .and_modify(|(_, existing)| existing.extend(refs.iter().cloned()))
                .or_insert_with(|| ((*package_name).to_owned(), refs));
        }
    }
    let mut reachable = BTreeSet::new();
    for root in roots {
        let mut pending = Vec::new();
        for declaration in declarations(root) {
            let Some(id) = dependency_id(declaration) else {
                continue;
            };
            let key = format!("{}::{}", root.manifest.pack_id, id);
            if let Some((_, refs)) = declaration_refs.get(&key) {
                pending.extend(
                    refs.iter()
                        .cloned()
                        .map(|reference| (root.manifest.name.clone(), reference)),
                );
            }
        }
        let mut configured_packages = vec![root];
        // A package's named counting rule often lives in a dependency (for
        // example, frcp's `frcp6` lives in core). Seed that declaration in
        // its owning package so its calendar references enter the closure.
        if let Some(counting) = root.manifest.counting.as_deref() {
            if let Some(owner) = packages.values().find(|package| {
                declarations(package).into_iter().any(|declaration| {
                    meta(declaration).is_some_and(|m| {
                        m.id == counting
                            || format!("{}::{}", package.manifest.pack_id, m.id) == counting
                    })
                })
            }) {
                if !std::ptr::eq(*owner, root) {
                    configured_packages.push(owner);
                }
            }
        }
        configured_packages.extend(root.manifest.recounts.iter().filter_map(|name| {
            packages
                .values()
                .find(|p| p.manifest.name == *name)
                .copied()
        }));
        for configured in configured_packages {
            let Some(counting) = configured.manifest.counting.as_deref() else {
                continue;
            };
            for declaration in declarations(configured) {
                let Some(id) = dependency_id(declaration) else {
                    continue;
                };
                let key = format!("{}::{}", configured.manifest.pack_id, id);
                if id == counting || key == counting {
                    reachable.insert(key);
                    let key = format!("{}::{}", configured.manifest.pack_id, id);
                    if let Some((_, refs)) = declaration_refs.get(&key) {
                        pending.extend(
                            refs.iter()
                                .cloned()
                                .map(|reference| (configured.manifest.name.clone(), reference)),
                        );
                    }
                }
            }
        }
        let mut seen = BTreeSet::new();
        while let Some((source_package, reference)) = pending.pop() {
            let target = if declaration_refs.contains_key(&reference) {
                Some(reference)
            } else {
                local_ids.get(&(source_package, reference)).cloned()
            };
            let Some(key) = target else { continue };
            if !seen.insert(key.clone()) {
                continue;
            }
            reachable.insert(key.clone());
            if let Some((package, refs)) = declaration_refs.get(&key) {
                pending.extend(
                    refs.iter()
                        .cloned()
                        .map(|reference| (package.clone(), reference)),
                );
            }
        }
    }
    let mut omitted: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (index, diagnostic) in diagnostics.iter_mut().enumerate() {
        if diagnostic.code != DiagnosticCode::HoleInStrict {
            continue;
        }
        let Some(owner) = session
            .packages
            .iter()
            .find(|p| !p.requested && p.files.iter().any(|(f, _)| f == &diagnostic.span.file.0))
        else {
            continue;
        };
        let package = packages.get(owner.name.as_str());
        let is_reachable = package.is_some_and(|p| {
            declarations(p).into_iter().any(|d| {
                let Some(m) = meta(d) else { return false };
                let Some(s) = m.span.as_ref() else {
                    return false;
                };
                s.file == diagnostic.span.file
                    && (s.line < diagnostic.span.line
                        || (s.line == diagnostic.span.line && s.col <= diagnostic.span.col))
                    && (s.end_line > diagnostic.span.end_line
                        || (s.end_line == diagnostic.span.end_line
                            && s.end_col >= diagnostic.span.end_col))
                    && reachable.contains(&format!("{}::{}", p.manifest.pack_id, m.id))
            })
        });
        if !is_reachable {
            omitted.entry(owner.name.clone()).or_default().push(index);
        }
    }
    let mut removed = BTreeSet::new();
    for (package, indexes) in omitted {
        let count = indexes.len();
        let Some((&first, rest)) = indexes.split_first() else {
            continue;
        };
        diagnostics[first].severity = Severity::Info;
        diagnostics[first].message = format!("unreachable dependency holes in `{package}`");
        diagnostics[first].notes.push(Note {
            message: format!("{count} holes in declarations unreachable from the requested package; omitted from its strict verdict"),
        });
        removed.extend(rest.iter().copied());
    }
    if !removed.is_empty() {
        let old = std::mem::take(diagnostics);
        *diagnostics = old
            .into_iter()
            .enumerate()
            .filter_map(|(i, d)| (!removed.contains(&i)).then_some(d))
            .collect();
    }
}

/// Whether the matter at `path` is a world fixture (SPEC §5.8): a `.world`
/// file the session checks sits in its directory, so the matter is replayed
/// under the worlds (`litigate worlds`), not only in the actual world.
fn fixture(session: &Session, path: &str) -> bool {
    let dir = std::path::Path::new(path).parent();
    session
        .worlds
        .iter()
        .any(|w| std::path::Path::new(&w.path).parent() == dir)
}

/// A world fixture's late filings are outcomes, never errors (SPEC §5.8):
/// its actual world lacks the law its worlds bind (a circuit's precedent),
/// so each L4002 there is a warning, the outcome `worlds` judges per world.
fn outcomes(diagnostics: &mut [Diagnostic]) {
    for d in diagnostics
        .iter_mut()
        .filter(|d| d.code == DiagnosticCode::DeadlineMissed && d.severity == Severity::Error)
    {
        d.severity = Severity::Warning;
        d.notes.push(Note {
            message: "a world fixture (a `.world` file beside it): a late filing is an outcome; `litigate worlds` judges it in each world".to_owned(),
        });
    }
}

impl Checked {
    /// Repository-relative paths of every checked file.
    #[must_use]
    pub fn files(&self) -> Vec<String> {
        let mut files: Vec<String> = self
            .session
            .packages
            .iter()
            .flat_map(|p| p.files.iter().map(|(f, _)| f.clone()))
            .chain(self.cases.iter().map(|c| c.path.clone()))
            .chain(self.worlds.iter().map(|w| w.path.clone()))
            .collect();
        files.sort();
        files
    }

    /// Per-package summary.
    #[must_use]
    pub fn packages(&self) -> Vec<Value> {
        self.session
            .packages
            .iter()
            .zip(&self.verdicts)
            .map(|(p, (_, v))| {
                // Written holes and the cites no amendment history proves.
                let holes = v.findings.iter().find(|f| f.name == Obligation::Holes);
                json!({
                    "name": p.name,
                    "version": p.version,
                    "digest": p.digest,
                    "root": p.root,
                    "requested": p.requested,
                    "holes": holes.map_or(0, |f| f.count),
                    "tests": {
                        "passed": v.tests.iter().filter(|t| t.passed).count(),
                        "failed": v.tests.iter().filter(|t| !t.passed).count(),
                    },
                })
            })
            .collect()
    }

    /// Judges every matter's deadlines at `today` ([`crate::matter::judge_at`]),
    /// reporting the L4016 of each whose `as_of` is older.
    pub fn judge_at(&mut self, today: &str) {
        for case in &mut self.cases {
            let text = self
                .session
                .cases
                .iter()
                .find(|c| c.path == case.path)
                .map_or("", |c| c.text.as_str());
            self.diagnostics
                .extend(crate::matter::judge_at(case, text, today));
        }
    }

    /// Envelope provenance.
    #[must_use]
    pub fn provenance(&self, as_of: Option<&str>, known_at: Option<&str>) -> Value {
        crate::prov::session(&self.session.packages, as_of, known_at)
    }
}

/// A world's summary: its file, the world, and its layers as it has them.
fn world_summary(w: &WorldCheck) -> Value {
    json!({"path": w.path, "world": w.world, "layers": w.layers})
}

/// `litigate check` over `src`, judging deadlines at `today` when given.
pub fn check(
    src: &dyn Source,
    paths: &[String],
    profile: Profile,
    (dates, today): (&Dates, Option<&str>),
    lib: &[String],
    verbose: bool,
) -> Outcome {
    let today = today.map(super::timeline::date).transpose()?;
    let mut c = checked_in(src, paths, profile, &dates.options(), lib)?;
    if let Some(today) = today {
        c.judge_at(today);
    }
    let profile_name = match profile {
        Profile::Dev => "dev",
        Profile::Strict => "strict",
    };
    let own_files: std::collections::BTreeSet<String> = c
        .session
        .packages
        .iter()
        .filter(|p| p.requested)
        .flat_map(|p| p.files.iter().map(|(f, _)| f.clone()))
        .chain(c.session.cases.iter().map(|case| case.path.clone()))
        .chain(c.session.worlds.iter().map(|w| w.path.clone()))
        .collect();
    let package_by_file: std::collections::BTreeMap<String, String> = c
        .session
        .packages
        .iter()
        .flat_map(|p| {
            p.files
                .iter()
                .map(move |(f, _)| (f.clone(), p.name.clone()))
        })
        .collect();
    let mut result = json!({
        "profile": profile_name,
        "files": c.files(),
        "obligations": c.findings,
        "packages": c.packages(),
        "cases": c.cases.iter().map(CaseCheck::summary).collect::<Vec<_>>(),
    });
    if !c.worlds.is_empty() {
        result["worlds"] = c.worlds.iter().map(world_summary).collect();
    }
    Ok(Report {
        result,
        provenance: c.provenance(dates.as_of.as_deref(), dates.known_at.as_deref()),
        sources: c.session.sources(),
        diagnostics: c.diagnostics,
        check_render: Some(crate::out::CheckRender {
            verbose,
            own_files,
            package_by_file,
        }),
        profile: Some(profile),
        text: None,
        failed: false,
    })
}
