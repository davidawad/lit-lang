//! `litigate coverage [ROOT] [--check]`: `ROOT/docs/COVERAGE.md`, the
//! per-jurisdiction coverage table, generated from `ROOT/lit`,
//! `ROOT/outlines` and `ROOT/tests/expect`, never written by hand (lg-wvlf.2). `--check` writes
//! nothing and exits 2 when the committed table differs from the one the
//! repository yields.
//!
//! Each row counts, over its packages' `.lit` modules (parsed, so comments
//! and notes never count): rules modeled (`deadline` and `rule`
//! declarations), precedents modeled (bound case cites: the distinct
//! `[sources.*]` bindings titled as a case, plus the distinct
//! `court_listener_url` opinions) and expect tests (`test` blocks); and,
//! for a `lit/<d>` package, its matter expect tests (the
//! `tests/expect/<d>/*.toml` naming a `case`; one naming a `package` runs
//! that package's `test` blocks, already counted).
//!
//! Rows: every package in `lit/`, then every state, DC and territory. A
//! jurisdiction's packages are its state-court outlines
//! (`outlines/<circuit>/<jurisdiction>/<court>/`, `<court>` not
//! `federal-*`) and the `lit/` packages promoted from them, recognized by
//! the outline's `forum` or, once the outline is deleted, by
//! [`PROMOTED`]; its status is `promoted` when a `lit/` package has its
//! forum, else `outline` when an outline exists, else `none` (zero counts
//! until modeled). Its counts are those of the packages at its status. A
//! `lit/` package of `StateRule` authority promoted from no jurisdiction
//! fails: it is no federal package (lg-uyoh).
use crate::cmd::impact::{Manifest, EXPECT_DIR};
use crate::out::{Failure, Outcome, Report};
use litigate_sema::pkg::manifest::parse_manifest;
use litigate_syntax::{SyntaxKind, SyntaxToken};
use serde::Serialize;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// Kind of a jurisdiction row.
const STATE: &str = "state";
const DISTRICT: &str = "district";
const TERRITORY: &str = "territory";

/// Every US state, DC and territory: `(outline directory, name, kind)`,
/// in table order.
const JURISDICTIONS: [(&str, &str, &str); 56] = [
    ("alabama", "Alabama", STATE),
    ("alaska", "Alaska", STATE),
    ("arizona", "Arizona", STATE),
    ("arkansas", "Arkansas", STATE),
    ("california", "California", STATE),
    ("colorado", "Colorado", STATE),
    ("connecticut", "Connecticut", STATE),
    ("delaware", "Delaware", STATE),
    ("florida", "Florida", STATE),
    ("georgia", "Georgia", STATE),
    ("hawaii", "Hawaii", STATE),
    ("idaho", "Idaho", STATE),
    ("illinois", "Illinois", STATE),
    ("indiana", "Indiana", STATE),
    ("iowa", "Iowa", STATE),
    ("kansas", "Kansas", STATE),
    ("kentucky", "Kentucky", STATE),
    ("louisiana", "Louisiana", STATE),
    ("maine", "Maine", STATE),
    ("maryland", "Maryland", STATE),
    ("massachusetts", "Massachusetts", STATE),
    ("michigan", "Michigan", STATE),
    ("minnesota", "Minnesota", STATE),
    ("mississippi", "Mississippi", STATE),
    ("missouri", "Missouri", STATE),
    ("montana", "Montana", STATE),
    ("nebraska", "Nebraska", STATE),
    ("nevada", "Nevada", STATE),
    ("new-hampshire", "New Hampshire", STATE),
    ("new-jersey", "New Jersey", STATE),
    ("new-mexico", "New Mexico", STATE),
    ("new-york", "New York", STATE),
    ("north-carolina", "North Carolina", STATE),
    ("north-dakota", "North Dakota", STATE),
    ("ohio", "Ohio", STATE),
    ("oklahoma", "Oklahoma", STATE),
    ("oregon", "Oregon", STATE),
    ("pennsylvania", "Pennsylvania", STATE),
    ("rhode-island", "Rhode Island", STATE),
    ("south-carolina", "South Carolina", STATE),
    ("south-dakota", "South Dakota", STATE),
    ("tennessee", "Tennessee", STATE),
    ("texas", "Texas", STATE),
    ("utah", "Utah", STATE),
    ("vermont", "Vermont", STATE),
    ("virginia", "Virginia", STATE),
    ("washington", "Washington", STATE),
    ("west-virginia", "West Virginia", STATE),
    ("wisconsin", "Wisconsin", STATE),
    ("wyoming", "Wyoming", STATE),
    ("district-of-columbia", "District of Columbia", DISTRICT),
    ("american-samoa", "American Samoa", TERRITORY),
    ("guam", "Guam", TERRITORY),
    (
        "northern-mariana-islands",
        "Northern Mariana Islands",
        TERRITORY,
    ),
    ("puerto-rico", "Puerto Rico", TERRITORY),
    ("virgin-islands", "U.S. Virgin Islands", TERRITORY),
];

