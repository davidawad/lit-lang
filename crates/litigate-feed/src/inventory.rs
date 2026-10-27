//! What the feed watches, read from the packages under `lit/`: the courts
//! of the promoted packages (through `courts.toml`), the precedent layers,
//! the cite prefixes the packages declare, and each package's elements
//! (the nodes of its built pack).

use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

/// The committed court table (`courts.toml`).
const COURTS: &str = include_str!("../courts.toml");

/// A package under `lit/`, as the feed needs it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Package {
    /// Its directory, repository-relative with `/` separators (`lit/ca8`).
    pub dir: String,
    /// `[package].name`.
    pub name: String,
    /// `[package].pack_id`: absent for a links package.
    pub pack_id: Option<String>,
    /// `[package].kind` (`base` when absent).
    pub kind: String,
    /// The directories of its path dependencies, normalized like `dir`.
    pub deps: Vec<String>,
    /// `[package.instrument]`, for a layer.
    pub instrument: Option<Instrument>,
    /// The URLs of its `[sources.*]`, in declaration order.
    pub source_urls: Vec<String>,
    /// Its cite prefixes, one group per `[[cite_syntax.rules]]` (equivalent
    /// spellings) and one per source's `cite_prefixes`.
    pub prefix_groups: Vec<Vec<String>>,
}

/// `[package.instrument]`: the fields the feed reads.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct Instrument {
    /// The issuing court (a precedent's).
    pub court: Option<String>,
    /// An order number or reporter cite.
    pub designation: Option<String>,
    /// When it issued.
    pub date: Option<String>,
}

/// A precedent layer whose citing opinions the feed polls.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Precedent {
    /// The precedent package's name.
    pub package: String,
    /// What `CourtListener`'s citation lookup is given: its designation.
    pub citation: Option<String>,
    /// The `CourtListener` cluster id, when a source URL is a `CourtListener`
    /// opinion page.
    pub cluster: Option<u64>,
}

/// Everything the feed watches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inventory {
    /// Every package under `lit/`, by directory.
    pub packages: Vec<Package>,
    /// `CourtListener` court id -> the names of the packages modeling it.
    pub courts: BTreeMap<String, Vec<String>>,
    /// The precedent layers.
    pub precedents: Vec<Precedent>,
}

#[derive(Deserialize)]
struct Manifest {
    package: ManifestPackage,
    #[serde(default)]
    dependencies: BTreeMap<String, toml::Value>,
    #[serde(default)]
    sources: BTreeMap<String, ManifestSource>,
    #[serde(default)]
    cite_syntax: CiteSyntax,
}

#[derive(Deserialize)]
struct ManifestPackage {
    name: String,
    pack_id: Option<String>,
    kind: Option<String>,
    instrument: Option<Instrument>,
}

#[derive(Deserialize)]
struct ManifestSource {
    url: Option<String>,
    #[serde(default)]
    cite_prefixes: Vec<String>,
}

#[derive(Default, Deserialize)]
struct CiteSyntax {
    #[serde(default)]
    rules: Vec<CiteRule>,
}

#[derive(Deserialize)]
struct CiteRule {
    prefixes: Vec<String>,
}

#[derive(Deserialize)]
struct CourtTable {
    packages: BTreeMap<String, CourtEntry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CourtEntry {
    #[serde(default)]
    courts: Vec<String>,
    none: Option<String>,
}

impl Inventory {
    /// Reads every `Lit.toml` under `repo/lit` against the committed court
    /// table.
    ///
    /// # Errors
    /// An unreadable or malformed manifest, or a court table that does not
    /// list exactly the packages with a pack identity.
    pub fn read(repo: &Path) -> Result<Self, String> {
        Self::read_with(repo, COURTS)
    }

