//! `litigate test --impact BASE`: the change impact report (lg-8fdj.3).
//! Every expect test under `tests/expect/<forum>/<name>.toml` runs twice,
//! on the tree at git revision BASE and on the working tree, and the report
//! lists each value that moved: one row of forum, case, world, field,
//! before and after. An expect test names either a matter and the world it
//! runs in (`case`, `world`, and the package directories `paths` it needs),
//! whose rows are each world's filing verdicts and deadlines, or a package
//! (`package`), whose rows are its inline `expect` values. A world or value
//! on one side only shows `-` on the other; a test on one side only is one
//! `expect test` row. The report never fails on a change: it is for a
//! reviewer to read what a branch moves.
use super::{test, worlds};
use crate::out::{Failure, Outcome, Report};
use crate::ws::{self, Session};
use litigate_sema::pkg::source::{MemoryTree, Source};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Where the expect tests live, relative to the repository root.
pub const EXPECT_DIR: &str = "tests/expect";

/// One expect test file.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    /// The `.case` file.
    case: Option<String>,
    /// The `.world` file it runs in.
    world: Option<String>,
    /// Package directories to load besides the library.
    #[serde(default)]
    paths: Vec<String>,
    /// A package whose inline tests are the expect test.
    package: Option<String>,
}

/// `(forum, name)`: one expect test file.
type Test = (String, String);

/// `(case, world, field)`: what one row of a test is about.
type Row = (String, String, String);

/// One value that moved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Change {
    /// The directory under `tests/expect` the test sits in.
    pub forum: String,
    /// The matter (`case`), the package and test (`package::test`), or
    /// the test file's name for a test on one side only.
    pub case: String,
    /// The world (its id, then its choices), `-` for a package's tests.
    pub world: String,
    /// What moved: `verdict KEY`, `deadline ID @DATE`, `lawful`, `error`,
    /// an expectation's text, or `expect test` (`present` on one side).
    pub field: String,
    /// Its value at BASE (`-`: none).
    pub before: String,
    /// Its value in the working tree (`-`: none).
    pub after: String,
}

/// `litigate test --impact BASE` over `head` (the working tree).
///
/// # Errors
/// BASE is not a revision, git cannot be run, or the working directory is
/// not the repository root.
pub fn impact(head: &dyn Source, base: &str, lib: &[String]) -> Outcome {
    let (sha, tree) = tree_at(base)?;
    let mut report = compare(&tree, head, lib);
    report.result["base"] = Value::String(sha);
    Ok(report)
}

/// The report of every expect test of `base` and `head`.
#[must_use]
pub fn compare(base: &dyn Source, head: &dyn Source, lib: &[String]) -> Report {
    let before = observe(base, lib);
    let after = observe(head, lib);
    let mut tests: Vec<&Test> = before.keys().chain(after.keys()).collect();
    tests.sort();
    tests.dedup();
    let none = String::from("-");
    let mut changes = Vec::new();
    for test in &tests {
        let change = |(case, world, field): &Row, b: Option<&String>, a: Option<&String>| Change {
            forum: test.0.clone(),
            case: case.clone(),
            world: world.clone(),
            field: field.clone(),
            before: b.unwrap_or(&none).clone(),
            after: a.unwrap_or(&none).clone(),
        };
        let (b, a) = match (before.get(*test), after.get(*test)) {
            (Some(b), Some(a)) => (b, a),
            // A test on one side only is one row, not each of its values.
            (b, a) => {
                let present = String::from("present");
                let row = (test.1.clone(), none.clone(), "expect test".to_owned());
                changes.push(change(&row, b.map(|_| &present), a.map(|_| &present)));
                continue;
            }
        };
        let mut rows: Vec<&Row> = b.keys().chain(a.keys()).collect();
        rows.sort();
        rows.dedup();
        for row in rows {
            let (b, a) = (b.get(row), a.get(row));
            if b != a {
                changes.push(change(row, b, a));
            }
        }
    }
    let mut report = Report::of(json!({
        "tests": tests.len(),
        "changed": changes.len(),
        "changes": changes,
    }));
    report.text = Some(table(&changes, tests.len()));
    report
}

