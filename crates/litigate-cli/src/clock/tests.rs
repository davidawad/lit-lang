#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::hir_with;
    use litigate_sema::hir::{ExprKind, Type, Value};

    /// A version recorded after the package's corpus date and one whose
    /// cite binds no source URL (L6001).
    const EXTRA: &str = "
counting late known_at 2026-01-01 cite \"FRCP 6(a)\"
{
  exclude_trigger_day
  include_last_day
  service_extension {
    Personal: 0.days
    Electronic: 0.days
    Mail: 3.days
    Clerk: unrecognized
    OtherConsented: unrecognized
    CommercialCarrier: unrecognized
    PrisonMail: unrecognized
    MailForeign: unrecognized
    Express: unrecognized
    ExpressForeign: unrecognized
  }
  holidays federal-holidays
}

counting nourl cite \"XYZ 6(a)\"
{
  exclude_trigger_day
  include_last_day
  holidays federal-holidays
}

deadline chained: 10.days from deadline(answer-period).nominal counted_by frcp6
deadline chained-adjusted: 10.days from deadline(chained).adjusted counted_by frcp6
deadline loop-a: 1.days from deadline(loop-b).nominal counted_by frcp6
deadline loop-b: 1.days from deadline(loop-a).nominal counted_by frcp6
deadline chained-early: 1.days from deadline(answer-period).nominal counted_by frcp6 known_at 2000-01-01
deadline nourl-base: 1.days from Served() counted_by nourl
deadline chained-nourl: 1.days from deadline(nourl-base).nominal counted_by frcp6
";

    fn expr(kind: ExprKind) -> Expr {
        Expr {
            ty: Type::Unit,
            kind,
            span: None,
        }
    }

    /// `Served(complaint, <method>)`.
    fn served(method: &str) -> Expr {
        let variant = |v: &str| {
            expr(ExprKind::Literal {
                value: Value::Variant(v.into()),
            })
        };
        expr(ExprKind::Call {
            callee: "Served".into(),
            args: vec![variant("Document::complaint"), variant(method)],
        })
    }

    /// The example package with `EXTRA`, its `answer-period` changed by
    /// `f`.
    fn package(f: &dyn Fn(&mut hir::Deadline)) -> hir::Package {
        let mut pkg = hir_with(EXTRA);
        for d in pkg.modules.iter_mut().flat_map(|m| &mut m.declarations) {
            if let Declaration::Deadline(d) = d {
                f(d);
            }
        }
        pkg
    }

    /// `answer-period` triggered on `date` (personal service), at
    /// `known_at`.
    fn due_on(
        pkg: &hir::Package,
        name: &str,
        date: &str,
        known_at: Option<&str>,
    ) -> Result<Due, String> {
        let action = served("ServiceMethod::Personal");
        let events = [MatterEvent::on(date, &action)];
        let clocks = Clocks::new(&[pkg]);
        let world = clocks.scope(&[&pkg.manifest.name]);
        due(&clocks, &world, pkg, name, &events, 0, known_at)
    }

    #[test]
    fn every_declared_version_lowers_or_says_why() {
        let mut pkg = hir_with(EXTRA);
        // Two versions of `frcp6`, the first counting rule, in one window:
        // neither is selected at its start, so neither lowers.
        let twice = pkg.modules[0]
            .declarations
            .iter()
            .find(|d| matches!(d, Declaration::Counting(_)))
            .cloned()
            .unwrap();
        pkg.modules[0].declarations.push(twice);
        let clocks = Clocks::new(&[&pkg]);
        let world = World::from_packages(&[&pkg]);
        let found: Vec<(&str, Result<usize, &str>)> = world
            .decls
            .iter()
            .filter(|d| matches!(d.decl, Declaration::Counting(_)))
            .map(|d| {
                (
                    d.meta.id.as_str(),
                    clocks
                        .fragment(d.decl)
                        .as_ref()
                        .map(|l| l.counting_rules.len())
                        .map_err(String::as_str),
                )
            })
            .collect();
        assert_eq!(clocks.fragments.len(), 4);
        assert_eq!(found.len(), 4, "{found:?}");
        assert_eq!(
            found[0],
            ("frcp6", Err("counting rule `frcp6` does not lower"))
        );
        // Lowered regardless of `known_at`: selection decides whether it
        // applies.
        assert_eq!(found[1], ("late", Ok(1)));
        assert_eq!(found[2].0, "nourl");
        assert!(
            found[2]
                .1
                .is_err_and(|e| e.starts_with("L6001: counting rule `nourl` needs a `cite`")),
            "{found:?}"
        );
    }

    #[test]
    fn only_calls_carry_a_service_method() {
        let literal = expr(ExprKind::Literal {
            value: Value::Variant("Mail".into()),
        });
        assert_eq!(method_of(&literal, None), None);
    }

    #[test]
    fn due_counts_under_the_versions_governing_the_trigger() {
        let date = |pkg: &hir::Package| {
            due_on(pkg, "answer-period", "2025-03-03", None).map(|d| d.computed.due_date)
        };
        let pkg = package(&|_| ());
        let found = due_on(&pkg, "answer-period", "2025-03-03", None).unwrap();
        assert_eq!(found.computed.due_date, "2025-03-24");
        assert_eq!(found.version, "answer-period@2009-12-01");
        assert_eq!(found.cite.as_deref(), Some("FRCP 6(a)"));
        assert_eq!(
            date(&package(&|d| d.unit = DayUnit::Court)),
            Ok("2025-04-01".to_owned())
        );
        let weeks = package(&|d| {
            d.amount = 3;
            d.unit = DayUnit::Week;
        });
        assert_eq!(date(&weeks), Ok("2025-03-24".to_owned()));
        // This package's rule declares no `end_of_month`.
        for unit in [DayUnit::Month, DayUnit::Year] {
            let months = package(&|d| d.unit = unit);
            assert!(date(&months).is_err_and(|e| e.contains("no calendar-month provision")));
        }
        let before = package(&|d| d.direction = hir::Direction::Backward);
        assert_eq!(date(&before), Ok("2025-02-10".to_owned()));
        // Mail service adds the counting rule's three days.
        let action = served("ServiceMethod::Mail");
        let events = [MatterEvent::on("2025-03-03", &action)];
        let clocks = Clocks::new(&[&pkg]);
        let mailed = due(
            &clocks,
            &clocks.scope(&[&pkg.manifest.name]),
            &pkg,
            "answer-period",
            &events,
            0,
            None,
        )
        .unwrap();
        assert_eq!(
            (mailed.computed.due_date.as_str(), mailed.method.as_deref()),
            ("2025-03-27", Some("Mail"))
        );
    }

    #[test]
    fn a_designated_due_date_is_the_trigger_argument_rolled() {
        let pkg = package(&|d| {
            d.designated = Some(hir::Designated {
                event: "Served".into(),
                param: 1,
                name: "due".into(),
                interpretation: None,
            });
        });
        let on = |designated: &str| {
            let action = expr(ExprKind::Call {
                callee: "Served".into(),
                args: vec![
                    expr(ExprKind::Literal {
                        value: Value::Variant("Document::complaint".into()),
                    }),
                    expr(ExprKind::Literal {
                        value: Value::Date(designated.into()),
                    }),
                ],
            });
            let events = [MatterEvent::on("2025-03-03", &action)];
            let clocks = Clocks::new(&[&pkg]);
            due(
                &clocks,
                &clocks.scope(&[&pkg.manifest.name]),
                &pkg,
                "answer-period",
                &events,
                0,
                None,
            )
            .map(|d| (d.designated, d.computed.due_date))
        };
        // A Saturday rolls to Monday; no service days are added.
        assert_eq!(
            on("2025-04-05"),
            Ok((Some("2025-04-05".to_owned()), "2025-04-07".to_owned()))
        );
        assert!(on("2025-03-10").is_err_and(|e| e.contains("earlier than the least time")));
    }

    #[test]
    fn known_at_hides_versions_recorded_later() {
        let pkg = package(&|d| d.counting = "late".into());
        assert!(due_on(&pkg, "answer-period", "2025-03-03", None).is_ok());
        assert!(due_on(&pkg, "answer-period", "2025-03-03", Some("2026-01-01")).is_ok());
        assert_eq!(
            due_on(&pkg, "answer-period", "2025-03-03", Some("2025-01-01")).map(|_| ()),
            Err("no version of counting rule `frcp-rule12-example::late` recorded by 2025-01-01 governs 2025-03-03".to_owned())
        );
        assert_eq!(
            due_on(&pkg, "answer-period", "2025-03-03", Some("1900-01-01")).map(|_| ()),
            Err("no version of deadline `frcp-rule12-example::answer-period` recorded by 1900-01-01 governs 2025-03-03".to_owned())
        );
    }

    #[test]
    fn unordered_claimants_govern_nothing() {
        // Two claimants displace `answer-period` and nothing orders them
        // (SPEC §5.5): no single declaration counts it.
        let mut pkg = hir_with(
            "deadline quick: 14.days from Served(complaint) counted_by frcp6 \
             notwithstanding answer-period",
        );
        let mut quicker = hirx::deadlines(&pkg, "quick").next().unwrap().clone();
        quicker.meta.id = "quicker".into();
        pkg.modules[0]
            .declarations
            .push(Declaration::Deadline(quicker));
        let why = due_on(&pkg, "answer-period", "2025-03-03", None).unwrap_err();
        assert!(
            why.starts_with(
                "no single declaration governs deadline `frcp-rule12-example::answer-period` on 2025-03-03: "
            ),
            "{why}"
        );
    }

    #[test]
    fn due_explains_every_refusal() {
        let refusal = |f: &dyn Fn(&mut hir::Deadline), name: &str, trigger: &str| {
            due_on(&package(f), name, trigger, None).unwrap_err()
        };
        let none = |_: &mut hir::Deadline| ();
        let name = "frcp-rule12-example";
        assert_eq!(
            refusal(&none, "nope", "2025-03-03"),
            format!("`nope` is not a deadline of `{name}`")
        );
        // A declaration of another kind is not a deadline either.
        assert_eq!(
            refusal(&none, "respond", "2025-03-03"),
            format!("`respond` is not a deadline of `{name}`")
        );
        assert_eq!(
            refusal(&none, "answer-period", "2000-01-01"),
            format!(
                "deadline `{name}::answer-period` is not encoded before 2009-12-01, so nothing governs 2000-01-01: {}",
                temporal::not_encoded("2009-12-01")
            )
        );
        // Business-day periods now lower through the shared clock library.
        let business = due_on(
            &package(&|d| d.unit = DayUnit::Business),
            "answer-period",
            "2025-03-03",
            None,
        )
        .unwrap();
        assert_eq!(business.computed.due_date, "2025-04-01");
        assert!(!refusal(&none, "answer-period", "2025-13-45").is_empty());
        assert!(!refusal(&|d| d.amount = u64::MAX, "answer-period", "2025-03-03").is_empty());
        assert!(refusal(
            &|d| d.counting = "nourl".into(),
            "answer-period",
            "2025-03-03"
        )
        .starts_with("L6001"));
        // No loaded package declares it: the shared library is asked, and
        // a node is no counting rule.
        for counting in ["no-such-rule", "respond"] {
            let why = refusal(
                &|d| d.counting = counting.into(),
                "answer-period",
                "2025-03-03",
            );
            assert!(!why.is_empty());
        }
        let too_short = |d: &mut hir::Deadline| {
            d.counting = "itc210".into();
            d.amount = 0;
        };
        assert_eq!(
            refusal(&too_short, "answer-period", "2025-03-03"),
            "invalid: 19 C.F.R. § 201.14(a): a period must be at least 1 day"
        );
    }

    /// Served 2025-03-03: `answer-period` ends 2025-03-24 (a Monday);
    /// `chained` runs 10 days from that nominal end, and `chained-adjusted`
    /// 10 days from `chained`'s adjusted end, Sunday April 13, to Monday.
    #[test]
    fn a_chained_period_counts_from_the_end_it_names() {
        let pkg = package(&|_| ());
        let date = |name: &str| due_on(&pkg, name, "2025-03-03", None).map(|d| d.computed.due_date);
        assert_eq!(date("chained"), Ok("2025-04-03".to_owned()));
        assert_eq!(date("chained-adjusted"), Ok("2025-04-14".to_owned()));
        let world = World::from_packages(&[&pkg]);
        let pack = &pkg.manifest.pack_id;
        let decl = |name: &str| hirx::deadlines(&pkg, name).next().unwrap();
        let root = root_trigger(&world, pack, decl("chained-adjusted"));
        assert_eq!(root, Some(&decl("answer-period").trigger));
        assert_eq!(root_trigger(&world, pack, decl("loop-a")), None);
        // A world without the base resolves no root.
        let empty = World::from_packages(&[]);
        assert_eq!(root_trigger(&empty, pack, decl("chained")), None);
    }

    /// A cycle, a base not recorded by `known_at`, and a base its clock
    /// cannot count: each refuses the chained period with the reason.
    #[test]
    fn a_chain_that_does_not_count_says_why() {
        let pkg = package(&|_| ());
        let why = |name: &str, known_at| due_on(&pkg, name, "2025-03-03", known_at).unwrap_err();
        assert!(why("loop-a", None).contains("is a cycle"));
        assert!(why("chained-early", Some("2001-01-01")).contains("answer-period"));
        assert!(why("chained-nourl", None).starts_with("L6001"));
    }

    #[test]
    fn methods_are_kebab_case() {
        assert_eq!(engine_method("Mail"), Some(ServiceMethod::Mail));
        assert_eq!(
            engine_method("OtherConsented"),
            Some(ServiceMethod::OtherConsented)
        );
        assert_eq!(engine_method("Personal"), Some(ServiceMethod::Personal));
        assert_eq!(engine_method("Fax"), None);
    }
}
