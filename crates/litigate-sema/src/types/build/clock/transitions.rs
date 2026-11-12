impl Cx<'_, '_> {
    /// `service_extension { Method: N.days | unrecognized, ... }`, total
    /// over [`SERVICE_METHODS`] (L3009 otherwise). A rule without one names
    /// no service method: every method is unrecognized, so a period served
    /// by any of them is refused, never counted as if it added nothing.
    fn service_extensions(&mut self, f: Option<&&Field>) -> BTreeMap<String, ServiceTime> {
        let mut out = BTreeMap::new();
        let Some(field) = f else {
            return SERVICE_METHODS
                .iter()
                .map(|m| ((*m).to_owned(), ServiceTime::Unrecognized))
                .collect();
        };
        let Some(ExprKind::Record(methods)) = first_expr(field).map(|e| &e.kind) else {
            return out;
        };
        for (method, e) in methods {
            if !SERVICE_METHODS.contains(&method.text.as_str()) {
                let mut report = Report::new(
                    C::InvalidCounting,
                    method.range,
                    format!("`{}` is not a service method", method.text),
                )
                .note(format!(
                    "the service methods are {}",
                    SERVICE_METHODS.join(", ")
                ));
                if let Some(best) = suggest::nearest(&method.text, SERVICE_METHODS) {
                    report = report.fix(method.range, best, Applicability::MaybeIncorrect);
                }
                self.report(report);
                continue;
            }
            let time = match &e.kind {
                ExprKind::Ref(r) if r.is_simple() && r.head.text == "unrecognized" => {
                    Some(ServiceTime::Unrecognized)
                }
                ExprKind::Ref(r) => {
                    self.bad_counting(r, &["unrecognized"]);
                    None
                }
                _ => self.days(e).map(|(n, unit)| ServiceTime::Adds(n, unit)),
            };
            if let Some(time) = time {
                if out.insert(method.text.clone(), time).is_some() {
                    self.report(Report::new(
                        C::InvalidCounting,
                        method.range,
                        format!("service method `{}` is listed twice", method.text),
                    ));
                }
            }
        }
        let missing: Vec<&str> = SERVICE_METHODS
            .iter()
            .copied()
            .filter(|m| !methods.iter().any(|(k, _)| k.text == *m))
            .collect();
        if !missing.is_empty() {
            self.report(
                Report::new(
                    C::InvalidCounting,
                    field.key.range,
                    format!(
                        "`service_extension` is not total: it does not say what {} add",
                        missing.join(", ")
                    ),
                )
                .note("give each `N.days` (`0.days`: it adds none) or `unrecognized`"),
            );
        }
        out
    }

    /// The calendar of `forward_holidays C cite X` or `declared_holidays C
    /// cite X`: `X` is the rule that makes `C`'s days ones the forum closes
    /// on, and is required. A `declared_holidays` calendar lists declared
    /// days: each of its holidays is a `once(date)` with no observance.
    fn closing_calendar(&mut self, f: Option<&&Field>, declared: bool) -> Option<String> {
        let field = f?;
        let cited = field.values.iter().any(|(a, _)| {
            matches!(
                a,
                Atom::Expr(Expr {
                    kind: ExprKind::Cite(..) | ExprKind::Hole(_),
                    ..
                })
            )
        });
        if !cited {
            self.report(
                Report::new(
                    C::InvalidCounting,
                    field.key.range,
                    format!("`{}` needs the `cite` of the rule closing on it", field.key.text),
                )
                .note(if declared {
                    "whether a declared day closes the forum is a legal question its rule answers (FRCP 6(a)(6)(B))"
                } else {
                    "name the rule closing these days for forward periods (FRCP 6(a)(6)(C))"
                }),
            );
        }
        let r = first_ref(f)?;
        let id = self.value_ref(r, "calendar", |k| matches!(k, ValueKind::Calendar(_)))?;
        let lists_days = matches!(
            self.ck.scope.value(r),
            Ok((_, found)) if found.item.kind == ValueKind::Calendar(true)
        );
        if declared && !lists_days {
            self.report(
                Report::new(
                    C::InvalidCounting,
                    r.range,
                    format!("`declared_holidays {}` names a calendar of rules, not of declared days", r.display()),
                )
                .note("a declared-closure calendar lists each declared day as `once(date)`, with no `observed`"),
            );
        }
        Some(id)
    }

    fn short_period(&mut self, f: Option<&&Field>) -> Option<ShortPeriod> {
        match f.and_then(|x| first_expr(x)).map(|e| (&e.kind, e.range)) {
            Some((ExprKind::Record(parts), range)) => {
                let below = parts
                    .iter()
                    .find(|(k, _)| k.text == "below")
                    .and_then(|(_, e)| self.days(e));
                let count = match parts.iter().find(|(k, _)| k.text == "count") {
                    Some((
                        _,
                        Expr {
                            kind: ExprKind::Ref(r),
                            ..
                        },
                    )) => self.keyword(
                        r,
                        &[
                            ("calendar_days", DayUnit::Calendar),
                            ("court_days", DayUnit::Court),
                            ("business_days", DayUnit::Business),
                        ],
                    ),
                    _ => None,
                };
                if parts.len() != 2 || below.is_none() || count.is_none() {
                    self.report(Report::new(
                        C::InvalidCounting,
                        range,
                        "`short_period` is `{ below: N.days, count: <unit> }`",
                    ));
                }
                below
                    .zip(count)
                    .map(|((threshold, _), unit)| ShortPeriod { threshold, unit })
            }
            _ => None,
        }
    }

    /// `transition t amends d@D { unless C -> v, otherwise -> w }`.
    pub fn transition(
        &mut self,
        item: &Item,
        rule: Option<&Item>,
        name: &Name,
        amends: &RefPath,
        arms: &[ArmItem],
    ) -> Option<Transition> {
        let meta = self.meta(name, None, item, rule);
        let versioned = |cx: &mut Self, r: &RefPath| {
            if r.version.is_none() {
                cx.report(Report::new(
                    C::InvalidTransition,
                    r.range,
                    format!(
                        "`{}` must name a version (`{}@YYYY-MM-DD`)",
                        r.display(),
                        r.display()
                    ),
                ));
                return None;
            }
            cx.value_ref(r, "declaration", |_| true)
        };
        let amendment = versioned(self, amends);
        let mut out = Vec::new();
        for (i, arm) in arms.iter().enumerate() {
            let last = i + 1 == arms.len();
            if arm.unless.is_none() != last {
                let message = if last {
                    "the last arm must be `otherwise -> <version>`"
                } else {
                    "`otherwise` must be the last arm"
                };
                self.report(Report::new(C::InvalidTransition, arm.range, message));
            }
            let unless = arm.unless.as_ref().map(|c| self.ck.condition(c).hir);
            if let Some(selects) = versioned(self, &arm.target) {
                out.push(TransitionArm { unless, selects });
            }
        }
        if arms.is_empty() {
            self.report(Report::new(
                C::InvalidTransition,
                name.range,
                "a transition needs at least an `otherwise` arm",
            ));
        }
        Some(Transition {
            meta,
            amendment: amendment?,
            arms: out,
        })
    }
}
