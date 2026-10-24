//! `delegation-respected` (SPEC §5.5 step 3, L3004): a lower authority may
//! change a higher one's rule only by a typed capability the rule declares
//! (`delegable { to L, op O, max N.unit }`, design review §2.4) or an
//! emergency `grant` gives it for a window. Every relation that displaces
//! its target needs one covering the operations it performs: `override`
//! and `notwithstanding` (the value it replaces the target's with),
//! `supersedes` and `revives` (suspension of another declaration). An
//! unlicensed relation is L3004 here and has no effect on selection
//! ([`licensed`]).
use super::lattice::{Lattice, Order};
use crate::hir::{
    Declaration, DelegatedOp, Delegation, Expr, ExprKind, Extension, Relation, Value,
};
use crate::temporal::condition::matches;
use crate::temporal::interval::Window;
use crate::temporal::world::{error, nowhere, span_of, Decl, DiagnosticExt, Key, Locate, World};
use litigate_base::{Applicability, Derivation, Diagnostic, DiagnosticCode as C, Span};

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

/// Pattern `e`, written in package `pack`'s view, with every event and
/// variant name fully qualified, so patterns of two packages compare
/// ([`matches`] ignores types).
fn qualified(pack: &str, e: &Expr) -> Expr {
    let mut e = e.clone();
    qualify_in(pack, &mut e);
    e
}

fn qualify_in(pack: &str, e: &mut Expr) {
    if let ExprKind::Call { callee, args } = &mut e.kind {
        *callee = qualify(pack, callee);
        for a in args {
            qualify_in(pack, a);
        }
    }
    if let ExprKind::Literal {
        value: Value::Variant(v),
    } = &mut e.kind
    {
        *v = qualify(pack, v);
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
    /// (`extend { never cite "X" }`, naming X).
    Locked(&'w str),
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
    let mine = &world.decls[by].meta.authority;
    if outranks(world.lattice(), mine, &world.decls[target].meta.authority) {
        return Ok(());
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
    // Every covering capability bounded, and no bound admits the claim.
    let bounds: Option<Vec<(u64, crate::hir::DayUnit)>> = covering.iter().map(|d| d.max).collect();
    if let Some(bounds) = bounds {
        if !bounds.iter().any(|b| within_bound(world, *b, by, target)) {
            return Err(Refusal::Bound(bounds[0].0, bounds[0].1));
        }
    }
    lock_widened(world, by, target).map_or(Ok(()), |lock| Err(Refusal::Locked(lock)))
}

/// Whether declaration `by` may displace version `target` by an
/// `override` or `notwithstanding` ([`license`]).
#[must_use]
pub fn licensed(world: &World<'_>, by: usize, target: usize) -> bool {
    license(world, by, target, false).is_ok()
}

/// The authority of deadline `target`'s lock (`extend { never cite "X" }`)
/// when deadline `by` grants an extension `target` does not state (FRAP
/// 26(b): a local rule or order cannot grant there), and no licensed
/// emergency grant `unlock`s `target` for `by`'s window.
#[must_use]
pub fn lock_widened<'w>(world: &'w World<'_>, by: usize, target: usize) -> Option<&'w str> {
    let (Declaration::Deadline(mine), Declaration::Deadline(theirs)) =
        (world.decls[by].decl, world.decls[target].decl)
    else {
        return None;
    };
    let lock = theirs.extend.iter().find_map(Extension::lock)?;
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
        .all(|v| license(world, by, v, suspend).is_ok())
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
            }
        }
        for d in &da.meta.delegations {
            out.extend(misdelegated(lattice, loc, da, &da.meta.authority, d));
        }
    }
    out.extend(grants(world, lattice, pack));
    out
}

