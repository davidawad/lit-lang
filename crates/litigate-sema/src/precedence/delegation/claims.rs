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
            Refusal::Locked(lock) => error(
                C::MissingDelegation,
                span.clone(),
                format!(
                    "`{}` ({mine}) {} `{name}` ({theirs}) with an extension it does not grant, but `{name}` is locked: `{}`",
                    da.meta.id,
                    verb(keyword),
                    lock.lock_source().unwrap_or_default(),
                ),
            )
            .with_note(
                db.meta.span.clone(),
                format!(
                    "`{versioned}` delegates to {mine}, but a delegation does not reach past its lock: no local rule or order grants an extension there ({})",
                    lock.lock().unwrap_or_default(),
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