/// The forum of each promoted state-court `lit/` package and its
/// jurisdiction (a [`JURISDICTIONS`] directory), so the package keeps its
/// row once its outline is deleted. List a package here when it is
/// promoted.
const PROMOTED: [(&str, &str); 15] = [
    ("ak-superior", "alaska"),
    ("ak-supreme", "alaska"),
    ("fl-state-appellate", "florida"),
    ("fl-state-civil", "florida"),
    ("ia-district", "iowa"),
    ("iowactapp", "iowa"),
    ("ma-appeals-sjc", "massachusetts"),
    ("ma-superior-civil", "massachusetts"),
    ("me-law-court", "maine"),
    ("me-superior-civil", "maine"),
    ("mt-district", "montana"),
    ("sd", "south-dakota"),
    ("sd-circuit", "south-dakota"),
    ("va-appellate-civil", "virginia"),
    ("va-circuit-civil", "virginia"),
];

/// What a row counts.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize)]
struct Counts {
    rules: usize,
    precedents: usize,
    tests: usize,
    matters: usize,
}

impl Counts {
    fn add(self, other: Self) -> Self {
        Self {
            rules: self.rules + other.rules,
            precedents: self.precedents + other.precedents,
            tests: self.tests + other.tests,
            matters: self.matters + other.matters,
        }
    }
}

/// One package directory, read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct Package {
    /// `ROOT`-relative directory.
    dir: String,
    name: String,
    #[serde(skip)]
    forum: String,
    #[serde(skip)]
    authority: String,
    #[serde(flatten)]
    counts: Counts,
}

/// Packages by jurisdiction (a [`JURISDICTIONS`] directory).
type ByJurisdiction = BTreeMap<&'static str, Vec<Package>>;

/// One state, DC or territory row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct Row {
    name: &'static str,
    kind: &'static str,
    status: &'static str,
    packages: Vec<String>,
    #[serde(flatten)]
    counts: Counts,
}

