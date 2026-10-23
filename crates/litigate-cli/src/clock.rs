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
use litigate_lower::{Lowerer, Options};
use litigate_sema::hir::{self, DayUnit, Declaration, Expr};

/// One lowered version of a declared counting rule.
#[derive(Debug, Clone)]
struct Version {
    /// Package name.
    package: String,
    /// The declaration.
    decl: hir::Counting,
    /// Its clock fragment, or why it does not lower.
    fragment: Result<ClockLibrary, String>,
}

/// Every version of every counting rule the loaded packages declare.
#[derive(Debug, Clone, Default)]
pub struct Clocks {
    versions: Vec<Version>,
}

impl Clocks {
    /// Lowers every version of every counting rule of `packages`.
    #[must_use]
    pub fn new(packages: &[&hir::Package]) -> Self {
        let lowerer = Lowerer::new(packages);
        let versions = packages
            .iter()
            .filter(|p| !p.manifest.links)
            .flat_map(|p| {
                hirx::declarations(p).filter_map(move |d| match d {
                    Declaration::Counting(c) => Some((*p, c)),
                    _ => None,
                })
            })
            .map(|(p, c)| {
                let options = Options {
                    as_of: Some(c.meta.effective.from.clone()),
                    known_at: None,
                };
                let fragment = match lowerer.clock_at(p, &c.meta.id, &options) {
                    Ok(Some(library)) => Ok(library),
                    Ok(None) => Err(format!("counting rule `{}` does not lower", c.meta.id)),
                    Err(diagnostics) => Err(diagnostics.first().map_or_else(String::new, |d| {
                        format!("{}: {}", d.code.as_str(), d.message)
                    })),
                };
                Version {
                    package: p.manifest.name.clone(),
                    decl: c.clone(),
                    fragment,
                }
            })
            .collect();
        Self { versions }
    }

