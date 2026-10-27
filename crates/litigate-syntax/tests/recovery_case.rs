//! Recovery paths of the `.case` grammar and the parser's nesting limit.
mod support;

use litigate_syntax::{parse, FileKind};
use support::diag::check_all;

const ITEMS: &str = "one of `case`, `forum`, `governing_law`, `uses`, `as_of`, `known_at`, `summary`, `notes`, `source`, \
                     `start`, `perspective`, `parties`, `events`, `facts`, `stakes`, `beliefs`, \
                     or `analysis`";

fn case(uses: &str, rest: &str) -> String {
    format!("case c\nuses p {uses}\nas_of 2025-01-01\nsummary \"s\"\n{rest}")
}

#[test]
fn version_requirements() {
    let cases = [
        "1.0.0",
        "^1.0.0-1",
        "^1.0.0-rc.beta.1",
        "^1.0.0 -beta",
        "^1.0.0-",
        "^1.0.0-rc .1",
        "^1_0.0.0",
        "^1.0_0.0",
    ]
    .map(|uses| case(uses, ""));
    let bad_version = |at: &str, found: &str| {
        format!(
            "L0001 {at} expected a version `MAJOR.MINOR.PATCH[-pre]` with no spaces, found {found}"
        )
    };
    let dangling_minus = bad_version("3:1-3:6", "`as_of`");
    let underscored = bad_version("2:14-2:15", "`.`");
    let stray_minus = format!("L0001 2:15-2:16 expected {ITEMS}, found `-`");
    let stray_dot = format!("L0001 2:18-2:19 expected {ITEMS}, found `.`");
    check_all(
        FileKind::Case,
        &[
            (
                &cases[0],
                &["L0001 2:8-2:11 expected one of `^`, `~`, `>=`, or `=`, found `1.0`"],
            ),
            (&cases[1], &[]),
            (&cases[2], &[]),
            (&cases[3], &[stray_minus.as_str()]),
            (&cases[4], &[dangling_minus.as_str()]),
            (&cases[5], &[stray_dot.as_str()]),
            (&cases[6], &[underscored.as_str()]),
            (&cases[7], &[underscored.as_str()]),
        ],
    );
}

#[test]
fn beliefs_need_a_reference_and_a_colon() {
    let no_ref = case("^1.0.0", "beliefs { observe 1 }\n");
    let no_colon = case("^1.0.0", "beliefs { observe x 1 }\n");
    let prior_no_colon = case("^1.0.0", "beliefs { prior x 1 }\n");
    check_all(
        FileKind::Case,
        &[
            (
                &no_ref,
                &["L0001 5:19-5:20 expected an identifier, found `1`"],
            ),
            (&no_colon, &["L0001 5:21-5:22 expected `:`, found `1`"]),
            (
                &prior_no_colon,
                &["L0001 5:19-5:20 expected `:`, found `1`"],
            ),
        ],
    );
}

#[test]
fn stakes_take_a_fee_shift_with_a_value_or_alone() {
    let both = case(
        "^1.0.0",
        "stakes { j: 1.usd fee_shift 0.5, k: unvalued fee_shift 1 }\n",
    );
    let alone = case("^1.0.0", "stakes { j: fee_shift 1 }\n");
    let no_fraction = case("^1.0.0", "stakes { j: 1.usd fee_shift }\n");
    let nothing = case("^1.0.0", "stakes { j: }\n");
    check_all(
        FileKind::Case,
        &[
            (&both, &[]),
            (&alone, &[]),
            (
                &no_fraction,
                &["L0001 5:29-5:30 expected a number, found `}`"],
            ),
            (
                &nothing,
                &["L0001 5:13-5:14 expected an identifier, found `}`"],
            ),
        ],
    );
}

#[test]
fn case_items() {
    // Line 5 holds the item under test, after the three required items.
    let cases: [(&str, &[&str]); 14] = [
        (
            "summary \"t\"",
            &["L0001 5:1-5:8 `summary` may appear only once"],
        ),
        ("source x", &["L0001 5:8-5:9 expected a string, found `x`"]),
        ("perspective { client: a }", &[]),
        (
            "perspective 5",
            &["L0001 5:13-5:14 expected an identifier or `{`, found `5`"],
        ),
        ("parties x", &["L0001 5:9-5:10 expected `{`, found `x`"]),
        (
            "parties { 5 }",
            &["L0001 5:11-5:12 expected an identifier or `}`, found `5`"],
        ),
        (
            "parties { a X Y }",
            &["L0001 5:13-5:14 expected `:`, found `X`"],
        ),
        (
            "parties { a: 5 }",
            &["L0001 5:14-5:15 expected an identifier, found `5`"],
        ),
        (
            "events { x }",
            &["L0001 5:10-5:11 expected an identifier or `}`, found `x`"],
        ),
        (
            "events { 2025-01-02 5 }",
            &["L0001 5:21-5:22 expected an identifier, found `5`"],
        ),
        (
            "events { 2025-01-02 X by a }",
            &["L0001 5:23-5:25 expected `(`, found `by`"],
        ),
        (
            "facts { a 1 }",
            &["L0001 5:11-5:12 expected `:`, found `1`"],
        ),
        (
            "beliefs { x }",
            &["L0001 5:11-5:12 expected an identifier or `}`, found `x`"],
        ),
        ("analysis { k: 1, j: [1, \"x\", e], r: { a: -1 } }", &[]),
    ];
    let texts: Vec<String> = cases
        .iter()
        .map(|(item, _)| case("^1.0.0", &format!("{item}\n")))
        .collect();
    let table: Vec<(&str, &[&str])> = texts
        .iter()
        .zip(cases)
        .map(|(text, (_, expected))| (text.as_str(), expected))
        .collect();
    check_all(FileKind::Case, &table);
}

