//! Lowering over recovered trees (SPEC §3): a declaration missing a part
//! the parser already reported is dropped, a clause or value missing one is
//! dropped from its declaration, and every complete expression lowers.
use litigate_sema::db::parse::tree::{
    Atom, Expr, ExprKind, Field, Item, ItemKind, PathSep, Pattern, RefPath,
};
use litigate_sema::db::parse::{parse, Parsed};

fn lower(body: &str) -> Parsed {
    parse(&format!("module m\n{body}"))
}

/// `body => CODE message; ...` for each body.
fn reports(bodies: &[&str]) -> Vec<String> {
    bodies
        .iter()
        .map(|body| {
            let messages: Vec<String> = lower(body)
                .reports
                .iter()
                .map(|r| format!("{} {}", r.code.as_str(), r.message))
                .collect();
            format!("{body} => {}", messages.join("; "))
        })
        .collect()
}

const DROPPED: &[&str] = &[
    "package { forum f, authority A }",
    "import",
    "start",
    "group : \"G\"",
    "group g",
    "type = Date",
    "type T =",
    "enum { a }",
    "event (a: Date)",
    "event E(a)",
    "state : \"A\"",
    "state a:",
    "link \"L\": a -> b",
    "link x \"L\" -> b",
    "link x \"L\": a ->",
    "instance",
    "instance a",
    "rule answer-duty { }",
    "rule duty { }",
    "deadline d: 1.days counted_by c",
    "deadline d: 1.days from",
    "deadline d: 1.days from (1 +) counted_by c",
    "deadline d: 1.days from x",
    "counting { anchor trigger_day }",
    "counting c",
    "calendar { }",
    "transition { otherwise -> x }",
    "transition t { otherwise -> x }",
    "transition t amends x { otherwise }",
    "transition t amends x { unless -> y }",
    "transition t amends x { unless (1 +) -> y }",
    "default",
    "default cite \"FRCP 1 when true => 1\"",
    "default d",
    "default d when (1 +) => 1",
    "default d when true",
    "default d when true => (1 +)",
    "exception e when x",
    "exception e rebuts t when x =>",
    "exception e rebuts t when x => (1 +)",
    "exception e rebuts t when x => 1 priority",
    "exception rebuts t when x",
    "exception e rebuts cite \"FRCP 1 when x\"",
    "exception e rebuts t",
    "exception e rebuts t when (1 +)",
    "test { expect true }",
    "test \"t\" { expect }",
    "test \"t\" { expect (1 +) }",
    "test \"t\" { given x }",
    "default d when true => -",
    "default d when true => (1 +) + 1",
    "default d when true => 1 + (1 +)",
    "default d when true => (1 +).x",
    "default d when true => match",
    "default d when true => match x { a }",
    "default d when true => match x { a => (1 +) }",
    "default d when true => {a: }",
    "default d when true => {a: (1 +)}",
];

#[test]
fn incomplete_declarations_are_dropped() {
    for body in DROPPED {
        let parsed = lower(body);
        assert_eq!(parsed.tree.items, Vec::<Item>::new(), "{body}");
        assert!(parsed.tree.package.is_none(), "{body}");
        assert!(parsed.tree.imports.is_empty(), "{body}");
        assert!(!parsed.reports.is_empty(), "{body}");
    }
    insta::assert_debug_snapshot!(reports(DROPPED));
}

fn only_item(body: &str) -> ItemKind {
    let items = lower(body).tree.items;
    assert_eq!(items.len(), 1, "{body}: {items:#?}");
    items
        .into_iter()
        .map(|i| i.kind)
        .next()
        .unwrap_or(ItemKind::Authority(Vec::new()))
}

fn keys(fields: &[Field]) -> Vec<&str> {
    fields.iter().map(|f| f.key.text.as_str()).collect()
}

#[test]
fn incomplete_parts_are_dropped_from_their_declaration() {
    let ItemKind::Calendar(_, _, holidays) = only_item(
        "calendar cal {\n  a: fixed(1, 1)\n  b: fixed((1 +), 1) effective [2020-01-01, ..)\n  c: fixed(1, 1) effective [2020-01-01, ..) observed next_day\n}",
    ) else {
        panic!("calendar");
    };
    let names: Vec<_> = holidays.iter().map(|h| h.name.text.as_str()).collect();
    assert_eq!(names, ["c"]);
    assert_eq!(
        holidays[0].observed.as_ref().map(|o| o.text.as_str()),
        Some("next_day")
    );

    let node = |body: &str| match only_item(&format!("state s: \"S\" {{ {body} }}")) {
        ItemKind::Node(n) => {
            let edges = n
                .edges
                .iter()
                .map(|e| (e.label.text.clone(), keys(&e.clauses).join(" ")));
            (keys(&n.fields).join(" "), edges.collect::<Vec<_>>())
        }
        other => panic!("{other:?}"),
    };
    let edge = |label: &str, clauses: &str| vec![(label.to_owned(), clauses.to_owned())];
    assert_eq!(node("note"), ("note".to_owned(), vec![]));
    assert_eq!(node("x \"X\" ->"), (String::new(), vec![]));
    assert_eq!(node("y \"Y\" -> s p 1/"), (String::new(), edge("Y", "")));
    assert_eq!(node("z \"Z\" -> s p 1/2"), (String::new(), edge("Z", "p")));
    assert_eq!(node("1"), (String::new(), vec![]));

    let ItemKind::Deadline { clauses, opts, .. } = only_item(
        "deadline d: 1.days from x counted_by c effective [2020-01-01 known_at 2021-01-01 cite { extendable true }",
    ) else {
        panic!("deadline");
    };
    assert_eq!(keys(&clauses), ["known_at"]);
    assert_eq!(keys(&opts), ["extendable"]);

    for incomplete in ["effective", "effective [", "effective [.., 2020-01-01)"] {
        let ItemKind::Deadline { clauses, .. } = only_item(&format!(
            "deadline d: 1.days from x counted_by c {incomplete}"
        )) else {
            panic!("deadline");
        };
        assert_eq!(keys(&clauses), Vec::<&str>::new(), "{incomplete}");
    }
}

