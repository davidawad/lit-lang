//! Module header and declarations (SPEC §3.1).
use super::fields::{
    CITE_CLAUSES, COUNTING_ITEMS, DECL_CLAUSES, EDGE_CLAUSES, EDGE_FIELDS, INSTANCE_FIELDS,
    NODE_CLAUSES, NODE_FIELDS, PACKAGE_FIELDS, VERSION_CLAUSES,
};
use super::lex::Tok;
use super::tree::{
    ArmItem, EdgeItem, HolidayItem, Import, Item, ItemKind, ItemTree, Name, NodeItem,
    PackageHeader, TestStmt,
};
use super::{PResult, Parser};
use crate::db::span::{Report, TextRange};
use litigate_base::DiagnosticCode as C;

impl Parser {
    pub(crate) fn file(&mut self) -> ItemTree {
        let mut tree = ItemTree::default();
        let header = self.header(&mut tree);
        if header.is_err() {
            self.recover_top();
        }
        while self.peek().kind != Tok::Eof {
            match self.item_with_doc() {
                Ok(Some(item)) => tree.items.push(item),
                Ok(None) => {}
                Err(_) => self.recover_top(),
            }
        }
        tree
    }

    fn module_path(&mut self) -> PResult<Vec<Name>> {
        let mut path = vec![self.ident()?];
        while self.at_punct(".") && self.nth(1).kind == Tok::Ident {
            self.bump();
            path.push(self.ident()?);
        }
        Ok(path)
    }

    fn header(&mut self, tree: &mut ItemTree) -> PResult<()> {
        self.skip_docs_before_header();
        self.expect_kw("module")?;
        tree.module = self.module_path()?;
        while self.at_kw("import") {
            let start = self.bump().range.start;
            let path = self.module_path()?;
            let mut names = None;
            if self.at_punct(".") && self.nth_punct(1, "{") {
                self.bump();
                let open = self.bump().range;
                let mut list = Vec::new();
                while !self.at_punct("}") && self.peek().kind != Tok::Eof {
                    list.push(self.ident()?);
                    if !self.eat_punct(",") {
                        break;
                    }
                }
                self.close(open, "}")?;
                names = Some(list);
            }
            let range = TextRange::new(start, self.prev_end());
            tree.imports.push(Import { path, names, range });
        }
        if self.at_kw("package") {
            let start = self.bump().range.start;
            let name = self.ident()?;
            let fields = self.field_block(PACKAGE_FIELDS)?;
            let range = TextRange::new(start, self.prev_end());
            tree.package = Some(PackageHeader {
                name,
                fields,
                range,
            });
        }
        Ok(())
    }

    fn skip_docs_before_header(&mut self) {
        while self.peek().kind == Tok::DocComment {
            let t = self.bump();
            self.report(Report::new(
                C::DanglingDocComment,
                t.range,
                "doc comment is not followed by a declaration",
            ));
        }
    }

    fn item_with_doc(&mut self) -> PResult<Option<Item>> {
        let mut doc: Vec<String> = Vec::new();
        let mut doc_range = None::<TextRange>;
        while self.peek().kind == Tok::DocComment {
            let t = self.bump();
            doc_range = Some(doc_range.map_or(t.range, |r| r.cover(t.range)));
            doc.push(t.text);
        }
        if self.peek().kind == Tok::Eof {
            if let Some(range) = doc_range {
                self.report(Report::new(
                    C::DanglingDocComment,
                    range,
                    "doc comment is not followed by a declaration",
                ));
            }
            return Ok(None);
        }
        let mut item = self.item()?;
        if !doc.is_empty() {
            item.doc = Some(doc.join("\n"));
        }
        Ok(Some(item))
    }

    fn label(&mut self) -> PResult<Name> {
        self.expect_punct(":")?;
        self.string()
    }

