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
            if let Some(shape) = self.at_key(table) {
                match self.field(shape) {
                    Ok(f) => out.push(f),
                    Err(_) => self.recover_in_body(),
                }
            } else {
                self.unknown_key(table);
                self.recover_in_body();
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
        let start = self.peek().range.start;
        let mut values = Vec::new();
        let first = match shape {
            Shape::Flag => None,
            Shape::Interval => {
                let (from, to) = self.interval()?;
                Some(Atom::Interval(from, to))
            }
            Shape::Triangular if self.at_punct("{") => {
                Some(Atom::Fields(self.field_block(TRIANGULAR)?))
            }
            Shape::Cite => {
                self.pos -= 1;
                Some(Atom::Expr(self.cite_expr()?))
            }
            _ => Some(Atom::Expr(self.shaped_expr(shape)?)),
        };
        if let Some(atom) = first {
            values.push((atom, TextRange::new(start, self.prev_end())));
        }
        let next = self.peek().range.start;
        let second = match shape {
            Shape::Within | Shape::InlineDeadline if self.at_punct("{") => {
                Some(Atom::Fields(self.field_block(DEADLINE_OPTS)?))
            }
            Shape::ServiceExtension if self.at_kw("cite") => Some(Atom::Expr(self.cite_expr()?)),
            Shape::Patch => Some(Atom::Json(self.json_record()?)),
            _ => None,
        };
        if let Some(atom) = second {
            values.push((atom, TextRange::new(next, self.prev_end())));
        }
        let range = TextRange::new(key.range.start, self.prev_end().max(key.range.end));
        Ok(Field { key, values, range })
    }

    /// The expression-shaped value of `shape`.
    fn shaped_expr(&mut self, shape: Shape) -> PResult<Expr> {
        let token_expr = |p: &mut Self, kind: Tok, what: &str, make: fn(String) -> ExprKind| {
            p.token_name(kind, what).map(|n| Expr {
                kind: make(n.text),
                range: n.range,
            })
        };
        match shape {
            Shape::StrOrCite if self.at_kw("cite") => self.cite_expr(),
            Shape::Str | Shape::StrOrCite => token_expr(self, Tok::Str, "a string", ExprKind::Str),
            Shape::Date => token_expr(self, Tok::Date, "a date", ExprKind::Date),
            Shape::Ref | Shape::Within | Shape::Patch => {
                let r = self.ref_path()?;
                Ok(Expr {
                    range: r.range,
                    kind: ExprKind::Ref(r),
                })
            }
            Shape::IdList => self.id_list(),
            Shape::Record | Shape::ServiceExtension => self.record(),
            Shape::Prob => self.prob_value(),
            Shape::Bool => {
                let t = self.peek().clone();
                if t.kind == Tok::Ident && (t.text == "true" || t.text == "false") {
                    self.bump();
                    Ok(Expr {
                        kind: ExprKind::Bool(t.text == "true"),
                        range: t.range,
                    })
                } else {
                    self.unexpected("`true` or `false`")
                }
            }
            _ => self.atom(),
        }
    }

    /// `cite` text as an expression; the `cite` token is current.
    fn cite_expr(&mut self) -> PResult<Expr> {
        let c = self.cite_text()?;
        Ok(Expr {
            kind: ExprKind::Cite(c.text),
            range: c.range,
        })
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
