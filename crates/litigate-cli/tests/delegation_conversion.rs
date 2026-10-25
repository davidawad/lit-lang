//! The corpus's `delegable_to [L]` clauses became `delegable { to L, op
//! set }` mechanically (lg-962.4); this proves the conversion keeps their
//! meaning. A legacy `delegable_to [L]` licensed any displacement by
//! authority `L` (no operation check), which is `delegable { to L, op
//! [extend, shorten, set, suspend, compute_method] }` today. The corpus is
//! lowered as committed and again with every converted clause widened back
//! to that legacy meaning and the listed corrections undone: every pack,
//! clock fragment, `links.json` and named scenario is byte-identical but for
//! the source digests in its provenance (they hash the edited text), and
//! each package's HIR differs only in the operations of its delegations and
//! in the corrected declarations. No package or matter in the corpus
//! displaces a delegating declaration from below, so the lowering does not
//! turn on the operations at all: `op set`'s narrowing (no `suspend` or
//! `compute_method`, which a legacy clause licensed) shows only in the
//! HIR's operations, which the comparison blanks.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use serde_json::Value;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;
use support::{copy_dir, litigate_in, repo, scratch};

/// The legacy meaning of `delegable_to [L]`: every operation.
const LEGACY_OPS: &str = "op [extend, shorten, set, suspend, compute_method]";

/// Clauses converted mechanically: lg-962.4 converted 99 on its base;
/// lg-flu.29 then removed the four on the certiorari periods (frap, cafc)
/// and the Federal Circuit PTAB and TTAB notice-of-appeal periods, now
/// extended only by a Justice or the Director.
const CONVERTED: usize = 94;

/// There are no extra corrections in this corpus: the district-court
/// notice-of-appeal period is authored once in `appeal-periods.lit`.
const CORRECTIONS: &[(&str, &str, &str, &str)] = &[];

/// `line` with each `delegable { to L, op set }` widened to the legacy
/// meaning, and how many it widened. Comments are left alone.
fn widen(line: &str) -> (String, usize) {
    if line.trim_start().starts_with("//") {
        return (line.to_owned(), 0);
    }
    let mut parts = line.split("delegable { to ");
    let mut out = parts.next().unwrap().to_owned();
    let mut n = 0;
    for part in parts {
        out.push_str("delegable { to ");
        let body = &part[..part.find('}').unwrap()];
        match body.strip_suffix(", op set ") {
            Some(to) => {
                n += 1;
                let _ = write!(out, "{to}, {LEGACY_OPS} {}", &part[body.len()..]);
            }
            None => out.push_str(part),
        }
    }
    (out, n)
}

/// Widens every converted clause of the `.lit` and `.case` files under
/// `dir`; how many.
fn widen_all(dir: &Path) -> usize {
    let mut n = 0;
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            n += widen_all(&path);
        } else if path.extension().is_some_and(|e| e == "lit" || e == "case") {
            let text = std::fs::read_to_string(&path).unwrap();
            let mut out = String::with_capacity(text.len());
            for line in text.split_inclusive('\n') {
                let (line, k) = widen(line);
                out.push_str(&line);
                n += k;
            }
            // The formatter writes a long delegation as a multiline block.
            // Keep the round-trip check independent of that layout.
            let multiline = "\n    op set\n  }".to_owned();
            let widened = format!("\n    {LEGACY_OPS}\n  }}");
            n += out.matches(&multiline).count();
            out = out.replace(&multiline, &widened);
            std::fs::write(&path, out).unwrap();
        }
    }
    n
}

/// The `litigate build` arguments of the repository's packs and scenarios.
fn build_args() -> Vec<String> {
    std::fs::read_to_string(repo().join("crates/litgraph-packs/build.args"))
        .unwrap()
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_owned)
        .collect()
}

/// Builds the corpus under `root` into `out`; every built file by path.
fn build(root: &Path, out: &Path) -> BTreeMap<String, String> {
    let mut args = vec!["build".to_owned()];
    args.extend(build_args());
    args.extend(["--out".to_owned(), out.to_str().unwrap().to_owned()]);
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let run = litigate_in(root, &args);
    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
    let mut files = BTreeMap::new();
    tree(out, out, &mut files);
    files
}