/// `litigate coverage`.
pub fn coverage(root: Option<&str>, check: bool) -> Outcome {
    let root = Path::new(root.unwrap_or("."));
    let lit = files(&root.join("lit"))?;
    let outlines = files(&root.join("outlines"))?;
    let mut federal_and_promoted = packages(root, &lit)?;
    for (dir, n) in matters(root)? {
        let package = federal_and_promoted.iter_mut().find(|p| p.dir == dir);
        let Some(package) = package else {
            return Err(Failure::new(format!(
                "{EXPECT_DIR}/{}: no package `{dir}`",
                dir.trim_start_matches("lit/")
            )));
        };
        package.counts.matters = n;
    }
    let outlines = packages(root, &outlines)?;
    let mut by_slug = ByJurisdiction::new();
    for package in outlines {
        if let Some(slug) = state_court(&package.dir)? {
            by_slug.entry(slug).or_default().push(package);
        }
    }
    let (mut promoted, federal) = split(federal_and_promoted, &by_slug)?;
    let rows: Vec<Row> = JURISDICTIONS
        .iter()
        .map(|(slug, name, kind)| {
            let (status, packages) = match (promoted.remove(slug), by_slug.remove(slug)) {
                (Some(packages), _) => ("promoted", packages),
                (None, Some(packages)) => ("outline", packages),
                (None, None) => ("none", Vec::new()),
            };
            Row {
                name,
                kind,
                status,
                counts: packages
                    .iter()
                    .fold(Counts::default(), |sum, p| sum.add(p.counts)),
                packages: packages.into_iter().map(|p| p.dir).collect(),
            }
        })
        .collect();
    let states = rows
        .iter()
        .filter(|r| r.kind == STATE && r.status == "promoted")
        .count();
    let text = render(states, &federal, &rows);
    let file = root.join("docs/COVERAGE.md");
    let changed = std::fs::read_to_string(&file).ok().as_deref() != Some(text.as_str());
    if changed && !check {
        crate::cmd::build::write_atomic(&file, &text)?;
    }
    let mut report = Report::of(json!({
        "path": "docs/COVERAGE.md",
        "changed": changed,
        "promoted_states": states,
        "federal": federal,
        "jurisdictions": rows,
    }));
    report.failed = changed && check;
    Ok(report)
}

/// `lit` split into the packages promoted from each jurisdiction (by an
/// outline's forum in `by_slug` or by [`PROMOTED`]) and the federal rest.
///
/// # Errors
/// A `StateRule` package promoted from no jurisdiction.
fn split(
    lit: Vec<Package>,
    by_slug: &ByJurisdiction,
) -> Result<(ByJurisdiction, Vec<Package>), Failure> {
    let forums: BTreeMap<&str, &'static str> = PROMOTED
        .into_iter()
        .chain(
            by_slug
                .iter()
                .flat_map(|(slug, packages)| packages.iter().map(|p| (p.forum.as_str(), *slug))),
        )
        .collect();
    let mut promoted = ByJurisdiction::new();
    let mut federal = Vec::new();
    for package in lit {
        match forums.get(package.forum.as_str()) {
            Some(slug) => promoted.entry(*slug).or_default().push(package),
            None if package.authority == "StateRule" => {
                return Err(Failure::new(format!(
                    "{}: state-court forum `{}` has no outline and no `PROMOTED` jurisdiction",
                    package.dir, package.forum
                )));
            }
            None => federal.push(package),
        }
    }
    Ok((promoted, federal))
}

/// The jurisdiction of the state-court outline at `dir`
/// (`outlines/<circuit>/<jurisdiction>/<court>`), `None` for a federal
/// district or bankruptcy court's outline (`federal-*`, `bankruptcy-*`).
///
/// # Errors
/// A state-court outline under a directory naming no jurisdiction.
fn state_court(dir: &str) -> Result<Option<&'static str>, Failure> {
    let parts: Vec<&str> = dir.split('/').collect();
    let [_, _, slug, court] = parts[..] else {
        return Ok(None);
    };
    if court.starts_with("federal-") || court.starts_with("bankruptcy-") {
        return Ok(None);
    }
    JURISDICTIONS
        .iter()
        .find(|(s, _, _)| *s == slug)
        .map(|(s, _, _)| Some(*s))
        .ok_or_else(|| Failure::new(format!("{dir}: `{slug}` is no US state, DC or territory")))
}

/// The number of matter expect tests of each `lit/<d>` package directory:
/// the `tests/expect/<d>/*.toml` naming a `case`. No `tests/expect` is none.
///
/// # Errors
/// An unreadable or invalid expect test.
fn matters(root: &Path) -> Result<BTreeMap<String, usize>, Failure> {
    let dir = root.join(EXPECT_DIR);
    let mut out = BTreeMap::new();
    if !dir.is_dir() {
        return Ok(out);
    }
    for file in files(&dir)? {
        let relative = file.strip_prefix(&dir).unwrap_or(&file);
        let [forum, name] = relative.iter().collect::<Vec<_>>()[..] else {
            continue;
        };
        if Path::new(name).extension().is_none_or(|e| e != "toml") {
            continue;
        }
        let shown = format!("{EXPECT_DIR}/{}", relative.to_string_lossy());
        let test: Manifest = toml::from_str(&read_text(&file)?)
            .map_err(|_| Failure::new(format!("{shown}: invalid expect test")))?;
        if test.case.is_some() {
            *out.entry(format!("lit/{}", forum.to_string_lossy()))
                .or_default() += 1;
        }
    }
    Ok(out)
}

