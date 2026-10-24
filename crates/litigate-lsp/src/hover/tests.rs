use super::*;

#[test]
fn every_type_reads_as_written() {
    let shown: Vec<String> = [
        Type::Date,
        Type::Days(DayUnit::Calendar),
        Type::Days(DayUnit::Court),
        Type::Days(DayUnit::Business),
        Type::Days(DayUnit::Week),
        Type::Days(DayUnit::Month),
        Type::Days(DayUnit::Year),
        Type::Hours,
        Type::Money("usd".into()),
        Type::Probability,
        Type::Distribution("Outcome".into()),
        Type::Conditional("Outcome".into(), "Judge".into()),
        Type::Credal,
        Type::Cite,
        Type::DateInterval,
        Type::Role,
        Type::Boolean,
        Type::Number,
        Type::String,
        Type::Edge,
        Type::List(Box::new(Type::Date)),
        Type::Record(vec![("a".into(), Type::Number), ("b".into(), Type::Unit)]),
        Type::Named("core::Document".into()),
        Type::Unit,
    ]
    .iter()
    .map(show_type)
    .collect();
    assert_eq!(
        shown,
        [
            "Date",
            "Days<CalendarDays>",
            "Days<CourtDays>",
            "Days<BusinessDays>",
            "Days<Weeks>",
            "Days<Months>",
            "Days<Years>",
            "Hours",
            "Money<USD>",
            "Probability",
            "Dist<Outcome>",
            "Dist<Outcome, Judge>",
            "Credal",
            "Cite",
            "Interval<Date>",
            "Role",
            "Bool",
            "Number",
            "String",
            "EdgeRef",
            "[Date]",
            "{ a: Number, b: Unit }",
            "Document",
            "Unit",
        ]
    );
}

fn located(path: Option<&str>, lines: Option<(u32, u32)>, text: Option<&str>) -> Located {
    Located {
        heading: "Rule 12".into(),
        source: None,
        pinpoint: vec!["a".into(), "1".into()],
        path: path.map(str::to_owned),
        lines,
        text: text.map(str::to_owned),
    }
}

fn block(located: &[Located]) -> String {
    let mut out = String::new();
    l0(&mut out, located);
    out
}

#[test]
fn an_l0_section_names_where_it_was_found() {
    let one = block(&[located(
        Some("sources/frcp.txt"),
        Some((7, 7)),
        Some("  (1) x"),
    )]);
    assert_eq!(
        one,
        "\n**L0** Rule 12(a)(1) — `sources/frcp.txt:7`\n\n```text\n(1) x\n```\n"
    );
    let span = block(&[located(Some("sources/frcp.txt"), Some((7, 9)), None)]);
    assert_eq!(span, "\n**L0** Rule 12(a)(1) — `sources/frcp.txt:7-9`\n");
    // Vendored, but the heading was not found in the text.
    let file = block(&[located(Some("sources/frcp.txt"), None, None)]);
    assert_eq!(file, "\n**L0** Rule 12(a)(1) — `sources/frcp.txt`\n");
    // Neither vendored nor bound to a source.
    let nowhere = block(&[located(None, None, None)]);
    assert_eq!(
        nowhere,
        "\n**L0** Rule 12(a)(1) — not in the vendored corpus\n"
    );
}

#[test]
fn a_long_l0_excerpt_is_cut() {
    let text: Vec<String> = (1..=EXCERPT + 1).map(|i| format!("line {i}")).collect();
    let out = block(&[located(
        Some("s.txt"),
        Some((1, 13)),
        Some(&text.join("\n")),
    )]);
    assert!(out.contains("line 12\n…\n```"), "{out}");
    assert!(!out.contains("line 13"), "{out}");
}
