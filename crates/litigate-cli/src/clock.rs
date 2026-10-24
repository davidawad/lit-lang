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
use litigate_sema::hir::{self, DayUnit, Declaration, DueEnd, Expr};
use litigate_sema::precedence::select::unordered;
use litigate_sema::temporal::{self, Key, MatterEvent, Ungoverned, World};
use std::collections::BTreeMap;

/// Every version of every declaration the loaded packages declare, and
/// the clock fragment of each counting version. Declarations are selected
/// by sema's one query (`temporal::governing`: precedence over the
/// versions `temporal::select` finds), in the world of the packages asking
/// ([`Clocks::scope`]), as lowering and case replay select them.
#[derive(Debug, Default)]
pub struct Clocks<'a> {
    packages: Vec<&'a hir::Package>,
    /// Each counting version's fragment (by declaration address, which
    /// every scope shares), or why it does not lower.
    fragments: BTreeMap<usize, Result<ClockLibrary, String>>,
}

/// The identity of a loaded declaration, the same in every world over it.
fn address(decl: &Declaration) -> usize {
    std::ptr::from_ref(decl).addr()
}

impl<'a> Clocks<'a> {
    /// Lowers every version of every counting rule of `packages`, each at
    /// its own `from` date.
    #[must_use]
    pub fn new(packages: &[&'a hir::Package]) -> Self {
        let lowerer = Lowerer::new(packages);
        let fragments = World::from_packages(packages)
            .decls
            .iter()
            .filter_map(|d| match d.decl {
                Declaration::Counting(c) => Some((address(d.decl), packages[d.pack], c)),
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
        Self {
            packages: packages.to_vec(),
            fragments,
        }
    }

    /// The law the packages named `roots` see among those loaded: they,
    /// their dependencies and `core` (SPEC §5.5), so a downstream
    /// package's claims never change what an upstream one computes.
    #[must_use]
    pub fn scope(&self, roots: &[&str]) -> World<'a> {
        let roots: Vec<&'a hir::Package> = self
            .packages
            .iter()
            .copied()
            .filter(|p| roots.contains(&p.manifest.name.as_str()))
            .collect();
        World::closure(&roots, &self.packages)
    }

    /// The clock fragment of counting declaration `decl` (of a loaded
    /// package, as every scope's are).
    fn fragment(&self, decl: &Declaration) -> &Result<ClockLibrary, String> {
        &self.fragments[&address(decl)]
    }
}

/// The declaration (index into `world`, a [`Clocks::scope`]) governing
/// `key` for `events[at]` (`at` indexes `events`) among those recorded by
/// `known_at` (`temporal::governing`: the version in force on the event's
/// date or the claimant displacing it, unless a transition decides);
/// `None` when no package of `world` declares `key` (the shared library
/// has it, or nothing does).
///
/// # Errors
/// When packages declare `key` but nothing, or no single declaration,
/// governs the event.
fn governing(
    world: &World<'_>,
    what: &str,
    key: &Key,
    events: &[MatterEvent<'_>],
    at: usize,
    known_at: Option<&str>,
) -> Result<Option<usize>, String> {
    if world.versions(key).versions.is_empty() {
        return Ok(None);
    }
    let date = events[at].date;
    temporal::governing(world, key, events, at, known_at)
        .map(|g| Some(g.version))
        .map_err(|why| {
            let name = key.display("");
            if let Ungoverned::Unresolved(pairs) = why {
                let pairs = unordered(world, &pairs, "");
                return format!("no single declaration governs {what} `{name}` on {date}: {pairs}");
            }
            let recorded = known_at.map_or_else(String::new, |k| format!(" recorded by {k}"));
            format!("no version of {what} `{name}`{recorded} governs {date}")
        })
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

/// The engine's unit for `d`'s period.
///
/// # Errors
/// Business days, which have no engine lowering.
fn engine_unit(d: &hir::Deadline) -> Result<Unit, String> {
    Ok(match d.unit {
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
    /// The due date the trigger designates (`designated p`), before any
    /// roll; `None` when the period is counted.
    pub designated: Option<String>,
    /// The engine's result.
    pub computed: ComputedDeadline,
}

/// Computes deadline `name` of `pkg` for `events[at]` in `world` (a
/// [`Clocks::scope`] of `clocks`), under the declarations of the deadline
/// and of its counting rule governing that event among those recorded by
/// `known_at` (`None`: the latest corpus): a version of each, or the
/// claimant displacing it (SPEC §5.5). It counts from the latest event up
/// to `events[at]` matching the governing deadline's own trigger. A
/// date-only query is a one-event history; `at` indexes `events`.
///
/// # Errors
/// A message for an unknown deadline, one nothing governs on the event or
/// whose trigger no event matches, an invalid date, business-day units (no
/// engine lowering), a counting rule nothing governs on the event, or an
/// engine refusal.
pub fn due(
    clocks: &Clocks<'_>,
    world: &World<'_>,
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
    let missing = format!("`{name}` is not a deadline of `{}`", pkg.manifest.name);
    let (owner, d) = deadline_of(world, &key, (events, at, known_at), missing)?;
    let judged = Judged {
        clocks,
        world,
        events,
        at,
        known_at,
    };
    judged.due(owner, d, CHAIN_LIMIT)
}

/// How many chained periods a chain may follow before it is a cycle.
const CHAIN_LIMIT: usize = 16;

/// The deadline declaration (with its pack) governing `key` for
/// `events[at]` among those recorded by `known_at`; `missing` when `key`
/// names no deadline.
fn deadline_of<'r, 'w>(
    world: &'r World<'w>,
    key: &Key,
    (events, at, known_at): (&[MatterEvent<'_>], usize, Option<&str>),
    missing: String,
) -> Result<(&'r str, &'w hir::Deadline), String> {
    governing(world, "deadline", key, events, at, known_at)?
        .and_then(|i| match world.decls[i].decl {
            Declaration::Deadline(d) => Some((world.pack_of(i), d)),
            _ => None,
        })
        .ok_or(missing)
}

/// What a due date is judged against.
struct Judged<'j, 'w> {
    clocks: &'j Clocks<'w>,
    world: &'j World<'w>,
    events: &'j [MatterEvent<'j>],
    at: usize,
    known_at: Option<&'j str>,
}

impl Judged<'_, '_> {
    /// The due date of deadline `d` (of pack `owner`): from the latest
    /// event up to `events[at]` matching its trigger, or, chained, from
    /// the end of the period it follows (at most `depth` links deep).
    fn due(&self, owner: &str, d: &hir::Deadline, depth: usize) -> Result<Due, String> {
        let (events, at, known_at) = (self.events, self.at, self.known_at);
        let (trigger, action) = match &d.chain {
            None => {
                let trigger = events[..=at]
                    .iter()
                    .rev()
                    .find(|e| hirx::matches(&d.trigger, e.action))
                    .ok_or_else(|| {
                        format!(
                            "no event matches the trigger {} of `{}`",
                            hirx::event_text(&d.trigger),
                            d.meta.id
                        )
                    })?;
                (trigger.date.to_owned(), Some(trigger.action))
            }
            Some(chain) => {
                // A cycle's root trigger matches no event, so a count never
                // follows one this deep; this refusal guards that.
                let depth = depth
                    .checked_sub(1)
                    .ok_or(format!("the chain through `{}` is a cycle", d.meta.id))?;
                let key = self.world.target(owner, &chain.deadline).key;
                let missing = format!("`{}` is not a deadline", chain.deadline);
                let (pack, base) = deadline_of(self.world, &key, (events, at, known_at), missing)?;
                let base = self.due(pack, base, depth)?;
                let date = match chain.end {
                    DueEnd::Nominal => base.computed.nominal_date,
                    DueEnd::Adjusted => base.computed.due_date,
                };
                (date, None)
            }
        };
        self.count(owner, d, &trigger, action)
    }

    /// `d` counted from `trigger` (a date), served as event `action`
    /// says (a chained period has none).
    fn count(
        &self,
        owner: &str,
        d: &hir::Deadline,
        trigger: &str,
        action: Option<&Expr>,
    ) -> Result<Due, String> {
        let (world, events, at, known_at) = (self.world, self.events, self.at, self.known_at);
        let unit = engine_unit(d)?;
        let counting_key = world.target(owner, &d.counting).key;
        let version = governing(world, "counting rule", &counting_key, events, at, known_at)?
            .and_then(|i| match world.decls[i].decl {
                Declaration::Counting(c) => Some((world.decls[i].decl, c)),
                _ => None,
            });
        // A displacing counting rule counts under its own name.
        let rule = version.map_or_else(
            || hirx::local(&d.counting).to_owned(),
            |(_, c)| c.meta.id.clone(),
        );
        let method = action.and_then(|a| method_of(a, version.map(|(_, c)| c)));
        // A designated due date is the notice's, rolled under the counting
        // rule like any last day: counted zero days from it, no service
        // days. A chained period (no event) designates nothing.
        let designated = action
            .and_then(|a| d.designation(trigger, a))
            .transpose()
            .map_err(|why| why.message().to_owned())?;
        let (start, days, method) = match &designated {
            Some(date) => (date.as_str(), 0, None),
            None => (trigger, d.amount, method),
        };
        let request = DayDeadline {
            trigger: Date::parse_iso(start).map_err(|e| e.to_string())?,
            days: i64::try_from(days).map_err(|e| e.to_string())?,
            direction: match d.direction {
                hir::Direction::Forward => Direction::Forward,
                hir::Direction::Backward => Direction::Backward,
            },
            unit,
            service_method: method.as_ref().map(|(_, m)| *m),
        };
        let library = match version {
            Some((decl, _)) => self.clocks.fragment(decl).as_ref().map_err(Clone::clone)?,
            None => &ClockLibrary::embedded().0,
        };
        let clock = library.clock(&rule).map_err(|e| e.to_string())?;
        let computed = compute_due_date(&request, &clock, &ClockConfig::default())
            .map_err(|e| e.to_string())?;
        Ok(Due {
            version: format!("{}@{}", d.meta.id, d.meta.effective.from),
            rule,
            cite: version.and_then(|(_, c)| c.meta.cite.clone()),
            trigger: trigger.to_owned(),
            method: method.map(|(name, _)| name),
            designated,
            computed,
        })
    }
}

/// The event pattern that starts deadline `d` (of pack `owner`) in
/// `world`: its own trigger, or for a chained period the one starting the
/// period it follows (its first version's); `None` for a chain that does
/// not resolve or is a cycle.
#[must_use]
pub fn root_trigger<'w>(world: &World<'w>, owner: &str, d: &'w hir::Deadline) -> Option<&'w Expr> {
    let mut d = (owner.to_owned(), d);
    for _ in 0..=CHAIN_LIMIT {
        let Some(chain) = &d.1.chain else {
            return Some(&d.1.trigger);
        };
        let key = world.target(&d.0, &chain.deadline).key;
        d = world
            .decls
            .iter()
            .enumerate()
            .find_map(|(i, x)| match x.decl {
                Declaration::Deadline(b) if world.key(i) == key => {
                    Some((world.pack_of(i).to_owned(), b))
                }
                _ => None,
            })?;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::hir_with;
    use litigate_sema::hir::{ExprKind, Type, Value};

    /// A version recorded after the package's corpus date and one whose
    /// cite binds no source URL (L6001).
    const EXTRA: &str = "
counting late known_at 2026-01-01 cite \"FRCP 6(a)\"
{
  exclude_trigger_day
  include_last_day
  service_extension {
    Personal: 0.days
    Electronic: 0.days
    Mail: 3.days
    Clerk: unrecognized
    OtherConsented: unrecognized
    CommercialCarrier: unrecognized
    PrisonMail: unrecognized
    MailForeign: unrecognized
    Express: unrecognized
    ExpressForeign: unrecognized
  }
  holidays federal-holidays
}

counting nourl cite \"XYZ 6(a)\"
{
  exclude_trigger_day
  include_last_day
  holidays federal-holidays
}

deadline chained: 10.days from deadline(answer-period).nominal counted_by frcp6
deadline chained-adjusted: 10.days from deadline(chained).adjusted counted_by frcp6
deadline loop-a: 1.days from deadline(loop-b).nominal counted_by frcp6
deadline loop-b: 1.days from deadline(loop-a).nominal counted_by frcp6
deadline chained-early: 1.days from deadline(answer-period).nominal counted_by frcp6 known_at 2000-01-01
deadline nourl-base: 1.days from Served() counted_by nourl
deadline chained-nourl: 1.days from deadline(nourl-base).nominal counted_by frcp6
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
        let events = [MatterEvent::on(date, &action)];
        let clocks = Clocks::new(&[pkg]);
        let world = clocks.scope(&[&pkg.manifest.name]);
        due(&clocks, &world, pkg, name, &events, 0, known_at)
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
        let world = World::from_packages(&[&pkg]);
        let found: Vec<(&str, Result<usize, &str>)> = world
            .decls
            .iter()
            .filter(|d| matches!(d.decl, Declaration::Counting(_)))
            .map(|d| {
                (
                    d.meta.id.as_str(),
                    clocks
                        .fragment(d.decl)
                        .as_ref()
                        .map(|l| l.counting_rules.len())
                        .map_err(String::as_str),
                )
            })
            .collect();
        assert_eq!(clocks.fragments.len(), 4);
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
        let weeks = package(&|d| {
            d.amount = 3;
            d.unit = DayUnit::Week;
        });
        assert_eq!(date(&weeks), Ok("2025-03-24".to_owned()));
        // This package's rule declares no `end_of_month`.
        for unit in [DayUnit::Month, DayUnit::Year] {
            let months = package(&|d| d.unit = unit);
            assert!(date(&months).is_err_and(|e| e.contains("no calendar-month provision")));
        }
        let before = package(&|d| d.direction = hir::Direction::Backward);
        assert_eq!(date(&before), Ok("2025-02-10".to_owned()));
        // Mail service adds the counting rule's three days.
        let action = served("ServiceMethod::Mail");
        let events = [MatterEvent::on("2025-03-03", &action)];
        let clocks = Clocks::new(&[&pkg]);
        let mailed = due(
            &clocks,
            &clocks.scope(&[&pkg.manifest.name]),
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
    fn a_designated_due_date_is_the_trigger_argument_rolled() {
        let pkg = package(&|d| {
            d.designated = Some(hir::Designated {
                param: 1,
                name: "due".into(),
            });
        });
        let on = |designated: &str| {
            let action = expr(ExprKind::Call {
                callee: "Served".into(),
                args: vec![
                    expr(ExprKind::Literal {
                        value: Value::Variant("Document::complaint".into()),
                    }),
                    expr(ExprKind::Literal {
                        value: Value::Date(designated.into()),
                    }),
                ],
            });
            let events = [MatterEvent::on("2025-03-03", &action)];
            let clocks = Clocks::new(&[&pkg]);
            due(
                &clocks,
                &clocks.scope(&[&pkg.manifest.name]),
                &pkg,
                "answer-period",
                &events,
                0,
                None,
            )
            .map(|d| (d.designated, d.computed.due_date))
        };
        // A Saturday rolls to Monday; no service days are added.
        assert_eq!(
            on("2025-04-05"),
            Ok((Some("2025-04-05".to_owned()), "2025-04-07".to_owned()))
        );
        assert!(on("2025-03-10").is_err_and(|e| e.contains("earlier than the least time")));
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
    fn unordered_claimants_govern_nothing() {
        // Two claimants displace `answer-period` and nothing orders them
        // (SPEC §5.5): no single declaration counts it.
        let mut pkg = hir_with(
            "deadline quick: 14.days from Served(complaint) counted_by frcp6 \
             notwithstanding answer-period",
        );
        let mut quicker = hirx::deadlines(&pkg, "quick").next().unwrap().clone();
        quicker.meta.id = "quicker".into();
        pkg.modules[0]
            .declarations
            .push(Declaration::Deadline(quicker));
        let why = due_on(&pkg, "answer-period", "2025-03-03", None).unwrap_err();
        assert!(
            why.starts_with(
                "no single declaration governs deadline `frcp-rule12-example::answer-period` on 2025-03-03: "
            ),
            "{why}"
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

    /// Served 2025-03-03: `answer-period` ends 2025-03-24 (a Monday);
    /// `chained` runs 10 days from that nominal end, and `chained-adjusted`
    /// 10 days from `chained`'s adjusted end, Sunday April 13, to Monday.
    #[test]
    fn a_chained_period_counts_from_the_end_it_names() {
        let pkg = package(&|_| ());
        let date = |name: &str| due_on(&pkg, name, "2025-03-03", None).map(|d| d.computed.due_date);
        assert_eq!(date("chained"), Ok("2025-04-03".to_owned()));
        assert_eq!(date("chained-adjusted"), Ok("2025-04-14".to_owned()));
        let world = World::from_packages(&[&pkg]);
        let pack = &pkg.manifest.pack_id;
        let decl = |name: &str| hirx::deadlines(&pkg, name).next().unwrap();
        let root = root_trigger(&world, pack, decl("chained-adjusted"));
        assert_eq!(root, Some(&decl("answer-period").trigger));
        assert_eq!(root_trigger(&world, pack, decl("loop-a")), None);
        // A world without the base resolves no root.
        let empty = World::from_packages(&[]);
        assert_eq!(root_trigger(&empty, pack, decl("chained")), None);
    }

    /// A cycle, a base not recorded by `known_at`, and a base its clock
    /// cannot count: each refuses the chained period with the reason.
    #[test]
    fn a_chain_that_does_not_count_says_why() {
        let pkg = package(&|_| ());
        let why = |name: &str, known_at| due_on(&pkg, name, "2025-03-03", known_at).unwrap_err();
        assert!(why("loop-a", None).contains("is a cycle"));
        assert!(why("chained-early", Some("2001-01-01")).contains("answer-period"));
        assert!(why("chained-nourl", None).starts_with("L6001"));
    }

    #[test]
    fn methods_are_kebab_case() {
        assert_eq!(engine_method("Mail"), Some(ServiceMethod::Mail));
        assert_eq!(
            engine_method("OtherConsented"),
            Some(ServiceMethod::OtherConsented)
        );
        assert_eq!(engine_method("Personal"), Some(ServiceMethod::Personal));
        assert_eq!(engine_method("Fax"), None);
    }
}
