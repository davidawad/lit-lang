/// A counting version's identity across loads: pack id, local id, and its
/// effective `from` and `known_at`.
pub type VersionKey = (String, String, String, String);

/// The clock fragment of every counting version, or why it does not lower.
pub type Fragments = BTreeMap<VersionKey, Result<ClockLibrary, String>>;

/// The identity of a loaded declaration, the same in every world over it.
fn address(decl: &Declaration) -> usize {
    std::ptr::from_ref(decl).addr()
}

fn version_key(pkg: &hir::Package, c: &hir::Counting) -> VersionKey {
    (
        pkg.manifest.pack_id.clone(),
        c.meta.id.clone(),
        c.meta.effective.from.clone(),
        c.meta.effective.known_at.clone(),
    )
}

/// The counting versions of `packages`: address, package, declaration.
fn countings<'a>(
    packages: &[&'a hir::Package],
) -> Vec<(usize, &'a hir::Package, &'a hir::Counting)> {
    World::from_packages(packages)
        .decls
        .iter()
        .filter_map(|d| match d.decl {
            Declaration::Counting(c) => Some((address(d.decl), packages[d.pack], c)),
            _ => None,
        })
        .collect()
}

/// The clock fragment of version `c` of `p`, or why it does not lower.
fn lower(
    lowerer: &Lowerer<'_>,
    p: &hir::Package,
    c: &hir::Counting,
) -> Result<ClockLibrary, String> {
    match lowerer.clock_version(p, c) {
        Ok(Some(library)) => Ok(library),
        Ok(None) => Err(format!("counting rule `{}` does not lower", c.meta.id)),
        Err(diagnostics) => Err(diagnostics.first().map_or_else(String::new, |d| {
            format!("{}: {}", d.code.as_str(), d.message)
        })),
    }
}

/// Lowers every version of every counting rule of `packages`, each at its
/// own `from` date (what [`Clocks::new`] does, kept by version so it can
/// be computed once and shared, [`Clocks::with_fragments`]).
#[must_use]
pub fn fragments(packages: &[&hir::Package]) -> Fragments {
    let lowerer = Lowerer::new(packages);
    countings(packages)
        .into_iter()
        .map(|(_, p, c)| (version_key(p, c), lower(&lowerer, p, c)))
        .collect()
}

impl<'a> Clocks<'a> {
    /// Lowers every version of every counting rule of `packages`, each at
    /// its own `from` date.
    #[must_use]
    pub fn new(packages: &[&'a hir::Package]) -> Self {
        let lowerer = Lowerer::new(packages);
        let fragments = countings(packages)
            .into_iter()
            .map(|(i, p, c)| (i, Cow::Owned(lower(&lowerer, p, c))))
            .collect();
        Self {
            packages: packages.to_vec(),
            fragments,
        }
    }

    /// [`Clocks::new`] over fragments lowered already, by [`fragments`] of
    /// these packages or more.
    #[must_use]
    pub fn with_fragments(packages: &[&'a hir::Package], lowered: &'a Fragments) -> Self {
        let fragments = countings(packages)
            .into_iter()
            .filter_map(|(i, p, c)| Some((i, Cow::Borrowed(lowered.get(&version_key(p, c))?))))
            .collect();
        Self {
            packages: packages.to_vec(),
            fragments,
        }
    }

    /// The loaded package whose pack id is `pack`.
    #[must_use]
    pub fn package(&self, pack: &str) -> Option<&'a hir::Package> {
        self.packages
            .iter()
            .copied()
            .find(|p| p.manifest.pack_id == pack)
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

    /// How many counting versions have a fragment.
    #[cfg(test)]
    pub(crate) fn lowered_versions(&self) -> usize {
        self.fragments.len()
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
            match why {
                Ungoverned::Unresolved(pairs) => {
                    let pairs = unordered(world, &pairs, "");
                    return format!(
                        "no single declaration governs {what} `{name}` on {date}: {pairs}"
                    );
                }
                Ungoverned::Inactive(Inactive::Miss(Miss::Before { earliest })) => {
                    return format!(
                        "{what} `{name}` is not encoded before {earliest}, so nothing governs {date}: {}",
                        temporal::not_encoded(&earliest)
                    );
                }
                _ => {}
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
const fn engine_unit(d: &hir::Deadline) -> Unit {
    match d.unit {
        DayUnit::Calendar => Unit::Calendar,
        DayUnit::Court => Unit::Court,
        DayUnit::Week => Unit::Week,
        DayUnit::Month => Unit::Month,
        DayUnit::Year => Unit::Year,
        DayUnit::Hour => Unit::Hour,
        DayUnit::Business => Unit::Business,
    }
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
/// whose trigger no event matches, an invalid date, a counting rule nothing
/// governs on the event, or an engine refusal.
pub fn due(
    clocks: &Clocks<'_>,
    world: &World<'_>,
    pkg: &hir::Package,
    name: &str,
    events: &[MatterEvent<'_>],
    at: usize,
    known_at: Option<&str>,
) -> Result<Due, String> {
    let governing = deadline(world, pkg, name, events, at, known_at)?;
    due_of(clocks, world, governing, events, at, known_at)
}

/// The declaration of deadline `name` of `pkg` governing `events[at]`
/// among those recorded by `known_at`, with its pack, as [`due`] selects
/// it: a version, or the claimant displacing it.
///
/// # Errors
/// A message for an unknown deadline or one nothing governs on the event.
pub fn deadline<'r, 'w>(
    world: &'r World<'w>,
    pkg: &hir::Package,
    name: &str,
    events: &[MatterEvent<'_>],
    at: usize,
    known_at: Option<&str>,
) -> Result<(&'r str, &'w hir::Deadline), String> {
    let key = Key {
        pack: pkg.manifest.pack_id.clone(),
        name: hirx::local(name).to_owned(),
    };
    let missing = format!("`{name}` is not a deadline of `{}`", pkg.manifest.name);
    deadline_of(world, &key, (events, at, known_at), missing)
}

/// [`due`] under `governing`, the declaration [`deadline`] selects (with
/// its pack).
///
/// # Errors
/// As [`due`], once the deadline is selected.
pub fn due_of(
    clocks: &Clocks<'_>,
    world: &World<'_>,
    (owner, d): (&str, &hir::Deadline),
    events: &[MatterEvent<'_>],
    at: usize,
    known_at: Option<&str>,
) -> Result<Due, String> {
    let judged = Judged {
        clocks,
        world,
        events,
        at,
        known_at,
    };
    judged.due(owner, d, CHAIN_LIMIT)
}

/// Counts `months` from the same trigger under the declaration and clock
/// that counted `d`'s period.
///
/// # Errors
/// When the clock governing the trigger cannot count this month period.
pub fn months_from(
    clocks: &Clocks<'_>,
    world: &World<'_>,
    (owner, d): (&str, &hir::Deadline),
    events: &[MatterEvent<'_>],
    at: usize,
    known_at: Option<&str>,
    months: u64,
) -> Result<String, String> {
    let mut bound = d.clone();
    bound.amount = months;
    bound.unit = DayUnit::Month;
    bound.direction = hir::Direction::Forward;
    due_of(clocks, world, (owner, &bound), events, at, known_at).map(|due| due.computed.due_date)
}
