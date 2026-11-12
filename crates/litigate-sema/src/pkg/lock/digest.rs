/// The `Lit.lock` format this compiler reads and writes.
pub const FORMAT: u32 = 3;

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

/// The Merkle digest of a lock row: SHA-256 over the format 3 tag, `name`,
/// `version`, `language`, `content` (the semantic digest, or the source
/// digest of a package that does not check), the number of `texts` and
/// each entry, then the number of `dependencies` and each entry (which
/// carries the dependency's own Merkle digest). A path dependency's row
/// has no language or content, so its digest covers its name, version and
/// dependencies only. Returns `sha256:<hex>`.
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
    let head = ["litlock 3", name, version, &language, content];
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

/// One resolved package in `Lit.lock`. A path dependency (a package
/// resolved from inside the same repository, as Cargo treats one) is
/// pinned by name, version and dependencies only: its `language`,
/// `source`, `semantic` and `texts` are empty, since it is checked from
/// source on every build, so an edit to it never rewrites a dependent's
/// lock.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LockedPackage {
    /// Package name.
    pub name: String,
    /// Exact version.
    pub version: String,
    /// Language version (0 for a path dependency).
    #[serde(default)]
    pub language: u32,
    /// Source digest ([`digest`]), `sha256:<hex>` (empty for a path
    /// dependency).
    #[serde(default)]
    pub source: String,
    /// Semantic digest ([`semantic_digest`]), `sha256:<hex>`, or
    /// [`UNCHECKED`] (empty for a path dependency).
    #[serde(default)]
    pub semantic: String,
    /// Sorted per-module `path source semantic` digests in format 3.
    #[serde(default)]
    pub modules: Vec<String>,
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
    /// The row of a path dependency: `name`, `version` and
    /// `dependencies`, without digests.
    #[must_use]
    pub fn path(name: String, version: String, dependencies: Vec<String>) -> Self {
        Self {
            name,
            version,
            dependencies,
            ..Self::default()
        }
    }

    /// This row without its digests: the row of a path dependency.
    #[must_use]
    pub fn into_path(self) -> Self {
        Self::path(self.name, self.version, self.dependencies)
    }

    /// Whether the row pins a path dependency (it records no digests).
    #[must_use]
    pub fn is_path(&self) -> bool {
        self.source.is_empty() && self.modules.is_empty()
    }

    /// `version`, then the content digest unless a path dependency.
    #[must_use]
    pub fn shown(&self) -> String {
        if self.is_path() {
            self.version.clone()
        } else {
            format!("{} {}", self.version, self.content())
        }
    }

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
