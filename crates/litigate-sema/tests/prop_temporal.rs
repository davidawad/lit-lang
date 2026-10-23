//! A matter beginning on an amendment's effective date is not pending across it.
mod legal;

use legal::{pkg, with_world, workspace};
use litigate_sema::hir::{Expr, ExprKind, Type};
use litigate_sema::temporal::{governing, Key, MatterEvent};
use proptest::prelude::*;

const RULES: &str = "module demo.rules
start s
enum Document { complaint }
event Served(document: Document)
counting c6 { exclude_trigger_day, include_last_day, holidays cal }
calendar cal { new-year: fixed(1, 1) effective [1870-06-28, ..) }
state s: \"S\" { _ \"go\" -> t }
terminal t: \"T\"
deadline p: 20.days from Served(complaint) counted_by c6 effective [2009-12-01, 2016-12-01)
deadline p: 21.days from Served(complaint) counted_by c6 effective [2016-12-01, ..)
transition amend amends p@2016-12-01 { otherwise -> p@2016-12-01 }
";

proptest! {
    #[test]
    fn prop_starting_on_amendment_date_is_not_pending(day in 1_u32..=28) {
        let package = pkg("demo", "FederalRule", "2009-12-01", &[], &[("rules.lit", RULES)]);
        let (db, ws) = workspace(&[package]);
        let key = Key { pack: "demo".into(), name: "p".into() };
        let date = format!("2016-12-{day:02}");
        let action = Expr {
            ty: Type::Unit,
            kind: ExprKind::Call { callee: "Served".into(), args: Vec::new() },
            span: None,
        };
        let events = [MatterEvent { date: &date, action: &action }];
        with_world(&db, ws, |world| {
            let result = governing(world, &key, &events, 0, None);
            prop_assert!(result.is_ok(), "{result:?}");
            let governing = result.unwrap_or_else(|error| panic!("{error:?}"));
            prop_assert_eq!(governing.via, None);
            prop_assert_eq!(
                &world.decls[governing.version].meta.effective.from,
                "2016-12-01"
            );
            Ok(())
        })?;
    }
}
