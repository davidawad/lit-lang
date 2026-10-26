//! Package digests and the deterministic, Merkle `Lit.lock` (SPEC §4.2).
use crate::db::span::{Report, TextRange};
use crate::hir;
use litigate_base::{Applicability, DiagnosticCode as C};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
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
/// It is never stored: [`LockFile::merkle`] computes it on read.
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
    /// Sorted `"name version"` entries, each naming exactly one row of
    /// the same lock.
    pub dependencies: Vec<String>,
}

impl LockedPackage {
    /// What the row's Merkle digest covers of the package itself: the
    /// semantic digest, or the source digest of a package that does not
    /// check.
    #[must_use]
    pub fn content(&self) -> &str {
        if self.semantic == UNCHECKED {
            &self.source
        } else {
            &self.semantic
        }
    }
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
                "\n[[package]]\nname = {}\nversion = {}\nlanguage = {}\nsource = {}\nsemantic = {}\ntexts = [{}]\ndependencies = [{}]\n",
                quote(&p.name),
                quote(&p.version),
                p.language,
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
        let lock: Self = toml::from_str(text).map_err(|e| invalid(&e))?;
        lock.pins()?;
        Ok(lock)
    }

    /// The row `"name version"` names, if exactly one does.
    fn row(&self, pin: &str) -> Option<&LockedPackage> {
        let (name, version) = pin.rsplit_once(' ')?;
        self.find(name, version)
    }

    /// The row of `name` at `version`, if exactly one is.
    fn find(&self, name: &str, version: &str) -> Option<&LockedPackage> {
        let mut rows = self
            .packages
            .iter()
            .filter(|p| p.name == name && p.version == version);
        let row = rows.next();
        rows.next().is_none().then_some(row).flatten()
    }

    /// Every row is the only one of its name and version, and every
    /// dependency names exactly one row.
    ///
    /// # Errors
    /// L1007 naming the first row or dependency that does not.
    fn pins(&self) -> Result<(), Report> {
        let fail = |message: String| {
            Err(Report::new(C::StaleLock, TextRange::default(), message)
                .note("regenerate it with `litigate lock`"))
        };
        for p in &self.packages {
            if self.find(&p.name, &p.version).is_none() {
                return fail(format!(
                    "Lit.lock locks `{}` {} more than once",
                    p.name, p.version
                ));
            }
            if let Some(dep) = p.dependencies.iter().find(|d| self.row(d).is_none()) {
                return fail(format!(
                    "Lit.lock: `{}` depends on `{dep}`, which names no locked package",
                    p.name
                ));
            }
        }
        Ok(())
    }

    /// The Merkle digest of every row, computed on read: [`merkle`] over
    /// the row's own digests and each dependency's Merkle digest, keyed by
    /// `"name version"`. A dependency that closes a cycle is entered
    /// without a digest. Rows come from a lock [`LockFile::parse`] (or
    /// [`LockFile::new`] over resolved rows) accepted, so every dependency
    /// names exactly one row.
    #[must_use]
    pub fn merkle(&self) -> BTreeMap<String, String> {
        let mut done = BTreeMap::new();
        for p in &self.packages {
            self.digest_of(p, &mut Vec::new(), &mut done);
        }
        done
    }

    /// `p`'s Merkle digest, and whether it entered a dependency that
    /// closes a cycle through `path` (so depends on where the walk began
    /// and is recorded only for the row the walk began at).
    fn digest_of(
        &self,
        p: &LockedPackage,
        path: &mut Vec<String>,
        done: &mut BTreeMap<String, String>,
    ) -> (String, bool) {
        let key = format!("{} {}", p.name, p.version);
        if let Some(d) = done.get(&key) {
            return (d.clone(), false);
        }
        path.push(key.clone());
        let mut cyclic = false;
        let mut deps = Vec::new();
        for d in &p.dependencies {
            match self.row(d) {
                Some(row) if !path.contains(d) => {
                    let (digest, through) = self.digest_of(row, path, done);
                    cyclic |= through;
                    deps.push(format!("{d} {digest}"));
                }
                _ => {
                    cyclic = true;
                    deps.push(d.clone());
                }
            }
        }
        path.pop();
        let digest = merkle(
            &p.name,
            &p.version,
            p.language,
            p.content(),
            &p.texts,
            &deps,
        );
        if !cyclic || path.is_empty() {
            done.insert(key, digest.clone());
        }
        (digest, cyclic)
    }

    /// The Merkle root: SHA-256 over the format tag and each row's
    /// [`LockFile::merkle`] digest in row order. Returns `sha256:<hex>`.
    #[must_use]
    pub fn root(&self) -> String {
        let rows = self.merkle();
        format!(
            "sha256:{}",
            framed(
                std::iter::once("litlock root 2".as_bytes())
                    .chain(rows.iter().flat_map(|(k, d)| [k.as_bytes(), d.as_bytes()]))
            )
        )
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

/// L1007 for `text`, a lock in a format older than [`FORMAT`]: it names
/// `litigate lock DIR` and carries the machine-applicable fix replacing
/// the whole lock with `upgraded`, the text that command writes there.
/// `None` for a lock in this format, a newer one, or no readable format.
#[must_use]
pub fn upgrade(text: &str, dir: &str, upgraded: impl FnOnce() -> String) -> Option<Report> {
    let version = LockFile::format(text).ok().filter(|v| *v < FORMAT)?;
    let end = u32::try_from(text.len()).unwrap_or(u32::MAX);
    Some(
        Report::new(
            C::StaleLock,
            TextRange::default(),
            format!(
                "Lit.lock format {version} is not supported (expected {FORMAT}): run `litigate lock {dir}` to upgrade it"
            ),
        )
        .fix(
            TextRange::new(0, end),
            upgraded(),
            Applicability::MachineApplicable,
        ),
    )
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
                    p.name,
                    f.version,
                    f.content(),
                    p.version,
                    p.content()
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

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn pkg(name: &str, deps: &[&str]) -> LockedPackage {
        LockedPackage {
            name: name.into(),
            version: "1.0.0".into(),
            language: 1,
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
            pkg("alpha", &["core 1.0.0"]),
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
    fn only_an_older_format_is_upgraded() {
        let v1 = "version = 1\n\n[[package]]\nname = \"core\"\n";
        let report = upgrade(v1, "lit/core", || "version = 2\n".to_owned());
        assert_eq!(
            report,
            Some(
                Report::new(
                    C::StaleLock,
                    TextRange::default(),
                    "Lit.lock format 1 is not supported (expected 2): run `litigate lock lit/core` to upgrade it",
                )
                .fix(
                    TextRange::new(0, 39),
                    "version = 2\n",
                    Applicability::MachineApplicable
                )
            )
        );
        for current in ["version = 2", "version = 3", "version = "] {
            assert_eq!(upgrade(current, ".", String::new), None, "{current}");
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
        let source = &changed.source;
        assert_eq!(
            notes,
            [
                "run `litigate lock` to regenerate Lit.lock".to_owned(),
                format!("`core` changed (locked 1.0.0 {source}, now 1.1.0 {source})"),
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
    fn every_pin_names_exactly_one_row() {
        let twice = LockFile::new(vec![pkg("core", &[]), pkg("core", &[])]).render();
        let dangling = LockFile::new(vec![pkg("app", &["core 1.0.0"])]).render();
        let unversioned = LockFile::new(vec![pkg("core", &[]), pkg("app", &["core"])]).render();
        for (text, message) in [
            (twice, "Lit.lock locks `core` 1.0.0 more than once"),
            (
                dangling,
                "Lit.lock: `app` depends on `core 1.0.0`, which names no locked package",
            ),
            (
                unversioned,
                "Lit.lock: `app` depends on `core`, which names no locked package",
            ),
        ] {
            let report = LockFile::parse(&text).err();
            assert_eq!(
                report.map(|r| (r.code, r.message)),
                Some((C::StaleLock, message.to_owned()))
            );
        }
    }

    #[test]
    fn merkle_digests_are_computed_on_read_over_dependencies() {
        let lock = LockFile::new(vec![
            pkg("core", &[]),
            pkg("frcp", &["core 1.0.0"]),
            pkg("links", &["core 1.0.0", "frcp 1.0.0"]),
            pkg("ttab", &["core 1.0.0"]),
        ]);
        let rows = lock.merkle();
        let core = merkle(
            "core",
            "1.0.0",
            1,
            &lock.packages[0].source,
            &lock.packages[0].texts,
            &[],
        );
        assert_eq!(rows["core 1.0.0"], core);
        let frcp = merkle(
            "frcp",
            "1.0.0",
            1,
            &lock.packages[1].source,
            &lock.packages[1].texts,
            &[format!("core 1.0.0 {core}")],
        );
        assert_eq!(rows["frcp 1.0.0"], frcp);
        let root = lock.root();
        assert!(root.starts_with("sha256:"), "{root}");
        assert_eq!(root.len(), 71);
        // A change to frcp's own digest moves frcp, links and the root,
        // never ttab or core.
        let mut edited = lock.clone();
        edited.packages[1].semantic = "sha256:1".to_owned();
        let moved = edited.merkle();
        let same: Vec<&str> = rows
            .iter()
            .filter(|(k, d)| moved[*k] == **d)
            .map(|(k, _)| k.as_str())
            .collect();
        assert_eq!(same, ["core 1.0.0", "ttab 1.0.0"]);
        assert_ne!(edited.root(), root);
    }

    #[test]
    fn a_cycle_is_digested_from_where_the_walk_begins() {
        let lock = LockFile::new(vec![
            pkg("a", &["b 1.0.0"]),
            pkg("b", &["a 1.0.0"]),
            pkg("c", &["a 1.0.0"]),
        ]);
        let rows = lock.merkle();
        let b_in_a = merkle(
            "b",
            "1.0.0",
            1,
            &lock.packages[1].source,
            &lock.packages[1].texts,
            &["a 1.0.0".to_owned()],
        );
        let a = merkle(
            "a",
            "1.0.0",
            1,
            &lock.packages[0].source,
            &lock.packages[0].texts,
            &[format!("b 1.0.0 {b_in_a}")],
        );
        assert_eq!(rows["a 1.0.0"], a);
        assert_eq!(
            rows["b 1.0.0"],
            merkle(
                "b",
                "1.0.0",
                1,
                &lock.packages[1].source,
                &lock.packages[1].texts,
                &[format!("a 1.0.0 {a}")]
            )
        );
        assert_eq!(rows.len(), 3);
    }

    proptest! {
        #[test]
        fn prop_quoted_strings_parse_back(s in "\\PC{0,24}") {
            let lock = LockFile::new(vec![LockedPackage { name: s.clone(), version: s.clone(), source: s.clone(), semantic: s.clone(), texts: vec![s.clone()], dependencies: Vec::new(), language: 1 }]);
            prop_assert_eq!(LockFile::parse(&lock.render()).ok(), Some(lock));
        }
    }
}
