//! References, type expressions, expressions (SPEC §3.3), and JSON records.
use super::lex::Tok;
use super::tree::{Expr, ExprKind, Json, Name, PathSep, Pattern, RefPath, TypeRef};
use super::{PResult, Parser};
use crate::db::span::{Report, TextRange};
use crate::hir::{BinaryOp, UnaryOp};
use litigate_base::DiagnosticCode as C;

impl Parser {
    /// `Ident ["@" Ident] {("::" | ".") Ident} ["@" Date]`.
    pub(crate) fn ref_path(&mut self) -> PResult<RefPath> {
        let head = self.ident()?;
        let mut instance = None;
        if self.at_punct("@") && self.nth(1).kind == Tok::Ident {
            self.bump();
            instance = Some(self.ident()?);
        }
        let mut rest = Vec::new();
        loop {
            let sep = if self.at_punct(".") {
                PathSep::Dot
            } else if self.at_punct("::") {
                PathSep::Colons
            } else {
                break;
            };
            if self.nth(1).kind != Tok::Ident {
                break;
            }
            self.bump();
            rest.push((sep, self.ident()?));
        }
        let mut version = None;
        if self.at_punct("@") && self.nth(1).kind == Tok::Date {
            self.bump();
            version = Some(self.date()?);
        }
        let range = TextRange::new(head.range.start, self.prev_end());
        Ok(RefPath {
            head,
            instance,
            rest,
            version,
            range,
        })
    }

    /// `"[" [Ref {"," Ref} [","]] "]"` as a list expression of refs.
    pub(crate) fn id_list(&mut self) -> PResult<Expr> {
        let open = self.expect_punct("[")?.range;
        let mut items = Vec::new();
        while !self.at_punct("]") && self.peek().kind != Tok::Eof {
            let r = self.ref_path()?;
            items.push(Expr {
                range: r.range,
                kind: ExprKind::Ref(r),
            });
            if !self.eat_punct(",") {
                break;
            }
        }
        self.close(open, "]")?;
        Ok(Expr {
            kind: ExprKind::List(items),
            range: open.cover(TextRange::new(open.start, self.prev_end())),
        })
    }

    /// `Ident ["<" Type {"," Type} ">"]`.
    pub(crate) fn type_ref(&mut self) -> PResult<TypeRef> {
        let name = self.ident()?;
        let mut args = Vec::new();
        if self.eat_punct("<") {
            loop {
                args.push(self.type_ref()?);
                if !self.eat_punct(",") {
                    break;
                }
            }
            self.expect_punct(">")?;
        }
        let range = TextRange::new(name.range.start, self.prev_end());
        Ok(TypeRef { name, args, range })
    }

    fn finish(&self, start: u32, kind: ExprKind) -> Expr {
        Expr {
            kind,
            range: TextRange::new(start, self.prev_end()),
        }
    }

    /// `Expr = MatchExpr | OrExpr`.
    pub(crate) fn expr(&mut self) -> PResult<Expr> {
        if self.at_kw("match") && !self.nth_punct(1, "(") {
            let start = self.bump().range.start;
            let scrutinee = self.binary(0)?;
            let open = self.expect_punct("{")?.range;
            let mut arms = Vec::new();
            while !self.at_punct("}") && self.peek().kind != Tok::Eof {
                let pattern = if self.peek().kind == Tok::Underscore {
                    Pattern::Wildcard(self.bump().range)
                } else {
                    Pattern::Ref(self.ref_path()?)
                };
                self.expect_punct("=>")?;
                arms.push((pattern, self.expr()?));
                self.sep();
            }
            self.close(open, "}")?;
            return Ok(self.finish(start, ExprKind::Match(Box::new(scrutinee), arms)));
        }
        self.binary(0)
    }

