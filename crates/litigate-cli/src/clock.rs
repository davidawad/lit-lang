//! Deadline computation through litgraph's own clock interpreter, over the
//! counting rule a package lowers to (its clock fragment, else litgraph's
//! shared clock library). One code path serves `test`, case replay, and
//! `explain-value`, so they cannot disagree with the engine.
use crate::hirx;
use litgraph::clock::{
    compute_due_date, ClockConfig, ClockLibrary, ComputedDeadline, Date, DayDeadline, Direction,
    ServiceMethod, Unit,
};
use litigate_lower::{Lowerer, Options};
use litigate_sema::hir::{self, DayUnit, Expr};
use std::collections::BTreeMap;

/// Clock fragments of the loaded packages, by package name.
#[derive(Debug, Clone, Default)]
pub struct Clocks {
    fragments: BTreeMap<String, ClockLibrary>,
}

impl Clocks {
    /// Lowers every package to collect its own counting rules and calendars.
    #[must_use]
    pub fn new(packages: &[&hir::Package]) -> Self {
        let lowerer = Lowerer::new(packages);
        let fragments = packages
            .iter()
            .filter_map(|p| {
                let lowered = lowerer.package(p, &Options::default()).ok()?;
                Some((p.manifest.name.clone(), lowered.clock?.0))
            })
            .collect();
        Self { fragments }
    }

    /// The library holding counting rule `rule` for package `pkg`: its own
    /// fragment when it declares the rule, else the shared library.
    #[must_use]
    pub fn library(&self, pkg: &str, rule: &str) -> &ClockLibrary {
        self.fragments
            .get(pkg)
            .filter(|lib| lib.counting_rules.iter().any(|r| r.id == rule))
            .unwrap_or(&ClockLibrary::embedded().0)
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

/// Computes deadline `d` of `pkg` triggered on `trigger` by `event`.
///
/// # Errors
/// A message for an invalid date, business-day units (no engine lowering),
/// an unknown counting rule, or an engine refusal.
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
    let decl = hirx::counting(pkg, &rule);
    let method = event.and_then(|e| method_of(e, decl));
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
    let library = clocks.library(&pkg.manifest.name, &rule);
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