/// L3004 for each emergency grant of `pack` whose authority is below a
/// version of a target it reaches in its window (a grant never amplifies
/// its own power), and for each of its delegations not below that
/// version's authority.
fn grants(world: &World<'_>, lattice: &Lattice, pack: usize) -> Vec<Diagnostic> {
    let mut out: Vec<Diagnostic> = Vec::new();
    for (key, granted) in world.granted() {
        for g in granted.iter().filter(|g| g.pack == pack) {
            let window = Window::of(&g.grant.meta.effective);
            let at = g
                .grant
                .meta
                .span
                .clone()
                .unwrap_or_else(|| nowhere(&g.grant.meta.id));
            let ours = &g.grant.meta.authority;
            let name = key.display(&world.packs[pack].pack);
            for v in world
                .versions(key)
                .versions
                .iter()
                .filter(|v| v.in_force.overlaps(&window))
            {
                let db = &world.decls[v.decl];
                let theirs = &db.meta.authority;
                let mut found = Vec::new();
                if !outranks(lattice, ours, theirs) {
                    found.push(
                        error(
                            C::MissingDelegation,
                            at.clone(),
                            format!(
                                "grant `{}` ({ours}) reaches `{name}` ({theirs}), which it does not outrank: a grant never widens its own power",
                                g.grant.meta.id,
                            ),
                        )
                        .with_note(db.meta.span.clone(), format!("`{name}` is declared here"))
                        .derived(
                            Derivation::new("unlicensed", format!("unlicensed(grant `{}`)", g.grant.meta.id))
                                .at(Some(at.clone()))
                                .premise(Derivation::new("rank", format!("¬outranks({ours}, {theirs})"))),
                        ),
                    );
                }
                for d in g
                    .grant
                    .delegations
                    .iter()
                    .filter(|d| lattice.compare(&d.to, theirs) != Order::Below)
                {
                    found.push(error(
                        C::MissingDelegation,
                        at.clone(),
                        format!(
                            "grant `{}` delegates `{name}` ({theirs}) to `{}`, which is not a lower authority",
                            g.grant.meta.id, d.to
                        ),
                    ));
                }
                found.extend(granted_capabilities(g.grant, db, &name, &at));
                for d in found {
                    if !out.iter().any(|o| o.message == d.message) {
                        out.push(d);
                    }
                }
            }
        }
    }
    out
}

/// What grant `g` adds to version `db` (named `name`) that the version
/// does not admit: L2009 for a waiver, an unbounded extension or a
/// delegation of a jurisdictional deadline (as if declared there), and
/// L3004 for extensions of a locked deadline the grant does not `unlock`
/// (they would have no effect).
fn granted_capabilities(
    g: &crate::hir::Grant,
    db: &Decl<'_>,
    name: &str,
    at: &Span,
) -> Vec<Diagnostic> {
    let Declaration::Deadline(d) = db.decl else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let jurisdictional = d
        .characterization
        .as_ref()
        .is_some_and(|c| c.kind == crate::hir::CharacterKind::Jurisdictional);
    if jurisdictional {
        let mut forbidden = Vec::new();
        if g.waive.is_some() {
            forbidden.push("a waiver");
        }
        if g.extend
            .iter()
            .any(|x| matches!(x, Extension::Discretionary { .. }))
        {
            forbidden.push("an extension without a bound");
        }
        if !g.delegations.is_empty() {
            forbidden.push("a delegation to a lower authority");
        }
        if !forbidden.is_empty() {
            out.push(
                error(
                    C::InvalidCapability,
                    at.clone(),
                    format!(
                        "grant `{}` gives jurisdictional deadline `{name}` {}, which it admits from no instrument",
                        g.meta.id,
                        forbidden.join(" and ")
                    ),
                )
                .with_note(db.meta.span.clone(), format!("`{name}` is characterized jurisdictional here")),
            );
        }
    }
    let lock = d.extend.iter().find_map(Extension::lock);
    if let (Some(lock), false, false) = (lock, g.unlock, g.extend.is_empty()) {
        out.push(
            error(
                C::MissingDelegation,
                at.clone(),
                format!(
                    "grant `{}` extends `{name}`, which is locked (`extend {{ never cite \"{lock}\" }}`), without `unlock`: its extensions would have no effect",
                    g.meta.id
                ),
            )
            .with_note(db.meta.span.clone(), format!("`{name}` is locked here")),
        );
    }
    out
}

