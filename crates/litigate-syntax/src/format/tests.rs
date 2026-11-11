use super::format;
use crate::parse::FileKind;

fn fmt(text: &str) -> String {
    format(text, FileKind::Lit).unwrap_or_else(|error| panic!("{error}: {text}"))
}

#[test]
fn refuses_files_with_syntax_errors() {
    let error = format("module x\nstate", FileKind::Lit).err();
    assert!(error.is_some_and(|e| !e.errors.is_empty()));
}

#[test]
fn sorts_imports_and_separates_items() {
    let out = fmt("module m\nimport b\nimport a\nstart s\ngroup g: \"G\"\ngroup h: \"H\"\n");
    assert_eq!(
        out,
        "module m\n\nimport a\nimport b\n\nstart s\n\ngroup g: \"G\"\ngroup h: \"H\"\n"
    );
}

#[test]
fn derived_edge_ids_are_omitted_but_custom_ids_remain() {
    let out = fmt(
        "module m\nstate from: \"From\" {\n  edge-from-alone \"A\" -> alone\n  custom \"B\" -> to\n  \"C\" -> end\n}\n",
    );
    assert_eq!(
        out,
        "module m\n\nstate from: \"From\" { \"A\" -> alone, custom \"B\" -> to, \"C\" -> end }\n"
    );
}

#[test]
fn anonymous_marker_stays_distinct_from_an_omitted_edge_id() {
    let out =
        fmt("module m\nstate from: \"From\" {\n  _ \"Anonymous\" -> to\n  \"Derived\" -> end\n}\n");
    assert_eq!(
        out,
        "module m\n\nstate from: \"From\" { _ \"Anonymous\" -> to, \"Derived\" -> end }\n"
    );
}

#[test]
fn parallel_derived_edge_ids_number_every_edge_from_one() {
    let out = fmt(
        "module m\nstate from: \"From\" {\n  edge-from-to-1 \"A\" -> to\n  edge-from-to-2 \"B\" -> to\n}\n",
    );
    assert_eq!(
        out,
        "module m\n\nstate from: \"From\" { \"A\" -> to, \"B\" -> to }\n"
    );
}

#[test]
fn parallel_edge_id_elision_is_stable_and_formatting_is_idempotent() {
    let source =
        "module m\nstate from: \"From\" {\n  edge-from-to-1 \"A\" -> to\n  \"B\" -> to\n}\n";
    let out = fmt(source);
    assert_eq!(
        out,
        "module m\n\nstate from: \"From\" { \"A\" -> to, \"B\" -> to }\n"
    );
    assert_eq!(fmt(&out), out);
}

#[test]
fn shared_deadline_forum_is_hoisted_to_the_module() {
    let out = fmt(
        "module m\ndeadline a: 2.days from Served() counted_by c { forum frap }\ndeadline b: 3.days from Filed() counted_by c { forum frap }\n",
    );
    assert_eq!(
        out,
        "module m\n\nforum frap\n\ndeadline a: 2.days from Served() counted_by c\ndeadline b: 3.days from Filed() counted_by c\n"
    );
}

#[test]
fn nested_rule_deadlines_share_the_hoisted_module_forum() {
    let out = fmt(
        "module m\nrule duty r { deadline a: 2.days from Served() counted_by c { forum frap } }\ndeadline b: 3.days from Filed() counted_by c { forum frap }\n",
    );
    assert!(out.starts_with("module m\n\nforum frap\n\n"), "{out}");
    assert!(!out.contains("{ forum frap }"), "{out}");
    assert_eq!(fmt(&out), out);
}

#[test]
fn cite_before_body_breaks_the_body() {
    let out = fmt("module m\nrule duty r cite \"FRCP 1\" { start s }\n");
    assert_eq!(
        out,
        "module m\n\nrule duty r cite \"FRCP 1\"\n{\n  start s\n}\n"
    );
}

#[test]
fn long_bodies_break_and_drop_commas() {
    let long = "x".repeat(90);
    let out = fmt(&format!(
        "module m\nstate s: \"{long}\" {{ group g, note \"n\" }}\n"
    ));
    assert!(out.contains("{\n  group g\n  note \"n\"\n}"), "{out}");
}

