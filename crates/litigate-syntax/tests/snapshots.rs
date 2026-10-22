//! insta snapshots: the CST of every SPEC example and fixture, parse
//! diagnostics (JSON, SPEC §9.2 shape) for representative broken inputs,
//! and formatter output for messy input.
mod support;

use litigate_base::FileId;
use litigate_syntax::{format, parse, FileKind};
use support::repo_root;

const EXAMPLES: [&str; 5] = [
    "lit/examples/rule12.lit",
    "cases/examples/acme.case",
    "crates/litigate-syntax/tests/fixtures/constructs.lit",
    "crates/litigate-syntax/tests/fixtures/links.lit",
    "crates/litigate-syntax/tests/fixtures/matter.case",
];

fn kind_of(path: &str) -> FileKind {
    FileKind::from_path(std::path::Path::new(path)).unwrap_or(FileKind::Lit)
}

#[test]
fn cst_of_every_example() {
    for relative in EXAMPLES {
        let text = std::fs::read_to_string(repo_root().join(relative)).unwrap_or_default();
        let parsed = parse(&text, kind_of(relative));
        assert!(parsed.is_ok(), "{relative}: {:#?}", parsed.errors());
        assert_eq!(
            format(&text, kind_of(relative)).as_deref(),
            Ok(text.as_str()),
            "{relative}"
        );
        let name = relative
            .rsplit('/')
            .next()
            .unwrap_or(relative)
            .replace('.', "_");
        insta::assert_snapshot!(format!("cst_{name}"), parsed.debug_tree());
    }
}

/// `(name, kind, source)` of inputs exercising each recovery path.
const BROKEN: &[(&str, FileKind, &str)] = &[
    (
        "missing_arrow",
        FileKind::Lit,
        "module m\nstate s: \"S\" {\n  go \"Go\" { effort 1.hours }\n}\n",
    ),
    (
        "missing_brace",
        FileKind::Lit,
        "module m\nstate s: \"S\" {\n  go \"Go\" -> t\n\nstate t: \"T\"\n",
    ),
    (
        "missing_cite",
        FileKind::Lit,
        "module m\nchance c: \"C\" by Court FRCP 8(c)(1) {\n  a \"A\" -> c p 1\n}\n",
    ),
    (
        "misspelled_keyword",
        FileKind::Lit,
        "module m\nchanse c: \"C\"\nstate s: \"S\" { grop g }\n",
    ),
    (
        "lexical_errors",
        FileKind::Lit,
        "module m\ngroup g: \"open\ndeadline d: 21.dayz from X counted_by c\nstart 2025-02-30 $\n",
    ),
    (
        "dangling_doc_and_order",
        FileKind::Lit,
        "module m\nstart s\nimport x\n/// orphan\n",
    ),
    (
        "half_open_interval",
        FileKind::Lit,
        "module m\ncounting c effective [2020-01-01, ..] { allow_hours }\n",
    ),
    (
        "case_order_and_versions",
        FileKind::Case,
        "case c\nas_of 2025-01-01\nuses p ^1 .0.0\nevents {\n  2025-01-02 X(a) at b\n}\nsummary \"late\"\n",
    ),
    (
        "missing_required_case_items",
        FileKind::Case,
        "case c\nnotes \"n\"\n",
    ),
    (
        "unclosed_everything",
        FileKind::Lit,
        "module m\ntest \"t\" { expect f((1 + [2, {a: \"x",
    ),
];

#[test]
fn diagnostics_of_broken_inputs() {
    for (name, kind, text) in BROKEN {
        let parsed = parse(text, *kind);
        assert_eq!(parsed.syntax().to_string(), *text, "{name} is not lossless");
        assert!(!parsed.is_ok(), "{name} should not parse cleanly");
        let file = FileId::from(
            format!(
                "{name}.{}",
                if *kind == FileKind::Lit {
                    "lit"
                } else {
                    "case"
                }
            )
            .as_str(),
        );
        let json = serde_json::to_string_pretty(&parsed.diagnostics(&file)).unwrap_or_default();
        insta::assert_snapshot!(format!("diagnostics_{name}"), json);
    }
}

const MESSY: &str = "// leading file comment\nmodule   frcp.x\nimport b.c\nimport a.{ y,z }\npackage p{forum f authority A}\nstart s   // trailing start\n/// doc for group\ngroup g :\"G\"\ntype T=List<Map<A,B>>\nstate s:\"S\" by Court cite FRCP 1\n  effective [2020-01-01,..)\n{\n  // leading comment on edge\n  a \"A\"->t p 1 / 3 { cost -5.usd,duration {min 1.days mode 2.days max 3.days} } // trailing\n  b \"B\" -> t {within d {extendable true}}\n\n  tags [x,y]\n}\nterminal t:\"T\"{payoff ?( \"why\" )}\ndefault dd when has_event(X(a)) && !(a.b == 1) || c<d => match x { A => 1, _ => -2 }\n/* block comment at end */\n";

#[test]
fn formatter_output_for_messy_input() {
    let out = format(MESSY, FileKind::Lit).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(format(&out, FileKind::Lit).as_deref(), Ok(out.as_str()));
    insta::assert_snapshot!("format_messy", out);
}

#[test]
fn check_format_reports_l9004_with_a_whole_file_fix() {
    let file = FileId::from("x.lit");
    let diagnostic = litigate_syntax::check_format("module   m\n", FileKind::Lit, &file);
    let json = serde_json::to_string_pretty(&diagnostic).unwrap_or_default();
    insta::assert_snapshot!("check_format_l9004", json);
}
