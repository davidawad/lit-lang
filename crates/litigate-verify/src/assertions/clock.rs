/// How many chained periods a chain may follow before it is a cycle.
const CHAIN_LIMIT: usize = 16;

/// The event pattern starting deadline `d` (of `owner`): its trigger, or
/// for a chained period the one starting the period it follows (its first
/// version's); `None` for a cycle.
fn root_trigger<'a>(
    cx: &Context<'a>,
    clocks: &Clocks<'a>,
    owner: &'a hir::Package,
    d: &'a hir::Deadline,
) -> Option<&'a Expr> {
    let mut d = (owner, d);
    for _ in 0..=CHAIN_LIMIT {
        let Some(chain) = &d.1.chain else {
            return Some(&d.1.trigger);
        };
        let key = clocks
            .world
            .target(&d.0.manifest.pack_id, &chain.deadline)
            .key;
        let target = format!("{}::{}", key.pack, key.name);
        let (pkg, _) = cx.owner(&target);
        d = (pkg, *cx.deadline_versions(&target).first()?);
    }
    None
}

/// The nominal and adjusted due dates of deadline `key`'s version in force
/// on `date` (with that declaration), from `given` (whose trigger
/// arguments are `event`) on `date`; a chained period counts from the end
/// of the one it follows, at most `depth` links deep.
fn count<'a>(
    clocks: &Clocks<'a>,
    (key, target): (&Key, &str),
    (given, event, date): (&Expr, &[Expr], &str),
    depth: usize,
) -> Result<(String, String, &'a hir::Deadline), String> {
    let Some((owner, Declaration::Deadline(deadline))) = clocks.select(key, date) else {
        return Err(format!(
            "no version of `{target}` recorded by {} is in force on {date}",
            clocks.known_at
        ));
    };
    let (start, event) = match &deadline.chain {
        Some(chain) => {
            // A cycle's root trigger matches no `given`, so a count never
            // follows one this deep; this refusal guards that.
            let depth = depth
                .checked_sub(1)
                .ok_or(format!("the chain through `{target}` is a cycle"))?;
            let base = clocks
                .world
                .target(&owner.manifest.pack_id, &chain.deadline)
                .key;
            let name = base.display("");
            let (nominal, adjusted, _) =
                count(clocks, (&base, &name), (given, event, date), depth)?;
            let start = match chain.end {
                hir::DueEnd::Nominal => nominal,
                hir::DueEnd::Adjusted => adjusted,
            };
            // A chained period runs from a date, not from service.
            (start, &[][..])
        }
        // A displacing claimant counts from its own trigger.
        None if !matches(&deadline.trigger, given) => {
            return Err(format!(
                "the given event does not match the trigger of `{}`",
                deadline.meta.id
            ));
        }
        None => (date.to_owned(), event),
    };
    let counting = counting_of(clocks, owner, deadline, date)?;
    let library = match counting {
        Some((pkg, c)) => clocks.get(pkg, c)?,
        None => ClockLibrary::embedded().0.clone(),
    };
    // A displacing counting rule counts under its own name.
    let rule = counting.map_or_else(
        || {
            deadline
                .counting
                .rsplit("::")
                .next()
                .unwrap_or(&deadline.counting)
        },
        |(_, c)| c.meta.id.as_str(),
    );
    // A designated due date is the given event's, rolled under the
    // counting rule like any last day: counted zero days from it.
    let designated = deadline
        .designation(date, given)
        .transpose()
        .map_err(|why| why.message().to_owned())?;
    let (start, days) = match &designated {
        Some(d) => (d.clone(), 0),
        None => (start, deadline.amount),
    };
    let trigger = clock::Date::parse_iso(&start).map_err(|e| e.to_string())?;
    let request = DayDeadline {
        trigger,
        days: i64::try_from(days)
            .map_err(|_| format!("`{target}` is too long for the clock to count"))?,
        direction: match deadline.direction {
            hir::Direction::Forward => clock::Direction::Forward,
            hir::Direction::Backward => clock::Direction::Backward,
        },
        unit: engine_unit(deadline.unit),
        service_method: service_method(counting.map(|(_, c)| c), event)
            .filter(|_| designated.is_none()),
    };
    library
        .clock(rule)
        .and_then(|clock| clock::compute_due_date(&request, &clock, &ClockConfig::default()))
        .map(|c| (c.nominal_date, c.due_date, deadline))
        .map_err(|e| e.to_string())
}

/// The engine's unit for `unit`.
fn engine_unit(unit: hir::DayUnit) -> Unit {
    match unit {
        hir::DayUnit::Calendar => Unit::Calendar,
        hir::DayUnit::Court => Unit::Court,
        hir::DayUnit::Business => Unit::Business,
        hir::DayUnit::Week => Unit::Week,
        hir::DayUnit::Month => Unit::Month,
        hir::DayUnit::Year => Unit::Year,
        hir::DayUnit::Hour => Unit::Hour,
    }
}

