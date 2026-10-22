//! Everything type-correct lowers to HIR: versions, transitions, authority,
//! defaults, exceptions, links, and instances, from real `.lit` sources.
mod common;
use common::{package, render, workspace};

const RULES: &str = r#"module rules
start pending
enum Document { complaint }
enum Method { Personal, Mail }
event Served(document: Document, method: Method)
event Answered(document: Document)
authority Statute > FederalRule > LocalRule
counting c6 { exclude_trigger_day, include_last_day, roll_forward_on [weekend], holidays cal }
calendar cal { new-year: fixed(1, 1) effective [1870-06-28, ..) observed nearest_weekday }
deadline answer: 20.days from Served(complaint) counted_by c6 effective [2009-12-01, 2024-12-01)
deadline answer: 21.days from Served(complaint) counted_by c6 effective [2024-12-01, ..)
transition t amends answer@2024-12-01 cite FRCP 86(a)
{
  unless has_event(Answered(complaint)) -> answer@2009-12-01
  otherwise -> answer@2024-12-01
}
default standing when has_event(Served(complaint, Mail)) => 14.days
exception local rebuts standing when has_event(Answered(complaint)) => 7.days priority 2
exception moot undercuts standing when !has_event(Served(complaint, Mail)) priority 3
state pending: "Pending" by Defendant {
  respond "Respond" -> done { within answer, effort 2.hours, cost 1_500.usd }
}
terminal done: "Done" outcome [judgment] { payoff -1_000.50.usd }
test "answer due" {
  given Served(complaint, Personal) on 2025-03-03
  expect deadline(answer) == 2025-03-24
}
"#;

const LINKS: &str = r#"module links
link appeal "Appeal": rules::done -> rules@copy::pending by Plaintiff p 1/2 { note "x" }
instance rules@copy { pack rules, probabilities { respond: 1/3 }, patch respond { "hours": unset } }
"#;

#[test]
fn every_declaration_kind_lowers_to_hir() {
    let (db, ws) = workspace(&[
        package("rules", &[], &[("rules.lit", RULES)]),
        package("links", &["rules"], &[("links.lit", LINKS)]),
    ]);
    assert_eq!(render(&db, ws), Vec::<String>::new());
    let checks = litigate_sema::check_workspace(&db, ws);
    let json = serde_json::to_value(checks.iter().map(|c| &c.package).collect::<Vec<_>>())
        .unwrap_or_default();
    let text = json.to_string();
    for needle in [
        "\"answer\"",
        "2024-12-01",
        "\"Statute\"",
        "\"standing\"",
        "\"local\"",
        "\"moot\"",
        "\"appeal\"",
        "rules@copy",
        "\"answer due\"",
    ] {
        assert!(text.contains(needle), "{needle} missing from HIR");
    }
    insta::assert_json_snapshot!(json);
}
