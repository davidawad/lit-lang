impl Cx<'_, '_> {
    /// A deadline's trigger: an event pattern, or the end of another
    /// deadline's period, `deadline(d).nominal` or `deadline(d).adjusted`.
    /// A chain must say which end (design review 2026-10-23, §2.1): a bare
    /// `deadline(d)` is L2001 with both readings as fixes, and a period
    /// cannot run from its own end.
    fn trigger(&mut self, name: &Name, x: &Expr) -> (Option<crate::hir::Expr>, Option<Chain>) {
        if let Some(args) = deadline_call(x) {
            let text = |end: &str| format!("deadline({}).{end}", args_text(args));
            self.report(
                Report::new(
                    C::TypeMismatch,
                    x.range,
                    "a chained period says which end it runs from: `.nominal` (the last day as \
                     counted) or `.adjusted` (after every roll and extension)",
                )
                .fix(x.range, text("nominal"), Applicability::MaybeIncorrect)
                .fix(x.range, text("adjusted"), Applicability::MaybeIncorrect),
            );
            return (None, None);
        }
        let ExprKind::Field(base, end) = &x.kind else {
            return (self.pattern(x), None);
        };
        let Some(args) = deadline_call(base) else {
            return (self.pattern(x), None);
        };
        let end = self.due_end(end);
        let target = self.chain_target(name, base, args);
        let (Some(end), Some(deadline)) = (end, target) else {
            return (None, None);
        };
        (
            Some(chain_expr(&deadline, end)),
            Some(Chain { deadline, end }),
        )
    }

    /// The end `.nominal` or `.adjusted` names; L2001 for any other field.
    fn due_end(&mut self, end: &Name) -> Option<DueEnd> {
        match end.text.as_str() {
            "nominal" => Some(DueEnd::Nominal),
            "adjusted" => Some(DueEnd::Adjusted),
            other => {
                let mut report = Report::new(
                    C::TypeMismatch,
                    end.range,
                    format!("a period ends `nominal` or `adjusted`, not `{other}`"),
                );
                if let Some(best) = suggest::nearest(other, ["nominal", "adjusted"]) {
                    report = report.fix(end.range, best, Applicability::MaybeIncorrect);
                }
                self.report(report);
                None
            }
        }
    }

    /// The deadline `deadline(...)` (`base`, with `args`) names for
    /// deadline `name`'s chain: exactly one, and not `name` itself.
    fn chain_target(&mut self, name: &Name, base: &Expr, args: &[Expr]) -> Option<String> {
        let [Expr {
            kind: ExprKind::Ref(r),
            ..
        }] = args
        else {
            self.report(Report::new(
                C::TypeMismatch,
                base.range,
                "`deadline(d)` names exactly one deadline",
            ));
            return None;
        };
        if r.display() == name.text {
            self.report(Report::new(
                C::TypeMismatch,
                r.range,
                format!("`{}` cannot run from the end of its own period", name.text),
            ));
            return None;
        }
        self.value_ref(r, "deadline", |k| matches!(k, ValueKind::Deadline))
    }

    /// L3009 when deadline `name`, a period of `forum`'s rules, is counted
    /// by `counted_by`, a rule computing other forums' periods (design
    /// review 2026-10-23, L2-L4: no cross-forum borrowing). A rule computes
    /// the periods of every forum in the family of a forum it names: FRCP
    /// 6(a) and FRAP 26(a) count "any local rule or court order", so a
    /// district's or a circuit's own periods are its own forum's.
    fn borrowing(&mut self, name: &Name, counted_by: &RefPath, forum: &str) {
        let family = crate::change::family_of(self.families, forum);
        let computes = |forums: &std::collections::BTreeSet<String>| {
            forums.contains(forum)
                || family.is_some_and(|f| f.forums.iter().any(|x| forums.contains(x)))
        };
        // `counted_by` resolved to a counting rule before this is asked.
        let borrowed =
            self.ck
                .scope
                .value(counted_by)
                .ok()
                .and_then(|(_, found)| match &found.item.kind {
                    ValueKind::Counting { forums, .. } if !computes(forums) => Some(forums.clone()),
                    _ => None,
                });
        let Some(forums) = borrowed else {
            return;
        };
        let list = forums.iter().cloned().collect::<Vec<_>>().join(", ");
        self.report(
            Report::new(
                C::InvalidCounting,
                counted_by.range,
                format!(
                    "deadline `{}` is a period of forum `{forum}`, but counting rule `{}` \
                     computes periods of {list}",
                    name.text,
                    counted_by.display()
                ),
            )
            .note(
                "a period is counted by the rule of the forum whose rules set it; count it by \
                 that forum's rule, or name that forum with `{ forum F }`",
            ),
        );
    }

    /// `counting c { ... }`.
    pub fn counting(
        &mut self,
        item: &Item,
        rule: Option<&Item>,
        name: &Name,
        items: &[Field],
    ) -> Option<Counting> {
        let meta = self.meta(name, None, item, rule);
        let f = self.once(items);
        let roll_on = Self::refs(f.get("roll_forward_on"))
            .into_iter()
            .filter_map(|r| {
                self.keyword(
                    r,
                    &[
                        ("weekend", RollOn::Weekend),
                        ("sunday", RollOn::Sunday),
                        ("legal_holiday", RollOn::LegalHoliday),
                        ("inaccessible", RollOn::Inaccessible),
                    ],
                )
            })
            .collect::<Vec<_>>();
        let service_extensions = self.service_extensions(f.get("service_extension"));
        let forward_calendar = self.closing_calendar(f.get("forward_holidays"), false);
        let declared_calendar = self.closing_calendar(f.get("declared_holidays"), true);
        let forums = match f.get("forums") {
            Some(field) => Self::refs(Some(field))
                .into_iter()
                .map(RefPath::display)
                .collect(),
            None => vec![self.manifest.map(|m| m.forum.clone()).unwrap_or_default()],
        };
        let calendar = if let Some(r) = first_ref(f.get("holidays")) {
            self.value_ref(r, "calendar", |k| matches!(k, ValueKind::Calendar(_)))
        } else {
            self.report(Report::new(
                C::InvalidCounting,
                name.range,
                format!("counting rule `{}` needs `holidays <calendar>`", name.text),
            ));
            None
        };
        let anchor = if f.contains_key("anchor") {
            first_ref(f.get("anchor")).and_then(|r| {
                self.keyword(
                    r,
                    &[
                        ("trigger_day", Anchor::TriggerDay),
                        ("next_business_day", Anchor::NextBusinessDay),
                    ],
                )
            })
        } else {
            Some(Anchor::TriggerDay)
        };
        let short_period = self.short_period(f.get("short_period"));
        let end_of_month = first_ref(f.get("end_of_month")).and_then(|r| {
            self.keyword(
                r,
                &[("clamp", EndOfMonth::Clamp), ("roll", EndOfMonth::Roll)],
            )
        });
        Some(Counting {
            meta,
            exclude_trigger_day: f.contains_key("exclude_trigger_day"),
            include_last_day: f.contains_key("include_last_day"),
            roll_on,
            service_extensions,
            calendar: calendar?,
            forward_calendar,
            declared_calendar,
            forums,
            anchor: anchor?,
            short_period,
            hours: f.contains_key("allow_hours"),
            end_of_month,
        })
    }

}