/// L3004 when `da` delegates to `d.to`, which is not below its authority
/// `own`.
fn misdelegated(
    lattice: &Lattice,
    loc: &dyn Locate,
    da: &Decl<'_>,
    own: &str,
    d: &Delegation,
) -> Option<Diagnostic> {
    let to = &d.to;
    let order = lattice.compare(to, own);
    if order == Order::Below {
        return None;
    }
    let why = match order {
        Order::Equal => format!(
            "`{to}` is `{}`'s own authority, which needs no delegation",
            da.meta.id
        ),
        Order::Above => format!("`{to}` outranks `{own}`"),
        _ => format!("`{to}` and `{own}` are not ranked by any `authority` chain"),
    };
    let at = span_of(loc, da, "delegable");
    let derivation = Derivation::new(
        "unlicensed",
        format!("unlicensed(`{}` delegable to {to})", da.meta.id),
    )
    .at(Some(at.clone()))
    .premise(Derivation::new("rank", format!("¬below({to}, {own})")));
    Some(
        error(
            C::MissingDelegation,
            at,
            format!(
                "`{}` ({own}) delegates to `{to}`, which is not a lower authority",
                da.meta.id
            ),
        )
        .with_note(None, why)
        .derived(derivation),
    )
}

/// The versions of `target` (only its `@D` version, when given) in force
/// at some point of `window`.
fn overlapping(
    world: &World<'_>,
    target: &crate::temporal::world::Target,
    window: &Window,
) -> Vec<usize> {
    world
        .versions(&target.key)
        .versions
        .iter()
        .filter(|v| {
            target
                .version
                .as_ref()
                .is_none_or(|d| *d == v.in_force.from)
        })
        .filter(|v| v.in_force.overlaps(window))
        .map(|v| v.decl)
        .collect()
}

