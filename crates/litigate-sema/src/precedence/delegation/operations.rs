/// Clauses after whose value a `delegable` clause can be inserted.
const ANCHORS: [&str; 7] = [
    "effective",
    "known_at",
    "supersedes",
    "revives",
    "subject_to",
    "notwithstanding",
    "override",
];

fn end_of(s: &Span) -> Span {
    Span {
        file: s.file.clone(),
        line: s.end_line,
        col: s.end_col,
        end_line: s.end_line,
        end_col: s.end_col,
    }
}

/// The operations declaration `by` performs on version `target` when it
/// displaces it: suspension (`suspend`: a `supersedes` or `revives` of
/// another declaration), else the change of value it amounts to. Between
/// two deadlines that is `extend` or `shorten` for a longer or shorter
/// period in the same unit and direction, `set` for another unit,
/// direction, trigger, bearer, policy or grants, and `compute_method` for
/// another counting rule; between two counting rules `compute_method`;
/// otherwise `set`, which then stands for `extend` and `shorten` too.
/// Sorted; empty when the claim changes nothing.
#[must_use]
pub fn operations(world: &World<'_>, by: usize, target: usize, suspend: bool) -> Vec<DelegatedOp> {
    if suspend {
        return vec![DelegatedOp::Suspend];
    }
    let mut ops = match (world.decls[by].decl, world.decls[target].decl) {
        (Declaration::Deadline(a), Declaration::Deadline(b)) => {
            let mut ops = Vec::new();
            if a.unit == b.unit && a.direction == b.direction {
                match a.amount.cmp(&b.amount) {
                    std::cmp::Ordering::Greater => ops.push(DelegatedOp::Extend),
                    std::cmp::Ordering::Less => ops.push(DelegatedOp::Shorten),
                    std::cmp::Ordering::Equal => {}
                }
            } else {
                ops.push(DelegatedOp::Set);
            }
            let (pa, pb) = (world.pack_of(by), world.pack_of(target));
            if world.target(pa, &a.counting).key != world.target(pb, &b.counting).key {
                ops.push(DelegatedOp::ComputeMethod);
            }
            let (ta, tb) = (qualified(pa, &a.trigger), qualified(pb, &b.trigger));
            let same_trigger = matches(&ta, &tb) && matches(&tb, &ta);
            if !same_trigger
                || a.bearer != b.bearer
                || a.bearer_other != b.bearer_other
                || a.policy != b.policy
                || a.extend != b.extend
                || a.waive != b.waive
            {
                ops.push(DelegatedOp::Set);
            }
            ops
        }
        (Declaration::Counting(_), Declaration::Counting(_)) => vec![DelegatedOp::ComputeMethod],
        _ => vec![DelegatedOp::Set],
    };
    // `set` covers `extend` and `shorten`: name only what is needed.
    if ops.contains(&DelegatedOp::Set) {
        ops.retain(|o| !matches!(o, DelegatedOp::Extend | DelegatedOp::Shorten));
    }
    ops.sort();
    ops.dedup();
    ops
}

/// `name`, written in package `pack`'s view, fully qualified: a bare name
/// is declared by `pack`.
fn qualify(pack: &str, name: &str) -> String {
    if name.contains("::") {
        name.to_owned()
    } else {
        format!("{pack}::{name}")
    }
}

/// Pattern `e`, written in package `pack`'s view, with every event, type
/// and variant name fully qualified, so patterns of two packages compare
/// ([`matches`] compares argument types).
fn qualified(pack: &str, e: &Expr) -> Expr {
    let mut e = e.clone();
    qualify_in(pack, &mut e);
    e
}

fn qualify_in(pack: &str, e: &mut Expr) {
    if let Type::Named(n) = &mut e.ty {
        *n = qualify(pack, n);
    }
    if let ExprKind::Call { callee, args } = &mut e.kind {
        *callee = qualify(pack, callee);
        for a in args {
            qualify_in(pack, a);
        }
    }
    // A variant is `Enum::variant` in its own package, `pack::Enum::variant`
    // elsewhere.
    if let ExprKind::Literal {
        value: Value::Variant(v),
    } = &mut e.kind
    {
        if v.matches("::").count() < 2 {
            *v = format!("{pack}::{v}");
        }
    }
}