    /// The version of counting rule `rule` in force on `date`, preferring
    /// one `pkg` declares; `None` when no loaded package declares `rule`
    /// (the shared library has it, or nothing does).
    ///
    /// # Errors
    /// When packages declare `rule` but no version (or more than one) is
    /// in force on `date`.
    fn version(&self, pkg: &str, rule: &str, date: &str) -> Result<Option<&Version>, String> {
        let declared: Vec<&Version> = self
            .versions
            .iter()
            .filter(|v| v.decl.meta.id == rule)
            .collect();
        let owner = declared
            .iter()
            .find(|v| v.package == pkg)
            .or_else(|| declared.first())
            .map(|v| v.package.clone());
        let Some(owner) = owner else {
            return Ok(None);
        };
        let live: Vec<&Version> = declared
            .into_iter()
            .filter(|v| v.package == owner && hirx::in_force(&v.decl.meta, date, None))
            .collect();
        match live.as_slice() {
            [one] => Ok(Some(one)),
            [] => Err(format!(
                "no version of counting rule `{rule}` ({owner}) is in force on {date}"
            )),
            _ => Err(format!(
                "{} versions of counting rule `{rule}` ({owner}) are in force on {date}",
                live.len()
            )),
        }
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
    /// Counting rule id.
    pub rule: String,
    /// The trigger date.
    pub trigger: String,
    /// Service method name applied, if any.
    pub method: Option<String>,
    /// The engine's result.
    pub computed: ComputedDeadline,
}

/// Computes deadline `d` of `pkg` triggered on `trigger` by `event`,
/// under the version of its counting rule in force on `trigger`.
///
/// # Errors
/// A message for an invalid date, business-day units (no engine lowering),
/// an unknown counting rule or one with no version in force, or an engine
/// refusal.
pub fn due(
    clocks: &Clocks,
    pkg: &hir::Package,
    d: &hir::Deadline,
    trigger: &str,
    event: Option<&Expr>,
) -> Result<Due, String> {
    let rule = hirx::local(&d.counting).to_owned();
    let unit = match d.unit {
        DayUnit::Calendar => Unit::Calendar,
        DayUnit::Court => Unit::Court,
        DayUnit::Business => {
            return Err(format!(
                "`{}` counts business days, which have no engine lowering",
                d.meta.id
            ))
        }
    };
    let version = clocks.version(&pkg.manifest.name, &rule, trigger)?;
    let method = event.and_then(|e| method_of(e, version.map(|v| &v.decl)));
    let request = DayDeadline {
        trigger: Date::parse_iso(trigger).map_err(|e| e.to_string())?,
        days: i64::try_from(d.amount).map_err(|e| e.to_string())?,
        direction: match d.direction {
            hir::Direction::Forward => Direction::Forward,
            hir::Direction::Backward => Direction::Backward,
        },
        unit,
        service_method: method.as_ref().map(|(_, m)| *m),
    };
    let library = match version {
        Some(v) => v.fragment.as_ref().map_err(Clone::clone)?,
        None => &ClockLibrary::embedded().0,
    };
    let clock = library.clock(&rule).map_err(|e| e.to_string())?;
    let computed =
        compute_due_date(&request, &clock, &ClockConfig::default()).map_err(|e| e.to_string())?;
    Ok(Due {
        rule,
        trigger: trigger.to_owned(),
        method: method.map(|(name, _)| name),
        computed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::hir_with;
    use litigate_sema::hir::{ExprKind, Type, Value};

    /// A version recorded after the package's corpus date (so it never
    /// lowers) and one whose cite binds no source URL (L6001).
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
    const PKG: &str = "frcp-rule12-example";

    fn fixture() -> (hir::Package, Clocks) {
        let pkg = hir_with(EXTRA);
        let clocks = Clocks::new(&[&pkg]);
        (pkg, clocks)
    }

    fn answer_period(pkg: &hir::Package) -> hir::Deadline {
        hirx::deadlines(pkg, "answer-period")
            .next()
            .unwrap()
            .clone()
    }

    #[test]
    fn every_declared_version_lowers_or_says_why() {
        let (_, clocks) = fixture();
        let found: Vec<(&str, Result<usize, &str>)> = clocks
            .versions
            .iter()
            .map(|v| {
                (
                    v.decl.meta.id.as_str(),
                    v.fragment
                        .as_ref()
                        .map(|l| l.counting_rules.len())
                        .map_err(String::as_str),
                )
            })
            .collect();
        assert_eq!(found.len(), 3, "{found:?}");
        assert_eq!(found[0], ("frcp6", Ok(1)));
        assert_eq!(
            found[1],
            ("late", Err("counting rule `late` does not lower"))
        );
        assert_eq!(found[2].0, "nourl");
        assert!(
            found[2]
                .1
                .is_err_and(|e| e.starts_with("L6001: counting rule `nourl` needs a `cite`")),
            "{found:?}"
        );
    }

    #[test]
    fn version_selection_by_owner_and_date() {
        let (_, mut clocks) = fixture();
        assert!(matches!(
            Clocks::default().version(PKG, "frcp6", "2025-01-01"),
            Ok(None)
        ));
        let other = clocks.version("another-package", "frcp6", "2025-01-01");
        assert_eq!(
            other.map(|v| v.map(|v| v.package.clone())),
            Ok(Some(PKG.to_owned()))
        );
        assert_eq!(
            clocks.version(PKG, "frcp6", "2000-01-01").map(|_| ()),
            Err(format!(
                "no version of counting rule `frcp6` ({PKG}) is in force on 2000-01-01"
            ))
        );
        clocks.versions.push(clocks.versions[0].clone());
        assert_eq!(
            clocks.version(PKG, "frcp6", "2025-01-01").map(|_| ()),
            Err(format!(
                "2 versions of counting rule `frcp6` ({PKG}) are in force on 2025-01-01"
            ))
        );
    }

    #[test]
    fn only_calls_carry_a_service_method() {
        let literal = Expr {
            ty: Type::Unit,
            kind: ExprKind::Literal {
                value: Value::Variant("Mail".into()),
            },
            span: None,
        };
        assert_eq!(method_of(&literal, None), None);
    }

    #[test]
    fn due_counts_under_the_version_in_force() {
        let (pkg, clocks) = fixture();
        let d = answer_period(&pkg);
        let date = |d: &hir::Deadline| {
            due(&clocks, &pkg, d, "2025-03-03", None).map(|d| d.computed.due_date)
        };
        assert_eq!(date(&d), Ok("2025-03-24".to_owned()));
        let mut court = d.clone();
        court.unit = DayUnit::Court;
        assert_eq!(date(&court), Ok("2025-04-01".to_owned()));
        let mut before = d;
        before.direction = hir::Direction::Backward;
        assert_eq!(date(&before), Ok("2025-02-10".to_owned()));
    }

    #[test]
    fn due_explains_every_refusal() {
        let (pkg, clocks) = fixture();
        let base = answer_period(&pkg);
        let with = |f: &dyn Fn(&mut hir::Deadline)| {
            let mut d = base.clone();
            f(&mut d);
            d
        };
        let refusal = |clocks: &Clocks, d: &hir::Deadline, trigger: &str| {
            due(clocks, &pkg, d, trigger, None).unwrap_err()
        };
        let business = with(&|d| d.unit = DayUnit::Business);
        assert_eq!(
            refusal(&clocks, &business, "2025-03-03"),
            "`answer-period` counts business days, which have no engine lowering"
        );
        assert!(refusal(&clocks, &base, "2000-01-01").starts_with("no version of counting rule"));
        assert!(!refusal(&clocks, &base, "2025-13-45").is_empty());
        let huge = with(&|d| d.amount = u64::MAX);
        assert!(!refusal(&clocks, &huge, "2025-03-03").is_empty());
        let unlowered = with(&|d| d.counting = "nourl".into());
        assert!(refusal(&clocks, &unlowered, "2025-03-03").starts_with("L6001"));
        let unknown = with(&|d| d.counting = "no-such-rule".into());
        assert!(!refusal(&Clocks::default(), &unknown, "2025-03-03").is_empty());
        let too_short = with(&|d| {
            d.counting = "itc210".into();
            d.amount = 0;
        });
        assert_eq!(
            refusal(&Clocks::default(), &too_short, "2025-03-03"),
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
