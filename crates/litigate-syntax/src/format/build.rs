//! CST to layout document: token spacing (SPEC §10), braced blocks, and
//! comment placement. Only whitespace and separator commas are rewritten;
//! every other token is printed verbatim and in order.
use super::doc::Doc;
use crate::parse::{SyntaxNode, SyntaxToken};
use crate::syntax_kind::SyntaxKind::{self, *};
use rowan::NodeOrToken;

type Element = NodeOrToken<SyntaxNode, SyntaxToken>;

/// The previous significant token and its parent node kind.
#[derive(Debug, Clone, Copy)]
struct Prev {
    kind: SyntaxKind,
    parent: SyntaxKind,
}

/// Walks one item (or block entry) and emits its document.
pub(crate) struct Builder {
    out: Vec<Doc>,
    prev: Option<Prev>,
    newlines: usize,
    after_cite: bool,
}

impl Builder {
    pub(crate) fn new() -> Self {
        Self {
            out: Vec::new(),
            prev: None,
            newlines: 0,
            after_cite: false,
        }
    }

    pub(crate) fn finish(self) -> Doc {
        Doc::Concat(self.out)
    }

    /// Emits every child of `node`.
    pub(crate) fn node(&mut self, node: &SyntaxNode) {
        let children: Vec<Element> = node.children_with_tokens().collect();
        let mut index = 0;
        while index < children.len() {
            match &children[index] {
                NodeOrToken::Node(child) => self.node(child),
                NodeOrToken::Token(token) if token.kind() == LBrace => {
                    index = self.block(node, &children, index);
                    continue;
                }
                NodeOrToken::Token(token) => self.token(token),
            }
            index += 1;
        }
    }

    /// Emits a trivia or significant token. (Separator commas never get
    /// here: [`Self::block`] drops them.)
    pub(crate) fn token(&mut self, token: &SyntaxToken) {
        match token.kind() {
            Whitespace => self.newlines += token.text().matches('\n').count(),
            Comment | DocComment => self.comment(token),
            kind => {
                let parent = token.parent().map_or(SourceFile, |p| p.kind());
                self.before_token(kind, parent);
                self.out.push(Doc::text(token.text()));
                self.after_cite = parent == CiteExpr && kind != KwCite && ends_node(token);
                self.prev = Some(Prev { kind, parent });
                self.newlines = 0;
            }
        }
    }

    fn before_token(&mut self, kind: SyntaxKind, parent: SyntaxKind) {
        if self.after_cite && !matches!(kind, RBrace | Comma) {
            self.out.push(if kind == LBrace {
                Doc::HardLine
            } else {
                Doc::Nest(vec![Doc::HardLine])
            });
            self.prev = None;
        }
        if let Some(prev) = self.prev {
            if spaced(prev, kind, parent) {
                self.out.push(Doc::text(" "));
            }
        }
    }

    fn comment(&mut self, token: &SyntaxToken) {
        let text = token.text().trim_end();
        let line_comment = !text.starts_with("/*");
        let at_line_start = self.prev.is_none();
        match self.prev {
            Some(_) if self.newlines > 0 => self.out.push(Doc::Nest(vec![Doc::HardLine])),
            Some(_) => self.out.push(Doc::text(" ")),
            None => {}
        }
        self.out.push(Doc::text(text));
        if line_comment {
            self.out.push(if at_line_start {
                Doc::HardLine
            } else {
                Doc::Nest(vec![Doc::HardLine])
            });
            self.prev = None;
        } else {
            self.prev = Some(Prev {
                kind: Comment,
                parent: SourceFile,
            });
        }
        self.after_cite = false;
        self.newlines = 0;
    }

