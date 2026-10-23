//! Package digests and the deterministic, Merkle `Lit.lock` (SPEC §4.2).
use crate::db::span::{Report, TextRange};
use crate::hir;
use litigate_base::DiagnosticCode as C;
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fmt::Write as _;

/// The `Lit.lock` format this compiler reads and writes.
pub const FORMAT: u32 = 2;

/// SHA-256 over `parts`, each framed as `u64 BE length, bytes`; lowercase
/// hex.
fn framed<'a>(parts: impl IntoIterator<Item = &'a [u8]>) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update((part.len() as u64).to_be_bytes());
        hasher.update(part);
    }
    hex(&hasher.finalize())
}

/// Lowercase hex of `bytes`.
#[must_use]
pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().fold(String::with_capacity(64), |mut s, b| {
        let _ = write!(s, "{b:02x}");
        s
    })
}

/// The source digest: SHA-256 over `Lit.toml` then every source file
/// sorted by package-relative POSIX path, each framed as `u64 BE path
/// length, path, u64 BE content length, content`. Returns lowercase hex.
#[must_use]
pub fn digest<'a>(manifest: &str, files: impl IntoIterator<Item = (&'a str, &'a str)>) -> String {
    let mut files: Vec<(&str, &str)> = files.into_iter().collect();
    files.sort_by(|a, b| a.0.cmp(b.0));
    framed(
        std::iter::once(("Lit.toml", manifest))
            .chain(files)
            .flat_map(|(path, content)| [path.as_bytes(), content.as_bytes()]),
    )
}

/// The semantic digest: SHA-256 of the package HIR as canonical JSON
/// (object keys sorted, no whitespace) with its source digest nulled and
/// without any `span`. Reformatting, comments, and moving declarations
/// between files leave it unchanged; any change to what the package
/// means changes it. Returns lowercase hex.
#[must_use]
pub fn semantic_digest(package: &hir::Package) -> String {
    let mut value = serde_json::to_value(package).unwrap_or(Value::Null);
    value["digest"] = Value::Null;
    let mut text = String::new();
    canonical(&value, &["span"], &mut text);
    hex(&Sha256::digest(text.as_bytes()))
}

/// Canonical JSON of `value` (object keys sorted, no whitespace) with every
/// object key in `skip` dropped, at any depth.
pub(crate) fn canonical(value: &Value, skip: &[&str], out: &mut String) {
    match value {
        Value::Object(map) => {
            let mut keys: Vec<&String> =
                map.keys().filter(|k| !skip.contains(&k.as_str())).collect();
            keys.sort();
            out.push('{');
            for (i, key) in keys.into_iter().enumerate() {
                out.push_str(if i == 0 { "" } else { "," });
                out.push_str(&Value::String(key.clone()).to_string());
                out.push(':');
                canonical(&map[key], skip, out);
            }
            out.push('}');
        }
        Value::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                out.push_str(if i == 0 { "" } else { "," });
                canonical(item, skip, out);
            }
            out.push(']');
        }
        scalar => out.push_str(&scalar.to_string()),
    }
}

/// The Merkle digest of a lock row: SHA-256 over the format tag, `name`,
/// `version`, `language`, `content` (the semantic digest, or the source
/// digest of a package that does not check), the number of `texts` and
/// each entry, then the number of `dependencies` and each entry (which
/// carries the dependency's own Merkle digest). Returns `sha256:<hex>`.
#[must_use]
pub fn merkle(
    name: &str,
    version: &str,
    language: u32,
    content: &str,
    texts: &[String],
    dependencies: &[String],
) -> String {
    let (language, t, d) = (
        language.to_string(),
        texts.len().to_string(),
        dependencies.len().to_string(),
    );
    let head = ["litlock 2", name, version, &language, content];
    format!(
        "sha256:{}",
        framed(
            head.iter()
                .map(|p| p.as_bytes())
                .chain([t.as_bytes()])
                .chain(texts.iter().map(String::as_bytes))
                .chain([d.as_bytes()])
                .chain(dependencies.iter().map(String::as_bytes))
        )
    )
}