/// The rows of every expect test in `src`, by test.
fn observe(src: &dyn Source, lib: &[String]) -> BTreeMap<Test, BTreeMap<Row, String>> {
    let mut tests = BTreeMap::new();
    for (forum, name, path) in manifests(src) {
        let mut rows = BTreeMap::new();
        let mut put = |case: &str, world: &str, field: &str, value: String| {
            rows.insert((case.to_owned(), world.to_owned(), field.to_owned()), value);
        };
        let manifest = src
            .read_to_string(&path)
            .map_err(|e| e.to_string())
            .and_then(|text| toml::from_str::<Manifest>(&text).map_err(|e| e.to_string()));
        let outcome = match manifest {
            Ok(m) => run(src, &name, &m, lib, &mut put),
            Err(e) => Err(Failure::new(e)),
        };
        if let Err(Failure(message)) = outcome {
            put(&name, "-", "error", message);
        }
        tests.insert((forum, name), rows);
    }
    tests
}

/// `(forum, test name, path)` of each `tests/expect/<forum>/<name>.toml`.
fn manifests(src: &dyn Source) -> Vec<(String, String, PathBuf)> {
    let mut out = Vec::new();
    let Ok(forums) = src.entries(Path::new(EXPECT_DIR)) else {
        return out;
    };
    for dir in forums.into_iter().filter(|d| src.is_dir(d)) {
        let forum = name_of(&dir);
        for path in src.entries(&dir).unwrap_or_default() {
            if src.is_file(&path) && path.extension().is_some_and(|e| e == "toml") {
                let stem = path.file_stem().map(|s| s.to_string_lossy().into_owned());
                out.push((forum.clone(), stem.unwrap_or_default(), path));
            }
        }
    }
    out.sort();
    out
}

fn name_of(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Runs one expect test, putting its rows.
fn run(
    src: &dyn Source,
    name: &str,
    m: &Manifest,
    lib: &[String],
    put: &mut dyn FnMut(&str, &str, &str, String),
) -> Result<(), Failure> {
    match (&m.case, &m.world, &m.package) {
        (Some(case), Some(world), None) => {
            let report = worlds::worlds(
                src,
                &worlds::Args {
                    case: case.clone(),
                    world: world.clone(),
                    paths: m.paths.clone(),
                    lib: lib.to_vec(),
                    ..worlds::Args::default()
                },
            )?;
            world_rows(&report, name, put);
            Ok(())
        }
        (None, None, Some(package)) => {
            let mut paths = vec![package.clone()];
            paths.extend(m.paths.iter().cloned());
            let targets = ws::discover_in(src, &paths)?;
            let session = Session::load_in(src, &targets, lib)?;
            let (results, _) = test::run(&session, None);
            for t in results {
                let case = format!("{}::{}", t.package, t.name);
                let mut seen: BTreeMap<&str, usize> = BTreeMap::new();
                for e in &t.expectations {
                    let n = seen.entry(&e.text).or_default();
                    *n += 1;
                    let field = if *n == 1 {
                        e.text.clone()
                    } else {
                        format!("{} #{n}", e.text)
                    };
                    let value = match (&e.error, &e.actual) {
                        (Some(error), _) => format!("error: {error}"),
                        (None, Some(actual)) => shown(actual),
                        (None, None) => e.passed.to_string(),
                    };
                    let value = if e.passed {
                        value
                    } else {
                        format!("{value} (fails)")
                    };
                    put(&case, "-", &field, value);
                }
            }
            Ok(())
        }
        _ => Err(Failure::new(
            "an expect test names `case` and `world`, or `package`",
        )),
    }
}

/// The rows of a `worlds` report of test `name`: the matter's errors,
/// then per world whether its law is determined, each filing's verdict
/// and each deadline's due date and status.
fn world_rows(report: &Report, name: &str, put: &mut dyn FnMut(&str, &str, &str, String)) {
    let result = &report.result;
    let case = result["case"].as_str().unwrap_or(name);
    let errors: std::collections::BTreeSet<String> = report
        .diagnostics
        .iter()
        .filter(|d| d.severity == litigate_base::Severity::Error)
        .map(|d| format!("{} {}", d.code.as_str(), d.message))
        .collect();
    if !errors.is_empty() {
        put(
            case,
            "-",
            "error",
            errors.into_iter().collect::<Vec<_>>().join("; "),
        );
    }
    for w in result["worlds"].as_array().into_iter().flatten() {
        let mut world = w["id"].as_str().unwrap_or("-").to_owned();
        for (choice, option) in w["choices"].as_object().into_iter().flatten() {
            let _ = write!(world, " {choice}={}", shown(option));
        }
        put(case, &world, "lawful", shown(&w["lawful"]));
        for (key, verdict) in w["filings"].as_object().into_iter().flatten() {
            put(case, &world, &format!("verdict {key}"), shown(verdict));
        }
        for d in w["deadlines"].as_array().into_iter().flatten() {
            let field = format!(
                "deadline {} @{}",
                shown(&d["deadline"]),
                shown(&d["trigger_date"])
            );
            let due = d["due"].as_str().unwrap_or("unknown");
            put(
                case,
                &world,
                &field,
                format!("{due} {}", shown(&d["status"])),
            );
        }
    }
}

/// A JSON value as table text: strings bare.
fn shown(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Null => "-".to_owned(),
        other => other.to_string(),
    }
}

