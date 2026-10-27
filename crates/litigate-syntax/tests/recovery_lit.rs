//! Recovery paths of the `.lit` grammar: each malformed input must parse
//! losslessly and report exactly these diagnostics (code, span, message,
//! notes, fixes). Spans and fixes are `line:col-line:col`, one-based.
mod support;

use litigate_syntax::{parse, FileKind, SyntaxKind};
use support::diag::check_all;

const ITEMS: &str = "one of `module`, `import`, `package`, `start`, `group`, `type`, \
                     `enum`, `event`, `state`, `decision`, `chance`, `terminal`, `link`, \
                     `instance`, `rule`, `deadline`, `counting`, `calendar`, `transition`, \
                     `authority`, `default`, `exception`, `test`, `limit`, `grant`, \
                     `recharacterize`, or `invalidate`";

const EXPR: &str = "one of an identifier, a number, a string, a date `YYYY-MM-DD`, or `(`";

#[test]
fn header_items_keep_their_order() {
    let stray_dot = format!("L0001 2:9-2:10 expected {ITEMS}, found `.`");
    check_all(
        FileKind::Lit,
        &[
            (
                "module m\nmodule n\n",
                &["L0001 2:1-2:7 `module` must be the first declaration and appear once"],
            ),
            (
                "module m\npackage p { forum f, authority A }\npackage q { forum f, authority A }\n",
                &["L0001 3:1-3:8 `package` must follow the imports, precede items, and appear once"],
            ),
            ("module m\nimport a.*\n", &[stray_dot.as_str()]),
            (
                "module m\npackage p { forum f }\n",
                &["L0001 2:21-2:22 expected `authority` before the closing `}`"],
            ),
            // A fixed field list stops at the next item instead of eating it.
            (
                "module m\npackage p {\nstart s\n",
                &[
                    "L0002 2:11-2:12 unclosed `{`: expected `}` | fix 2:12-2:12 \"}\"",
                    "L0001 3:1-3:6 expected `forum`, found `start`",
                ],
            ),
        ],
    );
}

#[test]
fn delimited_lists_recover_at_separators_and_closers() {
    check_all(
        FileKind::Lit,
        &[
            (
                "module m\ntest \"t\" { expect 1, expect 2 }\n",
                &["L0001 2:20-2:21 expected `}`, found `,`"],
            ),
            (
                "module m\nenum E { A",
                &["L0002 2:8-2:9 unclosed `{`: expected `}` | fix 2:11-2:11 \"}\""],
            ),
            // The inserted closer goes before a CRLF line ending.
            (
                "module m\r\nevent E(a: T\r\nstart s\r\n",
                &["L0002 2:8-2:9 unclosed `(`: expected `)` | fix 2:13-2:13 \")\""],
            ),
            // The rest of a broken entry is skipped up to the end of the file.
            (
                "module m\nstate s: \"S\" { group 5 6",
                &[
                    "L0002 2:14-2:15 unclosed `{`: expected `}` | fix 2:25-2:25 \"}\"",
                    "L0001 2:22-2:23 expected an identifier, found `5`",
                ],
            ),
        ],
    );
}

#[test]
fn edges_and_links() {
    check_all(
        FileKind::Lit,
        &[
            (
                "module m\nstate s: \"S\" { a \"A\" -> t by x \"X\" -> u }\n",
                &[
                    "L0001 2:27-2:29 expected one of `group`, `note`, `valence`, \
                   `court_listener_url`, `tags`, `attrs`, `payoff`, `payoff_by_flag`, \
                   `fee_shift`, `basis`, `formerly`, an identifier, or `}`, found `by`",
                ],
            ),
            (
                "module m\nlink l \"L\" a -> b\n",
                &["L0001 2:12-2:13 expected `:`, found `a`"],
            ),
            (
                "module m\nlink l \"L\": -> b\n",
                &["L0001 2:13-2:15 expected an identifier, found `->`"],
            ),
            (
                "module m\ninstance a @ { pack p }\n",
                &["L0001 2:14-2:15 expected an identifier, found `{`"],
            ),
            (
                "module m\ninstance a @ b { patch 5 }\n",
                &["L0001 2:24-2:25 expected an identifier, found `5`"],
            ),
            (
                "module m\ninstance a @ b { payoff_transform \"stake - payoff\" }\n",
                &["L0001 2:35-2:51 expected an identifier, found `\"stake - payoff\"`"],
            ),
            (
                "module m\ninstance a @ b { patch e }\n",
                &["L0001 2:26-2:27 expected `{`, found `}`"],
            ),
        ],
    );
}

#[test]
fn edge_ids_may_spell_edge_clause_keywords() {
    let text = "module m\nstate s: \"S\" {\n  a \"A\" -> t\n  by \"By\" -> t\n  p \"P\" -> t\n}\n";
    let parsed = parse(text, FileKind::Lit);
    assert!(parsed.is_ok(), "{:?}", parsed.errors());
    let edges = parsed
        .syntax()
        .descendants()
        .filter(|node| node.kind() == SyntaxKind::EdgeDecl)
        .count();
    assert_eq!(edges, 3);
}