    fn binary_op(&self, level: usize) -> Option<BinaryOp> {
        let t = self.peek();
        if t.kind != Tok::Punct {
            return None;
        }
        let op = match (level, t.text.as_str()) {
            (0, "||") => BinaryOp::Or,
            (1, "&&") => BinaryOp::And,
            (2, "==") => BinaryOp::Eq,
            (2, "!=") => BinaryOp::Ne,
            (2, "<") => BinaryOp::Lt,
            (2, "<=") => BinaryOp::Le,
            (2, ">") => BinaryOp::Gt,
            (2, ">=") => BinaryOp::Ge,
            (3, "+") => BinaryOp::Add,
            (3, "-") => BinaryOp::Sub,
            (4, "*") => BinaryOp::Mul,
            (4, "/") => BinaryOp::Div,
            (4, "%") => BinaryOp::Rem,
            _ => return None,
        };
        Some(op)
    }

    /// Precedence climbing over levels 0 (`||`) to 4 (`* / %`).
    fn binary(&mut self, level: usize) -> PResult<Expr> {
        if level > 4 {
            return self.unary();
        }
        let mut lhs = self.binary(level + 1)?;
        while let Some(op) = self.binary_op(level) {
            self.bump();
            let rhs = self.binary(level + 1)?;
            let range = lhs.range.cover(rhs.range);
            lhs = Expr {
                kind: ExprKind::Binary(op, Box::new(lhs), Box::new(rhs)),
                range,
            };
            if level == 2 {
                if self.binary_op(2).is_some() {
                    let t = self.peek().range;
                    self.report(Report::new(
                        C::UnexpectedToken,
                        t,
                        "comparison operators do not chain; parenthesize one side",
                    ));
                    return Err(super::Fail);
                }
                break;
            }
        }
        Ok(lhs)
    }

    fn unary(&mut self) -> PResult<Expr> {
        let start = self.peek().range.start;
        let op = if self.at_punct("!") {
            Some(UnaryOp::Not)
        } else if self.at_punct("-") {
            Some(UnaryOp::Neg)
        } else {
            None
        };
        if let Some(op) = op {
            self.bump();
            let operand = self.unary()?;
            return Ok(self.finish(start, ExprKind::Unary(op, Box::new(operand))));
        }
        let mut e = self.primary()?;
        while self.at_punct(".") && self.nth(1).kind == Tok::Ident {
            self.bump();
            let field = self.ident()?;
            e = self.finish(start, ExprKind::Field(Box::new(e), field));
        }
        Ok(e)
    }

    /// `?` or `?("reason")`; the `?` is current.
    pub(crate) fn hole(&mut self) -> PResult<Expr> {
        let start = self.bump().range.start;
        let mut reason = None;
        if self.at_punct("(") && self.nth(1).kind == Tok::Str {
            let open = self.bump().range;
            reason = Some(self.string()?.text);
            self.close(open, ")")?;
        }
        Ok(self.finish(start, ExprKind::Hole(reason)))
    }

    fn primary(&mut self) -> PResult<Expr> {
        let t = self.peek().clone();
        let start = t.range.start;
        let literal = match t.kind {
            Tok::Number => Some(ExprKind::Number(t.text.clone())),
            Tok::Money => Some(ExprKind::Money(t.text.clone())),
            Tok::Duration => Some(ExprKind::Duration(t.text.clone())),
            Tok::Str => Some(ExprKind::Str(t.text.clone())),
            Tok::Date => Some(ExprKind::Date(t.text.clone())),
            Tok::Ident if t.text == "true" || t.text == "false" => {
                Some(ExprKind::Bool(t.text == "true"))
            }
            Tok::Ident if t.text == "unit" => Some(ExprKind::Unit),
            _ => None,
        };
        if let Some(kind) = literal {
            self.bump();
            return Ok(self.finish(start, kind));
        }
        if t.kind == Tok::Ident && t.text == "cite" {
            let text = self.cite_text()?;
            return Ok(self.finish(start, ExprKind::Cite(text.text)));
        }
        if t.kind == Tok::Ident {
            let path = self.ref_path()?;
            if self.at_punct("(") {
                let open = self.bump().range;
                let args = self.comma_list(")")?;
                self.close(open, ")")?;
                return Ok(self.finish(start, ExprKind::Call(path, args)));
            }
            return Ok(self.finish(start, ExprKind::Ref(path)));
        }
        match t.text.as_str() {
            "?" if t.kind == Tok::Punct => self.hole(),
            "(" if t.kind == Tok::Punct => {
                let open = self.bump().range;
                let inner = self.expr()?;
                self.close(open, ")")?;
                Ok(self.finish(start, ExprKind::Paren(Box::new(inner))))
            }
            "[" if t.kind == Tok::Punct => {
                let open = self.bump().range;
                let items = self.comma_list("]")?;
                self.close(open, "]")?;
                Ok(self.finish(start, ExprKind::List(items)))
            }
            "{" if t.kind == Tok::Punct => self.record(),
            _ => self.unexpected("an expression"),
        }
    }

