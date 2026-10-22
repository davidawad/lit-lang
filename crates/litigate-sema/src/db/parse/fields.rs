//! Keyed fields and clauses. Each context has a closed table of keys and
//! the value shape each key takes, which is what keeps juxtaposed items
//! unambiguous (SPEC §3.4).
use super::lex::Tok;
use super::tree::{Atom, Expr, ExprKind, Field};
use super::{PResult, Parser};
use crate::db::span::{Report, TextRange};
use crate::hir::BinaryOp;
use litigate_base::{Applicability, DiagnosticCode as C};

/// Value shape of one key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Shape {
    Flag,
    Str,
    Ref,
    IdList,
    Record,
    Atom,
    Triangular,
    StrOrCite,
    Within,
    InlineDeadline,
    Prob,
    Bool,
    Interval,
    Date,
    Cite,
    Patch,
    ServiceExtension,
}

pub(crate) type Table = &'static [(&'static str, Shape)];

const VERSION: [(&str, Shape); 3] = [
    ("effective", Shape::Interval),
    ("known_at", Shape::Date),
    ("cite", Shape::Cite),
];
const RELATIONS: [(&str, Shape); 6] = [
    ("subject_to", Shape::Ref),
    ("notwithstanding", Shape::Ref),
    ("supersedes", Shape::Ref),
    ("revives", Shape::Ref),
    ("override", Shape::Ref),
    ("delegable_to", Shape::IdList),
];

/// Version, relation, and cite clauses (rules, deadlines, counting, defaults).
pub(crate) const DECL_CLAUSES: Table = &[
    VERSION[0],
    VERSION[1],
    VERSION[2],
    RELATIONS[0],
    RELATIONS[1],
    RELATIONS[2],
    RELATIONS[3],
    RELATIONS[4],
    RELATIONS[5],
];
/// Version and cite clauses (calendars).
pub(crate) const VERSION_CLAUSES: Table = &VERSION;
/// Cite only (transitions, exceptions, tests).
pub(crate) const CITE_CLAUSES: Table = &[("cite", Shape::Cite)];
/// Node header clauses.
pub(crate) const NODE_CLAUSES: Table = &[
    ("by", Shape::Ref),
    ("discretion", Shape::Flag),
    ("outcome", Shape::IdList),
    VERSION[0],
    VERSION[1],
    VERSION[2],
    RELATIONS[0],
    RELATIONS[1],
    RELATIONS[2],
    RELATIONS[3],
    RELATIONS[4],
    RELATIONS[5],
];
/// Node body fields.
pub(crate) const NODE_FIELDS: Table = &[
    ("group", Shape::Ref),
    ("note", Shape::Str),
    ("valence", Shape::Ref),
    ("court_listener_url", Shape::Str),
    ("tags", Shape::IdList),
    ("attrs", Shape::Record),
    ("payoff", Shape::Atom),
    ("payoff_by_flag", Shape::Record),
];
/// Edge clauses.
pub(crate) const EDGE_CLAUSES: Table = &[("by", Shape::Ref), ("p", Shape::Prob)];
/// Edge fields.
pub(crate) const EDGE_FIELDS: Table = &[
    ("event", Shape::Atom),
    ("authority", Shape::StrOrCite),
    ("effort", Shape::Atom),
    ("cost", Shape::Atom),
    ("within", Shape::Within),
    ("deadline", Shape::InlineDeadline),
    ("duration", Shape::Triangular),
    ("valence", Shape::Ref),
    ("court_listener_url", Shape::Str),
    ("action_id", Shape::Str),
    ("note", Shape::Str),
    ("tags", Shape::IdList),
    ("attrs", Shape::Record),
    ("sets", Shape::IdList),
    ("clears", Shape::IdList),
    ("requires", Shape::IdList),
    ("forbids", Shape::IdList),
    ("replaces", Shape::IdList),
];
/// Deadline options.
pub(crate) const DEADLINE_OPTS: Table = &[
    ("extendable", Shape::Bool),
    ("extension_authority", Shape::Str),
    ("note", Shape::Str),
];
/// Counting items.
pub(crate) const COUNTING_ITEMS: Table = &[
    ("exclude_trigger_day", Shape::Flag),
    ("include_last_day", Shape::Flag),
    ("allow_hours", Shape::Flag),
    ("roll_forward_on", Shape::IdList),
    ("service_extension", Shape::ServiceExtension),
    ("holidays", Shape::Ref),
    ("anchor", Shape::Ref),
    ("short_period", Shape::Record),
];
/// Instance fields.
pub(crate) const INSTANCE_FIELDS: Table = &[
    ("pack", Shape::Ref),
    ("note", Shape::Str),
    ("remove_edges", Shape::IdList),
    ("probabilities", Shape::Record),
    ("roles", Shape::Record),
    ("payoff_transform", Shape::Str),
    ("patch", Shape::Patch),
];
/// `package` header fields.
pub(crate) const PACKAGE_FIELDS: Table = &[("forum", Shape::Ref), ("authority", Shape::Ref)];
const TRIANGULAR: Table = &[
    ("min", Shape::Atom),
    ("mode", Shape::Atom),
    ("max", Shape::Atom),
];