#[test]
fn edge_fields() {
    let edge =
        |fields: &str| format!("module m\nstate s: \"S\" {{ a \"A\" -> t {{ {fields} }} }}\n");
    let replaces = edge("replaces x");
    let attrs = edge("attrs { a: -x }");
    let duration = edge("duration { min 1.days }");
    let deadline = edge("deadline x");
    let authority = edge("authority \"R. 5\"");
    let within = edge("within 5");
    check_all(
        FileKind::Lit,
        &[
            (
                &replaces,
                &["L0001 2:38-2:39 expected an identifier, found `x`"],
            ),
            (
                &attrs,
                &["L0001 2:40-2:41 expected an identifier, found `-`"],
            ),
            (
                &duration,
                &["L0001 2:51-2:52 expected `mode` before the closing `}`"],
            ),
            (
                &deadline,
                &["L0001 2:38-2:39 expected an identifier, found `x`"],
            ),
            (&authority, &[]),
            (
                &within,
                &["L0001 2:36-2:37 expected an identifier, found `5`"],
            ),
            (
                "module m\nstate s: \"S\" { a \"A\" -> t p 1 / x }\n",
                &["L0001 2:33-2:34 expected a number, found `x`"],
            ),
        ],
    );
}

#[test]
fn declarations() {
    let missing_expr =
        |at: &str, found: &str| format!("L0001 {at} expected {EXPR}, found `{found}`");
    let deadline = missing_expr("2:26-2:27", ")");
    let unless = missing_expr("2:32-2:34", "->");
    let default = missing_expr("2:16-2:18", "=>");
    let exception = missing_expr("2:27-2:28", ")");
    let given = missing_expr("2:18-2:19", "}");
    check_all(
        FileKind::Lit,
        &[
            (
                "module m\nrule duty r subject_to { start s }\n",
                &[
                    "L0001 2:13-2:23 expected one of `effective`, `known_at`, `retracted`, `unknown_before`, `commenced`, `subject_to`, \
                   `notwithstanding`, `supersedes`, `revives`, `override`, `interprets`, `delegable`, \
                   `cite`, or `{`, found `subject_to`",
                ],
            ),
            (
                "module m\nstate s: \"S\" FRCP 8",
                &[
                    "L0001 2:14-2:18 expected a clause, found `FRCP`; citations start with \
                   `cite` | fix 2:14-2:20 \"cite \\\"FRCP 8\\\"\"",
                ],
            ),
            (
                "module m\ndeadline d: 21.days from ) counted_by c\n",
                &[deadline.as_str()],
            ),
            (
                "module m\ncalendar c { h: f() known_at 2020-01-01 }\n",
                &["L0001 2:21-2:29 expected `effective`, found `known_at`"],
            ),
            (
                "module m\ntransition t amends x { 1 }\n",
                &["L0001 2:25-2:26 expected one of `unless`, `otherwise`, or `}`, found `1`"],
            ),
            (
                "module m\ntransition t amends x { unless -> y }\n",
                &[unless.as_str()],
            ),
            (
                "module m\ntransition t amends x { unless a b }\n",
                &["L0001 2:34-2:35 expected `->`, found `b`"],
            ),
            ("module m\ndefault d when => 1\n", &[default.as_str()]),
            (
                "module m\ndefault d when a b\n",
                &["L0001 2:18-2:19 expected `=>`, found `b`"],
            ),
            (
                "module m\nexception e rebuts x when )\n",
                &[exception.as_str()],
            ),
            ("module m\ntest \"t\" { given }\n", &[given.as_str()]),
        ],
    );
}