    /// One declaration (top level or inside a `rule`).
    pub(crate) fn item(&mut self) -> PResult<Item> {
        let t = self.peek().clone();
        let start = t.range.start;
        if t.kind != Tok::Ident {
            return self.unexpected("a declaration");
        }
        let kind = match t.text.as_str() {
            "start" => {
                self.bump();
                ItemKind::Start(self.ref_path()?)
            }
            "group" => {
                self.bump();
                let name = self.ident()?;
                ItemKind::Group(name, self.label()?)
            }
            "type" => {
                self.bump();
                let name = self.ident()?;
                self.expect_punct("=")?;
                ItemKind::TypeAlias(name, self.type_ref()?)
            }
            "enum" => {
                self.bump();
                let name = self.ident()?;
                let open = self.expect_punct("{")?.range;
                let mut variants = Vec::new();
                while !self.at_punct("}") && self.peek().kind != Tok::Eof {
                    variants.push(self.ident()?);
                    if !self.eat_punct(",") {
                        break;
                    }
                }
                self.close(open, "}")?;
                ItemKind::Enum(name, variants)
            }
            "event" => {
                self.bump();
                let name = self.ident()?;
                let open = self.expect_punct("(")?.range;
                let mut params = Vec::new();
                while !self.at_punct(")") && self.peek().kind != Tok::Eof {
                    let p = self.ident()?;
                    self.expect_punct(":")?;
                    params.push((p, self.type_ref()?));
                    if !self.eat_punct(",") {
                        break;
                    }
                }
                self.close(open, ")")?;
                ItemKind::Event(name, params)
            }
            "state" | "decision" | "chance" | "terminal" => ItemKind::Node(self.node()?),
            "link" => {
                self.bump();
                ItemKind::Link(self.edge(true)?)
            }
            "instance" => {
                self.bump();
                let base = self.ident()?;
                self.expect_punct("@")?;
                let origin = self.ident()?;
                ItemKind::Instance(base, origin, self.field_block(INSTANCE_FIELDS)?)
            }
            "rule" => self.rule()?,
            "deadline" => self.deadline()?,
            "counting" => {
                self.bump();
                let name = self.ident()?;
                let clauses = self.clauses(DECL_CLAUSES)?;
                ItemKind::Counting(name, clauses, self.field_block(COUNTING_ITEMS)?)
            }
            "calendar" => self.calendar()?,
            "transition" => self.transition()?,
            "authority" => {
                self.bump();
                let mut chain = vec![self.ident()?];
                while self.eat_punct(">") {
                    chain.push(self.ident()?);
                }
                ItemKind::Authority(chain)
            }
            "default" => {
                self.bump();
                let name = self.ident()?;
                let clauses = self.clauses(DECL_CLAUSES)?;
                self.expect_kw("when")?;
                let when = self.expr()?;
                self.expect_punct("=>")?;
                ItemKind::Default(name, clauses, when, self.expr()?)
            }
            "exception" => self.exception()?,
            "test" => self.test()?,
            _ => return self.unexpected("a declaration"),
        };
        Ok(Item {
            doc: None,
            kind,
            range: TextRange::new(start, self.prev_end()),
        })
    }

    fn node(&mut self) -> PResult<NodeItem> {
        let keyword = self.ident()?;
        let name = self.ident()?;
        let label = self.label()?;
        let clauses = self.clauses(NODE_CLAUSES)?;
        let mut node = NodeItem {
            keyword,
            name,
            label,
            clauses,
            fields: Vec::new(),
            edges: Vec::new(),
        };
        if !self.at_punct("{") {
            return Ok(node);
        }
        let open = self.bump().range;
        while !self.at_punct("}") && self.peek().kind != Tok::Eof {
            let t = self.peek().clone();
            let is_edge = matches!(t.kind, Tok::Ident | Tok::Underscore)
                && self.nth(1).kind == Tok::Str
                && self.nth_punct(2, "->");
            let result = if t.kind == Tok::DocComment {
                self.bump();
                continue;
            } else if is_edge {
                self.edge(false).map(|e| node.edges.push(e))
            } else if let Some((_, shape)) = NODE_FIELDS
                .iter()
                .find(|(k, _)| t.kind == Tok::Ident && *k == t.text)
            {
                self.field(*shape).map(|f| node.fields.push(f))
            } else {
                self.unknown_key(NODE_FIELDS);
                Err(super::Fail)
            };
            if result.is_err() {
                self.recover_in_body();
            }
            self.sep();
        }
        self.close(open, "}")?;
        Ok(node)
    }

    /// `(Ident | "_") String "->" Ref {EdgeClause} [EdgeFields]`; links put
    /// `":" Ref` before the arrow.
    fn edge(&mut self, link: bool) -> PResult<EdgeItem> {
        let first = self.peek().clone();
        let start = first.range.start;
        let name = if first.kind == Tok::Underscore {
            self.bump();
            None
        } else {
            Some(self.ident()?)
        };
        let label = self.string()?;
        let from = if link {
            self.expect_punct(":")?;
            Some(self.ref_path()?)
        } else {
            None
        };
        self.expect_punct("->")?;
        let to = self.ref_path()?;
        let clauses = self.clauses(EDGE_CLAUSES)?;
        let fields = if self.at_punct("{") {
            self.field_block(EDGE_FIELDS)?
        } else {
            Vec::new()
        };
        Ok(EdgeItem {
            name,
            name_range: first.range,
            label,
            from,
            to,
            clauses,
            fields,
            range: TextRange::new(start, self.prev_end()),
        })
    }

    fn rule(&mut self) -> PResult<ItemKind> {
        self.bump();
        let kind = self.ident()?;
        if !matches!(kind.text.as_str(), "duty" | "power" | "liability") {
            self.report(Report::new(
                C::UnexpectedToken,
                kind.range,
                format!(
                    "expected `duty`, `power`, or `liability`, found `{}`",
                    kind.text
                ),
            ));
        }
        let name = self.ident()?;
        let clauses = self.clauses(DECL_CLAUSES)?;
        let open = self.expect_punct("{")?.range;
        let mut items = Vec::new();
        while !self.at_punct("}") && self.peek().kind != Tok::Eof {
            match self.item_with_doc() {
                Ok(Some(item)) => items.push(item),
                Ok(None) => {}
                Err(_) => self.recover_in_body(),
            }
            self.sep();
        }
        self.close(open, "}")?;
        Ok(ItemKind::Rule(kind, name, clauses, items))
    }