fn lookup(table: Table, key: &str) -> Option<Shape> {
    table.iter().find(|(k, _)| *k == key).map(|(_, s)| *s)
}

fn keys(table: Table) -> String {
    table
        .iter()
        .map(|(k, _)| format!("`{k}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

impl Parser {
    fn at_key(&self, table: Table) -> Option<Shape> {
        let t = self.peek();
        (t.kind == Tok::Ident)
            .then(|| lookup(table, &t.text))
            .flatten()
    }

    /// Juxtaposed header clauses from `table` (stops at the first other token).
    pub(crate) fn clauses(&mut self, table: Table) -> PResult<Vec<Field>> {
        let mut out = Vec::new();
        while let Some(shape) = self.at_key(table) {
            out.push(self.field(shape)?);
        }
        Ok(out)
    }

    /// `"{" { Field Sep } "}"` with every key drawn from `table`; unknown keys
    /// are L2007 and skipped.
    pub(crate) fn field_block(&mut self, table: Table) -> PResult<Vec<Field>> {
        let open = self.expect_punct("{")?.range;
        let mut out = Vec::new();
        while !self.at_punct("}") && self.peek().kind != Tok::Eof {
            if self.peek().kind == Tok::DocComment {
                self.bump();
                continue;
            }
            match self.at_key(table) {
                Some(shape) => match self.field(shape) {
                    Ok(f) => out.push(f),
                    Err(_) => self.recover_in_body(),
                },
                None => {
                    self.unknown_key(table);
                    self.recover_in_body();
                }
            }
            self.sep();
        }
        self.close(open, "}")?;
        Ok(out)
    }

    pub(crate) fn unknown_key(&mut self, table: Table) {
        let t = self.peek().clone();
        let report = if t.kind == Tok::Ident {
            Report::new(
                C::InvalidField,
                t.range,
                format!("unknown field `{}`", t.text),
            )
            .note(format!("expected one of {}", keys(table)))
        } else {
            Report::new(
                C::UnexpectedToken,
                t.range,
                format!("expected a field, found {}", Self::describe(&t)),
            )
        };
        self.report(report);
    }

    /// One `key value...`; the key token is current and known.
    pub(crate) fn field(&mut self, shape: Shape) -> PResult<Field> {
        let key = self.ident_any()?;
        let mut values = Vec::new();
        let mut push = |p: &Self, atom: Atom, start: u32| {
            values.push((atom, TextRange::new(start, p.prev_end())))
        };
        let start = self.peek().range.start;
        match shape {
            Shape::Flag => {}
            Shape::Str => {
                let s = self.string()?;
                push(
                    self,
                    Atom::Expr(Expr {
                        kind: ExprKind::Str(s.text),
                        range: s.range,
                    }),
                    start,
                );
            }
            Shape::Ref => {
                let r = self.ref_path()?;
                push(
                    self,
                    Atom::Expr(Expr {
                        range: r.range,
                        kind: ExprKind::Ref(r),
                    }),
                    start,
                );
            }
            Shape::IdList => {
                let e = self.id_list()?;
                push(self, Atom::Expr(e), start);
            }
            Shape::Record => {
                let e = self.record()?;
                push(self, Atom::Expr(e), start);
            }
            Shape::Atom => {
                let e = self.atom()?;
                push(self, Atom::Expr(e), start);
            }
            Shape::Triangular if self.at_punct("{") => {
                let f = self.field_block(TRIANGULAR)?;
                push(self, Atom::Fields(f), start);
            }
            Shape::Triangular => {
                let e = self.atom()?;
                push(self, Atom::Expr(e), start);
            }
            Shape::StrOrCite if self.at_kw("cite") => {
                let c = self.cite_text()?;
                push(
                    self,
                    Atom::Expr(Expr {
                        kind: ExprKind::Cite(c.text),
                        range: c.range,
                    }),
                    start,
                );
            }
            Shape::StrOrCite => {
                let s = self.string()?;
                push(
                    self,
                    Atom::Expr(Expr {
                        kind: ExprKind::Str(s.text),
                        range: s.range,
                    }),
                    start,
                );
            }
            Shape::Within | Shape::InlineDeadline => {
                let e = if shape == Shape::Within {
                    let r = self.ref_path()?;
                    Expr {
                        range: r.range,
                        kind: ExprKind::Ref(r),
                    }
                } else {
                    self.atom()?
                };
                push(self, Atom::Expr(e), start);
                if self.at_punct("{") {
                    let s = self.peek().range.start;
                    let opts = self.field_block(DEADLINE_OPTS)?;
                    push(self, Atom::Fields(opts), s);
                }
            }
            Shape::Prob => {
                let e = self.prob_value()?;
                push(self, Atom::Expr(e), start);
            }
            Shape::Bool => {
                let t = self.peek().clone();
                if t.kind == Tok::Ident && (t.text == "true" || t.text == "false") {
                    self.bump();
                    push(
                        self,
                        Atom::Expr(Expr {
                            kind: ExprKind::Bool(t.text == "true"),
                            range: t.range,
                        }),
                        start,
                    );
                } else {
                    return self.unexpected("`true` or `false`");
                }
            }
            Shape::Interval => {
                let (from, to) = self.interval()?;
                push(self, Atom::Interval(from, to), start);
            }
            Shape::Date => {
                let d = self.date()?;
                push(
                    self,
                    Atom::Expr(Expr {
                        kind: ExprKind::Date(d.text),
                        range: d.range,
                    }),
                    start,
                );
            }
            Shape::Cite => {
                self.pos -= 1;
                let c = self.cite_text()?;
                push(
                    self,
                    Atom::Expr(Expr {
                        kind: ExprKind::Cite(c.text),
                        range: c.range,
                    }),
                    start,
                );
            }
            Shape::Patch => {
                let r = self.ref_path()?;
                push(
                    self,
                    Atom::Expr(Expr {
                        range: r.range,
                        kind: ExprKind::Ref(r),
                    }),
                    start,
                );
                let s = self.peek().range.start;
                let j = self.json_record()?;
                push(self, Atom::Json(j), s);
            }
            Shape::ServiceExtension => {
                let e = self.record()?;
                push(self, Atom::Expr(e), start);
                if self.at_kw("cite") {
                    let s = self.peek().range.start;
                    let c = self.cite_text()?;
                    push(
                        self,
                        Atom::Expr(Expr {
                            kind: ExprKind::Cite(c.text),
                            range: c.range,
                        }),
                        s,
                    );
                }
            }
        }
        let range = TextRange::new(key.range.start, self.prev_end().max(key.range.end));
        Ok(Field { key, values, range })
    }

    /// Any identifier including the reserved `cite` (a clause key).
    fn ident_any(&mut self) -> PResult<super::tree::Name> {
        if self.peek().kind == Tok::Ident {
            let t = self.bump();
            Ok(super::tree::Name {
                text: t.text,
                range: t.range,
            })
        } else {
            self.unexpected("an identifier")
        }
    }

    /// A field-value atom: hole, or a unary expression (`-5.usd`, `Served(x)`).
    pub(crate) fn atom(&mut self) -> PResult<Expr> {
        if self.at_punct("?") {
            return self.hole();
        }
        let start = self.peek().range.start;
        if self.at_punct("-") {
            self.bump();
            let inner = self.atom()?;
            return Ok(Expr {
                kind: ExprKind::Unary(crate::hir::UnaryOp::Neg, Box::new(inner)),
                range: TextRange::new(start, self.prev_end()),
            });
        }
        let t = self.peek().clone();
        match t.kind {
            Tok::Number | Tok::Money | Tok::Duration | Tok::Str | Tok::Date => {
                self.bump();
                let kind = match t.kind {
                    Tok::Number => ExprKind::Number(t.text),
                    Tok::Money => ExprKind::Money(t.text),
                    Tok::Duration => ExprKind::Duration(t.text),
                    Tok::Str => ExprKind::Str(t.text),
                    _ => ExprKind::Date(t.text),
                };
                Ok(Expr {
                    kind,
                    range: t.range,
                })
            }
            Tok::Ident => {
                let r = self.ref_path()?;
                if self.at_punct("(") {
                    let open = self.bump().range;
                    let mut args = Vec::new();
                    while !self.at_punct(")") && self.peek().kind != Tok::Eof {
                        args.push(self.expr()?);
                        if !self.eat_punct(",") {
                            break;
                        }
                    }
                    self.close(open, ")")?;
                    return Ok(Expr {
                        kind: ExprKind::Call(r, args),
                        range: TextRange::new(start, self.prev_end()),
                    });
                }
                Ok(Expr {
                    range: r.range,
                    kind: ExprKind::Ref(r),
                })
            }
            _ => self.unexpected("a value"),
        }
    }

    /// `ProbValue = Number ["/" Number] | Hole | "(" Expr ")"`.
    pub(crate) fn prob_value(&mut self) -> PResult<Expr> {
        if self.at_punct("?") {
            return self.hole();
        }
        let start = self.peek().range.start;
        if self.at_punct("(") {
            let open = self.bump().range;
            let inner = self.expr()?;
            self.close(open, ")")?;
            return Ok(Expr {
                kind: ExprKind::Paren(Box::new(inner)),
                range: TextRange::new(start, self.prev_end()),
            });
        }
        let num = self.token_name(Tok::Number, "a probability")?;
        let lhs = Expr {
            kind: ExprKind::Number(num.text),
            range: num.range,
        };
        if !self.at_punct("/") {
            return Ok(lhs);
        }
        self.bump();
        if self.peek().kind != Tok::Number {
            let t = self.peek().range;
            self.report(
                Report::new(C::UnexpectedToken, t, "expected a denominator after `/`").fix(
                    TextRange::new(t.start, t.start),
                    "1 ",
                    Applicability::HasPlaceholders,
                ),
            );
            return Err(super::Fail);
        }
        let den = self.bump();
        let rhs = Expr {
            kind: ExprKind::Number(den.text),
            range: den.range,
        };
        Ok(Expr {
            kind: ExprKind::Binary(BinaryOp::Div, Box::new(lhs), Box::new(rhs)),
            range: TextRange::new(start, self.prev_end()),
        })
    }
}