/// `(source, expected diagnostics)` for declaration parts.
const DECLARATION_PARTS: &[(&str, &[&str])] = &[
    (
        "module m\ncounting c { service_extension x }\n",
        &["L0001 2:32-2:33 expected an identifier, found `x`"],
    ),
    (
        "module m\ncounting c { forward_holidays 3 }\n",
        &["L0001 2:31-2:32 expected an identifier, found `3`"],
    ),
    (
        "module m\ndeadline d: 1.days after E() counted_by c { forum 3 }\n",
        &["L0001 2:51-2:52 expected an identifier, found `3`"],
    ),
    (
        "module m\ndeadline d: 1.days after E() counted_by c { forum f cite \"X\" quote 3 }\n",
        &["L0001 2:68-2:69 expected a string, found `3`"],
    ),
    (
        "module m\ncounting c { declared_holidays closures, forums 3 }\n",
        &["L0001 2:49-2:50 expected an identifier, found `3`"],
    ),
    (
        "module m\ncounting c effective [2020-01-01, .. { allow_hours }\n",
        &["L0002 2:22-2:23 unclosed `[`: expected `)` | fix 2:53-2:53 \")\""],
    ),
    (
        "module m\ntype T = List<1>\n",
        &["L0001 2:15-2:16 expected an identifier or `>`, found `1`"],
    ),
    (
        "module m\nevent E(a T)\n",
        &["L0001 2:11-2:12 expected `:`, found `T`"],
    ),
    (
        "module m\nauthority 5\n",
        &["L0001 2:11-2:12 expected an identifier, found `5`"],
    ),
    (
        "module m\nauthority A > 5\n",
        &["L0001 2:15-2:16 expected an identifier, found `5`"],
    ),
    (
        "module m\ncalendar c { h: 5 }\n",
        &["L0001 2:17-2:18 expected an identifier, found `5`"],
    ),
    (
        "module m\ntest \"t\" { given a 2020-01-01 }\n",
        &["L0001 2:20-2:30 expected `on`, found `2020-01-01`"],
    ),
    (
        "module m\ncounting c { short_period { below 1.days } }\n",
        &[
            "L0001 2:35-2:41 expected `:`, found `1.days`",
            "L0001 2:42-2:43 expected `count` before the closing `}`",
        ],
    ),
    (
        "module m\ntest \"t\" { expect a < b < c }\n",
        &["L0001 2:25-2:26 comparison operators are non-associative; \
           parenthesize one side"],
    ),
];

#[test]
fn declaration_parts() {
    check_all(FileKind::Lit, DECLARATION_PARTS);
}

#[test]
fn expressions() {
    let expect = |e: &str| format!("module m\ntest \"t\" {{ expect {e} }}\n");
    let unary = expect("-");
    let field_of_number = expect("f(x).1");
    let no_scrutinee = expect("match }");
    let bad_arm = expect("match x { 1 => 2 }");
    let no_arrow = expect("match x { A 1 }");
    let nothing = expect("");
    let nothing_error = format!("L0001 2:20-2:21 expected {EXPR}, found `}}`");
    let unary_error = format!("L0001 2:21-2:22 expected {EXPR}, found `}}`");
    let scrutinee_error = format!("L0001 2:25-2:26 expected {EXPR}, found `}}`");
    let stray_brace = format!("L0001 2:27-2:28 expected {ITEMS}, found `}}`");
    check_all(
        FileKind::Lit,
        &[
            (&nothing, &[nothing_error.as_str()]),
            (&unary, &[unary_error.as_str()]),
            (
                &field_of_number,
                &["L0001 2:23-2:24 expected one of `given`, `expect`, or `}`, found `.`"],
            ),
            (
                &no_scrutinee,
                &[scrutinee_error.as_str(), stray_brace.as_str()],
            ),
            (
                &bad_arm,
                &["L0001 2:29-2:30 expected one of an identifier, `_`, or `}`, found `1`"],
            ),
            (&no_arrow, &["L0001 2:31-2:32 expected `=>`, found `1`"]),
        ],
    );
}

#[test]
fn postfix_fields_holes_and_calls_named_match() {
    use SyntaxKind::{CallExpr, FieldExpr, HoleExpr, ParenExpr};
    let kinds = |e: &str| {
        let parsed = parse(
            &format!("module m\ntest \"t\" {{ expect {e} }}\n"),
            FileKind::Lit,
        );
        assert!(parsed.is_ok(), "{e}: {:?}", parsed.errors());
        parsed
            .syntax()
            .descendants()
            .map(|node| node.kind())
            .filter(|kind| {
                matches!(
                    kind,
                    SyntaxKind::FieldExpr
                        | SyntaxKind::CallExpr
                        | SyntaxKind::MatchExpr
                        | SyntaxKind::HoleExpr
                        | SyntaxKind::ParenExpr
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(kinds("match(1)"), [CallExpr]);
    assert_eq!(kinds("f(x).y.z > 1"), [FieldExpr, FieldExpr, CallExpr]);
    assert_eq!(kinds("(a).b"), [FieldExpr, ParenExpr]);
    assert_eq!(kinds("? == ?(\"r\")"), [HoleExpr, HoleExpr]);
}

#[test]
fn recovery_does_not_depend_on_indentation() {
    // S6: an indented item after an error anchors recovery like one at
    // column 1 when the brackets before it are balanced; inside an open
    // bracket only column 1 anchors.
    let items = |text: &str| {
        let parsed = parse(text, FileKind::Lit);
        assert_eq!(parsed.syntax().to_string(), text);
        let terminals = parsed
            .syntax()
            .descendants()
            .filter(|n| n.kind() == SyntaxKind::TerminalDecl)
            .count();
        (terminals, parsed.errors().len())
    };
    let flush = items("module m\ndefault d when => 1 1\nterminal t: \"T\"\n");
    let indented = items("module m\ndefault d when => 1 1\n    terminal t: \"T\"\n");
    assert_eq!(flush, indented);
    assert_eq!(flush.0, 1);
    let open = items("module m\ntest \"t\" { expect (\n    terminal t: \"T\"\n");
    assert_eq!(open.0, 0);
}