/// The `semantic` value of a package that does not check: it has no HIR,
/// so its Merkle digest covers its source digest instead.
pub const UNCHECKED: &str = "unchecked";

/// One resolved package in `Lit.lock`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LockedPackage {
    /// Package name.
    pub name: String,
    /// Exact version.
    pub version: String,
    /// Language version.
    pub language: u32,
    /// Merkle digest ([`merkle`]), `sha256:<hex>`.
    pub digest: String,
    /// Source digest ([`digest`]), `sha256:<hex>`.
    pub source: String,
    /// Semantic digest ([`semantic_digest`]), `sha256:<hex>`, or
    /// [`UNCHECKED`].
    pub semantic: String,
    /// Sorted `"<source id> <path> sha256:<hex>"` entries, one per
    /// `[sources.*]` binding with a `path`: the vendored L0 text's bytes
    /// (`missing` in place of the digest when the file is absent).
    #[serde(default)]
    pub texts: Vec<String>,
    /// Sorted `"name version sha256:<merkle>"` entries (`"name version"`
    /// where the dependency closes a cycle).
    pub dependencies: Vec<String>,
}

/// Parsed `Lit.lock`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LockFile {
    /// Format version; must be [`FORMAT`].
    pub version: u32,
    /// Packages sorted by name.
    #[serde(default, rename = "package")]
    pub packages: Vec<LockedPackage>,
}

impl LockFile {
    /// Builds a normalized lock (packages and dependencies sorted).
    #[must_use]
    pub fn new(mut packages: Vec<LockedPackage>) -> Self {
        for p in &mut packages {
            p.dependencies.sort();
            p.texts.sort();
        }
        packages.sort_by(|a, b| (&a.name, &a.version).cmp(&(&b.name, &b.version)));
        Self {
            version: FORMAT,
            packages,
        }
    }

    /// Deterministic TOML text: no timestamps, no absolute paths.
    #[must_use]
    pub fn render(&self) -> String {
        // One entry per line, so a changed text or dependency is a one-line
        // diff.
        let list = |items: &[String]| {
            items
                .iter()
                .map(|d| format!("\n    {},", quote(d)))
                .chain((!items.is_empty()).then(|| "\n".to_owned()))
                .collect::<String>()
        };
        let mut out = format!("version = {FORMAT}\n");
        for p in &self.packages {
            let _ = write!(
                out,
                "\n[[package]]\nname = {}\nversion = {}\nlanguage = {}\ndigest = {}\nsource = {}\nsemantic = {}\ntexts = [{}]\ndependencies = [{}]\n",
                quote(&p.name),
                quote(&p.version),
                p.language,
                quote(&p.digest),
                quote(&p.source),
                quote(&p.semantic),
                list(&p.texts),
                list(&p.dependencies),
            );
        }
        out
    }

    /// Parses `Lit.lock` text.
    ///
    /// # Errors
    /// L1007 for unparseable text or an unknown format version.
    pub fn parse(text: &str) -> Result<Self, Report> {
        let version = Self::format(text)?;
        if version != FORMAT {
            return Err(Report::new(
                C::StaleLock,
                TextRange::default(),
                format!("Lit.lock format {version} is not supported (expected {FORMAT})"),
            )
            .note("regenerate it with `litigate lock`"));
        }
        toml::from_str(text).map_err(|e| invalid(&e))
    }

    /// The format version of `Lit.lock` text, read before its rows so an
    /// older lock is named as such rather than as malformed.
    ///
    /// # Errors
    /// L1007 for text with no readable `version`.
    pub fn format(text: &str) -> Result<u32, Report> {
        #[derive(Deserialize)]
        struct Head {
            version: u32,
        }
        toml::from_str::<Head>(text)
            .map(|h| h.version)
            .map_err(|e| invalid(&e))
    }
}

/// L1007 for unparseable lock text.
fn invalid(e: &toml::de::Error) -> Report {
    Report::new(
        C::StaleLock,
        TextRange::default(),
        format!("Lit.lock is invalid: {}", e.message().trim()),
    )
    .note("regenerate it with `litigate lock`")
}

