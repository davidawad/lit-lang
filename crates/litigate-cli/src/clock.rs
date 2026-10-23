//! Deadline computation through litgraph's own clock interpreter, over the
//! counting rules the loaded packages lower to (each version lowered at
//! its own `from` date), else litgraph's shared clock library. One code
//! path serves `test`, case replay, and `explain-value`, so they cannot
//! disagree with the engine.
use crate::hirx;
use litgraph::clock::{
    compute_due_date, ClockConfig, ClockLibrary, ComputedDeadline, Date, DayDeadline, Direction,
    ServiceMethod, Unit,
};
use litigate_lower::Lowerer;
use litigate_sema::hir::{self, DayUnit, Declaration, Expr};
use litigate_sema::temporal::{self, Key, MatterEvent, World};
use std::collections::BTreeMap;

/// Every version of every declaration the loaded packages declare, and
/// the clock fragment of each counting version. Versions are selected by
/// sema's one query (`temporal::select`, through `temporal::governing`),
/// as lowering and case replay select them.
#[derive(Debug, Default)]
pub struct Clocks<'a> {
    world: World<'a>,
    /// Each counting version's fragment (by declaration index), or why it
    /// does not lower.
    fragments: BTreeMap<usize, Result<ClockLibrary, String>>,
}

impl<'a> Clocks<'a> {
    /// Lowers every version of every counting rule of `packages`, each at
    /// its own `from` date.
    #[must_use]
    pub fn new(packages: &[&'a hir::Package]) -> Self {
        let lowerer = Lowerer::new(packages);
        let world = World::from_packages(packages);
        let fragments = world
            .decls
            .iter()
            .enumerate()
            .filter_map(|(i, d)| match d.decl {
                Declaration::Counting(c) => Some((i, packages[d.pack], c)),
                _ => None,
            })
            .map(|(i, p, c)| {
                let fragment = match lowerer.clock_version(p, c) {
                    Ok(Some(library)) => Ok(library),
                    Ok(None) => Err(format!("counting rule `{}` does not lower", c.meta.id)),
                    Err(diagnostics) => Err(diagnostics.first().map_or_else(String::new, |d| {
                        format!("{}: {}", d.code.as_str(), d.message)
                    })),
                };
                (i, fragment)
            })
            .collect();
        Self { world, fragments }
    }

    /// The version of `key` governing `events[at]` (`at` indexes
    /// `events`) among those recorded by `known_at`
    /// (`temporal::governing`: the version in force on the event's date,
    /// unless a transition decides); `None` when no loaded package
    /// declares `key` (the shared library has it, or nothing does).
    ///
    /// # Errors
    /// When packages declare `key` but no version governs the event.
    fn governing(
        &self,
        what: &str,
        key: &Key,
        events: &[MatterEvent<'_>],
        at: usize,
        known_at: Option<&str>,
    ) -> Result<Option<usize>, String> {
        if self.world.versions(key).versions.is_empty() {
            return Ok(None);
        }
        temporal::governing(&self.world, key, events, at, known_at)
            .map(|g| Some(g.version))
            .map_err(|_| {
                let recorded = known_at.map_or_else(String::new, |k| format!(" recorded by {k}"));
                format!(
                    "no version of {what} `{}`{recorded} governs {}",
                    key.display(""),
                    events[at].date
                )
            })
    }
}

/// The engine's service method for a `.lit` method name (`Mail`,
/// `OtherConsented`): its kebab-case spelling in the engine's closed set.
#[must_use]
pub fn engine_method(name: &str) -> Option<ServiceMethod> {
    let kebab = name
        .chars()
        .enumerate()
        .fold(String::new(), |mut s, (i, c)| {
            if c.is_ascii_uppercase() && i > 0 {
                s.push('-');
            }
            s.push(c.to_ascii_lowercase());
            s
        });
    serde_json::from_value(serde_json::Value::String(kebab)).ok()
}

/// The service method an event carries: an argument naming one of the
/// counting rule's `service_extension` keys (or, for a shared-library
/// rule, any engine service method).
#[must_use]
pub fn method_of(
    event: &Expr,
    counting: Option<&hir::Counting>,
) -> Option<(String, ServiceMethod)> {
    let (_, args) = hirx::event_parts(event)?;
    args.into_iter().find_map(|arg| {
        let declared = counting.is_none_or(|c| c.service_extensions.contains_key(&arg));
        declared
            .then(|| engine_method(&arg))
            .flatten()
            .map(|m| (arg, m))
    })
}

/// A computed due date with its trace.
#[derive(Debug, Clone)]
pub struct Due {
    /// The deadline version counted (`id@from`).
    pub version: String,
    /// Counting rule id.
    pub rule: String,
    /// The selected counting version's `cite` (none for a shared-library
    /// rule).
    pub cite: Option<String>,
    /// The trigger date.
    pub trigger: String,
    /// Service method name applied, if any.
    pub method: Option<String>,
    /// The engine's result.
    pub computed: ComputedDeadline,
}

/// Computes deadline `name` of `pkg` triggered by `events[at]`, under the
/// versions of the deadline and of its counting rule governing that event
/// among those recorded by `known_at` (`None`: the latest corpus). A
/// date-only query is a one-event history; `at` indexes `events`.
///
/// # Errors
/// A message for an unknown deadline or one with no version governing the
/// event, an invalid date, business-day units (no engine lowering), a
/// counting rule with no version governing the event, or an engine
/// refusal.
pub fn due(
    clocks: &Clocks<'_>,
    pkg: &hir::Package,
    name: &str,
    events: &[MatterEvent<'_>],
    at: usize,
    known_at: Option<&str>,
) -> Result<Due, String> {
    let key = Key {
        pack: pkg.manifest.pack_id.clone(),
        name: hirx::local(name).to_owned(),
    };
    let d = clocks
        .governing("deadline", &key, events, at, known_at)?
        .and_then(|i| match clocks.world.decls[i].decl {
            Declaration::Deadline(d) => Some(d),
            _ => None,
        })
        .ok_or_else(|| format!("`{name}` is not a deadline of `{}`", pkg.manifest.name))?;
    let trigger = &events[at];
    let rule = hirx::local(&d.counting).to_owned();
    let unit = match d.unit {
        DayUnit::Calendar => Unit::Calendar,
        DayUnit::Court => Unit::Court,
        DayUnit::Week => Unit::Week,
        DayUnit::Month => Unit::Month,
        DayUnit::Year => Unit::Year,
        DayUnit::Business => {
            return Err(format!(
                "`{}` counts business days, which have no engine lowering",
                d.meta.id
            ))
        }
    };
    let counting_key = clocks.world.target(&pkg.manifest.pack_id, &d.counting).key;
    let version = clocks
        .governing("counting rule", &counting_key, events, at, known_at)?
        .and_then(|i| match clocks.world.decls[i].decl {
            Declaration::Counting(c) => Some((i, c)),
            _ => None,
        });
    let method = method_of(trigger.action, version.map(|(_, c)| c));
    let request = DayDeadline {
        trigger: Date::parse_iso(trigger.date).map_err(|e| e.to_string())?,
        days: i64::try_from(d.amount).map_err(|e| e.to_string())?,
        direction: match d.direction {
            hir::Direction::Forward => Direction::Forward,
            hir::Direction::Backward => Direction::Backward,
        },
        unit,
        service_method: method.as_ref().map(|(_, m)| *m),
    };
    let library = match version {
        Some((i, _)) => clocks.fragments[&i].as_ref().map_err(Clone::clone)?,
        None => &ClockLibrary::embedded().0,
    };
    let clock = library.clock(&rule).map_err(|e| e.to_string())?;
    let computed =
        compute_due_date(&request, &clock, &ClockConfig::default()).map_err(|e| e.to_string())?;
    Ok(Due {
        version: format!("{}@{}", d.meta.id, d.meta.effective.from),
        rule,
        cite: version.and_then(|(_, c)| c.meta.cite.clone()),
        trigger: trigger.date.to_owned(),
        method: method.map(|(name, _)| name),
        computed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::hir_with;
    use litigate_sema::hir::{ExprKind, Type, Value};

    /// A version recorded after the package's corpus date and one whose
    /// cite binds no source URL (L6001).
    const EXTRA: &str = "
counting late known_at 2026-01-01 cite FRCP 6(a)
{
  exclude_trigger_day
  include_last_day
  holidays federal-holidays
}

counting nourl cite XYZ 6(a)
{
  exclude_trigger_day
  include_last_day
  holidays federal-holidays
}
";

    fn expr(kind: ExprKind) -> Expr {
        Expr {
            ty: Type::Unit,
            kind,
            span: None,
        }
    }

    /// `Served(complaint, <method>)`.
    fn served(method: &str) -> Expr {
        let variant = |v: &str| {
            expr(ExprKind::Literal {
                value: Value::Variant(v.into()),
            })
        };
        expr(ExprKind::Call {
            callee: "Served".into(),
            args: vec![variant("Document::complaint"), variant(method)],
        })
    }

    /// The example package with `EXTRA`, its `answer-period` changed by
    /// `f`.
    fn package(f: &dyn Fn(&mut hir::Deadline)) -> hir::Package {
        let mut pkg = hir_with(EXTRA);
        for d in pkg.modules.iter_mut().flat_map(|m| &mut m.declarations) {
            if let Declaration::Deadline(d) = d {
                f(d);
            }
        }
        pkg
    }

    /// `answer-period` triggered on `date` (personal service), at
    /// `known_at`.
    fn due_on(
        pkg: &hir::Package,
        name: &str,
        date: &str,
        known_at: Option<&str>,
    ) -> Result<Due, String> {
        let action = served("ServiceMethod::Personal");
        let events = [MatterEvent {
            date,
            action: &action,
        }];
        due(&Clocks::new(&[pkg]), pkg, name, &events, 0, known_at)
    }

    #[test]
    fn every_declared_version_lowers_or_says_why() {
        let mut pkg = hir_with(EXTRA);
        // Two versions of `frcp6`, the first counting rule, in one window:
        // neither is selected at its start, so neither lowers.
        let twice = pkg.modules[0]
            .declarations
            .iter()
            .find(|d| matches!(d, Declaration::Counting(_)))
            .cloned()
            .unwrap();
        pkg.modules[0].declarations.push(twice);
        let clocks = Clocks::new(&[&pkg]);
        let found: Vec<(&str, Result<usize, &str>)> = clocks
            .fragments
            .iter()
            .map(|(i, f)| {
                let id = clocks.world.decls[*i].meta.id.as_str();
                (
                    id,
                    f.as_ref()
                        .map(|l| l.counting_rules.len())
                        .map_err(String::as_str),
                )
            })
            .collect();
        assert_eq!(found.len(), 4, "{found:?}");
        assert_eq!(
            found[0],
            ("frcp6", Err("counting rule `frcp6` does not lower"))
        );
        // Lowered regardless of `known_at`: selection decides whether it
        // applies.
        assert_eq!(found[1], ("late", Ok(1)));
        assert_eq!(found[2].0, "nourl");
        assert!(
            found[2]
                .1
                .is_err_and(|e| e.starts_with("L6001: counting rule `nourl` needs a `cite`")),
            "{found:?}"
        );
    }

    #[test]
    fn only_calls_carry_a_service_method() {
        let literal = expr(ExprKind::Literal {
            value: Value::Variant("Mail".into()),
        });
        assert_eq!(method_of(&literal, None), None);
    }

    #[test]
    fn due_counts_under_the_versions_governing_the_trigger() {
        let date = |pkg: &hir::Package| {
            due_on(pkg, "answer-period", "2025-03-03", None).map(|d| d.computed.due_date)
        };
        let pkg = package(&|_| ());
        let found = due_on(&pkg, "answer-period", "2025-03-03", None).unwrap();
        assert_eq!(found.computed.due_date, "2025-03-24");
        assert_eq!(found.version, "answer-period@2009-12-01");
        assert_eq!(found.cite.as_deref(), Some("FRCP 6(a)"));
        assert_eq!(
            date(&package(&|d| d.unit = DayUnit::Court)),
            Ok("2025-04-01".to_owned())
        );
        let before = package(&|d| d.direction = hir::Direction::Backward);
        assert_eq!(date(&before), Ok("2025-02-10".to_owned()));
        // Mail service adds the counting rule's three days.
        let action = served("ServiceMethod::Mail");
        let events = [MatterEvent {
            date: "2025-03-03",
            action: &action,
        }];
        let mailed = due(
            &Clocks::new(&[&pkg]),
            &pkg,
            "answer-period",
            &events,
            0,
            None,
        )
        .unwrap();
        assert_eq!(
            (mailed.computed.due_date.as_str(), mailed.method.as_deref()),
            ("2025-03-27", Some("Mail"))
        );
    }

    #[test]
    fn known_at_hides_versions_recorded_later() {
        let pkg = package(&|d| d.counting = "late".into());
        assert!(due_on(&pkg, "answer-period", "2025-03-03", None).is_ok());
        assert!(due_on(&pkg, "answer-period", "2025-03-03", Some("2026-01-01")).is_ok());
        assert_eq!(
            due_on(&pkg, "answer-period", "2025-03-03", Some("2025-01-01")).map(|_| ()),
            Err("no version of counting rule `frcp-rule12-example::late` recorded by 2025-01-01 governs 2025-03-03".to_owned())
        );
        assert_eq!(
            due_on(&pkg, "answer-period", "2025-03-03", Some("1900-01-01")).map(|_| ()),
            Err("no version of deadline `frcp-rule12-example::answer-period` recorded by 1900-01-01 governs 2025-03-03".to_owned())
        );
    }

    #[test]
    fn due_explains_every_refusal() {
        let refusal = |f: &dyn Fn(&mut hir::Deadline), name: &str, trigger: &str| {
            due_on(&package(f), name, trigger, None).unwrap_err()
        };
        let none = |_: &mut hir::Deadline| ();
        let name = "frcp-rule12-example";
        assert_eq!(
            refusal(&none, "nope", "2025-03-03"),
            format!("`nope` is not a deadline of `{name}`")
        );
        // A declaration of another kind is not a deadline either.
        assert_eq!(
            refusal(&none, "respond", "2025-03-03"),
            format!("`respond` is not a deadline of `{name}`")
        );
        assert_eq!(
            refusal(&none, "answer-period", "2000-01-01"),
            format!("no version of deadline `{name}::answer-period` governs 2000-01-01")
        );
        assert_eq!(
            refusal(
                &|d| d.unit = DayUnit::Business,
                "answer-period",
                "2025-03-03"
            ),
            "`answer-period` counts business days, which have no engine lowering"
        );
        assert!(!refusal(&none, "answer-period", "2025-13-45").is_empty());
        assert!(!refusal(&|d| d.amount = u64::MAX, "answer-period", "2025-03-03").is_empty());
        assert!(refusal(
            &|d| d.counting = "nourl".into(),
            "answer-period",
            "2025-03-03"
        )
        .starts_with("L6001"));
        // No loaded package declares it: the shared library is asked, and
        // a node is no counting rule.
        for counting in ["no-such-rule", "respond"] {
            let why = refusal(
                &|d| d.counting = counting.into(),
                "answer-period",
                "2025-03-03",
            );
            assert!(!why.is_empty());
        }
        let too_short = |d: &mut hir::Deadline| {
            d.counting = "itc210".into();
            d.amount = 0;
        };
        assert_eq!(
            refusal(&too_short, "answer-period", "2025-03-03"),
            "invalid: 19 CFR 201.14(a) periods must be at least 1 day"
        );
    }

    #[test]
    fn methods_are_kebab_case() {
        assert_eq!(engine_method("Mail"), Some(ServiceMethod::Mail));
        assert_eq!(
            engine_method("OtherConsented"),
            Some(ServiceMethod::OtherConsented)
        );
        assert_eq!(engine_method("Personal"), None);
    }
}