fn overriding(
    world: &World<'_>,
    loc: &dyn Locate,
    a: usize,
    text: &str,
    window: &Window,
) -> Vec<Diagnostic> {
    let da = &world.decls[a];
    let target = world.target(world.pack_of(a), text);
    let set = world.versions(&target.key);
    if set.versions.is_empty() {
        return Vec::new();
    }
    let name = target.key.display(world.pack_of(a));
    let affected = overlapping(world, &target, window);
    if affected.is_empty() {
        return vec![error(
            C::NoVersionInForce,
            span_of(loc, da, "override"),
            format!(
                "`{}` overrides `{text}`, which is not in force during {}",
                da.meta.id,
                window.display()
            ),
        )
        .with_note(
            None,
            format!(
                "`{name}` is in force on {}",
                set.versions
                    .iter()
                    .map(|v| v.in_force.display())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        )];
    }
    unlicensed(world, loc, a, &target.key, "override", &affected)
}

/// The operations as source text: `extend` or `[extend, set]`.
fn ops_text(ops: &[DelegatedOp]) -> String {
    match ops {
        [one] => one.name().to_owned(),
        many => format!(
            "[{}]",
            many.iter().map(|o| o.name()).collect::<Vec<_>>().join(", ")
        ),
    }
}

/// L3004 for each version in `affected` that `a`'s `keyword` relation on
/// `key` displaces without a license: a missing delegation, operations no
/// capability grants, a bound exceeded, or a lock the relation would
/// widen. One diagnostic per refusal kind, at the relation clause.
fn unlicensed(
    world: &World<'_>,
    loc: &dyn Locate,
    a: usize,
    key: &Key,
    keyword: &str,
    affected: &[usize],
) -> Vec<Diagnostic> {
    let suspend = matches!(keyword, "supersedes" | "revives") && world.key(a) != *key;
    let refused: Vec<(usize, Refusal<'_>)> = affected
        .iter()
        .filter_map(|&b| license(world, a, b, suspend).err().map(|r| (b, r)))
        .collect();
    let name = key.display(world.pack_of(a));
    let span = span_of(loc, &world.decls[a], keyword);
    let mut out = Vec::new();
    let missing: Vec<usize> = refused
        .iter()
        .filter(|(_, r)| *r == Refusal::Missing)
        .map(|(b, _)| *b)
        .collect();
    out.extend(missing_delegation(
        world,
        loc,
        a,
        keyword,
        &name,
        &missing,
        suspend,
        span.clone(),
    ));
    let claim = Claim {
        world,
        a,
        keyword,
        name: &name,
        span: &span,
    };
    for (b, refusal) in &refused {
        let Some(d) = claim.refused(*b, refusal) else {
            continue;
        };
        if !out.iter().any(|o: &Diagnostic| o.message == d.message) {
            out.push(d);
        }
    }
    out
}

/// A displacing claim of `a` (`keyword` on `name`, at `span`) whose
/// refusals [`unlicensed`] reports.
struct Claim<'c, 'w> {
    world: &'c World<'w>,
    a: usize,
    keyword: &'c str,
    name: &'c str,
    span: &'c Span,
}

impl Claim<'_, '_> {
    /// L3004 for `refusal` of version `b`; `None` for a missing
    /// delegation, which [`missing_delegation`] reports for every version
    /// at once.
    fn refused(&self, b: usize, refusal: &Refusal<'_>) -> Option<Diagnostic> {
        let (world, a, keyword, name, span) =
            (self.world, self.a, self.keyword, self.name, self.span);
        let da = &world.decls[a];
        let db = &world.decls[b];
        let theirs = &db.meta.authority;
        let mine = &da.meta.authority;
        let versioned = world.versioned(b, a);
        let d = match refusal {
            Refusal::Missing => return None,
            Refusal::Uncovered(ops) => {
                let granted: Vec<DelegatedOp> = capabilities(world, a, b)
                    .iter()
                    .flat_map(|d| d.ops.iter().copied())
                    .collect();
                error(
                    C::MissingDelegation,
                    span.clone(),
                    format!(
                        "`{}` ({mine}) {} `{name}` ({theirs}) by `{}`, but `{name}` delegates to {mine} only `{}`",
                        da.meta.id,
                        verb(keyword),
                        ops_text(ops),
                        ops_text(&granted)
                    ),
                )
                .with_note(
                    db.meta.span.clone(),
                    format!("`{versioned}` declares its capabilities here; `set` covers `extend` and `shorten`, nothing covers `compute_method` or `suspend` but itself"),
                )
                .derived(
                    Derivation::new("unlicensed", format!("unlicensed(`{}` {keyword} `{name}`)", da.meta.id))
                        .at(Some(span.clone()))
                        .premise(Derivation::new("operations", format!("performs(`{}`, {})", da.meta.id, ops_text(ops))))
                        .premise(Derivation::new("no-capability", format!("¬grants(`{versioned}`, {mine}, {})", ops_text(ops))).at(db.meta.span.clone())),
                )
            }
            Refusal::Bound(n, unit) => {
                let (n, unit) = (*n, *unit);
                error(
                    C::MissingDelegation,
                    span.clone(),
                    format!(
                        "`{}` ({mine}) {} `{name}` ({theirs}) beyond its delegation's bound: no more than {} past the declared period",
                        da.meta.id,
                        verb(keyword),
                        unit.prose(n)
                    ),
                )
                .with_note(db.meta.span.clone(), format!("`{versioned}` delegates to {mine} an extension of no more than {}", unit.prose(n)))
                .derived(
                    Derivation::new("unlicensed", format!("unlicensed(`{}` {keyword} `{name}`)", da.meta.id))
                        .at(Some(span.clone()))
                        .premise(Derivation::new("bound", format!("exceeds(`{}`, max {})", da.meta.id, unit.prose(n)))),
                )
            }
            Refusal::Locked(authority) => error(
                C::MissingDelegation,
                span.clone(),
                format!(
                    "`{}` ({mine}) {} `{name}` ({theirs}) with an extension it does not grant, but `{name}` is locked: `extend {{ never cite \"{authority}\" }}`",
                    da.meta.id,
                    verb(keyword),
                ),
            )
            .with_note(
                db.meta.span.clone(),
                format!(
                    "`{versioned}` delegates to {mine}, but a delegation does not reach past its lock: no local rule or order grants an extension there ({authority})",
                ),
            ),
        };
        Some(d)
    }
}

/// L3004 naming the delegation `missing` versions of the target of `a`'s
/// `keyword` relation lack.
#[allow(clippy::too_many_arguments)]
fn missing_delegation(
    world: &World<'_>,
    loc: &dyn Locate,
    a: usize,
    keyword: &str,
    name: &str,
    missing: &[usize],
    suspend: bool,
    span: Span,
) -> Option<Diagnostic> {
    let da = &world.decls[a];
    let mine = &da.meta.authority;
    let &first = missing.first()?;
    let db = &world.decls[first];
    let theirs = &db.meta.authority;
    let ops = operations(world, a, first, suspend);
    let wanted = if ops.is_empty() {
        "set".to_owned()
    } else {
        ops_text(&ops)
    };
    let mut d = error(
        C::MissingDelegation,
        span,
        format!(
            "`{}` ({mine}) {} `{name}` ({theirs}), but `{name}` does not delegate to {mine}: missing `delegable {{ to {mine}, op {wanted} }}`",
            da.meta.id,
            verb(keyword)
        ),
    );
    let rank = match world.lattice().compare(mine, theirs) {
        Order::Incomparable => {
            d = d.with_note(None, format!("{mine} and {theirs} are not ranked by any `authority` chain, so `{keyword}` needs an explicit delegation"));
            format!("incomparable({mine}, {theirs})")
        }
        _ => format!("below({mine}, {theirs})"),
    };
    let mut derivation = Derivation::new(
        "unlicensed",
        format!("unlicensed(`{}` {keyword} `{name}`)", da.meta.id),
    )
    .at(Some(d.span.clone()))
    .premise(
        Derivation::new("claim", format!("displaces(`{}`, `{name}`)", da.meta.id))
            .at(Some(d.span.clone())),
    )
    .premise(Derivation::new("rank", rank));
    for &b in missing {
        let db = &world.decls[b];
        derivation = derivation.premise(
            Derivation::new(
                "no-capability",
                format!("¬delegates(`{}`, {mine})", world.versioned(b, a)),
            )
            .at(db.meta.span.clone()),
        );
        d = d.with_note(
            db.meta.span.clone(),
            format!(
                "`{}` {} delegates to no {mine}",
                world.versioned(b, a),
                Window::of(&db.meta.effective).display()
            ),
        );
        let anchor = ANCHORS
            .iter()
            .find_map(|k| loc.clause(db, k).and_then(|(_, v)| v));
        if let Some(at) = anchor {
            d = d.with_fix(
                end_of(&at),
                format!(" delegable {{ to {mine}, op {wanted} }}"),
                Applicability::MaybeIncorrect,
            );
        }
    }
    Some(d.derived(derivation).with_note(
        None,
        format!("only a rule's own text can delegate; `{keyword}` cannot grant itself the power"),
    ))
}

/// How the message reads `keyword`'s effect.
fn verb(keyword: &str) -> &str {
    match keyword {
        "override" => "overrides",
        "notwithstanding" => "is declared notwithstanding",
        other => other,
    }
}