    fn comma_list(&mut self, closer: &str) -> PResult<Vec<Expr>> {
        let mut items = Vec::new();
        while !self.at_punct(closer) && self.peek().kind != Tok::Eof {
            items.push(self.expr()?);
            if !self.eat_punct(",") {
                break;
            }
        }
        Ok(items)
    }

    /// `"{" { Ident ":" Expr Sep } "}"`.
    pub(crate) fn record(&mut self) -> PResult<Expr> {
        let open = self.expect_punct("{")?.range;
        let mut fields = Vec::new();
        while !self.at_punct("}") && self.peek().kind != Tok::Eof {
            let name = self.ident()?;
            self.expect_punct(":")?;
            fields.push((name, self.expr()?));
            self.sep();
        }
        self.close(open, "}")?;
        Ok(self.finish(open.start, ExprKind::Record(fields)))
    }

    /// `JsonRecord`.
    pub(crate) fn json_record(&mut self) -> PResult<Json> {
        let open = self.expect_punct("{")?.range;
        let mut fields = Vec::new();
        while !self.at_punct("}") && self.peek().kind != Tok::Eof {
            let key = if self.peek().kind == Tok::Str {
                self.string()?
            } else {
                self.ident()?
            };
            self.expect_punct(":")?;
            fields.push((key, self.json_value()?));
            self.sep();
        }
        self.close(open, "}")?;
        Ok(Json::Object(fields))
    }

    fn json_value(&mut self) -> PResult<Json> {
        let t = self.peek().clone();
        match (t.kind, t.text.as_str()) {
            (Tok::Punct, "{") => self.json_record(),
            (Tok::Punct, "[") => {
                let open = self.bump().range;
                let mut items = Vec::new();
                while !self.at_punct("]") && self.peek().kind != Tok::Eof {
                    items.push(self.json_value()?);
                    if !self.eat_punct(",") {
                        break;
                    }
                }
                self.close(open, "]")?;
                Ok(Json::Array(items))
            }
            (Tok::Punct, "-") if self.nth(1).kind == Tok::Number => {
                self.bump();
                Ok(Json::Number(format!("-{}", self.bump().text)))
            }
            (Tok::Number, _) => Ok(Json::Number(self.bump().text)),
            (Tok::Str, _) => Ok(Json::Str(self.bump().text)),
            (Tok::Ident, "true" | "false") => Ok(Json::Bool(self.bump().text == "true")),
            (Tok::Ident, "unset") => {
                self.bump();
                Ok(Json::Unset)
            }
            _ => self.unexpected("a JSON value"),
        }
    }

    /// `Interval = "[" Date "," (Date | "..") ")"`.
    pub(crate) fn interval(&mut self) -> PResult<(Name, Option<Name>)> {
        self.expect_punct("[")?;
        let from = self.date()?;
        self.expect_punct(",")?;
        let to = if self.eat_punct("..") {
            None
        } else {
            Some(self.date()?)
        };
        self.expect_punct(")")?;
        Ok((from, to))
    }
}
