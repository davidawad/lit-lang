//! Facts `describe` reads straight from the normative SPEC (compiled in),
//! so the manual cannot drift from it: EBNF productions and the types table.
use super::{Production, TypeInfo};

/// `docs/litigate/SPEC.md` at build time.
pub const SPEC: &str = include_str!("../../../../docs/litigate/SPEC.md");

/// True when `line` starts a production (`Name = ...` at column 0).
fn starts_production(line: &str) -> Option<(&str, &str)> {
    let (name, rest) = line.split_once('=')?;
    let name = name.trim_end();
    let ident = !name.is_empty()
        && !line.starts_with(' ')
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
    ident.then_some((name, rest))
}

/// Splits `Name = rule ;  (* note *)` text into rule and note.
fn finish(name: &str, body: &str, out: &mut Vec<Production>) {
    // A second production on the same line (`IdStart = ... ;   IdContinue = ... ;`).
    let mut rest = body.trim();
    let mut name = name.to_owned();
    loop {
        let end = rest.find(" ;").or_else(|| rest.rfind(';'));
        let (rule, after) = match end {
            Some(i) => (
                &rest[..i],
                rest[i..]
                    .trim_start_matches(" ;")
                    .trim_start_matches(';')
                    .trim(),
            ),
            None => (rest, ""),
        };
        let (note, after) = match after.strip_prefix("(*").and_then(|a| a.split_once("*)")) {
            Some((note, tail)) => (Some(note.trim().to_owned()), tail.trim()),
            None => (None, after),
        };
        out.push(Production {
            name: name.clone(),
            rule: rule.split_whitespace().collect::<Vec<_>>().join(" "),
            note,
        });
        match starts_production(after) {
            Some((next, body)) => {
                next.clone_into(&mut name);
                rest = body.trim();
            }
            _ => break,
        }
    }
}

/// Every production of every `ebnf` code block, in SPEC order.
#[must_use]
pub fn grammar() -> Vec<Production> {
    grammar_of(SPEC)
}

/// Every production of every `ebnf` code block of markdown `spec`.
fn grammar_of(spec: &str) -> Vec<Production> {
    let mut out = Vec::new();
    let mut current: Option<(String, String)> = None;
    let mut inside = false;
    for line in spec.lines() {
        if line.trim_start().starts_with("```") {
            inside = line.trim() == "```ebnf";
            if let Some((name, body)) = current.take() {
                finish(&name, &body, &mut out);
            }
            continue;
        }
        if !inside || line.trim().is_empty() {
            continue;
        }
        if let Some((name, rest)) = starts_production(line) {
            if let Some((n, body)) = current.take() {
                finish(&n, &body, &mut out);
            }
            current = Some((name.to_owned(), rest.to_owned()));
        } else if let Some((_, body)) = current.as_mut() {
            body.push(' ');
            body.push_str(line.trim());
        }
    }
    out
}

/// Cells of a markdown table row (`\|` is a literal bar).
fn cells(row: &str) -> Vec<String> {
    let protected = row.replace("\\|", "\u{1}");
    protected
        .trim()
        .trim_matches('|')
        .split('|')
        .map(|c| c.trim().replace('\u{1}', "|"))
        .collect()
}

/// The SPEC §6 type table.
#[must_use]
pub fn types() -> Vec<TypeInfo> {
    types_of(SPEC)
}

/// The §6 type table of markdown `spec`.
fn types_of(spec: &str) -> Vec<TypeInfo> {
    let section = spec
        .split("\n## 6. ")
        .nth(1)
        .and_then(|s| s.split("\n## 7. ").next())
        .unwrap_or_default();
    section
        .lines()
        .filter(|l| l.starts_with("| `"))
        .map(cells)
        .filter_map(|c| match c.as_slice() {
            [name, literal, ops] => Some(TypeInfo {
                name: name.clone(),
                literal: literal.clone(),
                operations: ops.clone(),
            }),
            _ => None,
        })
        .collect()
}

/// The SPEC §7 obligation table: `(obligation, definition, code)`.
#[must_use]
pub fn obligations() -> Vec<(String, String, String)> {
    obligations_of(SPEC)
}

/// The §7 obligation table of markdown `spec`.
fn obligations_of(spec: &str) -> Vec<(String, String, String)> {
    let section = spec
        .split("\n## 7. ")
        .nth(1)
        .and_then(|s| s.split("\n## 8. ").next())
        .unwrap_or_default();
    section
        .lines()
        .filter(|l| l.starts_with("| `"))
        .map(cells)
        .filter_map(|c| match c.as_slice() {
            [name, definition, code] => Some((
                name.trim_matches('`').to_owned(),
                definition.clone(),
                code.clone(),
            )),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grammar_covers_both_file_kinds() {
        let g = grammar();
        let names: Vec<&str> = g.iter().map(|p| p.name.as_str()).collect();
        for wanted in [
            "LitFile",
            "CaseFile",
            "NodeDecl",
            "EdgeDecl",
            "DeadlineDecl",
            "EventsBlock",
            "IdStart",
            "IdContinue",
        ] {
            assert!(names.contains(&wanted), "{wanted} missing from {names:?}");
        }
        assert!(g.iter().all(|p| !p.rule.is_empty()), "{g:?}");
        assert!(g.iter().all(|p| !p.rule.ends_with(';')), "{g:?}");
        let date = g.iter().find(|p| p.name == "Date");
        assert_eq!(
            date.map(|p| p.rule.as_str()),
            Some(r#"D4, "-", D2, "-", D2"#)
        );
        assert!(date.is_some_and(|p| p.note.is_some()));
    }

    #[test]
    fn type_table_is_complete() {
        let t = types();
        assert!(t.len() >= 12, "{t:?}");
        assert!(t
            .iter()
            .any(|x| x.name == "`Bool`" && x.operations.contains("||")));
    }

    /// Every production shape: two on a line, a note, a continuation, a
    /// stray `=` line, `;` without a space, a missing `;`, and blocks that
    /// are not `ebnf`.
    const MARKDOWN: &str = r#"
```ebnf
(* a comment before any production *)
A_b = "x" ;   C = "y" ;  (* note *)
D = "z"
  | "w" ;
= stray
G = "g";
E = "no semicolon"
```

```text
F = "ignored" ;
```
"#;

    #[test]
    fn productions_of_every_shape() {
        insta::assert_debug_snapshot!(grammar_of(MARKDOWN));
    }

    #[test]
    fn table_rows_need_every_cell() {
        let spec = "\n## 6. Types\n| `A` | `1` | `+` |\n| `B` | short |\n\n## 7. Obligations\n| `o` | def | L5001 |\n| `p` | short |\n\n## 8. End\n";
        let types: Vec<String> = types_of(spec).into_iter().map(|t| t.name).collect();
        assert_eq!(types, ["`A`"]);
        assert_eq!(
            obligations_of(spec),
            [("o".to_owned(), "def".to_owned(), "L5001".to_owned())]
        );
    }
}
