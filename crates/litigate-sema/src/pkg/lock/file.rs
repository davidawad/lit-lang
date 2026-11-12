
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
            p.modules.sort();
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
            if p.is_path() {
                let _ = write!(
                    out,
                    "\n[[package]]\nname = {}\nversion = {}\ndependencies = [{}]\n",
                    quote(&p.name),
                    quote(&p.version),
                    list(&p.dependencies),
                );
                continue;
            }
            if p.modules.is_empty() {
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
                continue;
            }
            let _ = write!(
                out,
                "\n[[package]]\nname = {}\nversion = {}\nlanguage = {}\nmodules = [{}]\ntexts = [{}]\ndependencies = [{}]\n",
                quote(&p.name),
                quote(&p.version),
                p.language,
                list(&p.modules),
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
        if version != FORMAT && version != 2 {
            return Err(Report::new(
                C::StaleLock,
                TextRange::default(),
                format!("Lit.lock format {version} is not supported (expected {FORMAT})"),
            )
            .note("regenerate it with `litigate lock`"));
        }
        let mut lock: Self = toml::from_str(text).map_err(|e| invalid(&e))?;
        if version == 2 {
            lock.version = 2;
        }
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

    /// Every row is the only one of its name and version, records either
    /// all of its digests or (a path dependency) none, and every
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
            let digests = [
                !p.semantic.is_empty() || !p.modules.is_empty(),
                p.language != 0,
            ];
            let partial = if p.is_path() {
                digests.contains(&true) || !p.texts.is_empty()
            } else {
                digests.contains(&false) || (p.modules.is_empty() && p.semantic.is_empty())
            };
            if partial {
                return fail(format!(
                    "Lit.lock: `{}` {} records only some of `language`, `source` and `semantic`",
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
        let content = if p.modules.is_empty() {
            p.content().to_owned()
        } else {
            p.modules.join("\n")
        };
        let digest = merkle(&p.name, &p.version, p.language, &content, &p.texts, &deps);
        if !cyclic || path.is_empty() {
            done.insert(key, digest.clone());
        }
        (digest, cyclic)
    }

    /// The Merkle root: SHA-256 over the format 3 tag and each row's
    /// [`LockFile::merkle`] digest in row order. Returns `sha256:<hex>`.
    #[must_use]
    pub fn root(&self) -> String {
        let rows = self.merkle();
        format!(
            "sha256:{}",
            framed(
                std::iter::once("litlock root 3".as_bytes())
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
            Some(f) if f != p => report = report.note(changed(f, p)),
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

/// The note for `name`, locked as `was` and now resolved as `now`: the
/// version and content digest when either differs, then each other field
/// that does, so the note always names what changed.
fn changed(was: &LockedPackage, now: &LockedPackage) -> String {
    let mut note = format!("`{}` changed", now.name);
    if was.shown() != now.shown() {
        let _ = write!(note, " (locked {}, now {})", was.shown(), now.shown());
    }
    let mut parts = Vec::new();
    if was.language != now.language {
        parts.push(format!(
            "language (locked {}, now {})",
            was.language, now.language
        ));
    }
    // An unchecked row's content is its source digest, already shown.
    let shown = |p: &LockedPackage| p.content() == p.source;
    if was.source != now.source && !(shown(was) && shown(now)) {
        parts.push(format!(
            "source (locked {}, now {})",
            was.source, now.source
        ));
    }
    for (field, old, new) in [
        ("modules", &was.modules, &now.modules),
        ("texts", &was.texts, &now.texts),
        ("dependencies", &was.dependencies, &now.dependencies),
    ] {
        let gone: Vec<String> = old
            .iter()
            .filter(|e| !new.contains(e))
            .map(|e| format!("`{e}`"))
            .collect();
        let added: Vec<String> = new
            .iter()
            .filter(|e| !old.contains(e))
            .map(|e| format!("`{e}`"))
            .collect();
        let mut diff = Vec::new();
        if !gone.is_empty() {
            diff.push(format!("drops {}", gone.join(", ")));
        }
        if !added.is_empty() {
            diff.push(format!("adds {}", added.join(", ")));
        }
        if !diff.is_empty() {
            parts.push(format!("{field} ({})", diff.join(", ")));
        }
    }
    if !parts.is_empty() {
        let _ = write!(note, ": {}", parts.join("; "));
    }
    note
}