/// Why a displacing claim is not licensed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal<'w> {
    /// No capability names the claimant's authority.
    Missing,
    /// Capabilities name it, but none grants every operation.
    Uncovered(Vec<DelegatedOp>),
    /// Capabilities grant the operations, but the claim exceeds each one's
    /// bound (the first's: `max N.unit`).
    Bound(u64, crate::hir::DayUnit),
    /// The target is locked against an extension the claim grants
    /// (`extend { never cite "X" }`, or `lock against [...]` naming the
    /// claimant's authority): the lock.
    Locked(&'w Extension),
}

/// The delegations of version `target` naming `by`'s authority: its own,
/// then those of emergency grants reaching it whose window holds `by`'s.
fn capabilities<'w>(world: &'w World<'_>, by: usize, target: usize) -> Vec<&'w Delegation> {
    let mine = &world.decls[by].meta.authority;
    let window = Window::of(&world.decls[by].meta.effective);
    let theirs = &world.decls[target].meta;
    theirs
        .delegations
        .iter()
        .chain(
            in_window(world, target, &window)
                .into_iter()
                .flat_map(|g| &g.delegations),
        )
        .filter(|d| d.to == *mine)
        .collect()
}

/// The licensed grants reaching version `target` whose window holds
/// `window`: a grant licenses itself only when its authority equals or
/// outranks the target's (no amplification).
fn in_window<'w>(
    world: &'w World<'_>,
    target: usize,
    window: &Window,
) -> Vec<&'w crate::hir::Grant> {
    let key = world.key(target);
    let theirs = &world.decls[target].meta.authority;
    world
        .grants(&key)
        .iter()
        .filter(|g| window.within(&Window::of(&g.grant.meta.effective)))
        .filter(|g| outranks(world.lattice(), &g.grant.meta.authority, theirs))
        .map(|g| g.grant)
        .collect()
}

fn outranks(lattice: &Lattice, a: &str, b: &str) -> bool {
    matches!(lattice.compare(a, b), Order::Above | Order::Equal)
}

/// Whether bound `max n unit` admits the claim of `by` on `target`: it
/// caps the claimant's period at the target's plus `n`, in the same unit
/// and direction, and admits nothing but a deadline.
fn within_bound(
    world: &World<'_>,
    (n, unit): (u64, crate::hir::DayUnit),
    by: usize,
    target: usize,
) -> bool {
    match (world.decls[by].decl, world.decls[target].decl) {
        (Declaration::Deadline(a), Declaration::Deadline(b)) => {
            a.unit == unit
                && b.unit == unit
                && a.direction == b.direction
                && a.amount <= b.amount.saturating_add(n)
        }
        _ => false,
    }
}

/// Whether declaration `by` may displace version `target`, and why not
/// (SPEC §5.5 step 3): its authority equals or outranks the target's, or
/// a capability naming its authority grants every operation the claim
/// performs ([`operations`]) within its bound. A capability never
/// reaches past a lock: a lower authority may not displace a deadline
/// locked by `extend { never cite "X" }` with one granting an extension the
/// target does not ([`lock_widened`]), unless an emergency grant
/// `unlock`s it for the claim's window.
///
/// # Errors
/// The [`Refusal`].
pub fn license<'w>(
    world: &'w World<'_>,
    by: usize,
    target: usize,
    suspend: bool,
) -> Result<(), Refusal<'w>> {
    if admitting(world, by, target, suspend)?.is_none() {
        return Ok(());
    }
    lock_widened(world, by, target).map_or(Ok(()), |lock| Err(Refusal::Locked(lock)))
}