    /// [`Inventory::read`] against the court table `courts` (TOML).
    ///
    /// # Errors
    /// As [`Inventory::read`].
    pub fn read_with(repo: &Path, courts: &str) -> Result<Self, String> {
        let mut manifests = Vec::new();
        find_manifests(&repo.join("lit"), &mut manifests)?;
        manifests.sort();
        let packages = manifests
            .iter()
            .map(|path| read_package(repo, path))
            .collect::<Result<Vec<_>, _>>()?;
        let table: CourtTable =
            toml::from_str(courts).map_err(|e| format!("courts.toml: {}", e.message()))?;
        let mut court_map: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let promoted: BTreeSet<&str> = packages
            .iter()
            .filter(|p| p.pack_id.is_some() && p.kind == "base")
            .map(|p| p.name.as_str())
            .collect();
        for name in &promoted {
            match table.packages.get(*name) {
                None => {
                    return Err(format!(
                        "courts.toml: promoted package `{name}` is not listed: give its \
                         CourtListener `courts` or say why it has `none`"
                    ))
                }
                Some(entry) if entry.courts.is_empty() == entry.none.is_none() => {
                    return Err(format!(
                        "courts.toml: `{name}` needs exactly one of a non-empty `courts` or `none`"
                    ))
                }
                Some(entry) => {
                    for court in &entry.courts {
                        court_map
                            .entry(court.clone())
                            .or_default()
                            .push((*name).to_owned());
                    }
                }
            }
        }
        if let Some(stale) = table
            .packages
            .keys()
            .find(|name| !promoted.contains(name.as_str()))
        {
            return Err(format!(
                "courts.toml: `{stale}` is not a promoted package under lit/"
            ));
        }
        let precedents = packages
            .iter()
            .filter(|p| p.kind == "precedent")
            .map(|p| Precedent {
                package: p.name.clone(),
                citation: p.instrument.as_ref().and_then(|i| i.designation.clone()),
                cluster: p.source_urls.iter().find_map(|u| cluster_of(u)),
            })
            .collect();
        Ok(Self {
            packages,
            courts: court_map,
            precedents,
        })
    }

    /// The package named `name`.
    #[must_use]
    pub fn package(&self, name: &str) -> Option<&Package> {
        self.packages.iter().find(|p| p.name == name)
    }

    /// The names of `names` and every package they depend on, transitively.
    #[must_use]
    pub fn closure<'a>(&self, names: impl IntoIterator<Item = &'a str>) -> BTreeSet<String> {
        let mut seen = BTreeSet::new();
        let mut stack: Vec<&Package> = names.into_iter().filter_map(|n| self.package(n)).collect();
        while let Some(package) = stack.pop() {
            if seen.insert(package.name.clone()) {
                stack.extend(
                    package
                        .deps
                        .iter()
                        .filter_map(|d| self.packages.iter().find(|p| &p.dir == d)),
                );
            }
        }
        seen
    }

    /// Every package's cite prefix groups.
    #[must_use]
    pub fn prefix_groups(&self) -> Vec<Vec<String>> {
        self.packages
            .iter()
            .flat_map(|p| p.prefix_groups.iter().cloned())
            .collect()
    }
}

/// The `CourtListener` cluster id of an opinion page URL
/// (`https://www.courtlistener.com/opinion/<cluster>/<slug>/`).
#[must_use]
pub fn cluster_of(url: &str) -> Option<u64> {
    let rest = url.split("courtlistener.com/opinion/").nth(1)?;
    rest.split('/').next()?.parse().ok()
}

fn find_manifests(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            find_manifests(&path, out)?;
        } else if path.file_name().is_some_and(|n| n == "Lit.toml") {
            out.push(path);
        }
    }
    Ok(())
}

fn read_package(repo: &Path, manifest: &Path) -> Result<Package, String> {
    let shown = manifest.display();
    let text = std::fs::read_to_string(manifest).map_err(|e| format!("{shown}: {e}"))?;
    let parsed: Manifest =
        toml::from_str(&text).map_err(|e| format!("{shown}: {}", e.message()))?;
    let dir = manifest
        .parent()
        .and_then(|d| d.strip_prefix(repo).ok())
        .map(slashed)
        .unwrap_or_default();
    let deps = parsed
        .dependencies
        .values()
        .filter_map(|dep| dep.get("path").and_then(toml::Value::as_str))
        .map(|path| normalize(&format!("{dir}/{path}")))
        .collect();
    let mut prefix_groups: Vec<Vec<String>> = parsed
        .cite_syntax
        .rules
        .into_iter()
        .map(|r| r.prefixes)
        .collect();
    prefix_groups.extend(
        parsed
            .sources
            .values()
            .filter(|s| !s.cite_prefixes.is_empty())
            .map(|s| s.cite_prefixes.clone()),
    );
    Ok(Package {
        dir,
        name: parsed.package.name,
        pack_id: parsed.package.pack_id,
        kind: parsed.package.kind.unwrap_or_else(|| "base".to_owned()),
        deps,
        instrument: parsed.package.instrument,
        source_urls: parsed.sources.into_values().filter_map(|s| s.url).collect(),
        prefix_groups,
    })
}