/// TOML basic string.
fn quote(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if u32::from(c) < 0x20 || c == '\u{7f}' => {
                let _ = write!(out, "\\u{:04X}", u32::from(c));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Compares an on-disk lock with the expected one.
///
/// # Errors
/// L1007 when the lock is missing, unparseable, or differs (the notes name
/// each differing package).
pub fn verify(existing: Option<&str>, expected: &LockFile) -> Result<(), Report> {
    let fix = "run `litigate lock` to regenerate Lit.lock";
    let Some(text) = existing else {
        return Err(
            Report::new(C::StaleLock, TextRange::default(), "Lit.lock is missing").note(fix),
        );
    };
    let found = LockFile::parse(text)?;
    if &found == expected {
        return Ok(());
    }
    let mut report = Report::new(
        C::StaleLock,
        TextRange::default(),
        "Lit.lock does not match the resolved package graph",
    )
    .note(fix);
    for p in &expected.packages {
        match found.packages.iter().find(|f| f.name == p.name) {
            None => report = report.note(format!("`{}` is not locked", p.name)),
            Some(f) if f != p => {
                report = report.note(format!(
                    "`{}` changed (locked {} {}, now {} {})",
                    p.name, f.version, f.digest, p.version, p.digest
                ));
            }
            Some(_) => {}
        }
    }
    for f in found
        .packages
        .iter()
        .filter(|f| !expected.packages.iter().any(|p| p.name == f.name))
    {
        report = report.note(format!("`{}` is locked but no longer used", f.name));
    }
    Err(report)
}

/// The digest a three-way merge writes for a package both sides changed
/// differently. It is never a real digest, so the merged lock is stale
/// (L1007) until `litigate lock` regenerates it from the merged sources.
pub const UNMERGED: &str = "unmerged: run `litigate lock`";

/// A three-way merge of `Lit.lock` texts (the git merge driver).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Merged {
    /// The merged lock, sorted as [`LockFile::new`] sorts.
    pub lock: LockFile,
    /// Packages both sides changed differently, written with [`UNMERGED`].
    pub unresolved: Vec<String>,
}

/// Merges `ours` and `theirs` against `base` package by package. A row
/// only one side changed takes that side; a row both sides changed the
/// same way is kept. A row both sides changed differently (or one changed
/// and the other removed) keeps the changed row with an [`UNMERGED`]
/// digest: no lock text holds the digest of the merged sources, so the
/// result fails closed with L1007 instead of conflicting.
#[must_use]
pub fn merge(base: &LockFile, ours: &LockFile, theirs: &LockFile) -> Merged {
    let row = |lock: &LockFile, name: &str| lock.packages.iter().find(|p| p.name == name).cloned();
    let mut names: Vec<&str> = [base, ours, theirs]
        .iter()
        .flat_map(|l| l.packages.iter().map(|p| p.name.as_str()))
        .collect();
    names.sort_unstable();
    names.dedup();
    let mut packages = Vec::new();
    let mut unresolved = Vec::new();
    for name in names {
        let (o, a, b) = (row(base, name), row(ours, name), row(theirs, name));
        let merged = if a == o || a == b {
            b
        } else if b == o {
            a
        } else {
            unresolved.push(name.to_owned());
            a.or(b).map(|p| LockedPackage {
                digest: UNMERGED.to_owned(),
                ..p
            })
        };
        packages.extend(merged);
    }
    Merged {
        lock: LockFile::new(packages),
        unresolved,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn pkg(name: &str, deps: &[&str]) -> LockedPackage {
        LockedPackage {
            name: name.into(),
            version: "1.0.0".into(),
            language: 1,
            digest: merkle(name, "1.0.0", 1, "sha256:0", &[], &[]),
            source: format!("sha256:{}", digest("m", [])),
            semantic: UNCHECKED.to_owned(),
            texts: vec!["rules sources/rules.txt missing".to_owned()],
            dependencies: deps.iter().map(|d| (*d).to_owned()).collect(),
        }
    }

    #[test]
    fn digest_is_order_independent_and_framed() {
        let a = digest("m", [("b.lit", "x"), ("a.lit", "y")]);
        assert_eq!(a, digest("m", [("a.lit", "y"), ("b.lit", "x")]));
        assert_ne!(a, digest("m", [("a.lit", "yb.lit"), ("", "x")]));
        assert_eq!(a.len(), 64);
    }

    #[test]
    fn render_is_sorted_and_round_trips() {
        let lock = LockFile::new(vec![
            pkg("zeta", &["core 1.0.0", "alpha 1.0.0"]),
            pkg("core", &[]),
        ]);
        let text = lock.render();
        insta::assert_snapshot!(text);
        assert_eq!(LockFile::parse(&text), Ok(lock.clone()));
        assert_eq!(verify(Some(&text), &lock), Ok(()));
    }

    #[test]
    fn verify_reports_missing_and_stale() {
        let lock = LockFile::new(vec![pkg("core", &[])]);
        let missing = verify(None, &lock).err();
        assert_eq!(
            missing.map(|r| (r.code, r.message)),
            Some((C::StaleLock, "Lit.lock is missing".to_owned()))
        );
        let other = LockFile::new(vec![pkg("other", &[])]).render();
        let err = verify(Some(&other), &lock).err();
        let notes: Vec<String> = err
            .map(|r| r.notes.into_iter().map(|n| n.1).collect())
            .unwrap_or_default();
        assert_eq!(
            notes[1..],
            [
                "`core` is not locked".to_owned(),
                "`other` is locked but no longer used".to_owned()
            ]
        );
        assert!(LockFile::parse("version = 1").is_err());
    }

    #[test]
    fn unparseable_locks_are_stale() {
        let lock = LockFile::new(vec![pkg("core", &[])]);
        for (text, message) in [
            (
                "version = ",
                "Lit.lock is invalid: string values must be quoted, expected literal string",
            ),
            (
                "version = 2\ncolour = 2",
                "Lit.lock is invalid: unknown field `colour`, expected `version` or `package`",
            ),
            (
                "version = 1",
                "Lit.lock format 1 is not supported (expected 2)",
            ),
        ] {
            let report = verify(Some(text), &lock).err();
            assert_eq!(
                report.map(|r| (r.code, r.message)),
                Some((C::StaleLock, message.to_owned())),
                "{text}"
            );
        }
    }

    #[test]
    fn merkle_digests_frame_every_part() {
        let base = merkle(
            "a",
            "1.0.0",
            1,
            "sha256:x",
            &["t p d".into()],
            &["b 1.0.0 d".into()],
        );
        assert!(base.starts_with("sha256:"), "{base}");
        assert_eq!(base.len(), 71);
        for other in [
            merkle(
                "a",
                "1.0.1",
                1,
                "sha256:x",
                &["t p d".into()],
                &["b 1.0.0 d".into()],
            ),
            merkle(
                "a",
                "1.0.0",
                1,
                "sha256:y",
                &["t p d".into()],
                &["b 1.0.0 d".into()],
            ),
            merkle(
                "a",
                "1.0.0",
                1,
                "sha256:x",
                &["t p e".into()],
                &["b 1.0.0 d".into()],
            ),
            merkle(
                "a",
                "1.0.0",
                1,
                "sha256:x",
                &["t p d".into()],
                &["b 1.0.0 e".into()],
            ),
            merkle(
                "a",
                "1.0.0",
                1,
                "sha256:x",
                &[],
                &["t p d".into(), "b 1.0.0 d".into()],
            ),
        ] {
            assert_ne!(base, other);
        }
    }

    #[test]
    fn canonical_json_sorts_keys_and_drops_spans() {
        let value =
            serde_json::json!({"b": [1, {"span": 3, "z": null, "a": "x"}], "a": true, "span": {}});
        let mut out = String::new();
        canonical(&value, &["span"], &mut out);
        assert_eq!(out, r#"{"a":true,"b":[1,{"a":"x","z":null}]}"#);
    }

    #[test]
    fn verify_names_changed_packages_only() {
        let mut changed = pkg("core", &[]);
        changed.version = "1.1.0".into();
        let locked = LockFile::new(vec![pkg("core", &[]), pkg("same", &[])]).render();
        let expected = LockFile::new(vec![changed.clone(), pkg("same", &[])]);
        let notes: Vec<String> = verify(Some(&locked), &expected)
            .err()
            .map(|r| r.notes.into_iter().map(|n| n.1).collect())
            .unwrap_or_default();
        let digest = &changed.digest;
        assert_eq!(
            notes,
            [
                "run `litigate lock` to regenerate Lit.lock".to_owned(),
                format!("`core` changed (locked 1.0.0 {digest}, now 1.1.0 {digest})"),
            ]
        );
    }

    #[test]
    fn control_characters_are_escaped() {
        let mut p = pkg("a\u{1}b\u{7f}\"\\", &[]);
        p.version = "tab\there".into();
        let lock = LockFile::new(vec![p]);
        let text = lock.render();
        assert!(text.contains(r#"name = "a\u0001b\u007F\"\\""#), "{text}");
        assert!(text.contains(r#"version = "tab\u0009here""#), "{text}");
        assert_eq!(LockFile::parse(&text), Ok(lock));
    }

    #[test]
    fn merge_takes_each_sides_changes_and_marks_both_changed_rows() {
        let digested = |name: &str, d: &str| LockedPackage {
            digest: d.to_owned(),
            ..pkg(name, &[])
        };
        let base = LockFile::new(vec![
            pkg("core", &[]),
            pkg("frcp", &["core 1.0.0"]),
            pkg("gone", &[]),
            pkg("ptab", &["core 1.0.0"]),
            pkg("same", &[]),
        ]);
        let ours = LockFile::new(vec![
            digested("core", "a"),
            pkg("frcp", &["core 1.0.0"]),
            digested("ptab", "a"),
            digested("same", "x"),
        ]);
        let theirs = LockFile::new(vec![
            digested("core", "b"),
            digested("frcp", "b"),
            pkg("gone", &[]),
            digested("ptab", "b"),
            digested("same", "x"),
            digested("added", "b"),
        ]);
        let merged = merge(&base, &ours, &theirs);
        let unmerged = |name: &str| digested(name, UNMERGED);
        assert_eq!(
            merged,
            Merged {
                lock: LockFile::new(vec![
                    digested("added", "b"),
                    unmerged("core"),
                    digested("frcp", "b"),
                    unmerged("ptab"),
                    digested("same", "x"),
                ]),
                unresolved: vec!["core".to_owned(), "ptab".to_owned()],
            }
        );
        let stale = verify(Some(&merged.lock.render()), &base).err();
        assert_eq!(stale.map(|r| r.code), Some(C::StaleLock));
    }

    #[test]
    fn merge_marks_a_row_one_side_changed_and_the_other_removed() {
        let base = LockFile::new(vec![pkg("frcp", &[])]);
        let mut changed = pkg("frcp", &[]);
        changed.version = "2.0.0".into();
        let ours = LockFile::new(vec![]);
        let theirs = LockFile::new(vec![changed.clone()]);
        let marked = LockFile::new(vec![LockedPackage {
            digest: UNMERGED.to_owned(),
            ..changed
        }]);
        for (a, b) in [(&ours, &theirs), (&theirs, &ours)] {
            let merged = merge(&base, a, b);
            assert_eq!(merged.lock, marked);
            assert_eq!(merged.unresolved, ["frcp"]);
        }
    }

    proptest! {
        #[test]
        fn prop_quoted_strings_parse_back(s in "\\PC{0,24}") {
            let lock = LockFile::new(vec![LockedPackage { name: s.clone(), version: s.clone(), digest: s.clone(), source: s.clone(), semantic: s.clone(), texts: vec![s.clone()], dependencies: vec![s], language: 1 }]);
            prop_assert_eq!(LockFile::parse(&lock.render()).ok(), Some(lock));
        }
    }
}