#[test]
fn required_commas_survive_breaking() {
    let out = fmt("module m\nenum E { A, B // b\n}\n");
    assert_eq!(out, "module m\n\nenum E {\n  A,\n  B, // b\n}\n");
}

#[test]
fn comments_are_preserved() {
    let src = "// head\nmodule m\n\n/// Doc.\nstate s: \"S\" { // first\n  a \"A\" -> t /* inline */\n}\n";
    let out = fmt(src);
    for needle in ["// head", "/// Doc.", "// first", "/* inline */"] {
        assert!(out.contains(needle), "{needle} lost: {out}");
    }
    assert_eq!(fmt(&out), out);
}

#[test]
fn case_blocks_follow_case_layout() {
    let src = "case c uses p ^1.0.0 as_of 2025-01-01 summary \"s\" parties { a: X Y } \
               facts { n: e }";
    let out = format(src, FileKind::Case).unwrap_or_default();
    assert_eq!(
        out,
        "case c\nuses p ^1.0.0\nas_of 2025-01-01\nsummary \"s\"\n\nparties {\n  a: X Y\n}\n\nfacts { n: e }\n"
    );
}

#[test]
fn the_edition_marker_stays_first() {
    assert_eq!(
        fmt("edition   2026\nmodule m\n"),
        "edition 2026\n\nmodule m\n"
    );
    let case =
        "edition 2026 case c uses p ^1.0.0 as_of 2025-01-01 summary \"s\" parties { a: X Y }";
    assert_eq!(
        format(case, FileKind::Case).as_deref(),
        Ok("edition 2026\ncase c\nuses p ^1.0.0\nas_of 2025-01-01\nsummary \"s\"\n\nparties {\n  a: X Y\n}\n")
    );
}

#[test]
fn basis_fields_format_like_other_fields() {
    let text = "module a\nchance c: \"C\" {\n  basis   expert( \"Jane Roe\" )\n  x \"X\" -> y p 1/2 {basis sourced(\"R 1\"),note \"n\"}\n}\n";
    let out = format(text, FileKind::Lit).unwrap_or_default();
    assert_eq!(
        out,
        "module a\n\nchance c: \"C\" { basis expert(\"Jane Roe\"), x \"X\" -> y p 1/2 { basis sourced(\"R 1\"), note \"n\" } }\n"
    );
}

#[test]
fn numeric_provenance_is_canonical_and_idempotent() {
    let out =
        fmt("module m\nterminal t: \"T\" { payoff 1.usd basis placeholder vintage 2026-01-01 }\n");
    assert!(
        out.contains("payoff 1.usd basis placeholder vintage 2026-01-01"),
        "{out}"
    );
    assert_eq!(fmt(&out), out);
}

#[test]
fn guard_operations_keep_their_clauses_on_one_line() {
    let out = fmt(
        "module order\ninstance base@order {\n  pack base\n  patch timely {\n    \
         supersedes when has_event(Struck())\n      by CaseOrder effective [2010-01-15, ..)\n      \
         cite \"Order, Dkt. 12\"\n  }\n}\n",
    );
    assert_eq!(
        out,
        "module order\n\ninstance base@order {\n  pack base\n  patch timely {\n    supersedes \
         when has_event(Struck()) by CaseOrder effective [2010-01-15, ..) cite \"Order, Dkt. \
         12\"\n  }\n}\n"
    );
    assert_eq!(fmt(&out), out);
}

#[test]
fn a_rule_index_keeps_its_bracket_and_a_lock_its_list() {
    let out = fmt(
        "module m\nrule duty r Rule[FederalRule, _, [2010-01-01, ..), _, _] cite \"X\" {\n  deadline d: 1.days from E() counted_by c { lock against [LocalRule,CaseOrder] cite \"Y\" }\n}\n",
    );
    assert!(
        out.contains("rule duty r Rule[FederalRule, _, [2010-01-01, ..), _, _] cite \"X\""),
        "{out}"
    );
    assert!(
        out.contains("lock against [LocalRule, CaseOrder] cite \"Y\""),
        "{out}"
    );
}
