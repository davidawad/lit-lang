/// Check inline assertions and identify rules whose tests do not distinguish
/// counted dates, sharing one clock and each evaluated deadline value.
pub(crate) fn check(
    cx: &Context<'_>,
    known_at: &str,
) -> (Vec<Diagnostic>, std::collections::BTreeSet<String>) {
    let clocks = Clocks::new(cx, known_at);
    let mut tested = BTreeMap::<String, bool>::new();
    let mut failures = Vec::new();
    for declaration in cx.declarations() {
        let Declaration::Test(test) = declaration else {
            continue;
        };
        for exp in &test.expects {
            let (failure, sensitivity) = expectation(cx, &clocks, test, exp);
            failures.extend(failure);
            if let Some((target, insensitive)) = sensitivity {
                *tested.entry(target.clone()).or_insert(true) &= insensitive;
            }
        }
    }
    let insensitive = tested
        .into_iter()
        .filter_map(|(id, insensitive)| insensitive.then_some(id))
        .collect();
    (failures, insensitive)
}

fn date_deadline_expectation(expectation: &hir::Expectation) -> Option<(&str, bool, &str)> {
    let ExprKind::Literal {
        value: Value::Date(expected),
    } = &expectation.rhs.kind
    else {
        return None;
    };
    let (expression, nominal) = match &expectation.lhs.kind {
        ExprKind::Call { .. } => (&expectation.lhs, false),
        ExprKind::Field { base, field } => (base.as_ref(), field == "nominal"),
        _ => return None,
    };
    let ExprKind::Call { callee, args } = &expression.kind else {
        return None;
    };
    if callee != "deadline" {
        return None;
    }
    match args.first().map(|arg| &arg.kind) {
        Some(ExprKind::Ref { target }) => Some((target, nominal, expected)),
        _ => None,
    }
}

fn counting_expectation<'a>(
    cx: &Context<'_>,
    clocks: &Clocks,
    test: &hir::Test,
    expectation: &'a hir::Expectation,
    lhs: &Value,
) -> Option<(&'a str, bool)> {
    let (target, _nominal, expected) = date_deadline_expectation(expectation)?;
    let deadline = cx.deadline_versions(target).first().copied()?;
    if deadline.chain.is_some()
        || !matches!(
            deadline.unit,
            hir::DayUnit::Calendar
                | hir::DayUnit::Court
                | hir::DayUnit::Business
                | hir::DayUnit::Week
        )
    {
        return None;
    }
    let (_, date) = test
        .givens
        .iter()
        .find(|(given, _)| matches(&deadline.trigger, given))?;
    let Value::Date(actual) = lhs else {
        return None;
    };
    let trigger = clock::Date::parse_iso(date).ok()?;
    let (owner, _) = cx.owner(target);
    let counting = counting_of(clocks, owner, deadline, date).ok()?;
    let excluded_trigger = counting.is_none_or(|(_, rule)| rule.exclude_trigger_day);
    let amount = i64::try_from(deadline.amount).unwrap_or(i64::MAX);
    let amount_days = if deadline.unit == hir::DayUnit::Week {
        amount.saturating_mul(7)
    } else {
        amount
    };
    let delta = amount_days.saturating_sub(i64::from(!excluded_trigger));
    let naive = trigger
        .add_days(if deadline.direction == hir::Direction::Forward {
            delta
        } else {
            -delta
        })
        .to_iso();
    Some((
        cx.local(target),
        is_counting_insensitive(actual, &naive, expected),
    ))
}