#[test]
fn case_headers_and_uses() {
    let bad_version = "expected a version `MAJOR.MINOR.PATCH[-pre]` with no spaces, found";
    let short = case("^1.0", "");
    let short_error = format!("L0001 3:1-3:6 {bad_version} `as_of`");
    let spaced = case("^ 1.0.0", "");
    let spaced_error = format!("L0001 2:10-2:13 {bad_version} `1.0`");
    let second = case("^1.0.0, 5", "");
    check_all(
        FileKind::Case,
        &[
            (
                "uses p ^1.0.0\nas_of 2025-01-01\nsummary \"s\"\n",
                &["L0001 1:1-1:5 expected `case`, found `uses`"],
            ),
            (
                "case c // header\nuses p ^1.0.0\nas_of 2025-01-01\nsummary \"s\"\n",
                &[],
            ),
            (
                "case c\nuses ^1.0.0\nas_of 2025-01-01\nsummary \"s\"\n",
                &["L0001 2:6-2:7 expected an identifier, found `^`"],
            ),
            (&short, &[short_error.as_str()]),
            (&spaced, &[spaced_error.as_str()]),
            (
                &second,
                &["L0001 2:16-2:17 expected an identifier, found `5`"],
            ),
        ],
    );
}

#[test]
fn a_second_case_header_is_skipped() {
    let twice = case("^1.0.0", "case d\n");
    let header = format!("L0001 5:1-5:5 expected {ITEMS}, found `case`");
    let name = format!("L0001 5:6-5:7 expected {ITEMS}, found `d`");
    check_all(
        FileKind::Case,
        &[(&twice, &[header.as_str(), name.as_str()])],
    );
}

/// Every construct that nests stops at the limit instead of overflowing
/// the stack, including when the limit falls at the end of the file; below
/// the limit the same shapes parse cleanly.
#[test]
fn nesting_limit_is_reported_at_every_depth() {
    // `(open, inner, close, closed, unclosed)`: the first repeat count that
    // reaches the limit when closed, and when the file ends after the
    // openers. A paren's `1 + 2` hits it one level early (the right operand
    // nests); calls, lists, and records nest twice per level.
    let shapes: [(&str, &str, &str, usize, usize); 5] = [
        ("(", "1 + 2", ")", 127, 128),
        ("-", "1", "", 128, 128),
        ("f(", "1", ")", 64, 64),
        ("[", "1", "]", 64, 64),
        ("{a: ", "1", "}", 64, 64),
    ];
    let limit_errors = |text: &str| {
        let parsed = parse(text, FileKind::Lit);
        assert_eq!(parsed.syntax().to_string(), text);
        let limits = parsed
            .errors()
            .iter()
            .filter(|error| error.message == "nesting deeper than 128 levels")
            .count();
        (parsed.is_ok(), limits)
    };
    for (open, inner, close, closed, unclosed) in shapes {
        for depth in (closed - 4)..=(unclosed + 4) {
            let (open_all, close_all) = (open.repeat(depth), close.repeat(depth));
            let text = format!("module m\ntest \"t\" {{ expect {open_all}{inner}{close_all} }}\n");
            let (ok, limits) = limit_errors(&text);
            assert_eq!(ok, depth < closed, "{open}{inner} x{depth}");
            assert_eq!(limits > 0, depth >= closed, "{open}{inner} x{depth}");
            let at_eof = format!("module m\ntest \"t\" {{ expect {open_all}");
            let (_, limits) = limit_errors(&at_eof);
            assert_eq!(limits > 0, depth >= unclosed, "{open} x{depth} at EOF");
        }
    }
}

#[test]
fn docket_needs_a_number_to_be_a_keyword() {
    // Without a number, `docket` is read as the event call's callee.
    let bare = case("^1.0.0", "events { 2025-01-01 docket Answered(x) by a }\n");
    let call = case("^1.0.0", "events { 2025-01-01 docket(x) by a }\n");
    let parsed = parse(&call, FileKind::Case);
    assert!(parsed.is_ok(), "{:?}", parsed.errors());
    assert!(!parse(&bare, FileKind::Case).is_ok());
}

#[test]
fn an_event_id_needs_a_date_after_it() {
    let declared = case(
        "^1.0.0",
        "events {\n  answer: 2025-01-01 Answered(x) by a\n}\n",
    );
    let parsed = parse(&declared, FileKind::Case);
    assert!(parsed.is_ok(), "{:?}", parsed.errors());
    // An id followed by anything but a date stops the line there; an
    // identifier without `:` is not an event.
    for rest in [
        "events { answer: Answered(x) by a }\n",
        "events { answer Answered(x) by a }\n",
    ] {
        assert!(
            !parse(&case("^1.0.0", rest), FileKind::Case).is_ok(),
            "{rest}"
        );
    }
}
