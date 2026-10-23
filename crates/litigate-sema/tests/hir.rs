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
transition t amends answer@2024-12-01 cite "FRCP 86(a)"
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
instance rules@copy { pack rules, probabilities { respond: 1/3 }, patch respond { note "n" } }
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

/// `litigate import`'s `/// imported` doc line marks a declaration
/// imported in HIR (SPEC §7, §8.7); it never becomes a fallback label.
#[test]
fn the_imported_doc_line_marks_a_declaration() {
    let src = "module rules\nstart a\n\
               /// imported\nstate a: \"A\" { _ \"Go\" -> b }\n\
               terminal b: \"B\" { payoff 0.usd }\n\
               /// imported\n/// Federal holidays.\n\
               calendar fed { new-year: fixed(1, 1) effective [1870-06-28, ..) observed nearest_weekday }\n\
               /// imported\ncalendar bare { new-year: fixed(1, 1) effective [1870-06-28, ..) observed nearest_weekday }\n";
    let (db, ws) = workspace(&[package("rules", &[], &[("rules.lit", src)])]);
    let checks = litigate_sema::check_workspace(&db, ws);
    let Some(pkg) = checks.first().and_then(|c| c.package.as_ref()) else {
        panic!("{:?}", render(&db, ws));
    };
    let marks: Vec<(&str, &str, bool)> = pkg
        .modules
        .iter()
        .flat_map(|m| &m.declarations)
        .filter_map(litigate_sema::temporal::world::meta_of)
        .map(|m| (m.id.as_str(), m.label.as_str(), m.imported))
        .collect();
    assert_eq!(
        marks,
        [
            ("a", "A", true),
            ("b", "B", false),
            ("fed", "Federal holidays.", true),
            ("bare", "", true),
        ]
    );
    let json = serde_json::to_value(pkg).unwrap_or_default().to_string();
    assert_eq!(json.matches("\"imported\":true").count(), 3);
    assert!(!json.contains("\"imported\":false"));
}