/// The capabilities admitting `by`'s claim on version `target` (covering
/// its operations within their bounds); `None` when `by`'s own authority
/// suffices.
fn admitting<'w>(
    world: &'w World<'_>,
    by: usize,
    target: usize,
    suspend: bool,
) -> Result<Option<Vec<&'w Delegation>>, Refusal<'w>> {
    let mine = &world.decls[by].meta.authority;
    if outranks(world.lattice(), mine, &world.decls[target].meta.authority) {
        return Ok(None);
    }
    let caps = capabilities(world, by, target);
    if caps.is_empty() {
        return Err(Refusal::Missing);
    }
    let ops = operations(world, by, target, suspend);
    let covering: Vec<&Delegation> = caps
        .into_iter()
        .filter(|d| ops.iter().all(|op| d.ops.iter().any(|g| g.covers(*op))))
        .collect();
    if covering.is_empty() {
        return Err(Refusal::Uncovered(ops));
    }
    let admitted: Vec<&Delegation> = covering
        .iter()
        .copied()
        .filter(|d| d.max.is_none_or(|b| within_bound(world, b, by, target)))
        .collect();
    // Every covering capability bounded, and no bound admits the claim.
    match (admitted.is_empty(), covering[0].max) {
        (true, Some((n, unit))) => Err(Refusal::Bound(n, unit)),
        _ => Ok(Some(admitted)),
    }
}

/// The motion `by`'s licensed claim on version `target` still lacks in
/// the world's matter: every capability admitting it is exercised on
/// motion (`delegable { ..., on M() }`) and the matter records none
/// matching on or before `by`'s effective date (the first such motion,
/// qualified). `None` outside a matter, for a claim its own authority
/// licenses, and for one [`license`] refuses.
#[must_use]
pub fn unmoved(world: &World<'_>, by: usize, target: usize, suspend: bool) -> Option<Expr> {
    let caps = admitting(world, by, target, suspend).ok()??;
    let pack = world.pack_of(target);
    let date = &world.decls[by].meta.effective.from;
    let motions: Vec<Expr> = caps
        .iter()
        .map(|d| d.on.as_ref().map(|m| qualified(pack, m)))
        .collect::<Option<_>>()?;
    (!motions.iter().any(|m| world.moved(m, date))).then(|| motions[0].clone())
}

/// Every displacing claim of the world that [`unmoved`] holds back in its
/// matter: (claimant, target version, the motion it lacks), in
/// declaration order.
#[must_use]
pub fn unmoved_claims(world: &World<'_>) -> Vec<(usize, usize, Expr)> {
    let mut out = Vec::new();
    for (a, da) in world.decls.iter().enumerate() {
        let window = Window::of(&da.meta.effective);
        for r in &da.meta.relations {
            let (versions, suspend) = match r {
                Relation::Override(text) | Relation::Notwithstanding(text) => {
                    let target = world.target(world.pack_of(a), text);
                    (overlapping(world, &target, &window), false)
                }
                Relation::Supersedes(text) | Relation::Revives(text) => {
                    let target = world.target(world.pack_of(a), text);
                    let suspend = world.key(a) != target.key;
                    (
                        earlier(world, a, &target.key, target.version.as_deref()),
                        suspend,
                    )
                }
                Relation::SubjectTo(_) | Relation::Interprets(_) => continue,
            };
            out.extend(
                versions
                    .into_iter()
                    .filter_map(|v| unmoved(world, a, v, suspend).map(|m| (a, v, m))),
            );
        }
    }
    out
}

/// Whether declaration `by` may displace version `target` by an
/// `override` or `notwithstanding` ([`license`]), on the motion its
/// capability requires when there is one ([`unmoved`]).
#[must_use]
pub fn licensed(world: &World<'_>, by: usize, target: usize) -> bool {
    license(world, by, target, false).is_ok() && unmoved(world, by, target, false).is_none()
}

