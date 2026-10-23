//! A small program model: generated, printed with random layout, and read
//! back through the typed AST.
use litigate_syntax::ast::{
    AstNode, AuthorityDecl, DeadlineDecl, EnumDecl, Expr, GroupDecl, Literal, NodeDecl, Ratio,
};
use litigate_syntax::{Parse, SyntaxKind};
use proptest::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub items: Vec<Item>,
    /// Layout choices consumed while printing; ignored by equality checks
    /// via [`Program::canonical`].
    pub layout: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    Group {
        name: String,
        label: String,
    },
    Enum {
        name: String,
        variants: Vec<String>,
    },
    Authority(Vec<String>),
    Deadline {
        name: String,
        days: u32,
        event: String,
        args: Vec<String>,
        rule: String,
    },
    Node(Node),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub keyword: String,
    pub name: String,
    pub label: String,
    pub by: Option<String>,
    pub cite: Option<String>,
    pub tags: Vec<String>,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    pub id: Option<String>,
    pub label: String,
    pub target: String,
    pub p: Option<(u32, u32)>,
    pub effort: Option<u32>,
    pub within: Option<String>,
}

fn ident() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,5}(-[a-z0-9]{1,3})?".prop_filter("keywords are contextual but avoided", |s| {
        SyntaxKind::from_keyword(s).is_none()
    })
}

fn label() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 .,()§-]{0,12}"
}

fn edge() -> impl Strategy<Value = Edge> {
    (
        prop::option::of(ident()),
        label(),
        ident(),
        prop::option::of((1u32..20, 1u32..20)),
        prop::option::of(1u32..100),
        prop::option::of(ident()),
    )
        .prop_map(|(id, label, target, p, effort, within)| Edge {
            id,
            label,
            target,
            p,
            effort,
            within,
        })
}

fn node() -> impl Strategy<Value = Node> {
    (
        prop::sample::select(vec!["state", "decision", "chance", "terminal"]),
        ident(),
        label(),
        prop::option::of(ident()),
        // Citation text needs no escape hatch: commas, `§§`, braces and
        // `//` are plain string content.
        prop::option::of("[A-Z]{2,4} [0-9]{1,2}\\([a-z]\\)(, §§ [0-9]{1,2} [{}/]{0,2})?"),
        prop::collection::vec(ident(), 0..3),
        prop::collection::vec(edge(), 0..4),
    )
        .prop_map(|(keyword, name, label, by, cite, tags, edges)| Node {
            keyword: keyword.to_owned(),
            name,
            label,
            by,
            cite,
            tags,
            edges,
        })
}

fn item() -> impl Strategy<Value = Item> {
    prop_oneof![
        (ident(), label()).prop_map(|(name, label)| Item::Group { name, label }),
        (ident(), prop::collection::vec(ident(), 1..4))
            .prop_map(|(name, variants)| Item::Enum { name, variants }),
        prop::collection::vec(ident(), 1..4).prop_map(Item::Authority),
        (
            ident(),
            1u32..60,
            ident(),
            prop::collection::vec(ident(), 0..3),
            ident()
        )
            .prop_map(|(name, days, event, args, rule)| Item::Deadline {
                name,
                days,
                event,
                args,
                rule
            }),
        node().prop_map(Item::Node),
    ]
}

pub fn program() -> impl Strategy<Value = Program> {
    (
        prop::collection::vec(item(), 0..8),
        prop::collection::vec(any::<u8>(), 1..64),
    )
        .prop_map(|(items, layout)| Program { items, layout })
}

/// Emits tokens separated by layout-chosen trivia.
struct Printer<'a> {
    out: String,
    layout: &'a [u8],
    step: usize,
}

impl Printer<'_> {
    fn gap(&mut self) {
        const GAPS: [&str; 6] = [" ", "\n", "  ", "\n\n  ", " /* c */ ", " // c\n"];
        let choice = self.layout[self.step % self.layout.len()];
        self.step += 1;
        self.out.push_str(GAPS[usize::from(choice) % GAPS.len()]);
    }

    fn tok(&mut self, text: &str) {
        self.gap();
        self.out.push_str(text);
    }

    fn comma(&mut self) {
        if self.layout[self.step % self.layout.len()].is_multiple_of(2) {
            self.tok(",");
        }
        self.step += 1;
    }

    fn list(&mut self, items: &[String]) {
        self.tok("[");
        for (index, item) in items.iter().enumerate() {
            if index > 0 {
                self.tok(",");
            }
            self.tok(item);
        }
        self.tok("]");
    }
}

impl Program {
    pub fn canonical(&self) -> Program {
        Program {
            items: self.items.clone(),
            layout: Vec::new(),
        }
    }

    pub fn print(&self) -> String {
        let mut p = Printer {
            out: "module m".to_owned(),
            layout: &self.layout,
            step: 0,
        };
        for item in &self.items {
            p.out.push('\n');
            print_item(&mut p, item);
        }
        p.out.push('\n');
        p.out
    }

    pub fn read(parsed: &Parse) -> Program {
        let items = parsed
            .lit_file()
            .map(|file| file.items())
            .unwrap_or_default()
            .iter()
            .filter_map(|item| read_item(item.syntax()))
            .collect();
        Program {
            items,
            layout: Vec::new(),
        }
    }
}

