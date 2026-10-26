//! The verification cache (SPEC §4.2). "Package X verified against its
//! dependencies at digest Z" is a build-cache fact, never a `Lit.lock`
//! one: an entry is keyed by the Merkle root of X's resolution (X's own
//! digests and every dependency's, computed on read), the profile, the
//! build dates, source corpus and the compiler (its version and the digest
//! of its executable), so a change to any dependency's digest or source
//! text misses every dependent's entry, and so does an upgraded compiler
//! over the same tree. Only
//! a session read from disk uses it;
//! CI (`CI` set) re-verifies from scratch, neither reading nor writing it.
use litigate_base::Diagnostic;
use litigate_verify::{Verification, VerificationReport};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Where entries live under the working directory unless `LITIGATE_CACHE`
/// names another directory.
pub const DEFAULT_DIR: &str = "target/litigate/verified";

/// A verification cache directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cache {
    dir: PathBuf,
}

/// One stored verification.
#[derive(Serialize, Deserialize)]
struct Entry {
    report: VerificationReport,
    diagnostics: Vec<Diagnostic>,
}

impl Cache {
    /// The cache for this process under `root`, from the environment.
    #[must_use]
    pub fn from_env(root: &Path) -> Option<Self> {
        let var = |name: &str| std::env::var(name).ok();
        Self::configured(root, var("CI").as_deref(), var("LITIGATE_CACHE").as_deref())
    }

    /// The cache under `root`: none when `ci` is set (to anything but
    /// empty or `false`) or `dir` is `off`; else in `dir` (relative to
    /// `root`), default [`DEFAULT_DIR`].
    #[must_use]
    pub fn configured(root: &Path, ci: Option<&str>, dir: Option<&str>) -> Option<Self> {
        let in_ci = ci.is_some_and(|v| !v.is_empty() && v != "false");
        let dir = dir.unwrap_or(DEFAULT_DIR);
        (!in_ci && dir != "off").then(|| Self {
            dir: root.join(dir),
        })
    }

    /// The entry file for `key`, under this compiler.
    fn path(&self, key: &[&str]) -> PathBuf {
        let mut hasher = Sha256::new();
        for part in std::iter::once(compiler()).chain(key.iter().copied()) {
            hasher.update((part.len() as u64).to_be_bytes());
            hasher.update(part.as_bytes());
        }
        let name = litigate_sema::pkg::lock::hex(&hasher.finalize());
        self.dir.join(format!("{name}.json"))
    }

    /// The verification stored under `key`, or `verify`'s, stored for the
    /// next run. An unreadable or unwritable entry is a miss, never an
    /// error.
    pub fn verified(&self, key: &[&str], verify: impl FnOnce() -> Verification) -> Verification {
        let path = self.path(key);
        let stored = std::fs::read_to_string(&path)
            .ok()
            .and_then(|text| serde_json::from_str::<Entry>(&text).ok());
        if let Some(entry) = stored {
            return Verification {
                report: entry.report,
                diagnostics: entry.diagnostics,
            };
        }
        let fresh = verify();
        let entry = Entry {
            report: fresh.report.clone(),
            diagnostics: fresh.diagnostics.clone(),
        };
        let text = serde_json::to_string(&entry).unwrap_or_default();
        let _ = std::fs::create_dir_all(&self.dir);
        let tmp = path.with_extension(format!("{}.tmp", std::process::id()));
        let _ = std::fs::write(&tmp, text).and_then(|()| std::fs::rename(&tmp, &path));
        fresh
    }
}

/// This compiler: its version and the SHA-256 of its executable's bytes,
/// so an upgraded or rebuilt compiler never reads another build's entries,
/// even one installed with the same size and modification time. Computed
/// once per process, on the first lookup; an executable that cannot be
/// read keys on its version with a marker no readable build produces.
fn compiler() -> &'static str {
    static COMPILER: OnceLock<String> = OnceLock::new();
    COMPILER.get_or_init(|| {
        identity(
            env!("CARGO_PKG_VERSION"),
            std::env::current_exe()
                .and_then(std::fs::read)
                .ok()
                .as_deref(),
        )
    })
}

