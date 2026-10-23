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

#[test]
fn a_forced_group_ends_the_line_being_measured() {
    // Flat, `{ a, b } tail` overflows 10 columns; but the forced group after
    // the block breaks before `tail`, so the block still fits.
    let forced = Doc::Group {
        docs: vec![Doc::Line, Doc::text("tail")],
        force: true,
    };
    let doc = Doc::Concat(vec![block(&["a", "b"]), forced]);
    assert_eq!(print(&doc, 10), "{ a, b }\ntail");
}