    fn deadline(&mut self) -> PResult<ItemKind> {
        self.bump();
        let name = self.ident()?;
        self.expect_punct(":")?;
        let amount = self.atom()?;
        let direction = self.ident()?;
        if !matches!(direction.text.as_str(), "from" | "after" | "before") {
            self.report(Report::new(
                C::UnexpectedToken,
                direction.range,
                format!(
                    "expected `from`, `after`, or `before`, found `{}`",
                    direction.text
                ),
            ));
        }
        let trigger = self.expr()?;
        self.expect_kw("counted_by")?;
        let counted_by = self.ref_path()?;
        let clauses = self.clauses(DECL_CLAUSES)?;
        let opts = if self.at_punct("{") {
            self.field_block(super::fields::DEADLINE_OPTS)?
        } else {
            Vec::new()
        };
        Ok(ItemKind::Deadline {
            name,
            amount,
            direction,
            trigger,
            counted_by,
            clauses,
            opts,
        })
    }

    fn calendar(&mut self) -> PResult<ItemKind> {
        self.bump();
        let name = self.ident()?;
        let clauses = self.clauses(VERSION_CLAUSES)?;
        let open = self.expect_punct("{")?.range;
        let mut holidays = Vec::new();
        while !self.at_punct("}") && self.peek().kind != Tok::Eof {
            match self.holiday() {
                Ok(h) => holidays.push(h),
                Err(_) => self.recover_in_body(),
            }
            self.sep();
        }
        self.close(open, "}")?;
        Ok(ItemKind::Calendar(name, clauses, holidays))
    }

    fn holiday(&mut self) -> PResult<HolidayItem> {
        let name = self.ident()?;
        self.expect_punct(":")?;
        let rule = self.atom()?;
        self.expect_kw("effective")?;
        let effective = self.interval()?;
        let observed = if self.eat_kw("observed") {
            Some(self.ident()?)
        } else {
            None
        };
        Ok(HolidayItem {
            name,
            rule,
            effective,
            observed,
        })
    }

    fn transition(&mut self) -> PResult<ItemKind> {
        self.bump();
        let name = self.ident()?;
        self.expect_kw("amends")?;
        let amends = self.ref_path()?;
        let clauses = self.clauses(CITE_CLAUSES)?;
        let open = self.expect_punct("{")?.range;
        let mut arms = Vec::new();
        while !self.at_punct("}") && self.peek().kind != Tok::Eof {
            let start = self.peek().range.start;
            let unless = if self.eat_kw("unless") {
                Some(self.expr()?)
            } else {
                self.expect_kw("otherwise")?;
                None
            };
            self.expect_punct("->")?;
            let target = self.ref_path()?;
            arms.push(ArmItem {
                unless,
                target,
                range: TextRange::new(start, self.prev_end()),
            });
            self.sep();
        }
        self.close(open, "}")?;
        Ok(ItemKind::Transition(name, amends, clauses, arms))
    }

    fn exception(&mut self) -> PResult<ItemKind> {
        self.bump();
        let name = self.ident()?;
        let kind = self.ident()?;
        if !matches!(kind.text.as_str(), "rebuts" | "undercuts") {
            self.report(Report::new(
                C::UnexpectedToken,
                kind.range,
                format!("expected `rebuts` or `undercuts`, found `{}`", kind.text),
            ));
        }
        let target = self.ref_path()?;
        let clauses = self.clauses(CITE_CLAUSES)?;
        self.expect_kw("when")?;
        let when = self.expr()?;
        let then = if self.eat_punct("=>") {
            Some(self.expr()?)
        } else {
            None
        };
        let priority = if self.eat_kw("priority") {
            Some(self.atom()?)
        } else {
            None
        };
        Ok(ItemKind::Exception {
            name,
            kind,
            target,
            clauses,
            when,
            then,
            priority,
        })
    }

    fn test(&mut self) -> PResult<ItemKind> {
        self.bump();
        let name = self.string()?;
        let clauses = self.clauses(CITE_CLAUSES)?;
        let open = self.expect_punct("{")?.range;
        let mut stmts = Vec::new();
        while !self.at_punct("}") && self.peek().kind != Tok::Eof {
            let stmt = if self.eat_kw("given") {
                let e = self.expr()?;
                self.expect_kw("on")?;
                TestStmt::Given(e, self.date()?)
            } else if self.eat_kw("expect") {
                TestStmt::Expect(self.expr()?)
            } else {
                return self.unexpected("`given` or `expect`");
            };
            stmts.push(stmt);
        }
        self.close(open, "}")?;
        Ok(ItemKind::Test(name, clauses, stmts))
    }
}
