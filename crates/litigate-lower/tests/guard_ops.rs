//! The provenance of typed guard operations (SPEC §3.6): a links package's
//! source map records, per patched edge, the base guard, every operation
//! in the order it applies with its authority and whether it is in force
//! at the build dates, and the guard they compose; two change-file sets
//! differ there, so an alternate timeline can diff their guards.
use litigate_lower::source_map::GuardTrace;
use litigate_lower::{Lowerer, Options};
use litigate_sema::db::inputs::create_workspace;
use litigate_sema::hir::Package;
use litigate_sema::pkg::load::{LoadedFile, LoadedPackage};
use litigate_sema::LitDatabase;

const BASE_TOML: &str = "[package]
name = \"base\"
version = \"1.0.0\"
language = 1
pack_id = \"base\"
forum = \"demo\"
authority = \"FederalRule\"
title = \"T\"
in_force = \"2009-12-01\"
known_at = \"2025-01-01\"

[package.roles]
Plaintiff = \"self\"
Court = \"nature\"
";

const BASE: &str = "module base
start intake

authority FederalRule > LocalRule > CaseOrder

event Filed()
event Moved()
event Excused()

state intake: \"Intake\" by Plaintiff
{
  file \"File\" -> check { event Filed() }
}

chance check: \"Timely?\" by Court effective [2009-12-01, 2010-01-01) delegable_to [CaseOrder]
{
  tags [fact]
  timely \"Timely\" -> won p 1/2 { when has_event(Moved()) && has_event(Filed()) }
  late \"Late\" -> lost p 1/2 { when !has_event(Moved()) && !has_event(Excused()) }
}

chance check: \"Timely?\" by Court effective [2010-01-01, ..) delegable_to [CaseOrder]
{
  tags [fact]
  timely \"Timely\" -> won p 1/2 { when has_event(Moved()) }
  late \"Late\" -> lost p 1/2 { when !has_event(Moved()) && !has_event(Excused()) }
}

terminal won: \"Won\" { payoff 0.usd }
terminal lost: \"Lost\" { payoff 0.usd }
";

const LINKS_TOML: &str = "[package]
name = \"order\"
version = \"1.0.0\"
language = 1
in_force = \"2009-12-01\"
known_at = \"2025-01-01\"

[dependencies]
base = { version = \"^1.0.0\", path = \"../base\" }
";

const NARROW: &str = "    when and has_event(Excused()) by CaseOrder effective [2010-01-01, ..) \
                      cite \"Order, Dkt. 12\"";
const REPLACE: &str =
    "    supersedes when has_event(Excused()) by FederalRule effective [2026-01-01, ..) \
                       cite \"Fed. R. Civ. P. 1\"";

/// The base pack (two versions of `check`, the build emitting the
/// later) and a links package patching `timely` with `ops`.
fn compiled(ops: &[&str]) -> Vec<Package> {
    let links = format!(
        "module order\nimport base.{{ Moved, Excused, FederalRule, CaseOrder }}\n\
         instance base@order {{\n  pack base\n  patch timely {{\n{}\n  }}\n}}\n",
        ops.join("\n")
    );
    let package = |name: &str, manifest: &str, text: &str| LoadedPackage {
        root: format!("pkgs/{name}"),
        manifest: manifest.to_owned(),
        lock: None,
        files: vec![LoadedFile {
            relative: format!("{name}.lit"),
            text: text.to_owned(),
        }],
        texts: Default::default(),
    };
    // An unrelated pack loads first: its nodes are never the base's.
    let loaded = [
        package(
            "other",
            &BASE_TOML.replace("\"base\"", "\"other\""),
            "module other\nstart timely\nterminal timely: \"T\" { payoff 0.usd }\n",
        ),
        package("base", BASE_TOML, BASE),
        package("order", LINKS_TOML, &links),
    ];
    let mut db = LitDatabase::default();
    let ws = create_workspace(&mut db, &loaded);
    litigate_sema::check_workspace(&db, ws)
        .iter()
        .map(|c| {
            c.package.clone().unwrap_or_else(|| {
                panic!(
                    "{}",
                    serde_json::to_string_pretty(&c.diagnostics).unwrap_or_default()
                )
            })
        })
        .collect()
}

/// The links package's guard provenance at `as_of`.
fn guards(ops: &[&str], as_of: Option<&str>) -> Vec<GuardTrace> {
    let packages = compiled(ops);
    let refs: Vec<&Package> = packages.iter().collect();
    let links = packages
        .iter()
        .find(|p| p.manifest.links)
        .unwrap_or_else(|| panic!("no links package"));
    let options = Options {
        as_of: as_of.map(str::to_owned),
        ..Options::default()
    };
    Lowerer::new(&refs)
        .package(links, &options)
        .unwrap_or_else(|d| panic!("{d:#?}"))
        .source_map
        .guards
}

/// `traces` without source spans, which follow the text.
fn unspanned(mut traces: Vec<GuardTrace>) -> Vec<GuardTrace> {
    for op in traces.iter_mut().flat_map(|t| t.ops.iter_mut()) {
        op.span = None;
    }
    traces
}

#[test]
fn the_source_map_records_each_guard_operation_with_its_authority() {
    let after = Some("2026-06-01");
    let one = unspanned(guards(&[NARROW], after));
    let both = unspanned(guards(&[REPLACE, NARROW], after));
    // Source order is not consulted: the case order applies first.
    assert_eq!(both, unspanned(guards(&[NARROW, REPLACE], after)));
    // Before the replacement takes effect (the build's default date is
    // the package corpus date), only the narrowing applies.
    let before = unspanned(guards(&[REPLACE, NARROW], None));
    insta::assert_json_snapshot!([&one, &both, &before]);
    // The alternate timeline's diff: the second change set adds the
    // replacement, which reshapes the composed guard.
    assert_eq!(one.len(), both.len());
    assert_eq!(one[0].base, both[0].base);
    assert_ne!(one[0].composed, both[0].composed);
    let added: Vec<_> = both[0]
        .ops
        .iter()
        .filter(|op| !one[0].ops.contains(op))
        .map(|op| (op.op.as_str(), op.by.as_str(), op.cite.as_str()))
        .collect();
    assert_eq!(added, [("replace", "FederalRule", "Fed. R. Civ. P. 1")]);
}

#[test]
fn a_links_package_without_guard_operations_records_none() {
    let packages = compiled(&["    note \"no guard change\""]);
    let refs: Vec<&Package> = packages.iter().collect();
    let links = packages
        .iter()
        .find(|p| p.manifest.links)
        .unwrap_or_else(|| panic!("no links package"));
    let lowered = Lowerer::new(&refs)
        .package(links, &Options::default())
        .unwrap_or_else(|d| panic!("{d:#?}"));
    assert!(lowered.source_map.guards.is_empty());
    let json = serde_json::to_value(&lowered.source_map).unwrap_or_default();
    assert!(json.get("guards").is_none(), "{json}");
}