    /// Emits the braced block opened at `children[open]`; returns the index
    /// after its closing brace (formatted files are well nested).
    fn block(&mut self, owner: &SyntaxNode, children: &[Element], open: usize) -> usize {
        let force = matches!(owner.kind(), EventsBlock | PartiesBlock) || self.after_cite;
        self.before_token(LBrace, owner.kind());
        let sep = separator_of(owner.kind());
        let mut entries: Vec<Entry> = Vec::new();
        let mut pending = Entry::default();
        let body = &children[open + 1..];
        let close = body
            .iter()
            .position(|child| child.kind() == RBrace)
            .unwrap_or(body.len());
        let mut newlines = 0usize;
        for child in &body[..close] {
            match child {
                NodeOrToken::Token(token) if token.kind() == Whitespace => {
                    newlines += token.text().matches('\n').count();
                }
                NodeOrToken::Token(token) if token.kind() == Comma => {}
                NodeOrToken::Token(token) if matches!(token.kind(), Comment | DocComment) => {
                    let text = Doc::text(token.text().trim_end());
                    match entries.last_mut() {
                        Some(last) if newlines == 0 => last.trailing.push(text),
                        _ => {
                            pending.blank |= newlines > 1 && !entries.is_empty();
                            pending.leading.push(text);
                        }
                    }
                    newlines = 0;
                }
                element => {
                    pending.blank |= newlines > 1 && !entries.is_empty();
                    pending.doc = Some(entry_doc(element));
                    entries.push(std::mem::take(&mut pending));
                    newlines = 0;
                }
            }
        }
        self.out
            .push(block_doc(&entries, &pending.leading, sep, force));
        self.prev = Some(Prev {
            kind: RBrace,
            parent: owner.kind(),
        });
        self.newlines = 0;
        self.after_cite = false;
        open + 1 + close + 1
    }
}

/// True when nothing but trivia follows `token` in its parent node: the
/// citation text (or `historical`) that ends a `cite` clause, which ends
/// its line (SPEC §10).
fn ends_node(token: &SyntaxToken) -> bool {
    std::iter::successors(
        token.next_sibling_or_token(),
        Element::next_sibling_or_token,
    )
    .all(|e| matches!(e.kind(), Whitespace | Comment | DocComment))
}

#[derive(Debug, Default)]
struct Entry {
    leading: Vec<Doc>,
    doc: Option<Doc>,
    trailing: Vec<Doc>,
    blank: bool,
}

fn entry_doc(element: &Element) -> Doc {
    let mut builder = Builder::new();
    match element {
        NodeOrToken::Node(node) => builder.node(node),
        NodeOrToken::Token(token) => builder.token(token),
    }
    builder.finish()
}

/// `{ a, b }` flat, or one entry per line when broken.
fn block_doc(entries: &[Entry], dangling: &[Doc], sep: Sep, force: bool) -> Doc {
    if entries.is_empty() && dangling.is_empty() {
        return Doc::text("{}");
    }
    let mut inner = Vec::new();
    for (index, entry) in entries.iter().enumerate() {
        if entry.blank {
            inner.push(Doc::BlankLine);
        }
        for comment in &entry.leading {
            inner.extend([Doc::Line, comment.clone(), Doc::BreakParent]);
        }
        inner.push(Doc::Line);
        inner.extend(entry.doc.clone());
        let last = index + 1 == entries.len();
        inner.push(match (sep, last) {
            (Sep::Optional, false) => Doc::IfBreak("", ","),
            (Sep::Required, false) => Doc::IfBreak(",", ","),
            (Sep::Required, true) => Doc::IfBreak(",", ""),
            _ => Doc::IfBreak("", ""),
        });
        for comment in &entry.trailing {
            inner.extend([Doc::text(" "), comment.clone(), Doc::BreakParent]);
        }
    }
    for comment in dangling {
        inner.extend([Doc::Line, comment.clone(), Doc::BreakParent]);
    }
    Doc::group(
        vec![Doc::text("{"), Doc::Nest(inner), Doc::Line, Doc::text("}")],
        force,
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Sep {
    Optional,
    Required,
    Forbidden,
}

fn separator_of(owner: SyntaxKind) -> Sep {
    match owner {
        EnumDecl | ImportList => Sep::Required,
        TestDecl | EventsBlock => Sep::Forbidden,
        _ => Sep::Optional,
    }
}

/// Whether one space separates `prev` from the next token (SPEC §10).
fn spaced(prev: Prev, kind: SyntaxKind, parent: SyntaxKind) -> bool {
    let tight = |k: SyntaxKind| matches!(k, Dot | ColonColon | At);
    let version = |p: SyntaxKind| matches!(p, VersionReq | Version);
    !(matches!(kind, Comma | Colon | RParen | RBracket)
        || tight(kind)
        || tight(prev.kind)
        || matches!(prev.kind, LParen | LBracket)
        || (kind == LParen && matches!(parent, ArgList | ParamList | HoleExpr))
        || (kind == LBracket && parent == IndexClause)
        || (matches!(kind, Less | Greater) && parent == TypeArgs)
        || (prev.kind == Less && prev.parent == TypeArgs)
        || (matches!(prev.kind, Minus | Bang) && prev.parent == UnaryExpr)
        || (prev.parent == Ratio && parent == Ratio)
        || (version(prev.parent) && version(parent)))
}
