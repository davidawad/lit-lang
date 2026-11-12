//! Package digests and the deterministic, Merkle `Lit.lock` (SPEC §4.2).
use crate::db::span::{Report, TextRange};
use crate::hir;
use litigate_base::{Applicability, DiagnosticCode as C};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fmt::Write as _;
include!("lock/digest.rs");
include!("lock/file.rs");
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
            modules: Vec::new(),
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
    fn format_three_renders_sorted_module_pins_one_per_line() {
        let mut p = pkg("core", &[]);
        p.source.clear();
        p.semantic.clear();
        p.modules = vec![
            "states/hawaii.lit sha256:b sha256:d".into(),
            "states/delaware.lit sha256:a sha256:c".into(),
        ];
        let text = LockFile::new(vec![p]).render();
        assert!(text.starts_with("version = 3\n"));
        let delaware = text.find("states/delaware.lit").unwrap();
        let hawaii = text.find("states/hawaii.lit").unwrap();
        assert!(delaware < hawaii);
        assert_eq!(text.matches("states/").count(), 2);
        assert!(!text.contains("\nsource ="));
        assert!(!text.contains("\nsemantic ="));
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
                "Lit.lock format 1 is not supported (expected 3)",
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
    fn format_two_locks_remain_readable_for_migration_fixtures() {
        let text = "version = 2\n\n[[package]]\nname = \"core\"\nversion = \"1.0.0\"\nlanguage = 1\nsource = \"sha256:source\"\nsemantic = \"sha256:semantic\"\ntexts = []\ndependencies = []\n";
        let parsed = LockFile::parse(text).unwrap();
        assert_eq!(parsed.version, 2);
        assert_eq!(parsed.packages[0].source, "sha256:source");
    }

    #[test]
    fn only_an_older_format_is_upgraded() {
        let v1 = "version = 1\n\n[[package]]\nname = \"core\"\n";
        let report = upgrade(v1, "lit/core", || "version = 3\n".to_owned());
        assert_eq!(
            report,
            Some(
                Report::new(
                    C::StaleLock,
                    TextRange::default(),
                    "Lit.lock format 1 is not supported (expected 3): run `litigate lock lit/core` to upgrade it",
                )
                .fix(
                    TextRange::new(0, 39),
                    "version = 3\n",
                    Applicability::MachineApplicable
                )
            )
        );
        for current in ["version = 3", "version = "] {
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
        fn prop_quoted_strings_parse_back(s in "\\PC{1,24}") {
            let lock = LockFile::new(vec![LockedPackage { name: s.clone(), version: s.clone(), source: s.clone(), semantic: s.clone(), modules: Vec::new(), texts: vec![s.clone()], dependencies: Vec::new(), language: 1 }]);
            prop_assert_eq!(LockFile::parse(&lock.render()).ok(), Some(lock));
        }
    }
}
