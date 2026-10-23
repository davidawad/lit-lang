//! Recovery inside `.lit` declarations: every missing or misplaced part of
//! a declaration header or body reports exactly these diagnostics.
mod support;

use litigate_syntax::FileKind;
use support::diag::check_all;

/// `(source, expected diagnostics)`.
type Cases = &'static [(&'static str, &'static [&'static str])];

/// What may start a top-level `.lit` item.
macro_rules! items {
    () => {
        "one of `module`, `import`, `package`, `start`, `group`, `type`, `enum`, `event`, \
         `state`, `decision`, `chance`, `terminal`, `link`, `instance`, `rule`, `deadline`, \
         `counting`, `calendar`, `transition`, `authority`, `default`, `exception`, or `test`"
    };
}

/// What may follow a node declaration's header.
macro_rules! node_clauses {
    () => {
        "one of `by`, `discretion`, `info`, `outcome`, `effective`, `known_at`, `subject_to`, \
         `notwithstanding`, `supersedes`, `revives`, `override`, `delegable_to`, `cite`, or `{`"
    };
}

const FILE_HEADERS_AND_SIMPLE_ITEMS: Cases = &[
    ("foo\n", &["L0001 1:1-1:4 expected `module`, found `foo`"]),
    (
        "start s\n",
        &["L0001 1:1-1:6 expected `module`, found `start`"],
    ),
    (
        "module m\nfoo\n",
        &[concat!(
            "L0001 2:1-2:4 expected ",
            items!(),
            ", found `foo`"
        )],
    ),
    (
        "module m\nimport 5\n",
        &["L0001 2:8-2:9 expected an identifier, found `5`"],
    ),
    (
        "module m\npackage p x\n",
        &["L0001 2:11-2:12 expected `{`, found `x`"],
    ),
    (
        "module m\npackage p { , forum f, authority A }\n",
        &[
            "L0001 2:13-2:14 expected `forum`, found `,`",
            "L0001 2:15-2:20 expected `authority`, found `forum`",
            "L0001 2:24-2:33 expected one of `forum`, `authority`, or `}`, found `authority`",
        ],
    ),
    (
        "module m\ngroup : \"G\"\n",
        &["L0001 2:7-2:8 expected an identifier, found `:`"],
    ),
    (
        "module m\ngroup g \"G\"\n",
        &["L0001 2:9-2:12 expected `:`, found `\"G\"`"],
    ),
    (
        "module m\ntype = X\n",
        &["L0001 2:6-2:7 expected an identifier, found `=`"],
    ),
    (
        "module m\ntype T X\n",
        &["L0001 2:8-2:9 expected `=`, found `X`"],
    ),
    (
        "module m\nenum { A }\n",
        &["L0001 2:6-2:7 expected an identifier, found `{`"],
    ),
    (
        "module m\nenum E { A, , B }\n",
        &["L0001 2:13-2:14 expected an identifier or `}`, found `,`"],
    ),
    (
        "module m\nevent (a: T)\n",
        &["L0001 2:7-2:8 expected an identifier, found `(`"],
    ),
    (
        "module m\nevent E(5)\n",
        &["L0001 2:9-2:10 expected an identifier or `)`, found `5`"],
    ),
    (
        "module m\nstart a @ 5\n",
        &[concat!("L0001 2:9-2:10 expected ", items!(), ", found `@`")],
    ),
];

#[test]
fn file_headers_and_simple_items() {
    check_all(FileKind::Lit, FILE_HEADERS_AND_SIMPLE_ITEMS);
}

