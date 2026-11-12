#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn method_names_map_to_engine_spellings() {
        assert_eq!(kebab("Mail"), "mail");
        assert_eq!(kebab("OtherConsented"), "other-consented");
    }

    #[test]
    fn date_order_comparisons_are_iso_ordered() {
        let a = Value::Date("2025-03-24".into());
        let b = Value::Date("2025-03-27".into());
        assert_eq!(compare(&a, BinaryOp::Lt, &b), Some(true));
        assert_eq!(compare(&a, BinaryOp::Le, &a), Some(true));
        assert_eq!(compare(&a, BinaryOp::Gt, &b), Some(false));
        assert_eq!(compare(&a, BinaryOp::Ge, &b), Some(false));
        assert_eq!(compare(&a, BinaryOp::Eq, &a), Some(true));
        assert_eq!(compare(&a, BinaryOp::Ne, &b), Some(true));
        assert_eq!(compare(&Value::Boolean(true), BinaryOp::Lt, &a), None);
    }

    #[test]
    fn only_comparisons_compare_dates() {
        let a = Value::Date("2025-03-24".into());
        for o in [BinaryOp::Add, BinaryOp::Sub, BinaryOp::And, BinaryOp::Or] {
            assert_eq!(compare(&a, o, &a), None, "{o:?}");
        }
    }

    #[test]
    fn operators_render_as_written() {
        let all = [
            BinaryOp::Eq,
            BinaryOp::Ne,
            BinaryOp::Lt,
            BinaryOp::Le,
            BinaryOp::Gt,
            BinaryOp::Ge,
            BinaryOp::Mul,
        ];
        let shown: Vec<&str> = all.into_iter().map(op).collect();
        assert_eq!(shown, ["==", "!=", "<", "<=", ">", ">=", "?"]);
    }

    fn expr(kind: ExprKind) -> Expr {
        Expr {
            ty: hir::Type::Unit,
            kind,
            span: None,
        }
    }

    fn variant(v: &str) -> Expr {
        expr(ExprKind::Literal {
            value: Value::Variant(v.to_owned()),
        })
    }

    fn call(callee: &str, args: &[&str]) -> Expr {
        expr(ExprKind::Call {
            callee: callee.to_owned(),
            args: args.iter().map(|a| variant(a)).collect(),
        })
    }

    #[test]
    fn a_pattern_matches_its_constructor_and_leading_arguments() {
        let event = call("Served", &["Document::complaint", "Method::Mail"]);
        assert!(matches(&call("Served", &[]), &event));
        assert!(matches(&call("Served", &["Document::complaint"]), &event));
        assert!(matches(&event, &event));
        assert!(!matches(
            &call("Answered", &["Document::complaint"]),
            &event
        ));
        assert!(!matches(&call("Served", &["Document::reply"]), &event));
        let longer = call("Served", &["Document::complaint", "Method::Mail", "X::y"]);
        assert!(!matches(&longer, &event));
        let literal = variant("Document::complaint");
        assert!(!matches(&literal, &event));
        assert!(!matches(&event, &literal));
    }

    #[test]
    fn boundary_candidates_match_trigger_arguments() {
        let trigger = call("InitialAppearance", &["Custody::Released"]);
        let matching = call("InitialAppearance", &["Custody::Released", "Extra::field"]);
        let unrelated = call("InitialAppearance", &["Custody::InCustody"]);
        let ExprKind::Call { args, .. } = &trigger.kind else {
            unreachable!();
        };
        let ExprKind::Call { args: matching, .. } = &matching.kind else {
            unreachable!();
        };
        let ExprKind::Call {
            args: unrelated, ..
        } = &unrelated.kind
        else {
            unreachable!();
        };
        assert!(trigger_arguments_match(args, matching));
        assert!(!trigger_arguments_match(args, unrelated));
    }

    #[test]
    fn values_render_plainly_and_others_as_json() {
        assert_eq!(render(&Value::Date("2025-03-24".into())), "2025-03-24");
        assert_eq!(render(&Value::Boolean(false)), "false");
        assert_eq!(
            render(&Value::Variant("Document::complaint".into())),
            r#"{"kind":"variant","value":"Document::complaint"}"#
        );
    }

    #[test]
    fn a_call_shows_its_reference_or_its_callee() {
        let v = Value::Boolean(true);
        let target = expr(ExprKind::Ref {
            target: "answer-period".into(),
        });
        let deadline = expr(ExprKind::Call {
            callee: "deadline".into(),
            args: vec![target],
        });
        assert_eq!(show(&deadline, &v), "deadline(answer-period)");
        assert_eq!(show(&call("has_event", &["E::x"]), &v), "has_event");
        assert_eq!(show(&call("has_event", &[]), &v), "has_event");
        assert_eq!(show(&variant("E::x"), &v), "true");
    }

    #[test]
    fn plain_weekday_arithmetic_is_insensitive_but_a_rolled_boundary_is_not() {
        assert!(is_counting_insensitive(
            "2027-01-11",
            "2027-01-11",
            "2027-01-11"
        ));
        assert!(!is_counting_insensitive(
            "2027-01-11",
            "2027-01-09",
            "2027-01-11"
        ));
    }
}