/// The changes as one aligned table per forum.
fn table(changes: &[Change], tests: usize) -> String {
    let mut out = format!(
        "{} change(s) across {tests} expect test(s)\n",
        changes.len()
    );
    let header = ["case", "world", "field", "before", "after"];
    let mut forums: BTreeMap<&str, Vec<[&str; 5]>> = BTreeMap::new();
    for c in changes {
        forums
            .entry(&c.forum)
            .or_default()
            .push([&c.case, &c.world, &c.field, &c.before, &c.after]);
    }
    for (forum, rows) in forums {
        let mut width = header.map(str::len);
        for row in &rows {
            for (w, cell) in width.iter_mut().zip(row) {
                *w = (*w).max(cell.chars().count());
            }
        }
        let _ = writeln!(out, "\n{forum}");
        let line = |cells: &[&str; 5]| {
            let mut s = String::new();
            for (i, (cell, w)) in cells.iter().zip(width).enumerate() {
                if i == 4 {
                    s.push_str(cell);
                } else {
                    let _ = write!(s, "{cell:<w$} | ");
                }
            }
            s.trim_end().to_owned() + "\n"
        };
        out.push_str(&line(&header));
        out.push_str(&line(
            &width.map(|w| "-".repeat(w)).each_ref().map(String::as_str),
        ));
        for row in &rows {
            out.push_str(&line(row));
        }
    }
    out
}

