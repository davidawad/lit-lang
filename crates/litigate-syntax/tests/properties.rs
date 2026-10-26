//! Invariants: parsing is total and lossless on arbitrary input; formatting
//! is idempotent and structure-preserving; printing a generated program
//! and parsing it back recovers the program.
mod support;

use litigate_syntax::{format, parse, FileKind, SyntaxError};
use proptest::prelude::*;
use support::shape;

/// Checks the invariants every parse must satisfy, whatever the input.
fn check_total(text: &str, kind: FileKind) -> Result<(), TestCaseError> {
    let parsed = parse(text, kind);
    prop_assert_eq!(parsed.syntax().to_string(), text);
    let len = u32::try_from(text.len()).unwrap_or(u32::MAX);
    // Sorted by position; at one position at most one recovery error
    // (L0001/L0002) and one of each other code.
    let recovery = |c: &str| matches!(c, "L0001" | "L0002");
    let starts: Vec<(u32, &str)> = parsed
        .errors()
        .iter()
        .map(|e| (e.range.start().into(), e.code.as_str()))
        .collect();
    prop_assert!(
        starts.windows(2).all(|w| w[0].0 < w[1].0
            || (w[0].0 == w[1].0 && w[0].1 != w[1].1 && !(recovery(w[0].1) && recovery(w[1].1)))),
        "unsorted or duplicate: {starts:?}"
    );
    for error in parsed.errors() {
        prop_assert!(u32::from(error.range.end()) <= len);
        prop_assert!(error.code.as_str().starts_with("L0"), "{:?}", error.code);
    }
    if parsed.is_ok() {
        check_format(text, kind)?;
    }
    Ok(())
}

/// Formatting a clean file succeeds, is idempotent, and keeps the tree.
fn check_format(text: &str, kind: FileKind) -> Result<(), TestCaseError> {
    let once = format(text, kind).map_err(|e| TestCaseError::fail(format!("{e}: {text:?}")))?;
    let twice = format(&once, kind).map_err(|e| TestCaseError::fail(format!("{e}: {once:?}")))?;
    prop_assert_eq!(&once, &twice, "not idempotent for {:?}", text);
    prop_assert!(!once.ends_with("\n\n"), "blank line at the end: {:?}", once);
    prop_assert!(once.ends_with('\n'), "no final newline: {:?}", once);
    prop_assert_eq!(
        shape(&parse(text, kind).syntax()),
        shape(&parse(&once, kind).syntax())
    );
    Ok(())
}

const SOUP: &[&str] = &[
    "module m\n",
    "case c\n",
    "uses p ^1.0.0\n",
    "as_of 2025-01-01\n",
    "summary \"s\"\n",
    "state",
    "decision",
    "chance",
    "terminal",
    "rule",
    "duty",
    "deadline",
    "counting",
    "calendar",
    "transition",
    "default",
    "exception",
    "test",
    "link",
    "instance",
    "group",
    "enum",
    "event",
    "type",
    "import",
    "package",
    "start",
    "authority",
    "match",
    "when",
    "unless",
    "otherwise",
    "given",
    "expect",
    "on",
    "by",
    "p",
    "tags",
    "note",
    "within",
    "effective",
    "known_at",
    "events",
    "parties",
    "facts",
    "stakes",
    "beliefs",
    "analysis",
    "observe",
    "source",
    "perspective",
    "x",
    "a-b",
    "_",
    "\"lbl\"",
    "\"open",
    "21.days",
    "8.hours",
    "3.usd",
    "1/3",
    "0.5",
    "2025-03-03",
    "?",
    "?(\"r\")",
    "cite \"FRCP 1(a)\"",
    "cite",
    "->",
    "=>",
    "::",
    ".",
    "..",
    "@",
    ":",
    ",",
    "{",
    "}",
    "(",
    ")",
    "[",
    "]",
    "<",
    ">",
    "==",
    "&&",
    "||",
    "!",
    "-",
    "+",
    "*",
    "%",
    "=",
    "^",
    "~",
    ">=",
    " ",
    "\n",
    "\n\n",
    "// c\n",
    "/// d\n",
    "/* b */",
    "/* open",
    "\t",
    "§",
    "\u{feff}",
    "1_",
    "2025-02-30",
];

fn soup() -> impl Strategy<Value = String> {
    prop::collection::vec(prop::sample::select(SOUP), 0..60).prop_map(|parts| parts.join(" "))
}

