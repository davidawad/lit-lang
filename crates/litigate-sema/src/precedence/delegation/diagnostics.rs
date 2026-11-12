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
    let lock = d
        .extend
        .iter()
        .find(|x| x.locks(&g.meta.authority))
        .and_then(Extension::lock_source);
    if let (Some(lock), false, false) = (lock, g.unlock, g.extend.is_empty()) {
        out.push(
            error(
                C::MissingDelegation,
                at.clone(),
                format!(
                    "grant `{}` extends `{name}`, which is locked (`{lock}`), without `unlock`: its extensions would have no effect",
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

/// L3004 for each authority a named lock `x` of `da` binds that is above
/// `da`'s own or unranked against it: a rule binds its own level (an
/// emergency grant of that level needs `unlock`) and those below it,
/// never a higher authority.
fn mislocked(lattice: &Lattice, loc: &dyn Locate, da: &Decl<'_>, x: &Extension) -> Vec<Diagnostic> {
    let Extension::NonExtendable { against, .. } = x else {
        return Vec::new();
    };
    let own = &da.meta.authority;
    against
        .iter()
        .filter_map(|level| {
            let why = match lattice.compare(level, own) {
                Order::Below | Order::Equal => return None,
                Order::Above => format!("`{level}` outranks `{own}`"),
                Order::Incomparable => {
                    format!("`{level}` and `{own}` are not ranked by any `authority` chain")
                }
            };
            Some(
                error(
                    C::MissingDelegation,
                    span_of(loc, da, "lock"),
                    format!(
                        "`{}` ({own}) locks against `{level}`, which it does not bind: a lock binds the deadline's own authority and those below it",
                        da.meta.id
                    ),
                )
                .with_note(None, why),
            )
        })
        .collect()
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

/// The shape of `a`'s `interprets text` (SPEC §5.7): `a` a deadline, its
/// target another package's deadline in force during `a`'s window, and
/// `a`'s period the target's in every version it reads (L2001, L2007,
/// L3007): an interpretation changes a trigger or a counting rule, never
/// the period the text states.
fn interpreting(
    world: &World<'_>,
    loc: &dyn Locate,
    a: usize,
    text: &str,
    window: &Window,
) -> Vec<Diagnostic> {
    let da = &world.decls[a];
    let at = || span_of(loc, da, "interprets");
    let Declaration::Deadline(mine) = da.decl else {
        return vec![error(
            C::TypeMismatch,
            at(),
            format!("`{}` interprets `{text}`, but only a deadline can", da.meta.id),
        )
        .with_note(None, "an `interprets` reading is a deadline restating another package's deadline with the trigger or counting rule a binding precedent holds")];
    };
    let target = world.target(world.pack_of(a), text);
    if target.key.pack == world.pack_of(a) {
        return vec![error(
            C::InvalidField,
            at(),
            format!(
                "`{}` interprets `{text}`, a declaration of its own package",
                da.meta.id
            ),
        )
        .with_note(
            None,
            "a package changes its own declarations by versioning them (`effective`, `supersedes`)",
        )];
    }
    let set = world.versions(&target.key);
    if set.versions.is_empty() {
        return Vec::new();
    }
    let affected = overlapping(world, &target, window);
    if affected.is_empty() {
        return vec![error(
            C::NoVersionInForce,
            at(),
            format!(
                "`{}` interprets `{text}`, which is not in force during {}",
                da.meta.id,
                window.display()
            ),
        )];
    }
    let mut out = Vec::new();
    for v in affected {
        let theirs = &world.decls[v];
        match theirs.decl {
            Declaration::Deadline(d) => {
                if (d.amount, d.unit, d.direction) != (mine.amount, mine.unit, mine.direction) {
                    out.push(
                        error(
                            C::InvalidField,
                            at(),
                            format!(
                                "`{}` interprets `{}`, but states another period",
                                da.meta.id,
                                world.versioned(v, a)
                            ),
                        )
                        .with_note(None, "an interpretation changes a deadline's trigger or counting rule; its period is the text's"),
                    );
                }
            }
            _ => out.push(error(
                C::TypeMismatch,
                at(),
                format!(
                    "`{}` interprets `{}`, which is not a deadline",
                    da.meta.id,
                    world.versioned(v, a)
                ),
            )),
        }
    }
    out
}
