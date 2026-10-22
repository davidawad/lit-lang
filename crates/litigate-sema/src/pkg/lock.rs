//! Package digests and the deterministic `Lit.lock` (SPEC §4.2).
use crate::db::span::{Report, TextRange};
use litigate_base::DiagnosticCode as C;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fmt::Write as _;

/// SHA-256 over `Lit.toml` then every source file sorted by
/// package-relative POSIX path, each framed as `u64 BE path length, path,
/// u64 BE content length, content`. Returns lowercase hex.
#[must_use]
pub fn digest<'a>(manifest: &str, files: impl IntoIterator<Item = (&'a str, &'a str)>) -> String {
    let mut files: Vec<(&str, &str)> = files.into_iter().collect();
    files.sort_by(|a, b| a.0.cmp(b.0));
    let mut hasher = Sha256::new();
    for (path, content) in std::iter::once(("Lit.toml", manifest)).chain(files) {
        for part in [path.as_bytes(), content.as_bytes()] {
            hasher.update((part.len() as u64).to_be_bytes());
            hasher.update(part);
        }
    }
    hasher
        .finalize()
        .iter()
        .fold(String::with_capacity(64), |mut s, b| {
            let _ = write!(s, "{b:02x}");
            s
        })
}

/// One resolved package in `Lit.lock`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LockedPackage {
    /// Package name.
    pub name: String,
    /// Exact version.
    pub version: String,
    /// `sha256:<hex>`.
    pub digest: String,
    /// Sorted `"name version"` strings.
    pub dependencies: Vec<String>,
    /// Language version.
    pub language: u32,
}

/// Parsed `Lit.lock`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LockFile {
    /// Format version; must be 1.
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
        }
        packages.sort_by(|a, b| (&a.name, &a.version).cmp(&(&b.name, &b.version)));
        Self {
            version: 1,
            packages,
        }
    }

    /// Deterministic TOML text: no timestamps, no absolute paths.
    #[must_use]
    pub fn render(&self) -> String {
        let mut out = String::from("version = 1\n");
        for p in &self.packages {
            let deps: Vec<String> = p.dependencies.iter().map(|d| quote(d)).collect();
            let _ = write!(
                out,
                "\n[[package]]\nname = {}\nversion = {}\ndigest = {}\ndependencies = [{}]\nlanguage = {}\n",
                quote(&p.name),
                quote(&p.version),
                quote(&p.digest),
                deps.join(", "),
                p.language
            );
        }
        out
    }

    /// Parses `Lit.lock` text.
    ///
    /// # Errors
    /// L1007 for unparseable text or an unknown format version.
    pub fn parse(text: &str) -> Result<Self, Report> {
        let lock: Self = toml::from_str(text).map_err(|e| {
            Report::new(
                C::StaleLock,
                TextRange::default(),
                format!("Lit.lock is invalid: {}", e.message().trim()),
            )
            .note("regenerate it with `litigate lock`")
        })?;
        if lock.version != 1 {
            return Err(Report::new(
                C::StaleLock,
                TextRange::default(),
                format!(
                    "Lit.lock format {} is not supported (expected 1)",
                    lock.version
                ),
            ));
        }
        Ok(lock)
    }
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
                ))
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
            digest: format!("sha256:{}", digest("m", [])),
            dependencies: deps.iter().map(|d| (*d).to_owned()).collect(),
            language: 1,
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
        assert!(verify(None, &lock)
            .is_err_and(|r| r.code == C::StaleLock && r.message.contains("missing")));
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
        assert!(LockFile::parse("version = 2").is_err());
    }

    proptest! {
        #[test]
        fn prop_quoted_strings_parse_back(s in "\\PC{0,24}") {
            let lock = LockFile::new(vec![LockedPackage { name: s.clone(), version: s.clone(), digest: s, dependencies: vec![], language: 1 }]);
            prop_assert_eq!(LockFile::parse(&lock.render()).ok(), Some(lock));
        }
    }
}