/// Case count: `PROPTEST_CASES` when set, else 512.
fn cases() -> u32 {
    std::env::var("PROPTEST_CASES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(512)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(cases()))]

    #[test]
    fn parsing_arbitrary_text_is_total_and_lossless(text in any::<String>()) {
        check_total(&text, FileKind::Lit)?;
        check_total(&text, FileKind::Case)?;
    }

    #[test]
    fn parsing_token_soup_is_total_and_lossless(text in soup()) {
        check_total(&text, FileKind::Lit)?;
        check_total(&text, FileKind::Case)?;
    }

    #[test]
    fn printed_programs_parse_back_to_the_same_program(program in gen::program()) {
        let text = program.print();
        let parsed = parse(&text, FileKind::Lit);
        prop_assert!(parsed.is_ok(), "{:#?}\n{}", parsed.errors(), text);
        prop_assert_eq!(gen::Program::read(&parsed), program.canonical());
        check_format(&text, FileKind::Lit)?;
        let formatted = format(&text, FileKind::Lit).unwrap_or_default();
        prop_assert_eq!(gen::Program::read(&parse(&formatted, FileKind::Lit)), program.canonical());
    }
}

/// The worked examples and fixtures, as `(text, kind)`.
fn seeds() -> Vec<(String, FileKind)> {
    support::corpus()
        .into_iter()
        .chain(["constructs.lit", "links.lit", "matter.case"].map(|name| {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures")
                .join(name);
            let kind = FileKind::from_path(&path).unwrap_or(FileKind::Lit);
            (path, kind)
        }))
        .filter_map(|(path, kind)| std::fs::read_to_string(path).ok().map(|text| (text, kind)))
        .collect()
}

/// Byte offsets of token boundaries in `text`.
fn boundaries(text: &str) -> Vec<usize> {
    litigate_syntax::lexer::lex(text)
        .0
        .iter()
        .map(|token| usize::from(token.range.start()))
        .collect()
}

#[derive(Debug, Clone)]
enum Edit {
    Insert(usize, &'static str),
    Delete(usize),
    Duplicate(usize),
}

fn apply(text: &str, edits: &[Edit]) -> String {
    let mut out = text.to_owned();
    for edit in edits {
        let cuts = boundaries(&out);
        let at = |n: usize| cuts.get(n % cuts.len().max(1)).copied().unwrap_or(0);
        let next = |n: usize| {
            cuts.get(n % cuts.len().max(1) + 1)
                .copied()
                .unwrap_or(out.len())
        };
        match edit {
            Edit::Insert(n, piece) => out.insert_str(at(*n), piece),
            Edit::Delete(n) => out.replace_range(at(*n)..next(*n), ""),
            Edit::Duplicate(n) => {
                let piece = out[at(*n)..next(*n)].to_owned();
                out.insert_str(at(*n), &piece);
            }
        }
    }
    out
}

fn edits(pieces: &'static [&'static str]) -> impl Strategy<Value = Vec<Edit>> {
    let edit = prop_oneof![
        (any::<usize>(), prop::sample::select(pieces)).prop_map(|(n, p)| Edit::Insert(n, p)),
        any::<usize>().prop_map(Edit::Delete),
        any::<usize>().prop_map(Edit::Duplicate),
    ];
    prop::collection::vec(edit, 1..6)
}

const TRIVIA: &[&str] = &[
    " ", "\n", "\n\n", "  ", "// c\n", "/* c */", "\n// c\n", "/// d\n",
];

proptest! {
    #![proptest_config(ProptestConfig::with_cases(cases()))]

    #[test]
    fn mutated_fixtures_parse_totally(seed in 0usize..64, edits in edits(SOUP)) {
        let seeds = seeds();
        let (text, kind) = &seeds[seed % seeds.len()];
        check_total(&apply(text, &edits), *kind)?;
    }

    #[test]
    fn trivia_injection_keeps_formatting_stable(
        seed in 0usize..64,
        spots in prop::collection::vec((any::<usize>(), prop::sample::select(TRIVIA)), 1..8),
    ) {
        let seeds = seeds();
        let (text, kind) = &seeds[seed % seeds.len()];
        let edits: Vec<Edit> = spots.into_iter().map(|(n, p)| Edit::Insert(n, p)).collect();
        let mutated = apply(text, &edits);
        if parse(&mutated, *kind).is_ok() {
            check_format(&mutated, *kind)?;
        }
    }
}

#[test]
fn deep_nesting_does_not_overflow() {
    for (open, close) in [("(", ")"), ("[", "]"), ("{", "}"), ("-", ""), ("!", "")] {
        let text = format!(
            "module m\ndefault d when {} 1 {}",
            open.repeat(50_000),
            close.repeat(50_000)
        );
        let parsed = parse(&text, FileKind::Lit);
        assert_eq!(parsed.syntax().to_string(), text);
    }
    let chain = format!("module m\ndefault d when 1{} => 2\n", " + 1".repeat(50_000));
    assert_eq!(parse(&chain, FileKind::Lit).syntax().to_string(), chain);
    // S5: a `.field` chain stops at the same cap as an operator chain.
    let fields = |n: usize| format!("module m\ndefault d when (x){} => 2\n", ".a".repeat(n));
    assert!(parse(&fields(256), FileKind::Lit).is_ok());
    let long = fields(50_000);
    let parsed = parse(&long, FileKind::Lit);
    assert_eq!(parsed.syntax().to_string(), long);
    assert!(parsed
        .errors()
        .iter()
        .any(|e| e.message == "expression chains more than 256 field accesses; split it"));
}

/// Applies every machine-applicable fix (later offsets first).
fn apply_fixes(text: &str, errors: &[SyntaxError]) -> String {
    let mut fixes: Vec<_> = errors
        .iter()
        .flat_map(|error| error.fixes.iter())
        .filter(|fix| fix.applicability == litigate_base::Applicability::MachineApplicable)
        .collect();
    fixes.sort_by_key(|fix| std::cmp::Reverse(fix.range.start()));
    let mut out = text.to_owned();
    for fix in fixes {
        out.replace_range(
            usize::from(fix.range.start())..usize::from(fix.range.end()),
            &fix.replacement,
        );
    }
    out
}

#[test]
fn machine_applicable_fixes_repair_obvious_mistakes() {
    let cases = [
        "module m\nstate s: \"S\" {\n  go \"Go\" -> t\n\nstate t: \"T\"\n",
        "module m\nchance c: \"C\" by Court FRCP 8(c)(1) {\n  a \"A\" -> c p 1\n}\n",
        "module m\nchanse c: \"C\"\nstate s: \"S\" { grop g }\n",
        "module m\ncounting c effective [2020-01-01, ..] { allow_hours }\n",
        "module m\ntest \"t\" { expect f((1 + [2, {a: \"x",
        "module m\nenum E { A B }\ndeadline d: 2.dys from X counted_by c\n",
        "module m\nstate s: \"S\" { a \"A\" t }\n",
        "module m\nstate s: \"S\" bye Court {}\n",
        "module m\ndeadline d: 1.days from X counted_by c efective [2020-01-01, ..)\n",
    ];
    for text in cases {
        let parsed = parse(text, FileKind::Lit);
        assert!(!parsed.is_ok(), "{text}");
        let fixed = apply_fixes(text, parsed.errors());
        let reparsed = parse(&fixed, FileKind::Lit);
        assert!(reparsed.is_ok(), "{fixed}\n{:#?}", reparsed.errors());
    }
}

/// Regression (lg-scsc): the program generator once drew `not` as a
/// deadline's event argument, which the grammar rejects (L0012; `!` is the
/// negation operator). Pins that input and every other word `gen` now
/// excludes: each one fails to parse as an argument, so none may be drawn.
#[test]
fn generator_excludes_words_the_grammar_rejects_as_identifiers() {
    let program = |arg: &str| gen::Program {
        items: vec![gen::Item::Deadline {
            name: "a".to_owned(),
            days: 1,
            event: "a".to_owned(),
            args: vec![arg.to_owned()],
            rule: "a".to_owned(),
        }],
        layout: vec![0],
    };
    let excluded = gen::excluded_words();
    assert_eq!(
        excluded,
        [
            "not",
            "world",
            "layer",
            "precedent",
            "overrule",
            "lock",
            "where",
            "and",
            "or",
            "limit",
            "certificate",
            "excludes",
        ]
    );
    for word in excluded {
        assert!(!gen::is_ident(word), "{word}");
        let text = program(word).print();
        assert!(
            !parse(&text, FileKind::Lit).is_ok(),
            "{word} parses: {text}"
        );
    }
    let text = program("not").print();
    assert!(parse(&text, FileKind::Lit)
        .errors()
        .iter()
        .any(|error| error.code.as_str() == "L0012"));
    assert!(gen::is_ident("not-a") && gen::is_ident("nott"));
    let text = program("nott").print();
    assert!(parse(&text, FileKind::Lit).is_ok(), "{text}");
}

#[path = "support/gen.rs"]
mod gen;
