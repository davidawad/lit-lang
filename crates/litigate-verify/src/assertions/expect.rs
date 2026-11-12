fn expectation(
    cx: &Context<'_>,
    clocks: &Clocks,
    test: &hir::Test,
    e: &hir::Expectation,
) -> (Option<Diagnostic>, Option<(String, bool)>) {
    let span = cx.span_or_manifest(e.lhs.span.as_ref().or(test.span.as_ref()));
    let fail = |message: String| {
        note(
            emit(C::TestFailed, cx.profile, span.clone(), message),
            test.span.as_ref(),
            format!("in test \"{}\"", test.name),
        )
    };
    let lhs = eval(cx, clocks, test, &e.lhs);
    let rhs = eval(cx, clocks, test, &e.rhs);
    let sensitivity = lhs.as_ref().ok().and_then(|lhs| {
        counting_expectation(cx, clocks, test, e, lhs)
            .map(|(target, insensitive)| (target.to_owned(), insensitive))
    });
    let diagnostic = match (lhs, rhs) {
        (Ok(l), Ok(r)) => match compare(&l, e.op, &r) {
            Some(true) => None,
            Some(false) => Some(fail(format!(
                "expected {} {} {}, but it is {}",
                show(&e.lhs, &l),
                op(e.op),
                show(&e.rhs, &r),
                render(&l)
            ))),
            None => Some(fail(format!(
                "cannot compare {} {} {}",
                render(&l),
                op(e.op),
                render(&r)
            ))),
        },
        (Err(why), _) | (_, Err(why)) => {
            Some(fail(format!("cannot evaluate the assertion: {why}")))
        }
    };
    (diagnostic, sensitivity)
}

/// Evaluates a test-side expression to a literal value.
fn eval(cx: &Context<'_>, clocks: &Clocks, test: &hir::Test, e: &Expr) -> Result<Value, String> {
    match &e.kind {
        ExprKind::Literal { value } => Ok(value.clone()),
        ExprKind::Call { callee, args } if callee == "deadline" => {
            due_field(cx, clocks, test, args, "adjusted")
        }
        ExprKind::Field { base, field } => match &base.kind {
            ExprKind::Call { callee, args } if callee == "deadline" => {
                due_field(cx, clocks, test, args, field)
            }
            _ => Err("only a due date's fields are evaluated".to_owned()),
        },
        ExprKind::Call { callee, args } if callee == "has_event" => match args.first() {
            Some(p) => Ok(Value::Boolean(
                test.givens.iter().any(|(g, _)| matches(p, g)),
            )),
            None => Err("`has_event` takes an event pattern".to_owned()),
        },
        _ => Err("only literals, `deadline(d)`, and `has_event(p)` are evaluated".to_owned()),
    }
}

fn compare(l: &Value, o: BinaryOp, r: &Value) -> Option<bool> {
    match (o, l, r) {
        (BinaryOp::Eq, _, _) => Some(l == r),
        (BinaryOp::Ne, _, _) => Some(l != r),
        (_, Value::Date(a), Value::Date(b)) => match o {
            BinaryOp::Lt => Some(a < b),
            BinaryOp::Le => Some(a <= b),
            BinaryOp::Gt => Some(a > b),
            BinaryOp::Ge => Some(a >= b),
            _ => None,
        },
        _ => None,
    }
}

/// `pattern` matches `event`: same constructor, and each argument the
/// pattern gives equals the event's (omitted trailing arguments match
/// anything).
fn matches(pattern: &Expr, event: &Expr) -> bool {
    arguments(pattern, event).is_some()
}

/// The arguments of `event` when `pattern` matches it.
fn arguments<'e>(pattern: &Expr, event: &'e Expr) -> Option<&'e [Expr]> {
    match (&pattern.kind, &event.kind) {
        (
            ExprKind::Call {
                callee: p,
                args: pa,
            },
            ExprKind::Call {
                callee: e,
                args: ea,
            },
        ) => (p == e && pa.len() <= ea.len() && pa.iter().zip(ea).all(|(a, b)| same_expr(a, b)))
            .then_some(ea.as_slice()),
        _ => None,
    }
}

/// `deadline(d).field`: the `nominal` or (otherwise) `adjusted` date.
fn due_field(
    cx: &Context<'_>,
    clocks: &Clocks,
    test: &hir::Test,
    args: &[Expr],
    field: &str,
) -> Result<Value, String> {
    match args.first().map(|a| &a.kind) {
        Some(ExprKind::Ref { target }) => {
            due(cx, clocks, test, target, field == "nominal").map(Value::Date)
        }
        _ => Err("`deadline` takes a deadline name".to_owned()),
    }
}

/// A matching `given`: its index, event, trigger arguments and date.
type Given<'g> = (usize, &'g hir::Expr, &'g [hir::Expr], &'g str);