#[test]
fn patches_lower_to_edge_fields() {
    let patches = |body: &str| match only_item(&format!("instance a @ b {{ {body} }}")) {
        ItemKind::Instance(_, _, fields) => fields,
        other => panic!("{other:?}"),
    };
    assert_eq!(patches("patch { note \"x\" }"), Vec::<Field>::new());
    let fields = patches("patch ok { note \"n\", tags [a], within d { extendable true } }");
    let [(
        Atom::Expr(Expr {
            kind: ExprKind::Ref(edge),
            ..
        }),
        _,
    ), (Atom::Fields(written), _)] = &fields[0].values[..]
    else {
        panic!("{fields:#?}");
    };
    assert_eq!(edge.head.text, "ok");
    let keys: Vec<&str> = written.iter().map(|f| f.key.text.as_str()).collect();
    assert_eq!(keys, ["note", "tags", "within"]);
}

/// S-expression rendering of a lowered expression.
fn shape(e: &Expr) -> String {
    let list = |items: &[Expr]| items.iter().map(shape).collect::<Vec<_>>().join(" ");
    match &e.kind {
        ExprKind::Number(t) | ExprKind::Money(t) | ExprKind::Duration(t) | ExprKind::Date(t) => {
            t.clone()
        }
        ExprKind::Str(s) => format!("{s:?}"),
        ExprKind::Bool(b) => b.to_string(),
        ExprKind::Unit => "unit".to_owned(),
        ExprKind::Hole(reason) => format!("?{reason:?}"),
        ExprKind::Cite(c, h) => format!("(cite {c}{})", if *h { " historical" } else { "" }),
        ExprKind::Ref(r) => path(r),
        ExprKind::Call(f, args) => format!("({} {})", path(f), list(args)),
        ExprKind::Unary(op, x) => format!("({op:?} {})", shape(x)),
        ExprKind::Binary(op, l, r) => format!("({op:?} {} {})", shape(l), shape(r)),
        ExprKind::Paren(x) => format!("[{}]", shape(x)),
        ExprKind::Record(fields) => {
            let fields: Vec<String> = fields
                .iter()
                .map(|(k, v)| format!("{}: {}", k.text, shape(v)))
                .collect();
            format!("{{{}}}", fields.join(", "))
        }
        ExprKind::List(items) => format!("[{}]", list(items)),
        ExprKind::Field(x, name) => format!("(. {} {})", shape(x), name.text),
        ExprKind::Match(x, arms) => {
            let arms: Vec<String> = arms
                .iter()
                .map(|(p, v)| match p {
                    Pattern::Ref(r) => format!("{} => {}", path(r), shape(v)),
                    Pattern::Wildcard(_) => format!("_ => {}", shape(v)),
                })
                .collect();
            format!("(match {} {})", shape(x), arms.join(", "))
        }
    }
}

fn path(r: &RefPath) -> String {
    let mut s = r.head.text.clone();
    if let Some(i) = &r.instance {
        s = format!("{s}@{}", i.text);
    }
    for (sep, n) in &r.rest {
        let sep = if *sep == PathSep::Dot { "." } else { "::" };
        s = format!("{s}{sep}{}", n.text);
    }
    if let Some(v) = &r.version {
        s = format!("{s}@{}", v.text);
    }
    s
}

#[test]
fn every_expression_form_lowers() {
    let sources = [
        "a || b && c == d && e != f && g < h && i <= j && k > l && m >= n",
        "1 + 2 - 3 * 4 / 5 % 6",
        "-1 + !true",
        "[unit, 1.usd, 2.days, \"s\\\"q\", 2020-01-01, false, ?, ?(\"why\")]",
        "base@copy::x@2020-01-01 == pack::id && a.b.c",
        "f(1, g()).h.i",
        "(1).x",
        "match x { a.b => 1, _ => 2 }",
        "{k: 1, when: 2}",
    ];
    let out: Vec<String> = sources
        .iter()
        .map(|src| {
            let ItemKind::Default(_, _, _, then) =
                only_item(&format!("default d when true => {src}"))
            else {
                panic!("{src}");
            };
            format!("{src}  ~>  {}", shape(&then))
        })
        .collect();
    insta::assert_debug_snapshot!(out);
    let ItemKind::Rule(_, _, clauses, _) = only_item("rule duty r cite \"FRCP 1(a)\" { }") else {
        panic!("rule");
    };
    assert_eq!(keys(&clauses), ["cite"]);
    let ItemKind::Rule(_, _, clauses, _) = only_item("rule duty r cite ?(\"no text\") { }") else {
        panic!("rule");
    };
    assert!(
        matches!(&clauses[0].values[0].0, Atom::Expr(Expr { kind: ExprKind::Hole(Some(r)), .. }) if r == "no text")
    );
    let ItemKind::Rule(_, _, clauses, _) = only_item("rule duty r cite\n{ }") else {
        panic!("rule");
    };
    assert_eq!(keys(&clauses), Vec::<&str>::new());
}
