//! The engine clock as a [`litigate_sema::case::Clock`]: case replay counts
//! deadlines with litgraph's own interpreter over the lowered counting
//! rules (SPEC §8.2), so a matter is judged by the arithmetic its scenario
//! runs under.
use crate::{LoweredPackage, Lowerer};
use litgraph::clock::cycle::endless_roll;
use litgraph::clock::rule::Step;
use litgraph::clock::{
    compute_due_date, ClockConfig, ClockLibrary, Date, DayDeadline, Direction, ServiceMethod, Unit,
};
use litigate_base::Diagnostic;
use litigate_sema::case::{Clock, DueRequest};
use litigate_sema::hir;
use std::collections::BTreeMap;

/// The embedded clock library overlaid with lowered package fragments,
/// plus every version of the packages' own counting rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineClock {
    library: ClockLibrary,
    /// `(rule id, version from)` to that version's fragment.
    versions: BTreeMap<(String, String), ClockLibrary>,
    /// Versions some date never rolls out of (`cycle::endless_roll`):
    /// counting under them is refused, not run forever.
    endless: BTreeMap<(String, String), String>,
}

fn overlay(library: &mut ClockLibrary, f: ClockLibrary) {
    library
        .calendars
        .retain(|c| f.calendars.iter().all(|n| n.id != c.id));
    library
        .counting_rules
        .retain(|r| f.counting_rules.iter().all(|n| n.id != r.id));
    library.calendars.extend(f.calendars);
    library.counting_rules.extend(f.counting_rules);
}

impl EngineClock {
    /// The embedded library with `fragments` merged over it: a fragment's
    /// rule or calendar replaces the library's one with the same id.
    #[must_use]
    pub fn new(fragments: impl IntoIterator<Item = ClockLibrary>) -> Self {
        let mut library = ClockLibrary::embedded().0.clone();
        for f in fragments {
            overlay(&mut library, f);
        }
        Self {
            library,
            versions: BTreeMap::new(),
            endless: BTreeMap::new(),
        }
    }

    /// The embedded library plus every version of every counting rule the
    /// `packages` declare, each lowered at its own `from` date
    /// ([`Lowerer::clock_version`]); the caller's selection decides which
    /// one a request names.
    ///
    /// # Errors
    /// The diagnostics of a counting version that does not lower.
    pub fn lowered(
        lowerer: &Lowerer<'_>,
        packages: &[&hir::Package],
    ) -> Result<Self, Vec<Diagnostic>> {
        let mut clock = Self::new([]);
        for pkg in packages.iter().filter(|p| !p.manifest.links) {
            let versions =
                pkg.modules
                    .iter()
                    .flat_map(|m| &m.declarations)
                    .filter_map(|d| match d {
                        hir::Declaration::Counting(c) => Some(c),
                        _ => None,
                    });
            for c in versions {
                if let Some(fragment) = lowerer.clock_version(pkg, c)? {
                    let key = (c.meta.id.clone(), c.meta.effective.from.clone());
                    if let Some(why) = endless_roll(&fragment) {
                        clock.endless.insert(key.clone(), why);
                    }
                    clock.versions.insert(key, fragment);
                }
            }
        }
        Ok(clock)
    }

    /// The clock of lowered `packages` (their own counting fragments).
    #[must_use]
    pub fn of(packages: &[LoweredPackage]) -> Self {
        Self::new(
            packages
                .iter()
                .filter_map(|p| p.clock.as_ref().map(|(lib, _)| lib.clone())),
        )
    }
}

impl Clock for EngineClock {
    fn due(&self, r: &DueRequest) -> Result<String, String> {
        let key = r.version.clone().map(|from| (r.counting.clone(), from));
        if let Some(why) = key.and_then(|k| self.endless.get(&k)) {
            return Err(why.clone());
        }
        let versioned = r.version.as_ref().and_then(|from| {
            let fragment = self.versions.get(&(r.counting.clone(), from.clone()))?;
            let mut library = self.library.clone();
            overlay(&mut library, fragment.clone());
            Some(library)
        });
        let library = versioned.as_ref().unwrap_or(&self.library);
        let clock = library.clock(&r.counting).map_err(|e| e.to_string())?;
        // The first argument naming a service method is the trigger's; the
        // rule's total service step adds its days or refuses it.
        let service_method = r.service.iter().find_map(|m| {
            serde_json::from_value::<ServiceMethod>(serde_json::Value::String(m.clone())).ok()
        });
        let deadline = DayDeadline {
            trigger: Date::parse_iso(&r.trigger).map_err(|e| e.to_string())?,
            days: i64::try_from(r.amount).map_err(|e| e.to_string())?,
            direction: match r.direction {
                hir::Direction::Forward => Direction::Forward,
                hir::Direction::Backward => Direction::Backward,
            },
            unit: match r.unit {
                hir::DayUnit::Calendar => Unit::Calendar,
                hir::DayUnit::Court | hir::DayUnit::Business => Unit::Court,
                hir::DayUnit::Week => Unit::Week,
                hir::DayUnit::Month => Unit::Month,
                hir::DayUnit::Year => Unit::Year,
            },
            service_method,
        };
        compute_due_date(&deadline, &clock, &ClockConfig::default())
            .map(|c| c.due_date)
            .map_err(|e| e.to_string())
    }
}