/// The `nominal` (or adjusted, then extended) due date of deadline
/// `target` from the matching `given` selected by its trigger policy,
/// using the shared version-selection query and the build's `known_at`
/// date. A chained period's `given` is the one starting the period it
/// follows.
fn due(
    cx: &Context<'_>,
    clocks: &Clocks,
    test: &hir::Test,
    target: &str,
    nominal: bool,
) -> Result<String, String> {
    let versions = cx.deadline_versions(target);
    let first = versions
        .first()
        .ok_or_else(|| format!("`{target}` is not a deadline"))?;
    let (owner, _) = cx.owner(target);
    let root = root_trigger(cx, clocks, owner, first).unwrap_or(&first.trigger);
    // A deadline another package declares (one this package imports) is
    // written in that package's view: each `given` is respelled into it.
    let (from, to) = (&cx.package.manifest.pack_id, &owner.manifest.pack_id);
    let givens: Vec<Expr> = test
        .givens
        .iter()
        .map(|(g, _)| respelled(from, to, g))
        .collect();
    let matching = test
        .givens
        .iter()
        .zip(&givens)
        .enumerate()
        .filter_map(|(i, ((_, d), g))| Some((i, g, arguments(root, g)?, d.as_str())));
    let order = |(i, _, _, a): &Given<'_>, (j, _, _, b): &Given<'_>| a.cmp(b).then(i.cmp(j));
    // The trigger policy picks the event that runs the period (SPEC §5.3).
    let picked = if first.policy.is_first() {
        matching.min_by(order)
    } else {
        matching.max_by(order)
    };
    let (at, given, event, date) =
        picked.ok_or_else(|| format!("no `given` event matches the trigger of `{target}`"))?;
    let (pkg, id) = cx.owner(target);
    let key = Key {
        pack: pkg.manifest.pack_id.clone(),
        name: id.to_owned(),
    };
    let (counted, adjusted, governing) =
        count(clocks, (&key, target), (given, event, date), CHAIN_LIMIT)?;
    if nominal {
        return Ok(counted);
    }
    let mut order: Vec<usize> = (0..test.givens.len()).collect();
    order.sort_by(|&a, &b| test.givens[a].1.cmp(&test.givens[b].1).then(a.cmp(&b)));
    let history: Vec<grant::Given<'_>> = order
        .iter()
        .map(|&i| grant::Given {
            date: &test.givens[i].1,
            action: &test.givens[i].0,
            actor: test.actors.get(&i).map(String::as_str),
        })
        .collect();
    let trigger = order.iter().position(|&i| i == at).unwrap_or_default();
    let trigger_event = history[trigger];
    let mut applied = grant::extensions(
        governing,
        target,
        &history,
        trigger,
        &adjusted,
        &|months| {
            let (owner, _) = cx.owner(target);
            counted_months(
                clocks,
                owner,
                governing,
                trigger_event.action,
                trigger_event.date,
                months,
            )
            .ok()
        },
        |action| grant::extends(&clocks.world, cx.package, &key, action),
    )?;
    Ok(applied.pop().map_or(adjusted, |(_, new)| new))
}

fn counted_months(
    clocks: &Clocks<'_>,
    owner: &hir::Package,
    deadline: &hir::Deadline,
    event: &Expr,
    trigger: &str,
    months: u64,
) -> Result<String, String> {
    let counting = counting_of(clocks, owner, deadline, trigger)?;
    let library = match counting {
        Some((pkg, c)) => clocks.get(pkg, c)?,
        None => ClockLibrary::embedded().0.clone(),
    };
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
    let method = service_method(
        counting.map(|(_, c)| c),
        arguments(&deadline.trigger, event).unwrap_or(&[]),
    );
    let request = DayDeadline {
        trigger: clock::Date::parse_iso(trigger).map_err(|e| e.to_string())?,
        days: i64::try_from(months).map_err(|_| format!("{months} months is out of range"))?,
        direction: clock::Direction::Forward,
        unit: Unit::Month,
        service_method: method,
    };
    let rule = library.clock(rule).map_err(|e| e.to_string())?;
    clock::compute_due_date(&request, &rule, &ClockConfig::default())
        .map(|due| due.due_date)
        .map_err(|e| e.to_string())
}

/// Event `e`, written in package `from`'s view, respelled for package
/// `to`'s: a bare name is `from`'s and gains its package, and `to`'s own
/// names lose theirs, as `to`'s declarations spell them.
fn respelled(from: &str, to: &str, e: &Expr) -> Expr {
    let mut e = e.clone();
    if from != to {
        respell_in(from, to, &mut e);
    }
    e
}

fn respell_in(from: &str, to: &str, e: &mut Expr) {
    // A name is `Name` in its own package and `pack::Name` elsewhere; a
    // variant is `Enum::variant` and `pack::Enum::variant`.
    let respell = |name: &mut String, own: usize| {
        let qualified = if name.matches("::").count() > own {
            name.clone()
        } else {
            format!("{from}::{name}")
        };
        *name = match qualified.split_once("::") {
            Some((pack, local)) if pack == to => local.to_owned(),
            _ => qualified,
        };
    };
    if let hir::Type::Named(n) = &mut e.ty {
        respell(n, 0);
    }
    match &mut e.kind {
        ExprKind::Call { callee, args } => {
            respell(callee, 0);
            for a in args {
                respell_in(from, to, a);
            }
        }
        ExprKind::Literal {
            value: Value::Variant(v),
        } => respell(v, 1),
        _ => {}
    }
}