/// Runs git with `args`, its standard output.
fn git(args: &[&str]) -> Result<Vec<u8>, Failure> {
    let out = Command::new("git")
        .args(args)
        .output()
        .map_err(|e| Failure::new(format!("git: {e}")))?;
    if !out.status.success() {
        return Err(Failure::new(format!(
            "git {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    Ok(out.stdout)
}

/// Whether the tree at a revision carries `path` for the expect tests:
/// litigate sources, manifests, locks and JSON, outside the Rust crates
/// and the vendored raw sources (of which only the texts a `Lit.lock`
/// pins are read, by [`pinned`]).
fn wanted(path: &str) -> bool {
    let skipped = ["crates/", "sources/", "editors/", "target/"];
    let ext = Path::new(path).extension().and_then(|e| e.to_str());
    !skipped.iter().any(|p| path.starts_with(p))
        && matches!(
            ext,
            Some("lit" | "case" | "world" | "toml" | "lock" | "json")
        )
}

/// The paths of the texts a `Lit.lock` pins: each `texts` entry is
/// `"ID PATH sha256:DIGEST"`.
fn pinned(lock: &str) -> impl Iterator<Item = &str> {
    lock.lines().filter_map(|line| {
        let entry = line.trim().strip_prefix('"')?;
        let mut words = entry.split(' ');
        let (_, path, digest) = (words.next()?, words.next()?, words.next()?);
        digest.starts_with("sha256:").then_some(path)
    })
}

/// The contents of `objects` (`git cat-file --batch` names, `REV:PATH`
/// say), `None` for a missing one.
fn cat(objects: &[String]) -> Result<Vec<Option<Vec<u8>>>, Failure> {
    let mut child = Command::new("git")
        .args(["cat-file", "--batch"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| Failure::new(format!("git cat-file: {e}")))?;
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| Failure::new("git cat-file: no stdin"))?;
    let mut names = String::new();
    for o in objects {
        names.push_str(o);
        names.push('\n');
    }
    let writer = std::thread::spawn(move || stdin.write_all(names.as_bytes()));
    let mut stdout = BufReader::new(
        child
            .stdout
            .take()
            .ok_or_else(|| Failure::new("git cat-file: no stdout"))?,
    );
    let mut out = Vec::with_capacity(objects.len());
    for _ in objects {
        // `OBJECT TYPE SIZE\n`, the content, then `\n`; or `NAME missing\n`.
        let mut header = String::new();
        stdout.read_line(&mut header)?;
        let Some(size) = header
            .trim_end()
            .rsplit(' ')
            .next()
            .and_then(|s| s.parse::<usize>().ok())
        else {
            out.push(None);
            continue;
        };
        let mut content = vec![0; size + 1];
        stdout.read_exact(&mut content)?;
        content.pop();
        out.push(Some(content));
    }
    writer
        .join()
        .map_err(|_| Failure::new("git cat-file: writer panicked"))??;
    child.wait()?;
    Ok(out)
}

/// The UTF-8 files at `paths` of commit `sha`, into `files`.
fn fetch(
    sha: &str,
    paths: Vec<String>,
    files: &mut BTreeMap<String, String>,
) -> Result<(), Failure> {
    let objects: Vec<String> = paths.iter().map(|p| format!("{sha}:{p}")).collect();
    for (path, content) in paths.into_iter().zip(cat(&objects)?) {
        if let Some(text) = content.and_then(|c| String::from_utf8(c).ok()) {
            files.insert(path, text);
        }
    }
    Ok(())
}

/// The commit `rev` names, and its files the expect tests read.
fn tree_at(rev: &str) -> Result<(String, MemoryTree), Failure> {
    if !git(&["rev-parse", "--show-prefix"])?
        .trim_ascii()
        .is_empty()
    {
        return Err(Failure::new("--impact: run from the repository root"));
    }
    let sha = String::from_utf8_lossy(
        git(&["rev-parse", "--verify", &format!("{rev}^{{commit}}")])?.trim_ascii(),
    )
    .into_owned();
    let listing = git(&["ls-tree", "-r", "-z", "--name-only", "--full-tree", &sha])?;
    let paths: Vec<String> = listing
        .split(|b| *b == 0)
        .filter_map(|p| std::str::from_utf8(p).ok())
        .filter(|p| wanted(p))
        .map(str::to_owned)
        .collect();
    let mut files = BTreeMap::new();
    fetch(&sha, paths, &mut files)?;
    let texts: std::collections::BTreeSet<String> = files
        .iter()
        .filter(|(path, _)| path.ends_with("Lit.lock"))
        .flat_map(|(_, lock)| pinned(lock).map(str::to_owned))
        .filter(|p| !files.contains_key(p))
        .collect();
    fetch(&sha, texts.into_iter().collect(), &mut files)?;
    Ok((sha, MemoryTree::new(files)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_litigate_files_outside_crates_and_sources_are_read() {
        assert!(wanted("lit/frcp/periods.lit"));
        assert!(wanted("docs/litigate/change/bass-5cir-2000.case"));
        assert!(wanted("tests/expect/frap/bass.toml"));
        assert!(!wanted("crates/litigate-cli/Cargo.toml"));
        assert!(!wanted("sources/frcp.txt"));
        assert!(!wanted("sources/x.json"));
        assert!(!wanted("README.md"));
    }

    #[test]
    fn a_lock_pins_the_texts_it_lists() {
        let lock =
            "texts = [\n    \"frap4 sources/frap4.txt sha256:c2\",\n]\nsource = \"sha256:0c\"\n";
        assert_eq!(pinned(lock).collect::<Vec<_>>(), ["sources/frap4.txt"]);
    }

    #[test]
    fn the_table_aligns_each_forum() {
        let change = |forum: &str, field: &str| Change {
            forum: forum.to_owned(),
            case: "bass".to_owned(),
            world: "w1".to_owned(),
            field: field.to_owned(),
            before: "timely".to_owned(),
            after: "late".to_owned(),
        };
        let text = table(
            &[change("frap", "verdict filed"), change("frcp", "lawful")],
            2,
        );
        assert_eq!(
            text,
            "2 change(s) across 2 expect test(s)\n\
             \nfrap\n\
             case | world | field         | before | after\n\
             ---- | ----- | ------------- | ------ | -----\n\
             bass | w1    | verdict filed | timely | late\n\
             \nfrcp\n\
             case | world | field  | before | after\n\
             ---- | ----- | ------ | ------ | -----\n\
             bass | w1    | lawful | timely | late\n"
        );
    }
}