const NODE_HEADERS_AND_BODIES: Cases = &[
        (
            "module m\nstate s \"S\"\n",
            &[
                "L0001 2:9-2:12 expected `:`, found `\"S\"`",
            ],
        ),
        ("module m\nstate s: \"S\" start x\n", &[]),
        (
            "module m\nstate s: \"S\"\nfoo 8\n",
            &[
                concat!("L0001 3:1-3:4 expected ", items!(), ", found `foo`"),
            ],
        ),
        (
            "module m\nstate s: \"S\" FRCP 8\nstart x\n",
            &[
                "L0001 2:14-2:18 expected a clause, found `FRCP`; citations start with `cite` | fix 2:14-2:20 \"cite \\\"FRCP 8\\\"\"",
            ],
        ),
        (
            "module m\nfoo (a, [b]) {c} bar\nstart x\n",
            &[
                concat!("L0001 2:1-2:4 expected ", items!(), ", found `foo`"),
            ],
        ),
        (
            "module m\nstate s: \"S\" by \"x\"\n",
            &[
                concat!("L0001 2:14-2:16 expected ", node_clauses!(), ", found `by`"),
            ],
        ),
        ("module m\ndecision s: \"S\" info i\n", &[]),
        (
            "module m\ndecision s: \"S\" info \"x\"\n",
            &[
                concat!("L0001 2:17-2:21 expected ", node_clauses!(), ", found `info`"),
            ],
        ),
        ("module m\nchance s: \"S\" { a \"A\" -> t p 1 { when has_event(E()) } }\n", &[]),
        (
            "module m\nchance s: \"S\" { a \"A\" -> t p 1 { when } }\n",
            &[
                "L0001 2:39-2:40 expected an identifier, found `}`",
            ],
        ),
        (
            "module m\nstate s: \"S\" note x\n",
            &[
                concat!("L0001 2:14-2:18 expected ", node_clauses!(), ", found `note` | note: did you mean `cite`? | fix 2:14-2:18 \"cite\""),
            ],
        ),
        (
            "module m\nstate s: \"S\" FRCP x { }\n",
            &[
                concat!("L0001 2:14-2:18 expected ", node_clauses!(), ", found `FRCP`"),
            ],
        ),
        (
            "module m\nstate s: \"S\" FRCP 8 { }\n",
            &[
                "L0001 2:14-2:18 expected a clause, found `FRCP`; citations start with `cite` | fix 2:14-2:20 \"cite \\\"FRCP 8\\\"\"",
            ],
        ),
        (
            "module m\nstate s: \"S\" FRCP 8(a)\n",
            &[
                "L0001 2:14-2:18 expected a clause, found `FRCP`; citations start with `cite` | fix 2:14-2:23 \"cite \\\"FRCP 8(a)\\\"\"",
            ],
        ),
        (
            "module m\nstate s: \"S\" cite FRCP  8(a) \"x\"\n",
            &[
                "L0001 2:19-2:33 citation text must be a string: `cite \"FRCP 8(a) \\\"x\\\"\"` | fix 2:19-2:33 \"\\\"FRCP 8(a) \\\\\\\"x\\\\\\\"\\\"\"",
            ],
        ),
        (
            "module m\nstate s: \"S\" cite \"X 1\"\nhistorical\n",
            &[concat!("L0001 3:1-3:11 expected ", items!(), ", found `historical`")],
        ),
        (
            "module m\nstate s: \"S\" cite { }\n",
            &["L0001 2:19-2:20 expected a string or `?`, found `{`"],
        ),
        (
            "module m\nstate s: \"S\" cite",
            &["L0001 2:18-2:18 expected a string or `?`, found end of file"],
        ),
        (
            "module m\nstate s: \"S\" cite\n{ }\n",
            &["L0001 3:1-3:2 expected a string or `?`, found `{`"],
        ),
        (
            "module m\nstate s: \"S\" { 5 }\n",
            &[
                "L0001 2:16-2:17 expected one of `group`, `note`, `valence`, `court_listener_url`, `tags`, `attrs`, `payoff`, `payoff_by_flag`, `basis`, `formerly`, an identifier, or `}`, found `5`",
            ],
        ),
        (
            "module m\nstate s: \"S\" { tags x }\n",
            &[
                "L0001 2:21-2:22 expected an identifier, found `x`",
            ],
        ),
        (
            "module m\nstate s: \"S\" { payoff ?() }\n",
            &[
                "L0001 2:25-2:26 expected a string, found `)`",
            ],
        ),
        (
            "module m\nstate s: \"S\" { payoff ?(\"r\" }\n",
            &[
                "L0002 2:24-2:25 unclosed `(`: expected `)` | fix 2:30-2:30 \")\"",
            ],
        ),
        (
            "module m\nstate s: \"S\" { a \"A\" -> t p x }\n",
            &[
                "L0001 2:29-2:30 expected an identifier, found `x`",
            ],
        ),
        (
            "module m\nstate s: \"S\" { a \"A\" -> t { duration x } }\n",
            &[
                "L0001 2:38-2:39 expected an identifier, found `x`",
            ],
        ),
];

#[test]
fn node_headers_and_bodies() {
    check_all(FileKind::Lit, NODE_HEADERS_AND_BODIES);
}

const LINKS_INSTANCES_AND_RULES: Cases = &[
        ("module m\nlink _ \"L\": a -> b\n", &[]),
        (
            "module m\nlink \"L\": a -> b\n",
            &[
                "L0001 2:6-2:9 expected an identifier or `_`, found `\"L\"`",
            ],
        ),
        (
            "module m\nlink l: a -> b\n",
            &[
                "L0001 2:7-2:8 expected a string, found `:`",
            ],
        ),
        (
            "module m\ninstance { }\n",
            &[
                "L0001 2:10-2:11 expected an identifier, found `{`",
            ],
        ),
        (
            "module m\ninstance a b { }\n",
            &[
                "L0001 2:12-2:13 expected `@`, found `b`",
            ],
        ),
        (
            "module m\nrule r { }\n",
            &[
                "L0001 2:6-2:7 expected one of `duty`, `power`, or `liability`, found `r`",
            ],
        ),
        (
            "module m\nrule duty r { 5 }\n",
            &[
                "L0001 2:15-2:16 expected one of `start`, `group`, `type`, `enum`, `event`, `state`, `decision`, `chance`, `terminal`, `link`, `instance`, `rule`, `deadline`, `counting`, `calendar`, `transition`, `authority`, `default`, `exception`, `test`, or `}`, found `5`",
            ],
        ),
];

