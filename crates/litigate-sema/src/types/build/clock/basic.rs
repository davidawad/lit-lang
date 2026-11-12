impl Cx<'_, '_> {
    /// Reports `r` as none of the `expected` keywords.
    pub(super) fn bad_counting(&mut self, r: &RefPath, expected: &[&str]) {
        let shown = r.display();
        let mut report = Report::new(
            C::InvalidCounting,
            r.range,
            format!("unknown counting keyword `{shown}`"),
        )
        .note(format!("expected one of {}", expected.join(", ")));
        if let Some(best) = suggest::nearest(&shown, expected.iter().copied()) {
            report = report.fix(r.range, best, Applicability::MaybeIncorrect);
        }
        self.report(report);
    }

    pub(super) fn keyword<T: Copy>(&mut self, r: &RefPath, table: &[(&str, T)]) -> Option<T> {
        let hit = table
            .iter()
            .find(|(k, _)| r.is_simple() && *k == r.head.text)
            .map(|(_, v)| *v);
        if hit.is_none() {
            self.bad_counting(r, &table.iter().map(|(k, _)| *k).collect::<Vec<_>>());
        }
        hit
    }

    pub(super) fn days(&mut self, e: &Expr) -> Option<(u64, DayUnit)> {
        match self.ck.infer(e).value {
            // Day amounts here are unsigned duration literals (SPEC §3.1),
            // so the conversion cannot fail.
            Some(Value::Days {
                amount,
                unit: unit @ (DayUnit::Calendar | DayUnit::Court | DayUnit::Business),
            }) => u64::try_from(amount).ok().map(|n| (n, unit)),
            Some(Value::Hours(_) | Value::Days { .. }) => {
                self.report(Report::new(
                    C::InvalidCounting,
                    e.range,
                    "this period must be counted in days",
                ));
                None
            }
            _ => {
                self.report(Report::new(
                    C::TypeMismatch,
                    e.range,
                    "expected a day count such as `3.days`",
                ));
                None
            }
        }
    }

    /// `deadline d: N.unit from|after|before T counted_by c`.
    #[allow(clippy::too_many_arguments)]
    pub fn deadline(
        &mut self,
        item: &Item,
        rule: Option<&Item>,
        name: &Name,
        amount: &Expr,
        direction: &Name,
        trigger: &Expr,
        counted_by: &RefPath,
        opts: &[Field],
    ) -> Option<Deadline> {
        let meta = self.meta(name, None, item, rule);
        let counting = self.value_ref(counted_by, "counting rule", |k| {
            matches!(k, ValueKind::Counting { .. })
        });
        let period = self.period(amount, counted_by, counting.is_some());
        let direction = match direction.text.as_str() {
            "before" => Direction::Backward,
            _ => Direction::Forward,
        };
        let o = self.once_except(opts, &["extend", "derived"]);
        let mut designated = first_ref(o.get("designated"))
            .and_then(|r| self.designated(name, trigger, r, (direction, period.map(|p| p.1))));
        self.interpretation(name, &o, designated.as_mut());
        let (trigger, chain) = self.trigger(name, trigger);
        let forum = self.forum(name, o.get("forum"), counted_by, counting.is_some());
        let adoption = o.get("forum").and_then(|f| self.adoption(f));
        let obligation = self.obligation(name, opts, &o, rule);
        let characterization = self.characterized(item, opts, &o);
        let derived = self.derivation(name, opts, period);
        let (amount, unit) = period?;
        Some(Deadline {
            meta,
            amount,
            unit,
            direction,
            trigger: trigger?,
            chain,
            counting: counting?,
            forum,
            adoption,
            extendable: flag(o.get("extendable")),
            extension_authority: self.string_or_hole(o.get("extension_authority")),
            note: Self::string(o.get("note")),
            bearer: obligation.bearer,
            bearer_other: obligation.bearer_other,
            policy: obligation.policy,
            linear: obligation.linear,
            extend: obligation.extend,
            waive: obligation.waive,
            designated,
            characterization,
            derived,
        })
    }

    /// Deadline `amount`'s period, checked against counting rule
    /// `counted_by` (when it resolved, `counted`): a month or year period
    /// needs `end_of_month`, an hours period `allow_hours`.
    fn period(
        &mut self,
        amount: &Expr,
        counted_by: &RefPath,
        counted: bool,
    ) -> Option<(u64, DayUnit)> {
        // The rule's signature carries `allow_hours` and `end_of_month`,
        // local or imported.
        let lacks = |cx: &Self, key: fn(&ValueKind) -> bool| {
            counted
                && cx
                    .ck
                    .scope
                    .value(counted_by)
                    .is_ok_and(|(_, found)| !key(&found.item.kind))
        };
        match self.ck.infer(amount).value {
            Some(Value::Days { amount: n, unit }) => {
                let refused = unit.is_monthly()
                    && lacks(self, |k| {
                        matches!(
                            k,
                            ValueKind::Counting {
                                end_of_month: true,
                                ..
                            }
                        )
                    });
                if refused {
                    self.report(Report::new(
                        C::InvalidCounting,
                        amount.range,
                        "a month or year deadline needs a counting rule with `end_of_month`",
                    ));
                }
                u64::try_from(n).ok().map(|n| (n, unit))
            }
            Some(Value::Hours(h)) => {
                let refused = lacks(self, |k| {
                    matches!(k, ValueKind::Counting { hours: true, .. })
                });
                self.hours(h, amount, refused)
            }
            _ => {
                self.report(Report::new(
                    C::TypeMismatch,
                    amount.range,
                    "a deadline period is a duration such as `21.days`",
                ));
                None
            }
        }
    }

    /// The period of an `N.hours` deadline: whole hours, under a counting
    /// rule whose signature carries `allow_hours` (L3009 when `refused`).
    fn hours(&mut self, h: Rational, amount: &Expr, refused: bool) -> Option<(u64, DayUnit)> {
        let whole = (h.denominator == 1)
            .then(|| u64::try_from(h.numerator).ok())
            .flatten();
        if refused {
            self.report(Report::new(
                C::InvalidCounting,
                amount.range,
                "an hours deadline needs a counting rule with `allow_hours`",
            ));
        } else if whole.is_none() {
            self.report(Report::new(
                C::TypeMismatch,
                amount.range,
                "an hours deadline counts whole hours, such as `24.hours`",
            ));
        }
        whole.map(|n| (n, DayUnit::Hour))
    }

    /// The terms of deadline `name`'s period (`derived N.unit cite "X"
    /// [quote "..."]`, once per term, SPEC §5.3). They sum to `period`, in
    /// its unit or one that converts to it exactly (weeks to calendar
    /// days, years to months): L2007 otherwise, and a quote beside a cite
    /// hole, which has no text to quote, is L2007 too.
    fn derivation(
        &mut self,
        name: &Name,
        opts: &[Field],
        period: Option<(u64, DayUnit)>,
    ) -> Vec<Derivation> {
        let fields: Vec<&Field> = opts.iter().filter(|f| f.key.text == "derived").collect();
        let mut terms = Vec::new();
        for f in &fields {
            if let Some(term) = self.derived_term(f) {
                terms.push(term);
            }
        }
        let Some((amount, unit)) = period.filter(|_| terms.len() == fields.len()) else {
            return terms;
        };
        let base = |n: u64, u: DayUnit| match u {
            DayUnit::Week => (n * 7, DayUnit::Calendar),
            DayUnit::Year => (n * 12, DayUnit::Month),
            u => (n, u),
        };
        let (want, measure) = base(amount, unit);
        let sum = terms.iter().try_fold(0, |sum, t| {
            let (n, u) = base(t.amount, t.unit);
            (u == measure).then_some(sum + n)
        });
        if !terms.is_empty() && sum != Some(want) {
            let shown = terms
                .iter()
                .map(|t| t.unit.prose(t.amount))
                .collect::<Vec<_>>()
                .join(" + ");
            self.report(Report::new(
                C::InvalidField,
                fields[0].range,
                format!(
                    "the derived terms of `{}` ({shown}) do not sum to its period of {}",
                    name.text,
                    unit.prose(amount)
                ),
            ));
        }
        terms
    }

    /// One `derived N.unit cite "X" [quote "..."]` term, or `None` after
    /// reporting why it is malformed.
    fn derived_term(&mut self, f: &Field) -> Option<Derivation> {
        let e = first_expr(f)?;
        let Some(Value::Days { amount, unit }) = self.ck.infer(e).value else {
            self.report(Report::new(
                C::TypeMismatch,
                e.range,
                "a derived term is a duration such as `1.months`",
            ));
            return None;
        };
        let amount = u64::try_from(amount).ok()?;
        let rest = Field {
            key: f.key.clone(),
            values: f.values.iter().skip(1).cloned().collect(),
            range: f.range,
        };
        let cite = first_expr(&rest)?;
        let hole = matches!(cite.kind, ExprKind::Hole(_));
        let historical = matches!(cite.kind, ExprKind::Cite(_, true));
        let quote = match rest.values.get(1) {
            Some((
                Atom::Expr(Expr {
                    kind: ExprKind::Str(q),
                    range,
                }),
                _,
            )) => Some((q.clone(), *range)),
            _ => None,
        };
        if let (true, Some((_, range))) = (hole, &quote) {
            self.report(Report::new(
                C::InvalidField,
                *range,
                "a `quote` quotes the text of its cite, and a cite hole has none",
            ));
            return None;
        }
        let cite = self.cite_text(Some(&&rest))?;
        Some(Derivation {
            amount,
            unit,
            cite,
            historical,
            quote: quote.map(|(q, _)| q),
        })
    }

    /// Deadline `name`'s forum, its `forum F` option's or its package's;
    /// a resolved counting rule must compute that forum's periods.
    fn forum(
        &mut self,
        name: &Name,
        field: Option<&&Field>,
        counted_by: &RefPath,
        counted: bool,
    ) -> String {
        let forum = first_ref(field).map_or_else(
            || self.manifest.map(|m| m.forum.clone()).unwrap_or_default(),
            RefPath::display,
        );
        if counted {
            self.borrowing(name, counted_by, &forum);
        }
        forum
    }

    /// The adopting rule of `forum F cite "X" [quote "..."]`. A cite hole
    /// is recorded as a hole; a quote needs a cite whose text it quotes,
    /// so one beside a hole is L2007.
    fn adoption(&mut self, field: &Field) -> Option<Adoption> {
        let rest = Field {
            key: field.key.clone(),
            values: field.values.iter().skip(1).cloned().collect(),
            range: field.range,
        };
        let cite = first_expr(&rest)?;
        let hole = matches!(cite.kind, ExprKind::Hole(_));
        let historical = matches!(cite.kind, ExprKind::Cite(_, true));
        let quote = match rest.values.get(1) {
            Some((
                Atom::Expr(Expr {
                    kind: ExprKind::Str(q),
                    range,
                }),
                _,
            )) => Some((q.clone(), *range)),
            _ => None,
        };
        let cite = self.cite_text(Some(&&rest));
        if let (true, Some((_, range))) = (hole, &quote) {
            self.report(Report::new(
                C::InvalidField,
                *range,
                "a `quote` quotes the text of its cite, and a cite hole has none",
            ));
            return None;
        }
        Some(Adoption {
            cite: cite?,
            historical,
            quote: quote.map(|(q, _)| q),
        })
    }

    /// `interpretation "..."` of deadline `name`, qualifying its
    /// `designated` least time (SPEC §5.3): L2007 without one.
    fn interpretation(
        &mut self,
        name: &Name,
        o: &BTreeMap<&str, &Field>,
        designated: Option<&mut Designated>,
    ) {
        let Some(f) = o.get("interpretation") else {
            return;
        };
        match designated {
            Some(d) => d.interpretation = Self::string(Some(f)),
            // A designation already reported leaves nothing to qualify.
            None if o.contains_key("designated") => {}
            None => self.report(Report::new(
                C::InvalidField,
                f.range,
                format!(
                    "`interpretation` qualifies the least time of a designated due date, and `{}` designates none",
                    name.text
                ),
            )),
        }
    }

    /// The deadline's `characterization` (options `opts`, indexed `o`),
    /// and the capabilities a jurisdictional one forbids (L2009).
    fn characterized(
        &mut self,
        item: &Item,
        opts: &[Field],
        o: &BTreeMap<&str, &Field>,
    ) -> Option<crate::hir::Characterization> {
        let characterization = o
            .get("characterization")
            .and_then(|f| self.characterization(f));
        if characterization
            .as_ref()
            .is_some_and(|c| c.kind == CharacterKind::Jurisdictional)
        {
            self.jurisdictional(clauses_of(item), opts);
        }
        characterization
    }

    /// `designated p` of deadline `name`: `p` is a `Date` parameter of the
    /// event `trigger` matches, and the deadline counts forward in calendar
    /// days (its least time, SPEC §5.3).
    fn designated(
        &mut self,
        name: &Name,
        trigger: &Expr,
        r: &RefPath,
        (direction, unit): (Direction, Option<DayUnit>),
    ) -> Option<Designated> {
        if unit.is_some_and(|u| u != DayUnit::Calendar) {
            self.report(Report::new(
                C::InvalidField,
                r.range,
                format!(
                    "the least time a designated due date of `{}` allows is counted in calendar days",
                    name.text
                ),
            ));
            return None;
        }
        if direction == Direction::Backward {
            self.report(Report::new(
                C::InvalidField,
                r.range,
                format!(
                    "deadline `{}` counts backward: only a forward period's due date is designated",
                    name.text
                ),
            ));
            return None;
        }
        let ExprKind::Call(callee, _) = &trigger.kind else {
            return None;
        };
        // The trigger pattern itself reports a callee that is no event.
        let (event, found) = self.ck.scope.user_type(&callee.head, &[]).ok()?;
        let TypeSig::Event(params) = found.item else {
            return None;
        };
        let Some(param) = params
            .iter()
            .position(|(p, _)| r.is_simple() && *p == r.head.text)
        else {
            self.report(Report::new(
                C::UnresolvedName,
                r.range,
                format!(
                    "the trigger `{event}` of deadline `{}` has no parameter `{}`",
                    name.text,
                    r.display()
                ),
            ));
            return None;
        };
        if params[param].1 != Some(Type::Date) {
            self.report(Report::new(
                C::TypeMismatch,
                r.range,
                format!(
                    "parameter `{}` of `{event}` designates a due date, so it must have type `Date`",
                    params[param].0
                ),
            ));
            return None;
        }
        Some(Designated {
            event: event.clone(),
            param,
            name: params[param].0.clone(),
            interpretation: None,
        })
    }

}