/// Deadline `target`'s lock binding `by`'s authority (`extend { never
/// cite "X" }`, or `lock against [...]` naming it) when deadline `by`
/// grants an extension `target` does not state (FRAP 26(b): a local rule
/// or order cannot grant there), and no licensed emergency grant
/// `unlock`s `target` for `by`'s window.
#[must_use]
pub fn lock_widened<'w>(world: &'w World<'_>, by: usize, target: usize) -> Option<&'w Extension> {
    let (Declaration::Deadline(mine), Declaration::Deadline(theirs)) =
        (world.decls[by].decl, world.decls[target].decl)
    else {
        return None;
    };
    let level = &world.decls[by].meta.authority;
    let lock = theirs.extend.iter().find(|x| x.locks(level))?;
    let window = Window::of(&world.decls[by].meta.effective);
    let unlocked = in_window(world, target, &window).iter().any(|g| g.unlock);
    (!unlocked
        && mine
            .extend
            .iter()
            .any(|x| x.by().is_some() && !theirs.extend.contains(x)))
    .then_some(lock)
}

/// The versions of `key` (only the one starting on `version`, when given)
/// a `supersedes` or `revives` declared by `by` acts on: those that took
/// effect before it did.
fn earlier(world: &World<'_>, by: usize, key: &Key, version: Option<&str>) -> Vec<usize> {
    let from = &world.decls[by].meta.effective.from;
    world
        .versions(key)
        .versions
        .iter()
        .filter(|v| v.in_force.from < *from && version.is_none_or(|d| v.in_force.from == d))
        .map(|v| v.decl)
        .collect()
}

/// Whether `by` may supersede or revive `key` (`@version` when given):
/// [`license`] over every version it acts on, as a suspension when `key`
/// is another declaration.
#[must_use]
pub fn licensed_over(world: &World<'_>, by: usize, key: &Key, version: Option<&str>) -> bool {
    let suspend = world.key(by) != *key;
    earlier(world, by, key, version)
        .into_iter()
        .all(|v| license(world, by, v, suspend).is_ok() && unmoved(world, by, v, suspend).is_none())
}

/// L3004 for every displacing relation of pack `pack` that no capability
/// licenses, L3007 for overrides of a rule never in force alongside, L3004
/// for delegations that do not point downward, and L3004 for emergency
/// grants by an authority below a target's.
#[must_use]
pub fn check(
    world: &World<'_>,
    lattice: &Lattice,
    loc: &dyn Locate,
    pack: usize,
) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for (a, da) in world
        .decls
        .iter()
        .enumerate()
        .filter(|(_, d)| d.pack == pack)
    {
        let window = Window::of(&da.meta.effective);
        for r in &da.meta.relations {
            match r {
                Relation::Override(text) => {
                    out.extend(overriding(world, loc, a, text, &window));
                }
                Relation::Notwithstanding(text) => {
                    let target = world.target(world.pack_of(a), text);
                    let affected = overlapping(world, &target, &window);
                    out.extend(unlicensed(
                        world,
                        loc,
                        a,
                        &target.key,
                        "notwithstanding",
                        &affected,
                    ));
                }
                Relation::Supersedes(text) | Relation::Revives(text) => {
                    let keyword = if matches!(r, Relation::Supersedes(_)) {
                        "supersedes"
                    } else {
                        "revives"
                    };
                    let target = world.target(world.pack_of(a), text);
                    let affected = earlier(world, a, &target.key, target.version.as_deref());
                    out.extend(unlicensed(world, loc, a, &target.key, keyword, &affected));
                }
                // `subject_to` yields: it displaces nothing.
                Relation::SubjectTo(_) => {}
                // A binding reading needs no delegation: only its shape.
                Relation::Interprets(text) => {
                    out.extend(interpreting(world, loc, a, text, &window));
                }
            }
        }
        for d in &da.meta.delegations {
            out.extend(misdelegated(lattice, loc, da, &da.meta.authority, d));
        }
        if let Declaration::Deadline(dl) = da.decl {
            for x in &dl.extend {
                out.extend(mislocked(lattice, loc, da, x));
            }
        }
    }
    out.extend(grants(world, lattice, pack));
    out
}