fn print_item(p: &mut Printer<'_>, item: &Item) {
    match item {
        Item::Group { name, label } => {
            p.out.push_str("group");
            p.tok(name);
            p.tok(":");
            p.tok(&format!("\"{label}\""));
        }
        Item::Enum { name, variants } => {
            p.out.push_str("enum");
            p.tok(name);
            p.tok("{");
            p.tok(&variants.join(", "));
            p.tok("}");
        }
        Item::Authority(levels) => {
            p.out.push_str("authority");
            p.tok(&levels.join(" > "));
        }
        Item::Deadline {
            name,
            days,
            event,
            args,
            rule,
        } => {
            p.out.push_str("deadline");
            p.tok(name);
            p.tok(":");
            p.tok(&format!("{days}.days"));
            p.tok("from");
            p.tok(&format!("{event}({})", args.join(", ")));
            p.tok("counted_by");
            p.tok(rule);
        }
        Item::Node(node) => print_node(p, node),
    }
}

fn print_node(p: &mut Printer<'_>, node: &Node) {
    p.out.push_str(&node.keyword);
    p.tok(&node.name);
    p.tok(":");
    p.tok(&format!("\"{}\"", node.label));
    if let Some(by) = &node.by {
        p.tok("by");
        p.tok(by);
    }
    if let Some(cite) = &node.cite {
        p.tok("cite");
        p.out.push_str(" \"");
        p.out.push_str(cite);
        p.out.push('"');
        p.out.push('\n');
    }
    p.tok("{");
    if !node.tags.is_empty() {
        p.tok("tags");
        p.list(&node.tags);
        p.comma();
    }
    for edge in &node.edges {
        p.tok(edge.id.as_deref().unwrap_or("_"));
        p.tok(&format!("\"{}\"", edge.label));
        p.tok("->");
        p.tok(&edge.target);
        if let Some((n, d)) = edge.p {
            p.tok("p");
            p.tok(&format!("{n}/{d}"));
        }
        if edge.effort.is_some() || edge.within.is_some() {
            p.tok("{");
            if let Some(hours) = edge.effort {
                p.tok("effort");
                p.tok(&format!("{hours}.hours"));
                p.comma();
            }
            if let Some(within) = &edge.within {
                p.tok("within");
                p.tok(within);
            }
            p.tok("}");
        }
        p.comma();
    }
    p.tok("}");
}

fn text(token: Option<litigate_syntax::SyntaxToken>) -> String {
    token.map(|t| t.text().to_owned()).unwrap_or_default()
}

fn unquote(token: Option<litigate_syntax::SyntaxToken>) -> String {
    text(token).trim_matches('"').to_owned()
}

fn read_item(node: &litigate_syntax::SyntaxNode) -> Option<Item> {
    if let Some(group) = GroupDecl::cast(node.clone()) {
        return Some(Item::Group {
            name: text(group.name()),
            label: unquote(group.label()),
        });
    }
    if let Some(decl) = EnumDecl::cast(node.clone()) {
        let variants = decl
            .variants()
            .iter()
            .map(|t| t.text().to_owned())
            .collect();
        return Some(Item::Enum {
            name: text(decl.name()),
            variants,
        });
    }
    if let Some(decl) = AuthorityDecl::cast(node.clone()) {
        return Some(Item::Authority(
            decl.levels().iter().map(|t| t.text().to_owned()).collect(),
        ));
    }
    if let Some(decl) = DeadlineDecl::cast(node.clone()) {
        let call = decl
            .trigger()
            .and_then(|e| litigate_syntax::ast::CallExpr::cast(e.syntax().clone()));
        let args = call
            .as_ref()
            .map(litigate_syntax::ast::CallExpr::args)
            .unwrap_or_default();
        return Some(Item::Deadline {
            name: text(decl.name()),
            days: text(decl.period())
                .trim_end_matches(".days")
                .parse()
                .unwrap_or(0),
            event: call
                .and_then(|c| c.callee())
                .map(|r| r.text())
                .unwrap_or_default(),
            args: args
                .iter()
                .filter_map(Expr::reference)
                .map(|r| r.text())
                .collect(),
            rule: decl.counted_by().map(|r| r.text()).unwrap_or_default(),
        });
    }
    NodeDecl::cast(node.clone()).map(|decl| Item::Node(read_node(&decl)))
}

fn read_node(decl: &NodeDecl) -> Node {
    let tags = decl
        .fields()
        .iter()
        .filter(|f| f.keyword_kind() == Some(SyntaxKind::KwTags))
        .flat_map(|f| f.id_list().map(|l| l.items()).unwrap_or_default())
        .map(|r| r.text())
        .collect();
    let edges = decl
        .edges()
        .iter()
        .map(|edge| {
            let fields = edge.fields();
            let field = |kw: SyntaxKind| {
                fields
                    .iter()
                    .find(|f| f.keyword_kind() == Some(kw))
                    .cloned()
            };
            let ratio = edge
                .probability()
                .and_then(|e| Ratio::cast(e.syntax().clone()));
            let number = |t: Option<litigate_syntax::SyntaxToken>| text(t).parse().unwrap_or(0);
            Edge {
                id: edge.id().map(|t| t.text().to_owned()),
                label: unquote(edge.label()),
                target: edge.target().map(|r| r.text()).unwrap_or_default(),
                p: ratio.map(|r| (number(r.numerator()), number(r.denominator()))),
                effort: field(SyntaxKind::KwEffort)
                    .and_then(|f| f.value_expr())
                    .and_then(|e| Literal::cast(e.syntax().clone()))
                    .map(|l| {
                        text(l.token())
                            .trim_end_matches(".hours")
                            .parse()
                            .unwrap_or(0)
                    }),
                within: field(SyntaxKind::KwWithin)
                    .and_then(|f| f.value_ref())
                    .map(|r| r.text()),
            }
        })
        .collect();
    Node {
        keyword: text(decl.keyword()),
        name: text(decl.name()),
        label: unquote(decl.label()),
        by: decl.by().map(|r| r.text()),
        cite: decl
            .cites()
            .first()
            .and_then(litigate_syntax::ast::Cite::text),
        tags,
        edges,
    }
}
