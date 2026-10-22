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
