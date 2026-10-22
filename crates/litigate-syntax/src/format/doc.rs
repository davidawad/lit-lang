//! A small Wadler-style pretty-printer: groups print flat when the rest of
//! their line fits, and break (one entry per line) otherwise.

/// Layout document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Doc {
    /// Literal text (never contains a newline).
    Text(String),
    /// A space when flat, a newline when broken.
    Line,
    /// Always a newline; forces every enclosing group to break.
    HardLine,
    /// An empty line before the next `Line` when broken; nothing when flat.
    BlankLine,
    /// Prints nothing; forces every enclosing group to break.
    BreakParent,
    /// `(broken, flat)` text chosen by the enclosing group's mode.
    IfBreak(&'static str, &'static str),
    /// Contents indented two more columns after each newline.
    Nest(Vec<Doc>),
    /// A layout unit; `force` breaks it unconditionally.
    Group { docs: Vec<Doc>, force: bool },
    /// Sequence.
    Concat(Vec<Doc>),
}

impl Doc {
    pub(crate) fn text(text: impl Into<String>) -> Self {
        Self::Text(text.into())
    }

    pub(crate) fn group(docs: Vec<Doc>, force: bool) -> Self {
        let force = force || docs.iter().any(Doc::breaks);
        Self::Group { docs, force }
    }

    /// True when this document forces its enclosing group to break.
    fn breaks(&self) -> bool {
        match self {
            Self::HardLine | Self::BreakParent => true,
            Self::Group { force, .. } => *force,
            Self::Nest(docs) | Self::Concat(docs) => docs.iter().any(Doc::breaks),
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Flat,
    Break,
}

/// Renders `doc` to text with `width` columns.
pub(crate) fn print(doc: &Doc, width: usize) -> String {
    let mut out = String::new();
    let mut col = 0usize;
    let mut stack: Vec<(usize, Mode, &Doc)> = vec![(0, Mode::Break, doc)];
    while let Some((indent, mode, doc)) = stack.pop() {
        match doc {
            Doc::Text(text) => {
                out.push_str(text);
                col += text.chars().count();
            }
            Doc::Line if mode == Mode::Flat => {
                out.push(' ');
                col += 1;
            }
            Doc::Line | Doc::HardLine => {
                out.push('\n');
                out.push_str(&" ".repeat(indent));
                col = indent;
            }
            Doc::BlankLine if mode == Mode::Break => out.push('\n'),
            Doc::BlankLine | Doc::BreakParent => {}
            Doc::IfBreak(broken, flat) => {
                let text = if mode == Mode::Break { broken } else { flat };
                out.push_str(text);
                col += text.chars().count();
            }
            Doc::Nest(docs) => stack.extend(docs.iter().rev().map(|d| (indent + 2, mode, d))),
            Doc::Concat(docs) => stack.extend(docs.iter().rev().map(|d| (indent, mode, d))),
            Doc::Group { docs, force } => {
                let flat =
                    !force && (mode == Mode::Flat || fits(width.saturating_sub(col), docs, &stack));
                let mode = if flat { Mode::Flat } else { Mode::Break };
                stack.extend(docs.iter().rev().map(|d| (indent, mode, d)));
            }
        }
    }
    out
}

/// True when `docs` printed flat, followed by the rest of the stack up to
/// its next line break, takes at most `remaining` columns.
fn fits(remaining: usize, docs: &[Doc], rest: &[(usize, Mode, &Doc)]) -> bool {
    let mut remaining = remaining;
    let mut work: Vec<(Mode, &Doc)> = docs.iter().rev().map(|d| (Mode::Flat, d)).collect();
    let mut rest = rest.iter().rev();
    loop {
        let Some((mode, doc)) = work.pop().or_else(|| rest.next().map(|(_, m, d)| (*m, *d))) else {
            return true;
        };
        let used = match doc {
            Doc::Text(text) => text.chars().count(),
            Doc::Line if mode == Mode::Flat => 1,
            Doc::Line | Doc::HardLine => return true,
            Doc::BlankLine | Doc::BreakParent => 0,
            Doc::IfBreak(broken, flat) => if mode == Mode::Break { broken } else { flat }
                .chars()
                .count(),
            Doc::Nest(docs) | Doc::Concat(docs) => {
                work.extend(docs.iter().rev().map(|d| (mode, d)));
                0
            }
            Doc::Group { docs, force } => {
                let mode = if *force { Mode::Break } else { mode };
                work.extend(docs.iter().rev().map(|d| (mode, d)));
                0
            }
        };
        if used > remaining {
            return false;
        }
        remaining -= used;
    }
}

#[cfg(test)]
mod tests {
    use super::{print, Doc};

    fn block(entries: &[&str]) -> Doc {
        let mut inner = Vec::new();
        for (index, entry) in entries.iter().enumerate() {
            inner.push(Doc::Line);
            inner.push(Doc::text(*entry));
            if index + 1 < entries.len() {
                inner.push(Doc::IfBreak("", ","));
            }
        }
        Doc::group(
            vec![Doc::text("{"), Doc::Nest(inner), Doc::Line, Doc::text("}")],
            false,
        )
    }

    #[test]
    fn groups_print_flat_when_they_fit() {
        let doc = Doc::Concat(vec![Doc::text("x "), block(&["a", "b"])]);
        assert_eq!(print(&doc, 100), "x { a, b }");
    }

    #[test]
    fn groups_break_one_entry_per_line() {
        let doc = Doc::Concat(vec![Doc::text("x "), block(&["aaaa", "bbbb"])]);
        assert_eq!(print(&doc, 10), "x {\n  aaaa\n  bbbb\n}");
    }

    #[test]
    fn hard_lines_force_breaks() {
        let doc = block(&["a"]);
        let Doc::Group { docs, .. } = doc else {
            unreachable!("block builds a group")
        };
        let forced = Doc::group([docs, vec![Doc::HardLine]].concat(), false);
        assert!(print(&forced, 100).starts_with("{\n  a\n}"));
    }
}
