// SPDX-License-Identifier: GPL-3.0-or-later
//! Gate (`docs/litigate/DESIGN.md` §9.3): no forum knowledge in engine
//! source. Every `crates/*/src/**/*.rs` file is scanned, code and comments
//! alike, for any pack id or forum key authored in `packs/*.json`; the keys
//! are read from the packs at test time, so a new pack is covered without
//! touching this file. Excluded: `#[cfg(test)]` items, `*_tests.rs` and
//! `tests.rs` files, and anything under a `tests/` or `fixtures/`
//! directory. A key matches as a whole word (not inside a longer
//! identifier: `frcp6` is not `frcp`, but `cofc-1498` and `"cofc"` are
//! `cofc`), case-sensitively, the way pack ids and forum keys are written.
//!
//! Forum facts belong in pack data (`counting`, `citeSyntax`, `sources`,
//! `links.json`) — see `docs/PACK_SCHEMA.md`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use proptest::prelude::*;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Every `id` and `forum` of every pack in `dir` (files without an `id`,
/// like `links.json`, contribute nothing).
fn forum_keys(dir: &Path) -> BTreeSet<String> {
    let mut keys = BTreeSet::new();
    for entry in fs::read_dir(dir).expect("packs dir readable") {
        let path = entry.expect("dir entry").path();
        if path.extension().is_none_or(|e| e != "json") {
            continue;
        }
        let text = fs::read_to_string(&path).expect("pack readable");
        let doc: serde_json::Value = serde_json::from_str(&text).expect("pack is JSON");
        for field in ["id", "forum"] {
            if let Some(k) = doc.get(field).and_then(serde_json::Value::as_str) {
                keys.insert(k.to_string());
            }
        }
    }
    keys
}

fn is_excluded(path: &Path) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    name.ends_with("_tests.rs")
        || name == "tests.rs"
        || path
            .components()
            .any(|c| matches!(c.as_os_str().to_str(), Some("tests" | "fixtures")))
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") && !is_excluded(&path) {
            out.push(path);
        }
    }
}

/// `src` with every `#[cfg(test)]` item (attribute through the end of the
/// item: its `;`, or the `}` closing its first `{`) blanked to spaces,
/// newlines kept so line numbers still line up. Braces and semicolons in
/// comments, strings, raw strings, and char literals are not structure.
fn strip_test_items(src: &str) -> String {
    let code = code_mask(src.as_bytes());
    let mut out = src.as_bytes().to_vec();
    let attr = b"#[cfg(test)]";
    let mut i = 0;
    while i + attr.len() <= out.len() {
        if !(code[i] && src.as_bytes()[i..].starts_with(attr)) {
            i += 1;
            continue;
        }
        let start = i;
        let mut j = i + attr.len();
        let mut depth = 0usize;
        let mut opened = false;
        while j < out.len() {
            if code[j] {
                match out[j] {
                    b'{' => {
                        depth += 1;
                        opened = true;
                    }
                    b'}' => depth = depth.saturating_sub(1),
                    b';' if !opened => break,
                    _ => {}
                }
                if opened && depth == 0 {
                    break;
                }
            }
            j += 1;
        }
        let end = (j + 1).min(out.len());
        for b in &mut out[start..end] {
            if *b != b'\n' {
                *b = b' ';
            }
        }
        i = end;
    }
    // Spans start at `#` and end at `;`/`}`, so whole scalars are blanked
    // and the bytes stay valid UTF-8.
    String::from_utf8(out).expect("blanking whole scalars keeps UTF-8")
}