/// The counting rule `deadline` (of `owner`) counts under on `date`, with
/// its package; `None` for one no loaded package declares (the shared
/// library's).
fn counting_of<'a>(
    clocks: &Clocks<'a>,
    owner: &hir::Package,
    deadline: &hir::Deadline,
    date: &str,
) -> Result<Option<(&'a hir::Package, &'a hir::Counting)>, String> {
    let key = clocks
        .world
        .target(&owner.manifest.pack_id, &deadline.counting)
        .key;
    if clocks.world.versions(&key).versions.is_empty() {
        return Ok(None);
    }
    match clocks.select(&key, date) {
        Some((pkg, Declaration::Counting(c))) => Ok(Some((pkg, c))),
        _ => Err(format!(
            "no version of counting rule `{}` recorded by {} is in force on {date}",
            deadline.counting, clocks.known_at
        )),
    }
}

/// The engine service method of the first event argument naming one of
/// the counting rule's `service_extension` methods (any engine method for
/// a shared-library rule).
fn service_method(c: Option<&hir::Counting>, args: &[Expr]) -> Option<clock::ServiceMethod> {
    args.iter().find_map(|a| match &a.kind {
        ExprKind::Literal {
            value: Value::Variant(v),
        } => {
            let name = v.rsplit("::").next().unwrap_or(v);
            c.is_none_or(|c| c.service_extensions.contains_key(name))
                .then(|| kebab(name))
                .and_then(|k| serde_json::from_value(serde_json::Value::String(k)).ok())
        }
        _ => None,
    })
}

fn kebab(name: &str) -> String {
    name.chars()
        .enumerate()
        .flat_map(|(i, ch)| {
            let dash = (i > 0 && ch.is_ascii_uppercase()).then_some('-');
            dash.into_iter()
                .chain(std::iter::once(ch.to_ascii_lowercase()))
        })
        .collect()
}

/// The loaded declarations, and engine clock libraries per counting-rule
/// version, lowered once each.
struct Clocks<'a> {
    world: World<'a>,
    packages: Vec<&'a hir::Package>,
    /// The corpus date versions are selected at.
    known_at: &'a str,
    lowered: RefCell<BTreeMap<(String, String), Result<ClockLibrary, String>>>,
}

impl<'a> Clocks<'a> {
    fn new(cx: &Context<'a>, known_at: &'a str) -> Self {
        let mut loaded = cx.packages.to_vec();
        if !loaded.iter().any(|p| std::ptr::eq(*p, cx.package)) {
            loaded.push(cx.package);
        }
        // The law the package sees (SPEC §5.5): a downstream package's
        // claims never change its tests.
        let packages = World::closure_packages(&[cx.package], &loaded);
        Self {
            world: World::from_packages(&packages),
            packages,
            known_at,
            lowered: RefCell::default(),
        }
    }

    /// The declaration governing `key` on `date` among those recorded by
    /// `known_at` (`precedence::select` over the versions
    /// `temporal::select` finds: a version of `key`, or a claimant
    /// displacing it), with its package.
    fn select(&self, key: &Key, date: &str) -> Option<(&'a hir::Package, &'a Declaration)> {
        let i = select(&self.world, key, date, Some(self.known_at)).ok()?;
        let d = &self.world.decls[i];
        Some((self.packages[d.pack], d.decl))
    }

    fn get(&self, pkg: &hir::Package, counting: &hir::Counting) -> Result<ClockLibrary, String> {
        let key = (
            format!("{}::{}", pkg.manifest.pack_id, counting.meta.id),
            counting.meta.effective.from.clone(),
        );
        if let Some(found) = self.lowered.borrow().get(&key) {
            return found.clone();
        }
        let lowered = self.lower(pkg, counting);
        self.lowered.borrow_mut().insert(key, lowered.clone());
        lowered
    }

    /// The clock fragment lowering emits for version `counting` of `pkg`,
    /// lowered at that version's `from` date. A selected version is the
    /// one in force on its own `from`, so it always has a fragment or
    /// diagnostics; were it to have neither, the empty library names no
    /// rule and the count fails there.
    fn lower(&self, pkg: &hir::Package, counting: &hir::Counting) -> Result<ClockLibrary, String> {
        Lowerer::new(&self.packages)
            .clock_version(pkg, counting)
            .map(Option::unwrap_or_default)
            .map_err(|ds| {
                ds.iter()
                    .map(|d| format!("{} {}", d.code.as_str(), d.message))
                    .collect::<Vec<_>>()
                    .join("; ")
            })
            // Counting would not terminate (L3009 reports the rule).
            .and_then(|lib| clock::cycle::endless_roll(&lib).map_or(Ok(lib), Err))
    }
}

fn show(e: &Expr, v: &Value) -> String {
    match &e.kind {
        ExprKind::Call { callee, args } => match args.first().map(|a| &a.kind) {
            Some(ExprKind::Ref { target }) => format!("{callee}({target})"),
            _ => callee.clone(),
        },
        _ => render(v),
    }
}

fn render(v: &Value) -> String {
    match v {
        Value::Date(d) => d.clone(),
        Value::Boolean(b) => b.to_string(),
        other => serde_json::to_string(other).unwrap_or_default(),
    }
}

fn op(o: BinaryOp) -> &'static str {
    match o {
        BinaryOp::Eq => "==",
        BinaryOp::Ne => "!=",
        BinaryOp::Lt => "<",
        BinaryOp::Le => "<=",
        BinaryOp::Gt => ">",
        BinaryOp::Ge => ">=",
        _ => "?",
    }
}