/// Build a reproducible boundary test for a deadline, if its lowered clock
/// has a weekend/holiday roll that changes a plain calendar-day count.
pub(crate) fn counting_boundary(
    cx: &Context<'_>,
    deadline: &hir::Deadline,
) -> Result<BoundaryResult, String> {
    if deadline.chain.is_some()
        || !matches!(
            deadline.unit,
            hir::DayUnit::Calendar
                | hir::DayUnit::Court
                | hir::DayUnit::Business
                | hir::DayUnit::Week
        )
    {
        return Ok(BoundaryResult::Exempt(
            CountingExemption::NoDiscriminatingDate,
        ));
    }
    let ExprKind::Call {
        callee,
        args: trigger_args,
    } = &deadline.trigger.kind
    else {
        return Ok(BoundaryResult::Exempt(
            CountingExemption::NoDiscriminatingDate,
        ));
    };
    let owner = cx.owner(&deadline.meta.id).0;
    let clocks = Clocks::new(cx, &deadline.meta.effective.known_at);
    let (anchor, pkg, counting) = match counting_anchor(&clocks, owner, deadline) {
        Ok(anchor) => anchor,
        Err(AnchorError::DeadlinePredatesRule) => {
            return Ok(BoundaryResult::Exempt(
                CountingExemption::DeadlinePredatesRule,
            ))
        }
        Err(AnchorError::Unavailable(reason)) => return Err(reason),
    };
    let Some(event) = boundary_event(cx, callee, trigger_args, counting) else {
        return Ok(BoundaryResult::Exempt(
            CountingExemption::NoDiscriminatingDate,
        ));
    };
    let lib = clocks.get(pkg, counting)?;
    let rule_id = counting.meta.id.as_str();
    let rule = lib.clock(rule_id).map_err(|error| error.to_string())?;
    let unit = engine_unit(deadline.unit);
    let amount = i64::try_from(deadline.amount).map_err(|_| "deadline period is too large")?;
    let delta = if deadline.unit == hir::DayUnit::Week {
        amount.saturating_mul(7)
    } else {
        amount
    };
    let delta = delta.saturating_sub(i64::from(!counting.exclude_trigger_day));
    let start = clock::Date::parse_iso(&anchor).map_err(|e| e.to_string())?;
    let end = deadline
        .meta
        .effective
        .to
        .as_deref()
        .map(clock::Date::parse_iso)
        .transpose()
        .map_err(|e| e.to_string())?;
    let request = DayDeadline {
        trigger: start,
        days: amount,
        direction: if deadline.direction == hir::Direction::Forward {
            clock::Direction::Forward
        } else {
            clock::Direction::Backward
        },
        unit,
        service_method: None,
    };
    Ok(
        find_counting_boundary(deadline, &rule, &request, (start, end), delta, &event).map_or(
            BoundaryResult::Exempt(CountingExemption::NoDiscriminatingDate),
            BoundaryResult::Found,
        ),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CountingExemption {
    NoDiscriminatingDate,
    DeadlinePredatesRule,
}

pub(crate) enum BoundaryResult {
    Found((String, String)),
    Exempt(CountingExemption),
}

fn boundary_event(
    cx: &Context<'_>,
    callee: &str,
    trigger_args: &[Expr],
    counting: &hir::Counting,
) -> Option<String> {
    let args = cx
        .declarations()
        .filter_map(|d| match d {
            Declaration::Test(test) => Some(test.givens.iter().filter_map(|(given, _)| {
                match &given.kind {
                    ExprKind::Call {
                        callee: given_callee,
                        args,
                    } if given_callee == callee && trigger_arguments_match(trigger_args, args) => {
                        Some(args.as_slice())
                    }
                    _ => None,
                }
            })),
            _ => None,
        })
        .flatten()
        .find(|args| {
            args.iter().all(|arg| test_arg(arg).is_some())
                && service_method(Some(counting), args)
                    .map(|m| format!("{m:?}"))
                    .is_none_or(|m| {
                        !matches!(counting.service_extensions.get(&m), Some(hir::ServiceTime::Adds(n, _)) if *n > 0)
                    })
        })?;
    Some(format!(
        "{}({})",
        callee.rsplit("::").next().unwrap_or(callee),
        args.iter()
            .filter_map(test_arg)
            .collect::<Vec<_>>()
            .join(", ")
    ))
}

fn find_counting_boundary(
    deadline: &hir::Deadline,
    rule: &clock::Clock<'_>,
    request: &DayDeadline,
    window: (clock::Date, Option<clock::Date>),
    delta: i64,
    event: &str,
) -> Option<(String, String)> {
    let (start, end) = window;
    for offset in 0..=730 {
        let trigger = start.add_days(offset);
        if end.is_some_and(|end| trigger >= end) {
            break;
        }
        let naive = trigger.add_days(if deadline.direction == hir::Direction::Forward {
            delta
        } else {
            -delta
        });
        let legal_holiday = rule.is_legal_holiday(naive, &[])
            || (deadline.direction == hir::Direction::Forward && rule.is_forward_holiday(naive));
        if !naive.is_weekend() && !legal_holiday {
            continue;
        }
        let request = DayDeadline {
            trigger,
            ..request.clone()
        };
        let Ok(computed) = clock::compute_due_date(&request, rule, &ClockConfig::default()) else {
            continue;
        };
        if computed.due_date == naive.to_iso() {
            continue;
        }
        let boundary = if naive.is_weekend() {
            "weekend"
        } else {
            "holiday"
        };
        let cite = deadline
            .meta
            .cite
            .as_deref()
            .map(|c| format!(" cite {}", serde_json::to_string(c).unwrap_or_default()))
            .unwrap_or_default();
        let open = if cite.is_empty() { " {" } else { "\n{" };
        return Some((
            format!(
                "\n\ntest {}{}{}\n  given {} on {}\n  expect deadline({}) == {}\n}}",
                serde_json::to_string(&format!(
                    "Counting boundary: {} ({boundary})",
                    deadline.meta.id
                ))
                .unwrap_or_default(),
                cite,
                open,
                event,
                trigger.to_iso(),
                deadline.meta.id,
                computed.due_date
            ),
            format!("naive {} rolls to {}", naive.to_iso(), computed.due_date),
        ));
    }
    None
}

/// The first day on which both the deadline and its counting rule are in
/// force. A deadline can predate the first recorded version of its rule.
enum AnchorError {
    DeadlinePredatesRule,
    Unavailable(String),
}

fn counting_anchor<'a>(
    clocks: &Clocks<'a>,
    owner: &hir::Package,
    deadline: &hir::Deadline,
) -> Result<(String, &'a hir::Package, &'a hir::Counting), AnchorError> {
    let key = clocks
        .world
        .target(&owner.manifest.pack_id, &deadline.counting)
        .key;
    let versions = clocks.world.versions(&key);
    let deadline_start = &deadline.meta.effective.from;
    let deadline_end = deadline.meta.effective.to.as_deref();
    let mut candidates = std::collections::BTreeSet::from([deadline_start.clone()]);
    for version in &versions.versions {
        let start = version.in_force.from.clone().max(deadline_start.clone());
        if deadline_end.is_none_or(|end| start.as_str() < end) {
            candidates.insert(start);
        }
        if let Some(commenced) = &version.commenced {
            let start = commenced.from.clone().max(deadline_start.clone());
            if deadline_end.is_none_or(|end| start.as_str() < end) {
                candidates.insert(start);
            }
        }
    }
    for anchor in candidates {
        if deadline_end.is_some_and(|end| anchor.as_str() >= end) {
            continue;
        }
        if let Ok(Some((pkg, counting))) = counting_of(clocks, owner, deadline, &anchor) {
            return Ok((anchor, pkg, counting));
        }
    }
    let first_rule = versions
        .versions
        .iter()
        .filter(|version| version.held(Some(clocks.known_at)))
        .map(|version| version.in_force.from.as_str())
        .min();
    if deadline_end.is_some_and(|end| first_rule.is_some_and(|first| end <= first)) {
        return Err(AnchorError::DeadlinePredatesRule);
    }
    Err(AnchorError::Unavailable(format!(
        "no version of counting rule `{}` is in force during the deadline's effective window",
        deadline.counting
    )))
}

fn test_arg(arg: &Expr) -> Option<String> {
    match &arg.kind {
        ExprKind::Ref { target } => Some(target.rsplit("::").next().unwrap_or(target).to_owned()),
        ExprKind::Literal {
            value: Value::Variant(v),
        } => Some(v.rsplit("::").next().unwrap_or(v).to_owned()),
        _ => None,
    }
}

fn trigger_arguments_match(trigger: &[Expr], given: &[Expr]) -> bool {
    trigger.len() <= given.len() && trigger.iter().zip(given).all(|(a, b)| same_expr(a, b))
}

fn is_counting_insensitive(actual: &str, naive: &str, expected: &str) -> bool {
    actual == naive && actual == expected
}
