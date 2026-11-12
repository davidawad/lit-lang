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
        let unit = engine_unit(d);
        // A court that recounts `owner`'s periods counts them by its own rule.
        let counting_key = world
            .recounted(owner, &d.counting)
            .unwrap_or_else(|| world.target(owner, &d.counting).key);
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