/// Per byte: is it code (not inside a comment, string, or char literal)?
fn code_mask(s: &[u8]) -> Vec<bool> {
    let mut mask = vec![true; s.len()];
    let mut i = 0;
    let mark = |from: usize, to: usize, mask: &mut Vec<bool>| {
        for m in &mut mask[from..to.min(s.len())] {
            *m = false;
        }
    };
    while i < s.len() {
        let rest = &s[i..];
        let end = if rest.starts_with(b"//") {
            rest.iter()
                .position(|&b| b == b'\n')
                .map_or(s.len(), |p| i + p)
        } else if rest.starts_with(b"/*") {
            block_comment_end(s, i)
        } else if let Some(hashes) = raw_string_open(rest) {
            let close: Vec<u8> = std::iter::once(b'"')
                .chain(std::iter::repeat_n(b'#', hashes))
                .collect();
            let body = i + rest.iter().position(|&b| b == b'"').unwrap_or(0) + 1;
            find(s, body, &close).map_or(s.len(), |p| p + close.len())
        } else if rest[0] == b'"' {
            string_end(s, i + 1)
        } else if rest[0] == b'\'' {
            char_literal_end(s, i).unwrap_or(i)
        } else {
            i
        };
        if end > i {
            mark(i, end, &mut mask);
            i = end;
        } else {
            i += 1;
        }
    }
    mask
}

fn find(s: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
    s.get(from..)?
        .windows(needle.len())
        .position(|w| w == needle)
        .map(|p| from + p)
}

fn block_comment_end(s: &[u8], start: usize) -> usize {
    let (mut depth, mut i) = (0usize, start);
    while i + 1 < s.len() {
        if s[i] == b'/' && s[i + 1] == b'*' {
            depth += 1;
            i += 2;
        } else if s[i] == b'*' && s[i + 1] == b'/' {
            depth -= 1;
            i += 2;
            if depth == 0 {
                return i;
            }
        } else {
            i += 1;
        }
    }
    s.len()
}

/// `r"`, `r#"`, `br##"` ... at the start of `rest`: the number of `#`s.
/// A raw identifier (`r#type`) is not a raw string.
fn raw_string_open(rest: &[u8]) -> Option<usize> {
    let after_b = rest.strip_prefix(b"b").unwrap_or(rest);
    let after_r = after_b.strip_prefix(b"r")?;
    let hashes = after_r.iter().take_while(|&&b| b == b'#').count();
    (after_r.get(hashes) == Some(&b'"')).then_some(hashes)
}

fn string_end(s: &[u8], mut i: usize) -> usize {
    while i < s.len() {
        match s[i] {
            b'\\' => i += 2,
            b'"' => return i + 1,
            _ => i += 1,
        }
    }
    s.len()
}

/// `'x'`, `'\n'`, `'\u{1F600}'`, `'§'` — but not a lifetime `'a`.
fn char_literal_end(s: &[u8], start: usize) -> Option<usize> {
    let body = s.get(start + 1..)?;
    if body.first() == Some(&b'\\') {
        return body
            .iter()
            .skip(1)
            .position(|&b| b == b'\'')
            .map(|p| start + 1 + 1 + p + 1);
    }
    let ch = std::str::from_utf8(&body[..body.len().min(4)])
        .ok()
        .or_else(|| {
            (1..4)
                .rev()
                .find_map(|n| std::str::from_utf8(body.get(..n)?).ok())
        })?
        .chars()
        .next()?;
    let len = ch.len_utf8();
    (body.get(len) == Some(&b'\'')).then_some(start + 1 + len + 1)
}