fn tree(base: &Path, dir: &Path, out: &mut BTreeMap<String, String>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            tree(base, &path, out);
        } else {
            let rel = path.strip_prefix(base).unwrap().to_string_lossy();
            out.insert(
                rel.replace('\\', "/"),
                std::fs::read_to_string(&path).unwrap(),
            );
        }
    }
}

/// `text` with every `sha256:` digest blanked.
fn undigested(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(i) = rest.find("sha256:") {
        out.push_str(&rest[..i + 7]);
        rest = rest[i + 7..].trim_start_matches(|c: char| c.is_ascii_hexdigit());
    }
    out.push_str(rest);
    out
}

/// A package's built HIR without what the edit moves by construction:
/// digests, spans, the source map and the checker's evidence (positions
/// and counts of the edited text). Each delegation's operations are
/// blanked, and a corrected declaration loses its delegations and
/// characterization.
fn hir(text: &str) -> Value {
    fn strip(v: &mut Value) {
        match v {
            Value::Object(map) => {
                for key in [
                    "span",
                    "source_map",
                    "content_hash",
                    "digest",
                    "verification",
                ] {
                    map.remove(key);
                }
                if map.contains_key("to") {
                    if let Some(ops) = map.get_mut("ops") {
                        *ops = Value::Null;
                    }
                }
                let corrected = map
                    .get("meta")
                    .and_then(|m| m["id"].as_str())
                    .is_some_and(|id| CORRECTIONS.iter().any(|c| c.1 == id));
                if corrected {
                    map.remove("characterization");
                    map["meta"].as_object_mut().unwrap().remove("delegations");
                }
                map.values_mut().for_each(strip);
            }
            Value::Array(items) => items.iter_mut().for_each(strip),
            _ => {}
        }
    }
    let mut v: Value = serde_json::from_str(text).unwrap();
    strip(&mut v);
    v
}

#[test]
fn the_delegable_conversion_lowers_the_corpus_identically() {
    let dir = scratch("delegation-conversion");
    let before = dir.join("before");
    for sub in ["lit", "cases"] {
        copy_dir(&repo().join(sub), &before.join(sub));
    }
    assert_eq!(widen_all(&before), CONVERTED);
    for (file, _, now, then) in CORRECTIONS {
        let path = before.join(file);
        let text = std::fs::read_to_string(&path).unwrap();
        assert_eq!(text.matches(now).count(), 1, "{file}: {now}");
        std::fs::write(&path, text.replace(now, then)).unwrap();
    }
    let mut locks: Vec<String> = std::fs::read_dir(before.join("lit"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.join("Lit.lock").exists())
        .map(|p| format!("lit/{}", p.file_name().unwrap().to_string_lossy()))
        .collect();
    locks.push("cases".to_owned());
    for pkg in &locks {
        let run = litigate_in(&before, &["lock", pkg]);
        assert_eq!(run.code, 0, "{pkg}: {}{}", run.stdout, run.stderr);
    }

    let then = build(&before, &dir.join("out-before"));
    let now = build(&repo(), &dir.join("out-after"));
    assert_eq!(
        then.keys().collect::<Vec<_>>(),
        now.keys().collect::<Vec<_>>()
    );
    let mut lowered = 0;
    for (name, text) in &now {
        if name.starts_with("litigatables/") {
            assert_eq!(hir(&then[name]), hir(text), "{name}");
        } else {
            assert_eq!(undigested(&then[name]), undigested(text), "{name}");
            lowered += 1;
        }
    }
    // Nine packs and their source maps, links, clock fragments, scenarios.
    assert!(lowered > 20, "{lowered}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn widening_leaves_comments_and_other_operations_alone() {
    assert_eq!(
        widen("deadline d: 1.days after E() counted_by c delegable { to [LocalRule, CaseOrder], op set } cite \"X\"\n"),
        (
            format!("deadline d: 1.days after E() counted_by c delegable {{ to [LocalRule, CaseOrder], {LEGACY_OPS} }} cite \"X\"\n"),
            1
        )
    );
    assert_eq!(
        widen("  delegable { to CaseOrder, op extend, max 30.days }").1,
        0
    );
    assert_eq!(widen("// `delegable { to CaseOrder, op set }` marks").1, 0);
}