/// The compiler identity of `version` built as `exe`.
fn identity(version: &str, exe: Option<&[u8]>) -> String {
    let digest = exe.map_or_else(
        || "unreadable".to_owned(),
        |bytes| {
            format!(
                "sha256:{}",
                litigate_sema::pkg::lock::hex(&Sha256::digest(bytes))
            )
        },
    );
    format!("{version} {digest}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use litigate_base::Profile;

    fn scratch(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("litigate-cache-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    fn verification(detail: &str) -> Verification {
        let diagnostic = Diagnostic::error(
            litigate_base::DiagnosticCode::VerificationFailed,
            litigate_base::Span {
                file: "a.lit".into(),
                line: 1,
                col: 1,
                end_line: 1,
                end_col: 2,
            },
            detail,
        );
        Verification {
            report: VerificationReport {
                obligations: Vec::new(),
                profile: Profile::Dev,
            },
            diagnostics: vec![diagnostic],
        }
    }

    #[test]
    fn ci_or_off_disables_the_cache() {
        let root = Path::new("/r");
        for (ci, dir) in [
            (Some("true"), None),
            (Some("1"), Some("c")),
            (None, Some("off")),
        ] {
            assert_eq!(Cache::configured(root, ci, dir), None, "{ci:?} {dir:?}");
        }
        for (ci, dir, at) in [
            (None, None, "/r/target/litigate/verified"),
            (Some(""), Some("c"), "/r/c"),
            (Some("false"), Some("/abs"), "/abs"),
        ] {
            let cache = Cache::configured(root, ci, dir);
            assert_eq!(
                cache.map(|c| c.dir),
                Some(PathBuf::from(at)),
                "{ci:?} {dir:?}"
            );
        }
        assert_eq!(
            Cache::from_env(root).is_some(),
            Cache::configured(
                root,
                std::env::var("CI").ok().as_deref(),
                std::env::var("LITIGATE_CACHE").ok().as_deref()
            )
            .is_some()
        );
    }

    /// What a fresh verification returns in these tests.
    fn fresh() -> Verification {
        verification("fresh")
    }

    /// Plants `v` as the entry for `key`.
    fn plant(cache: &Cache, key: &[&str], v: &Verification) {
        let entry = Entry {
            report: v.report.clone(),
            diagnostics: v.diagnostics.clone(),
        };
        let text = serde_json::to_string(&entry).unwrap();
        std::fs::write(cache.path(key), text).unwrap();
    }

    #[test]
    fn an_entry_is_reused_under_its_key_only() {
        let dir = scratch("reuse");
        let cache = Cache::configured(Path::new("/"), None, dir.to_str()).unwrap();
        assert_eq!(cache.verified(&["root-a", "dev"], fresh), fresh());
        assert!(cache.path(&["root-a", "dev"]).is_file());
        plant(&cache, &["root-a", "dev"], &verification("stored"));
        assert_eq!(
            cache.verified(&["root-a", "dev"], fresh),
            verification("stored")
        );
        // Another dependency digest is another resolution root: a miss.
        assert_eq!(cache.verified(&["root-b", "dev"], fresh), fresh());
        // Framing: the parts are not concatenated.
        assert_ne!(cache.path(&["ab", "c"]), cache.path(&["a", "bc"]));
        // A corrupt entry is a miss, rewritten.
        std::fs::write(cache.path(&["root-a", "dev"]), "{").unwrap();
        assert_eq!(cache.verified(&["root-a", "dev"], fresh), fresh());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_unwritable_cache_is_a_miss() {
        let dir = scratch("unwritable");
        std::fs::write(&dir, "a file, not a directory").unwrap();
        let cache = Cache::configured(Path::new("/"), None, dir.to_str()).unwrap();
        assert_eq!(cache.verified(&["k"], fresh), fresh());
        assert!(!cache.path(&["k"]).exists());
        let _ = std::fs::remove_file(&dir);
    }

    #[test]
    fn the_compiler_is_its_version_and_executable_digest() {
        let a = identity("1.0.0", Some(b"build a"));
        // Same version, same size, other bytes: another compiler.
        assert_ne!(a, identity("1.0.0", Some(b"build b")));
        assert_ne!(a, identity("1.0.1", Some(b"build a")));
        assert_eq!(a, identity("1.0.0", Some(b"build a")));
        assert!(a.starts_with("1.0.0 sha256:"), "{a}");
        assert_eq!(identity("1.0.0", None), "1.0.0 unreadable");
        assert!(compiler().starts_with(concat!(env!("CARGO_PKG_VERSION"), " sha256:")));
    }
}