fn is_word(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// `(line, key)` for every whole-word occurrence of a key in `text`.
fn hits(text: &str, keys: &BTreeSet<String>) -> Vec<(usize, String)> {
    let bytes = text.as_bytes();
    let mut out = vec![];
    for key in keys {
        let mut from = 0;
        while let Some(pos) = text[from..].find(key.as_str()).map(|p| from + p) {
            let end = pos + key.len();
            let before_ok = pos == 0 || !is_word(bytes[pos - 1]);
            let after_ok = end >= bytes.len() || !is_word(bytes[end]);
            if before_ok && after_ok {
                let line = text[..pos].matches('\n').count() + 1;
                out.push((line, key.clone()));
            }
            from = pos + 1;
        }
    }
    out.sort();
    out
}

#[test]
fn no_forum_literals_in_engine() {
    let root = repo_root();
    let keys = forum_keys(&root.join("packs"));
    assert!(
        keys.len() >= 2,
        "expected pack ids and forum keys in packs/, found {keys:?}"
    );
    let mut files = vec![];
    for krate in fs::read_dir(root.join("crates")).expect("crates dir") {
        rust_files(&krate.expect("crate entry").path().join("src"), &mut files);
    }
    files.sort();
    assert!(!files.is_empty(), "no engine sources found");
    let mut violations = vec![];
    for file in &files {
        let src = fs::read_to_string(file).expect("source readable");
        for (line, key) in hits(&strip_test_items(&src), &keys) {
            let rel = file.strip_prefix(&root).unwrap_or(file).display();
            violations.push(format!("{rel}:{line}: `{key}`"));
        }
    }
    assert!(
        violations.is_empty(),
        "forum knowledge in engine source (move it into pack data; see docs/PACK_SCHEMA.md):\n{}",
        violations.join("\n")
    );
}

fn keyset(keys: &[&str]) -> BTreeSet<String> {
    keys.iter().map(|k| (*k).to_string()).collect()
}

#[test]
fn scanner_flags_code_comments_and_strings() {
    let keys = keyset(&["acme"]);
    let src =
        "// acme rules\nfn f() -> &'static str { \"acme\" }\nconst A: &str = r#\"acme-1\"#;\n";
    assert_eq!(
        hits(&strip_test_items(src), &keys),
        vec![(1, "acme".into()), (2, "acme".into()), (3, "acme".into())]
    );
}

#[test]
fn scanner_respects_word_boundaries() {
    let keys = keyset(&["acme"]);
    assert!(hits("acme6 acme_x xacme Acme", &keys).is_empty());
    assert_eq!(hits("acme::n acme@b", &keys).len(), 2);
}

#[test]
fn scanner_skips_cfg_test_items_only() {
    let keys = keyset(&["acme"]);
    let src = "fn a() {}\n#[cfg(test)]\nmod tests {\n    const S: &str = \"}\"; // }\n    fn t() { let _ = 'x'; \"acme\"; }\n}\nfn b() { \"acme\"; }\n#[cfg(test)]\n#[path = \"acme_tests.rs\"]\nmod t2;\nconst C: &str = \"acme\";\n";
    assert_eq!(
        hits(&strip_test_items(src), &keys),
        vec![(7, "acme".into()), (11, "acme".into())]
    );
}

#[test]
fn scanner_lifetimes_are_not_char_literals() {
    let keys = keyset(&["acme"]);
    let src = "#[cfg(test)]\nmod m { fn f<'a>(x: &'a str) -> &'a str { x } }\nconst C: &str = \"acme\";\n";
    assert_eq!(
        hits(&strip_test_items(src), &keys),
        vec![(3, "acme".into())]
    );
}

#[test]
fn excluded_paths() {
    assert!(is_excluded(Path::new("crates/x/src/lint_tests.rs")));
    assert!(is_excluded(Path::new("crates/x/src/expr/tests.rs")));
    assert!(is_excluded(Path::new("crates/x/src/fixtures/a.rs")));
    assert!(!is_excluded(Path::new("crates/x/src/lint.rs")));
}

proptest! {
    /// A key planted in ordinary code is always found on its line, and the
    /// same key planted inside a `#[cfg(test)]` module never is, whatever
    /// harmless code surrounds it.
    #[test]
    fn planted_key_found_outside_tests_only(
        pad_lines in 0usize..5,
        filler in "[a-z ]{0,12}",
        in_test in any::<bool>(),
    ) {
        let keys = keyset(&["acme"]);
        let mut src = String::new();
        for _ in 0..pad_lines {
            writeln!(src, "// {filler}").unwrap();
        }
        if in_test {
            writeln!(src, "#[cfg(test)]\nmod t {{ fn f() {{ let _ = \"{{ {filler}\"; }} const K: &str = \"acme\"; }}").unwrap();
        } else {
            src.push_str("const K: &str = \"acme\";\n");
        }
        let found = hits(&strip_test_items(&src), &keys);
        if in_test {
            prop_assert!(found.is_empty(), "{found:?}");
        } else {
            prop_assert_eq!(found, vec![(pad_lines + 1, "acme".to_string())]);
        }
    }
}