fn slashed(path: &Path) -> String {
    path.components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

/// `a/b/../c` -> `a/c`, lexically.
fn normalize(path: &str) -> String {
    let mut parts: Vec<String> = Vec::new();
    for component in Path::new(path).components() {
        match component {
            Component::ParentDir => {
                parts.pop();
            }
            Component::Normal(part) => parts.push(part.to_string_lossy().into_owned()),
            _ => {}
        }
    }
    parts.join("/")
}

/// An element: a node of a package's built pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element {
    /// The package's name.
    pub package: String,
    /// The node id.
    pub id: String,
    /// Its label.
    pub label: String,
    /// Its cite, when it has one.
    pub cite: Option<String>,
}

/// The elements of `names` (packages of `inventory`), from `litigate
/// build` over an in-memory copy of `repo/lit`: package name -> its nodes.
/// A package that emits no pack (a library, a links package) has none.
///
/// # Errors
/// An unreadable `lit/` file or a failed build.
pub fn build_elements(
    repo: &Path,
    inventory: &Inventory,
    names: &BTreeSet<String>,
) -> Result<BTreeMap<String, Vec<Element>>, String> {
    let built: Vec<&Package> = names
        .iter()
        .filter_map(|n| inventory.package(n))
        .filter(|p| p.pack_id.is_some())
        .collect();
    let mut out = BTreeMap::new();
    if built.is_empty() {
        return Ok(out);
    }
    let mut files = BTreeMap::new();
    collect_files(repo, &repo.join("lit"), &mut files)?;
    let mut args = vec!["build".to_owned()];
    args.extend(built.iter().map(|p| p.dir.clone()));
    args.extend(["--out".to_owned(), "feed-build".to_owned()]);
    let run = litigate_cli::execute_in_memory(files, &args, None);
    if run.code != 0 {
        return Err(format!(
            "`litigate {}` failed (exit {}): {}{}",
            args.join(" "),
            run.code,
            run.stdout,
            run.stderr
        ));
    }
    for package in built {
        let id = package.pack_id.as_deref().unwrap_or_default();
        let Some(text) = run.files.get(&format!("feed-build/packs/{id}.json")) else {
            continue;
        };
        let pack: serde_json::Value =
            serde_json::from_str(text).map_err(|e| format!("pack `{id}`: {e}"))?;
        let nodes = pack["nodes"].as_array().cloned().unwrap_or_default();
        let elements = nodes
            .iter()
            .map(|node| Element {
                package: package.name.clone(),
                id: node["id"].as_str().unwrap_or_default().to_owned(),
                label: node["label"].as_str().unwrap_or_default().to_owned(),
                cite: node["cite"].as_str().map(str::to_owned),
            })
            .collect();
        out.insert(package.name.clone(), elements);
    }
    Ok(out)
}

fn collect_files(
    repo: &Path,
    dir: &Path,
    out: &mut BTreeMap<String, String>,
) -> Result<(), String> {
    let entries = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files(repo, &path, out)?;
        } else if let Ok(text) = std::fs::read_to_string(&path) {
            let rel = path.strip_prefix(repo).map(slashed).unwrap_or_default();
            out.insert(rel, text);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cluster_ids_come_from_courtlistener_opinion_pages_only() {
        assert_eq!(
            cluster_of("https://www.courtlistener.com/opinion/145721/bowles-v-russell/"),
            Some(145_721)
        );
        assert_eq!(cluster_of("https://www.uspto.gov/x.pdf"), None);
        assert_eq!(cluster_of("https://www.courtlistener.com/opinion/x/"), None);
    }

    #[test]
    fn paths_normalize_lexically() {
        assert_eq!(normalize("lit/ca8/../frap"), "lit/frap");
        assert_eq!(normalize("./lit/../../x"), "x");
    }
}