#[test]
fn links_instances_and_rules() {
    check_all(FileKind::Lit, LINKS_INSTANCES_AND_RULES);
}

const DEADLINES_COUNTING_AND_CALENDARS: Cases = &[
        (
            "module m\ndeadline : 21.days from x counted_by c\n",
            &[
                "L0001 2:10-2:11 expected an identifier, found `:`",
            ],
        ),
        (
            "module m\ndeadline d 21.days from x counted_by c\n",
            &[
                "L0001 2:12-2:19 expected `:`, found `21.days`",
            ],
        ),
        (
            "module m\ndeadline d: 21.days from x by c\n",
            &[
                "L0001 2:28-2:30 expected `counted_by`, found `by`",
            ],
        ),
        (
            "module m\ndeadline d: 21.days x counted_by c\n",
            &[
                "L0001 2:21-2:22 expected one of `from`, `after`, or `before`, found `x`",
            ],
        ),
        (
            "module m\ndeadline d: 21.days from x counted_by c { trigger 3 }\n",
            &[
                "L0001 2:51-2:52 expected an identifier, found `3`",
            ],
        ),
        (
            "module m\ndeadline d: 21.days from x counted_by c { extend by c }\n",
            &[
                "L0001 2:50-2:52 expected an identifier, found `by`",
            ],
        ),
        (
            "module m\nevent E() removes 5\n",
            &[
                "L0001 2:19-2:20 expected an identifier, found `5`",
            ],
        ),
        (
            "module m\ncounting c x\n",
            &[
                "L0001 2:12-2:13 expected one of `effective`, `known_at`, `subject_to`, `notwithstanding`, `supersedes`, `revives`, `override`, `delegable_to`, `cite`, or `{`, found `x`",
            ],
        ),
        (
            "module m\ncounting c effective 2020-01-01 { allow_hours }\n",
            &[
                "L0001 2:22-2:32 expected `[`, found `2020-01-01`",
            ],
        ),
        (
            "module m\ncounting c effective [x, ..) { allow_hours }\n",
            &[
                "L0001 2:23-2:24 expected a date `YYYY-MM-DD`, found `x`",
            ],
        ),
        (
            "module m\ncounting c effective [2020-01-01 ..) { allow_hours }\n",
            &[
                "L0001 2:34-2:36 expected `,`, found `..`",
            ],
        ),
        (
            "module m\ncalendar c x\n",
            &[
                "L0001 2:12-2:13 expected one of `effective`, `known_at`, `cite`, or `{`, found `x`",
            ],
        ),
        (
            "module m\ncalendar c { 5 }\n",
            &[
                "L0001 2:14-2:15 expected an identifier or `}`, found `5`",
            ],
        ),
        (
            "module m\ncalendar c { h f() }\n",
            &[
                "L0001 2:16-2:17 expected `:`, found `f`",
            ],
        ),
];

#[test]
fn deadlines_counting_and_calendars() {
    check_all(FileKind::Lit, DEADLINES_COUNTING_AND_CALENDARS);
}

const TRANSITIONS_DEFAULTS_EXCEPTIONS_AND_TESTS: Cases = &[
        (
            "module m\ntransition amends x { }\n",
            &[
                "L0001 2:19-2:20 expected `amends`, found `x`",
            ],
        ),
        (
            "module m\ntransition 5 amends x { }\n",
            &[
                "L0001 2:12-2:13 expected an identifier, found `5`",
            ],
        ),
        (
            "module m\ntransition t x { }\n",
            &[
                "L0001 2:14-2:15 expected `amends`, found `x`",
            ],
        ),
        (
            "module m\ntransition t amends x\n",
            &[
                "L0001 3:1-3:1 expected `cite` or `{`, found end of file",
            ],
        ),
        (
            "module m\ndefault d x\n",
            &[
                "L0001 2:11-2:12 expected one of `effective`, `known_at`, `subject_to`, `notwithstanding`, `supersedes`, `revives`, `override`, `delegable_to`, `cite`, or `when`, found `x`",
            ],
        ),
        (
            "module m\nexception e rebuts x\n",
            &[
                "L0001 3:1-3:1 expected `cite` or `when`, found end of file",
            ],
        ),
        (
            "module m\nexception e x when y\n",
            &[
                "L0001 2:13-2:14 expected `rebuts` or `undercuts`, found `x`",
            ],
        ),
        (
            "module m\ntest \"t\" x\n",
            &[
                "L0001 2:10-2:11 expected `cite` or `{`, found `x`",
            ],
        ),
];

#[test]
fn transitions_defaults_exceptions_and_tests() {
    check_all(FileKind::Lit, TRANSITIONS_DEFAULTS_EXCEPTIONS_AND_TESTS);
}