/// Every file under `dir`, in path order.
fn files(dir: &Path) -> Result<Vec<PathBuf>, Failure> {
    let mut stack = vec![dir.to_path_buf()];
    let mut files = Vec::new();
    while let Some(dir) = stack.pop() {
        let shown = dir.display();
        let entries = std::fs::read_dir(&dir)
            .and_then(|it| {
                it.map(|e| e.map(|e| e.path()))
                    .collect::<Result<Vec<_>, _>>()
            })
            .map_err(|e| Failure::new(format!("{shown}: {e}")))?;
        for path in entries {
            if path.is_dir() {
                stack.push(path);
            } else {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

/// The package of each `Lit.toml` among `files`, in path order; its
/// modules are the `.lit` files under its directory (packages do not nest
/// in `lit/` or `outlines/`).
fn packages(root: &Path, files: &[PathBuf]) -> Result<Vec<Package>, Failure> {
    files
        .iter()
        .filter(|f| f.ends_with("Lit.toml"))
        .map(|manifest| {
            let dir = manifest.parent().unwrap_or(root);
            let modules = files
                .iter()
                .filter(|f| f.starts_with(dir) && f.extension().is_some_and(|e| e == "lit"));
            read(root, dir, modules)
        })
        .collect()
}

fn read_text(path: &Path) -> Result<String, Failure> {
    std::fs::read_to_string(path).map_err(|e| Failure::new(format!("{}: {e}", path.display())))
}

/// The package at `dir` with `modules`, counted.
fn read<'a>(
    root: &Path,
    dir: &Path,
    modules: impl Iterator<Item = &'a PathBuf>,
) -> Result<Package, Failure> {
    let relative = dir
        .strip_prefix(root)
        .unwrap_or(dir)
        .to_string_lossy()
        .replace('\\', "/");
    let manifest = parse_manifest(&read_text(&dir.join("Lit.toml"))?)
        .map_err(|_| Failure::new(format!("{relative}/Lit.toml: invalid manifest")))?
        .manifest;
    let mut cases: BTreeSet<String> = manifest
        .sources
        .iter()
        .filter(|s| s.title.as_deref().is_some_and(is_case))
        .map(|s| format!("source {}", s.id))
        .collect();
    let mut counts = Counts::default();
    for file in modules {
        let tree = litigate_syntax::parse_lit(&read_text(file)?).syntax();
        for node in tree.descendants() {
            match node.kind() {
                SyntaxKind::DeadlineDecl | SyntaxKind::RuleDecl => counts.rules += 1,
                SyntaxKind::TestDecl => counts.tests += 1,
                _ => {}
            }
        }
        // Each `court_listener_url` keyword's URL: the next string token.
        cases.extend(
            tokens(tree.first_token())
                .filter(|t| t.kind() == SyntaxKind::KwCourtListenerUrl)
                .filter_map(|t| tokens(t.next_token()).find(|t| t.kind() == SyntaxKind::String))
                .map(|t| t.text().to_owned()),
        );
    }
    counts.precedents = cases.len();
    Ok(Package {
        dir: relative,
        name: manifest.name,
        forum: manifest.forum,
        authority: manifest.authority,
        counts,
    })
}

/// `first` and every token after it, in file order.
fn tokens(first: Option<SyntaxToken>) -> impl Iterator<Item = SyntaxToken> {
    std::iter::successors(first, SyntaxToken::next_token)
}

/// A source title naming a case (`A v. B`, `In re A`, `Ex parte A`).
fn is_case(title: &str) -> bool {
    [" v. ", "In re ", "Ex parte "]
        .iter()
        .any(|form| title.contains(form))
}

/// The Markdown table.
fn render(states: usize, federal: &[Package], rows: &[Row]) -> String {
    let total = rows.iter().filter(|r| r.kind == STATE).count();
    let mut out = format!(
        "# Coverage\n\n\
         **States with a promoted package: {states} of {total}.**\n\n\
         <!-- Generated by `just coverage` (`litigate coverage`) from lit/, \
         outlines/ and tests/expect/; do not edit. `just coverage-check` (in `just ci`) fails \
         when this file is stale. -->\n\n\
         Per package or jurisdiction, counted over its `.lit` modules: **rules** \
         are `deadline` and `rule` declarations, **precedents** are bound case \
         cites (the distinct `[sources.*]` bindings titled as a case plus the \
         distinct `court_listener_url` opinions), **tests** are `test` blocks of \
         `expect`s, **matters** are a `lit/<d>` package's matter expect tests \
         (`tests/expect/<d>/*.toml` naming a `case` and the `world` it runs \
         in). A jurisdiction's packages are its state-court outlines \
         (`outlines/<circuit>/<jurisdiction>/`, federal district and \
         bankruptcy courts excluded) and the `lit/` packages promoted from them (an outline's \
         `forum`, kept once the outline is deleted). Status: `promoted` (a `lit/` package), `outline` (outlines \
         only, counted as such), `none` (zero until modeled). Public law only.\n\n\
         ## Federal packages (lit/)\n\n\
         | Package | Directory | Rules | Precedents | Tests | Matters |\n\
         |---|---|--:|--:|--:|--:|\n"
    );
    for p in federal {
        let c = p.counts;
        let _ = writeln!(
            out,
            "| {} | `{}` | {} | {} | {} | {} |",
            p.name, p.dir, c.rules, c.precedents, c.tests, c.matters
        );
    }
    out.push_str(
        "\n## States, DC and territories\n\n\
         | Jurisdiction | Kind | Status | Rules | Precedents | Tests | Matters | Packages |\n\
         |---|---|---|--:|--:|--:|--:|---|\n",
    );
    for r in rows {
        let c = r.counts;
        let packages: Vec<String> = r.packages.iter().map(|d| format!("`{d}`")).collect();
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} | {} | {} |",
            r.name,
            r.kind,
            r.status,
            c.rules,
            c.precedents,
            c.tests,
            c.matters,
            packages.join(", ")
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_case_title_is_one_naming_parties_or_a_proceeding() {
        assert!(is_case("Bowles v. Russell, 551 U.S. 205 (2007)"));
        assert!(is_case("In re Bilski, 545 F.3d 943 (Fed. Cir. 2008)"));
        assert!(is_case("Ex parte McCardle, 74 U.S. 506 (1869)"));
        assert!(!is_case("Federal Rules of Civil Procedure"));
    }

    #[test]
    fn only_a_state_court_outline_has_a_jurisdiction() {
        let court = |dir| state_court(dir).map_err(|f| f.0);
        assert_eq!(court("outlines/x/florida/state-civil"), Ok(Some("florida")));
        assert_eq!(court("outlines/x/florida/federal-sd-fla"), Ok(None));
        assert_eq!(court("outlines/national/tax-court"), Ok(None));
        assert_eq!(
            court("outlines/x/atlantis/state-civil"),
            Err(
                "outlines/x/atlantis/state-civil: `atlantis` is no US state, DC or territory"
                    .to_owned()
            )
        );
    }

    #[test]
    fn every_promoted_forum_names_a_jurisdiction() {
        for (forum, slug) in PROMOTED {
            assert!(
                JURISDICTIONS.iter().any(|(s, _, _)| *s == slug),
                "{forum}: `{slug}`"
            );
        }
    }

    #[test]
    fn counts_add_field_by_field() {
        let one = Counts {
            rules: 1,
            precedents: 2,
            tests: 3,
            matters: 4,
        };
        assert_eq!(
            one.add(one),
            Counts {
                rules: 2,
                precedents: 4,
                tests: 6,
                matters: 8
            }
        );
    }
}
